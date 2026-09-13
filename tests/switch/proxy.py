"""Fault injection confined to the disposable compositor socket."""

import select
import socket
import threading
import time
from contextlib import suppress
from pathlib import Path


class Proxy:
    def __init__(self, real_socket: Path) -> None:
        self.real_socket = real_socket
        self.stall = threading.Event()
        self.unsupported = threading.Event()
        self.malformed = threading.Event()
        self.fail_data = threading.Event()
        self.data_failed = threading.Event()
        self.queries = 0
        self.seen = threading.Event()
        self.stopped = threading.Event()
        self.lock = threading.Lock()
        self.pending: list[socket.socket] = []
        self.durations: list[float] = []
        self.overlaps = 0
        self.received = 0

    def serve(self, listener: socket.socket) -> None:
        listener.settimeout(0.1)
        while not self.stopped.is_set():
            try:
                client, _ = listener.accept()
            except TimeoutError:  # ruff: ignore[try-except-continue] -- Periodically check test shutdown.
                continue
            except OSError:
                break
            threading.Thread(
                target=self._connection, args=(client,), daemon=True
            ).start()

    def _connection(self, client: socket.socket) -> None:
        with client:
            client.settimeout(3)
            try:
                self._reply(client)
            except OSError:
                # Cancellation closes the socket before a reply is delivered.
                return

    def _reply(self, client: socket.socket) -> None:
        data = client.recv(65536)
        is_query = data.startswith(b"/repl ") and b"is_key_down" in data
        if is_query:
            self.queries += 1
        if data.startswith(b"j/monitors") and self.fail_data.is_set():
            self.fail_data.clear()
            client.sendall(b"injected malformed monitor response")
            self.data_failed.set()
        elif is_query and self.malformed.is_set():
            client.sendall(b"ok")
        elif is_query and self.unsupported.is_set():
            client.sendall(b"unknown request")
        elif is_query and self.stall.is_set():
            self._stall_query(client)
        else:
            self._forward(client, data)

    def _forward(self, client: socket.socket, data: bytes) -> None:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as upstream:
            upstream.settimeout(3)
            upstream.connect(str(self.real_socket))
            upstream.sendall(data)
            while data := upstream.recv(65536):
                client.sendall(data)

    def _stall_query(self, client: socket.socket) -> None:
        started = time.monotonic()
        with self.lock:
            for previous in self.pending:
                with suppress(OSError):
                    ready = select.select([previous], [], [], 0)[0]
                    if not ready or previous.recv(1, socket.MSG_PEEK):
                        self.overlaps += 1
            self.pending.append(client)
            self.received += 1
        self.seen.set()
        try:
            # The client must close this connection on timeout or cancellation.
            with suppress(OSError):
                while client.recv(256):
                    pass
        finally:
            with self.lock:
                self.pending.remove(client)
                self.durations.append(time.monotonic() - started)

    def stop(self) -> None:
        self.stopped.set()
        with self.lock:
            for client in self.pending:
                with suppress(OSError):
                    client.shutdown(socket.SHUT_RDWR)
