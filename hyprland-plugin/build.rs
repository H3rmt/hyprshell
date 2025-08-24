use std::error::Error;
use std::fs::read_dir;
use std::io::{Read, Write};
use std::{env, fs::File, path::Path};
use zip::ZipWriter;
use zip::write::FileOptions;

fn include_plugin() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;

    let status = std::process::Command::new("make")
        .arg("prepare-combined")
        .current_dir("plugin")
        .status()?;

    if !status.success() {
        return Err("Failed to run make prepare".into());
    }

    let zip_path = Path::new(&out_dir).join("plugin.zip");

    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(&file);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .compression_level(None)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    for file in read_dir("plugin/out")?.flatten() {
        if file.path().is_dir() {
            continue;
        }
        // we can use the name as we dont allow for folders here
        zip.start_file(file.file_name().to_string_lossy(), options)?;
        let mut f = File::open(file.path())?;
        f.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
        buffer.clear();
    }
    zip.finish()?;
    Ok(())
}

fn main() {
    include_plugin().expect("Failed to include plugin");
    println!("cargo:rerun-if-changed=plugin/src/*");
}
