use crate::config::Config;
use crate::store::util::create_storage_path;
use crate::store::write::get_storage_writer;
use anyhow::Context;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;
use tracing::trace;

pub fn store_text(text: &str, config: &Config, cache_dir: &Path) -> anyhow::Result<()> {
    let compress = text.len() > 100;
    let mut cursor = Cursor::new(Vec::new());
    let ext = {
        let (mut write, ext) = get_storage_writer(&mut cursor, config, compress);
        write
            .write_all(text.as_bytes())
            .context("Failed to write text to clipboard")?;
        ext
    };
    let storage_path = create_storage_path(cache_dir, "text", &format!("txt.{ext}"))
        .context("Failed to get storage path for clipboard data")?;
    let mut file = File::create(&storage_path).context("Failed to create clipboard data file")?;
    file.write_all(&cursor.into_inner())
        .context("Failed to write clipboard data")?;

    trace!(
        "Wrote text to {} ({} bytes)",
        storage_path.display(),
        file.metadata().map_or(0, |m| m.len())
    );
    Ok(())
}
