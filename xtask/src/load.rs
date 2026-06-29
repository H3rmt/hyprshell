use std::path::Path;

use anyhow::Context;

/// Load a toml file and parse it into a document.
pub fn load_toml(file_path: &Path) -> anyhow::Result<toml_edit::DocumentMut> {
    let content = std::fs::read_to_string(file_path).context("failed to read toml file")?;
    let doc = content
        .parse::<toml_edit::DocumentMut>()
        .context("failed to parse toml file")?;
    Ok(doc)
}

pub fn write_toml(file_path: &Path, doc: &toml_edit::DocumentMut) -> anyhow::Result<()> {
    std::fs::write(file_path, doc.to_string()).context("failed to write toml file")?;
    Ok(())
}
