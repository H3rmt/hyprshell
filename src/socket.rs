use anyhow::{Context, bail};
use async_channel::{Receiver, Sender};
use core_lib::transfer;
use core_lib::transfer::ExternalTransferType;
use core_lib::util::get_daemon_socket_path_buff;
use std::fs::remove_file;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net;
use std::os::unix::net::UnixStream;
use std::thread;
use tracing::{debug, debug_span, info, instrument, trace, warn};

pub fn socket_handler(event_sender: Sender<ExternalTransferType>) {
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
                handle_client(conn, &event_sender)
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

#[instrument(level = "debug", skip(stream, event_sender))]
fn handle_client(
    mut stream: UnixStream,
    event_sender: &Sender<ExternalTransferType>,
) -> anyhow::Result<()> {
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
    event_sender
        .send_blocking(transfer)
        .context("Failed to send transfer")?;
    let _ = stream
        .write_all(b"OK")
        .and_then(|()| stream.write_all(b"\0"));
    Ok(())
}
