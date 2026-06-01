use capture_proto::wayland_capture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = wayland_capture::capture()?;

    let pixels: Vec<u8> = result.pixels.chunks_exact(4)
        .flat_map(|b| [b[2], b[1], b[0], b[3]])
        .collect();

    image::RgbaImage::from_raw(result.width, result.height, pixels)
        .ok_or("Failed to create image from raw pixels")?
        .save("/tmp/capture-proto-test.png")?;

    println!("Image saved to /tmp/capture-proto-test.png");

    Ok(())
}
