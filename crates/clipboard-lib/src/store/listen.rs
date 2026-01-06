use crate::config::{Compression, Config, ConvolutionFilterType, Encryption};
use crate::store::mime::{filer_mimes, get_preferred_mime};
use crate::store::save_image::compress_and_store_image;
use crate::store::save_map::compress_and_store_map;
use crate::store::save_text::store_text;
use core_lib::WarnWithDetails;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use tracing::{debug, warn};
use wl_clipboard::paste::{CallbackData, Seat, get_all_contents_callback};

/// # Panics
pub fn test_clipboard(_data_dir: PathBuf, cache_dir: PathBuf) {
    debug!("Starting clipboard listener");
    let cl_config = Arc::new(RwLock::new(conf()));

    let config_clone = cl_config.clone();
    let _ = cl_config.type_id();

    let handle_values = move |val: CallbackData| -> bool {
        let (mut mimes, load) = match val {
            Ok(r) => r,
            Err(err) => {
                warn!("Failed to get clipboard contents: {err:?}");
                return false;
            }
        };
        let config = {
            let Ok(config) = config_clone.read() else {
                return true; // lock poisoned, aboard clipboard listener
            };
            config.clone()
        };

        filer_mimes(&mut mimes);

        let Some(pref_mime) = get_preferred_mime(&mimes) else {
            warn!("No preferred MIME type found, available: {mimes:?}");
            return false;
        };

        // load data for all mime types
        let mut data = HashMap::new();
        for mime in &mimes {
            data.insert(
                mime.clone(),
                load(mime.clone()).expect("mime type despawned while loading clipboard data"),
            );
        }

        if pref_mime.starts_with("image/") {
            let pref_data = data
                .get(&pref_mime)
                .expect("Preferred MIME type not found")
                .clone();
            let cache_dir_clone = cache_dir.clone();
            let config_clone = config.clone();
            thread::spawn(move || {
                compress_and_store_image(pref_data, &config_clone, &cache_dir_clone)
                    .warn_details("Failed to store clipboard image")
            });
        } else {
            let pref_data = data.get(&pref_mime).expect("Preferred MIME type not found");
            let text = String::from_utf8_lossy(pref_data);
            store_text(&text, &config, &cache_dir).warn_details("Failed to store clipboard text");
        }
        let cache_dir_clone = cache_dir.clone();
        thread::spawn(move || compress_and_store_map(data, &config, &cache_dir_clone));
        false
    };

    let _ = get_all_contents_callback(Seat::Unspecified, Box::new(handle_values))
        .expect("Failed to start clipboard listener")
        .join();
    warn!("Clipboard listener stopped");
}

fn conf() -> Config {
    let mut config = Config {
        encryption: Encryption::default(),
        compression: Compression::default(),
        image_conv_filter: ConvolutionFilterType::Lanczos3,
    };
    config.encryption = Encryption::default();
    config.compression = Compression::default();
    #[cfg(feature = "compress_brotli")]
    {
        config.compression = Compression::Brotli(6);
    }
    #[cfg(feature = "compress_zstd")]
    {
        config.compression = Compression::Zstd(16);
    }
    #[cfg(feature = "encrypt_aes_gcm")]
    {
        config.encryption = Encryption::AesGcm;
    }
    #[cfg(feature = "compress_lz4")]
    {
        config.compression = Compression::Lz4;
    }
    #[cfg(feature = "encrypt_chacha20poly1305")]
    {
        config.encryption = Encryption::ChaCha20Poly1305;
    }
    config
}
