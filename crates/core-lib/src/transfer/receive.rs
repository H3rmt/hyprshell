use crate::transfer::ExternalTransferType;
use anyhow::Context;
use tracing::debug;

pub fn receive_from_buffer(mut buffer: Vec<u8>) -> anyhow::Result<ExternalTransferType> {
    if buffer.last() == Some(&0) {
        let _ = buffer.pop();
    }
    let str =
        str::from_utf8(&buffer).with_context(|| format!("Failed to convert buffer: {buffer:?}"))?;
    let transfer: ExternalTransferType =
        serde_json::from_str(str).with_context(|| format!("Failed to deserialize str: {str:?}"))?;
    debug!("Received command: {transfer:?}");
    Ok(transfer)
}
