use crate::util::get_daemon_socket_path_buff;
use anyhow::Context;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use tracing::{debug, trace};

pub fn send_raw_to_socket(data: &str) -> anyhow::Result<()> {
    let path = get_daemon_socket_path_buff();
    debug!("Sending data to socket: {data}");
    trace!("Socket path: {}", path.display());
    let mut stream = UnixStream::connect(&path)
        .with_context(|| format!("Can't connect to daemon socket {}", path.display()))?;
    trace!("Socket connected, sending data");
    stream
        .write_all(data.as_bytes())
        .and_then(|()| stream.write_all(b"\0"))
        .context("Can't send data to socket")?;
    trace!("Data sent");
    let mut reader = BufReader::new(stream);
    let mut buffer = vec![];
    reader
        .read_until(b'\0', &mut buffer)
        .context("Can't read data from socket")?;
    let ret = String::from_utf8(buffer).context("Failed to convert buffer")?;
    debug!("Received data from socket: {ret}");
    if ret != "OK\0" {
        return Err(anyhow::anyhow!("Invalid response from daemon: {ret}"));
    }
    Ok(())
}
