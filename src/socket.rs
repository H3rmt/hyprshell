use anyhow::Context;
use async_channel::Sender;
use core_lib::transfer::TransferType;
use core_lib::{get_daemon_socket_path_buff, transfer};
use std::fs::remove_file;
use std::io::Read;
use std::os::unix::net::UnixStream;
use tracing::{Level, info, span, warn};

pub fn socket_handler(event_sender: Sender<TransferType>) {
    let _span = span!(Level::TRACE, "socket_handler").entered();
    let buf = get_daemon_socket_path_buff();
    let path = buf.as_path();
    let listener = {
        // remove old PATH
        if path.exists() {
            remove_file(path).expect("Unable to remove old socket file");
        }
        std::os::unix::net::UnixListener::bind(path)
            .unwrap_or_else(|_| panic!("Failed to bind to socket {path:?}"))
    };
    info!("Starting socket on {path:?}");

    loop {
        let path = listener.accept();
        match path {
            Ok((conn, _)) => {
                handle_client(conn, &event_sender)
                    .context("Failed to handle client")
                    .unwrap_or_else(|e| {
                        warn!("Failed to handle connection {:?}", e);
                    });
            }
            Err(e) => {
                warn!("Failed to accept connection: {e:?}");
            }
        };
    }
}

fn handle_client(
    mut stream: UnixStream,
    event_sender: &Sender<TransferType>,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "handle_client").entered();
    let mut buffer = Vec::with_capacity(1024);
    stream
        .read_to_end(&mut buffer)
        .context("Failed to read data from buffer")?;
    if buffer.is_empty() {
        return Ok(());
    }
    let transfer =
        transfer::receive_from_buffer(buffer).context("Failed to receive from buffer")?;
    event_sender
        .send_blocking(transfer)
        .context("Failed to send transfer")?;
    Ok(())
}
