use std::{fs, path::Path};

use anyhow::Context;
use toml_edit::{DocumentMut, value};

/// Parse a toml file and edit a property in it, then write it back to the file.
///
/// # Example
/// ```
/// edit_toml("Cargo.toml", "package.version", "1.2.3").unwrap();
/// ```
pub fn edit_toml(file_path: &Path, property_path: &str, new_value: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path).context("failed to read toml file")?;
    let mut doc = content
        .parse::<DocumentMut>()
        .context("failed to parse toml file")?;
    let mut parts = property_path.rsplitn(2, '.');
    let key = parts.next().unwrap();
    if let Some(table_path) = parts.next() {
        let mut table = doc.as_table_mut();
        for part in table_path.split('.') {
            table = table[part]
                .as_table_mut()
                .expect("failed to access nested table");
        }
        table[key] = value(new_value);
    } else {
        doc[key] = value(new_value);
    }
    fs::write(file_path, doc.to_string()).context("failed to write toml file")?;
    Ok(())
}
