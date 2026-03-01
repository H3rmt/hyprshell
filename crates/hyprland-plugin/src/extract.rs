use crate::ASSET_ZIP;
use anyhow::Context;
use std::fs::File;
use std::io::{Cursor, copy};
use std::path::Path;
use tracing::trace;
use zip::ZipArchive;

pub fn extract_plugin(path: &Path) -> anyhow::Result<()> {
    let mut archive = ZipArchive::new(Cursor::new(ASSET_ZIP)).expect("failed to read zip");

    let mut counter = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("error reading zip file")?;
        let out_path = path.join(file.name());

        if let Some(p) = out_path.parent() {
            std::fs::create_dir_all(p)
                .with_context(|| format!("unable to create dir: {}", p.display()))?;
        }
        let mut outfile = File::create(&out_path)
            .with_context(|| format!("unable to create file: {}", out_path.display()))?;
        copy(&mut file, &mut outfile)
            .with_context(|| format!("unable to copy file: {}", out_path.display()))?;
        counter += 1;
    }

    trace!("extracted {} files", counter);
    Ok(())
}
