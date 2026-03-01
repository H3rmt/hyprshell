use std::fs::read_dir;
use std::io::{Read, Write};
use std::{env, fs::File, path::Path};
use zip::ZipWriter;
use zip::write::FileOptions;

fn include_plugin() {
    let out_dir = env::var("OUT_DIR").expect("out dir missing??");
    let prepare_dir = Path::new("plugin/src");

    let zip_path = Path::new(&out_dir).join("plugin.zip");
    let file = File::create(&zip_path).expect("Failed to create zip file");
    let mut zip = ZipWriter::new(&file);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .compression_level(None)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    for file in read_dir(prepare_dir)
        .expect("Failed to read prepare dir")
        .flatten()
    {
        // we can use the name as we dont allow for folders here
        zip.start_file(file.file_name().to_string_lossy(), options)
            .expect("Failed to start file in zip");
        let mut f = File::open(file.path()).expect("Failed to open file");
        f.read_to_end(&mut buffer).expect("Failed to read file");
        zip.write_all(&buffer).expect("Failed to write file to zip");
        buffer.clear();
    }
    zip.finish().expect("Failed to finish plugin zip");
}

fn main() {
    include_plugin();
    println!("cargo::rerun-if-changed=plugin/*");
    println!("cargo::rerun-if-env-changed=FORCE_BUILD_PLUGIN");
}
