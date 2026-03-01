use anyhow::{Context, bail};
use async_channel::{Receiver, Sender};
use core_lib::transfer;
use core_lib::transfer::{PluginConfig, TransferType};
use core_lib::util::get_daemon_socket_path_buff;
use std::fs::remove_file;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net;
use std::os::unix::net::UnixStream;
use std::thread;
use tracing::{debug, debug_span, info, trace, warn};

pub fn socket_handler(event_sender: Sender<TransferType>, event_receiver: Receiver<TransferType>) {
    let _span = debug_span!("socket_handler").entered();
    let path = &get_daemon_socket_path_buff();
    let listener = {
        // remove old PATH
        if path.exists() {
            remove_file(path).expect("Unable to remove old socket file");
        }
        net::UnixListener::bind(path)
            .unwrap_or_else(|_| panic!("Failed to bind to socket {}", path.display()))
    };
    debug!("Starting socket on {}", path.display());

    loop {
        let path = listener.accept();
        match path {
            Ok((conn, _)) => {
                trace!(
                    "New connection from {}",
                    conn.peer_addr()
                        .map(|a| format!("{a:?}"))
                        .unwrap_or_default()
                );
                handle_client(conn, &event_sender, &event_receiver)
                    .context("Failed to handle client")
                    .unwrap_or_else(|e| {
                        warn!("Failed to handle connection: {e:?}");
                    });
            }
            Err(e) => {
                warn!("Failed to accept connection: {e:?}");
            }
        }
    }
}

pub fn remove_socket() {
    let path = &get_daemon_socket_path_buff();
    if path.exists() {
        remove_file(path).expect("Unable to remove old socket file");
    }
}

fn handle_client(
    mut stream: UnixStream,
    event_sender: &Sender<TransferType>,
    event_receiver: &Receiver<TransferType>,
) -> anyhow::Result<()> {
    let _span = debug_span!("handle_client").entered();
    let mut buffer = vec![];
    let mut reader = BufReader::new(&mut stream);
    reader
        .read_until(b'\0', &mut buffer)
        .context("Can't read data from socket")?;
    if buffer.is_empty() {
        return Ok(());
    }
    let transfer = match transfer::receive_from_buffer(buffer) {
        Ok(transfer) => transfer,
        Err(err) => {
            let _ = stream
                .write_all(b"ERR")
                .and_then(|()| stream.write_all(b"\0"));
            bail!("Invalid transfer received.\n{err:?}");
        }
    };
    match transfer {
        TransferType::GetConfigWatch => {
            trace!("Starting config update thread");
            let event_receiver = event_receiver.clone();
            let stream_clone = stream.try_clone().context("Failed to clone stream")?;
            thread::spawn(move || send_new_plugin_config(event_receiver, stream_clone));
        }
        _ => {
            event_sender
                .send_blocking(transfer)
                .context("Failed to send transfer")?;
            let _ = stream
                .write_all(b"OK")
                .and_then(|()| stream.write_all(b"\0"));
        }
    }
    Ok(())
}

fn send_new_plugin_config(event_receiver: Receiver<TransferType>, mut stream_clone: UnixStream) {
    loop {
        match event_receiver.recv_blocking() {
            Ok(TransferType::SetPluginConfig(conf)) => {
                if !send_config(conf, &mut stream_clone) {
                    return;
                }
            }
            Ok(_) => {
                // Check if stream is still open by trying to write a null byte
                if stream_clone.write_all(b"\0").is_err() {
                    return;
                }
            }
            Err(_) => return,
        }
    }
}

fn send_config(conf: PluginConfig, stream_clone: &mut UnixStream) -> bool {
    match serde_json::to_string(&conf) {
        Ok(str) => {
            if stream_clone.write_all(str.as_bytes()).is_err() {
                return false;
            }
        }
        Err(err) => {
            warn!("Failed to serialize config: {err:?} for config");
            return false;
        }
    };
    if stream_clone.write_all(b"\0").is_err() {
        return false;
    };
    true
}
