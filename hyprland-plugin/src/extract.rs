use crate::ASSET_ZIP;
use std::fs::File;
use std::io::{Cursor, copy};
use tempfile::{TempDir, tempdir};
use tracing::trace;
use zip::ZipArchive;

pub fn extract_plugin() -> anyhow::Result<TempDir> {
    let tmp_dir = tempdir().expect("create tempdir failed");
    let mut archive = ZipArchive::new(Cursor::new(ASSET_ZIP)).expect("failed to read zip");

    let mut counter = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = tmp_dir.path().join(file.name());

        if let Some(p) = out_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut outfile = File::create(&out_path)?;
        copy(&mut file, &mut outfile)?;
        counter += 1;
    }

    trace!("extracted {} files", counter);
    Ok(tmp_dir)
}
