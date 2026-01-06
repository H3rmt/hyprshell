use crate::config::Config;
use crate::store::util::create_storage_path;
use crate::store::write::get_storage_writer;
use anyhow::Context;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;
use std::time::SystemTime;
use tracing::{trace, warn};

#[derive(Debug, bitcode::Encode, bitcode::Decode)]
pub enum ClipboardDataType {
    Alias(Box<str>),
    Data(Vec<u8>),
}

pub fn compress_and_store_map(data: HashMap<String, Vec<u8>>, config: &Config, cache_dir: &Path) {
    let combined_size = data.values().map(Vec::len).sum::<usize>();
    let (data, contains_image) = deduplicate_clipboard_entries(data, true);
    let compressed_combined_size = data
        .values()
        .filter_map(|dt| {
            if let ClipboardDataType::Data(d) = dt {
                Some(d.len())
            } else {
                None
            }
        })
        .sum::<usize>();
    trace!(
        "Combined size: {} bytes, compressed size {} bytes, storing {} aliased and {} data entries",
        combined_size,
        compressed_combined_size,
        data.values()
            .filter(|dt| matches!(dt, ClipboardDataType::Alias(_)))
            .count(),
        data.values()
            .filter(|dt| matches!(dt, ClipboardDataType::Data(_)))
            .count()
    );

    // dont compress if contains image
    if let Err(err) = store_map(&data, config, !contains_image, cache_dir) {
        warn!("Failed to store clipboard data: {err}");
    }
}

fn store_map(
    data: &HashMap<Box<str>, ClipboardDataType>,
    config: &Config,
    compress: bool,
    cache_dir: &Path,
) -> anyhow::Result<()> {
    let now = SystemTime::now();
    let mut cursor = Cursor::new(Vec::new());
    let ext = {
        let (mut writer, ext) = get_storage_writer(&mut cursor, config, compress);
        let encoded = bitcode::encode(data);
        writer
            .write_all(&encoded)
            .context("Failed to write encoded clipboard data")?;
        ext
    };
    let storage_path = create_storage_path(cache_dir, "data", &format!("bin.{ext}"))
        .context("Failed to get storage path for clipboard data")?;
    let mut file = File::create(&storage_path).context("Failed to create clipboard data file")?;
    file.write_all(&cursor.into_inner())
        .context("Failed to write clipboard data")?;

    trace!(
        "Wrote clipboard data to {} ({} bytes) in {:?}",
        storage_path.display(),
        file.metadata().map(|m| m.len()).unwrap_or(0),
        now.elapsed()?
    );
    Ok(())
}

pub fn deduplicate_clipboard_entries(
    data: HashMap<String, Vec<u8>>,
    dedup: bool,
) -> (HashMap<Box<str>, ClipboardDataType>, bool) {
    let time = std::time::Instant::now();
    let mut image_found = false;
    let mut dedupted = 0u16;
    let mut map: HashMap<Box<str>, ClipboardDataType> = HashMap::new();
    'outer: for (mime, data) in data {
        if mime.starts_with("image/") {
            image_found = true;
        }
        if dedup {
            for (f_mime, f_dt) in map.iter().filter(|(m, _)| !m.starts_with("image/")) {
                if let ClipboardDataType::Data(check_data) = f_dt
                    && data.eq(check_data)
                {
                    trace!("Deduped MIME type {mime} to {f_mime}");
                    map.insert(
                        mime.into_boxed_str(),
                        ClipboardDataType::Alias(f_mime.clone()),
                    );
                    dedupted += 1;
                    continue 'outer;
                }
            }
        }
        map.insert(mime.into_boxed_str(), ClipboardDataType::Data(data));
    }
    trace!(
        "Deduplication took {:?}, dedupted {dedupted} entries",
        time.elapsed()
    );
    (map, image_found)
}
