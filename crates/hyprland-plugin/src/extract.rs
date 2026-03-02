use crate::ASSET_ZIP_52;
use crate::ASSET_ZIP_54;
use std::fs::File;
use std::io::{Cursor, copy};
use tempfile::TempDir;
use tracing::trace;
use zip::ZipArchive;

pub fn extract_plugin(version: &semver::Version) -> anyhow::Result<TempDir> {
    let asset = if version >= &semver::Version::new(0, 54, 0) {
        tracing::info!("Extracting plugin assets for version 54");
        ASSET_ZIP_54
    } else {
        tracing::info!("Extracting plugin assets for version 52");
        ASSET_ZIP_52
    };
    let tmp_dir = TempDir::with_suffix(env!("CARGO_PKG_NAME")).expect("create tempdir failed");
    let mut archive = ZipArchive::new(Cursor::new(asset)).expect("failed to read zip");

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
