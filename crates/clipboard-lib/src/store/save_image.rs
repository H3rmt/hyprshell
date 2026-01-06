use crate::store::util::create_storage_path;
use anyhow::Context;
use image::{ImageEncoder, ImageReader};
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;
use tracing::trace;

use crate::config::Config;
use crate::store::write::get_storage_writer;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::png::PngEncoder;

const IMAGE_HEIGHT: u32 = 150;

pub fn compress_and_store_image(
    pref_data: Vec<u8>,
    config: &Config,
    cache_dir: &Path,
) -> anyhow::Result<()> {
    let now = std::time::SystemTime::now();
    let img2 = ImageReader::new(Cursor::new(pref_data))
        .with_guessed_format()?
        .decode()?;
    trace!(
        "Loaded image in {:?}, Image size: {}x{}",
        now.elapsed()?,
        img2.width(),
        img2.height()
    );
    let now = std::time::SystemTime::now();

    #[allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
    let mut dst_image = Image::new(
        (img2.width() as f32 * (IMAGE_HEIGHT as f32 / img2.height() as f32)) as u32,
        IMAGE_HEIGHT,
        img2.pixel_type()
            .context("Failed to get pixel type for clipboard image")?,
    );

    let mut resizer = Resizer::new();
    resizer.resize(
        &img2,
        &mut dst_image,
        &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(config.image_conv_filter.into())),
    )?;
    trace!(
        "Resized image size: {}x{} in {:?}",
        dst_image.width(),
        dst_image.height(),
        now.elapsed()?
    );

    let mut cursor = Cursor::new(Vec::new());
    {
        let (mut write, _ext) = get_storage_writer(&mut cursor, config, false);
        PngEncoder::new(&mut write).write_image(
            dst_image.buffer(),
            dst_image.width(),
            dst_image.height(),
            img2.color().into(),
        )?;
    }

    let storage_path = create_storage_path(cache_dir, "images", "png")
        .context("Failed to get storage path for clipboard image")?;
    let mut file = File::create(&storage_path).context("Failed to create clipboard image file")?;
    file.write_all(&cursor.into_inner())
        .context("Failed to write clipboard data")?;
    trace!(
        "Wrote image to {:?} ({} bytes)",
        storage_path.display(),
        file.metadata().map(|m| m.len()).unwrap_or(0)
    );
    Ok(())
}
