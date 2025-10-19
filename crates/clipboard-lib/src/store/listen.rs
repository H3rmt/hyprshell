use crate::config::{Compression, Config, Encryption};
use crate::store::mime::{filer_mimes, get_preferred_mime};
use crate::store::save_image::compress_and_store_image;
use crate::store::save_map::compress_and_store_map;
use crate::store::save_text::store_text;
use core_lib::WarnWithDetails;
use std::collections::HashMap;
use std::thread;
use tracing::{debug, warn};
use wl_clipboard::paste::{CallbackData, Seat, get_all_contents_callback};

/// # Panics
pub fn test_clipboard() {
    debug!("Starting clipboard listener");
    let handle = get_all_contents_callback(Seat::Unspecified, Box::new(handle_values))
        .expect("Failed to start clipboard listener");
    let _ = handle.join();
    warn!("Clipboard listener stopped");
}

fn handle_values(val: CallbackData) -> bool {
    let mut config = Config {
        encryption: Encryption::default(),
        compression: Compression::default(),
    };
    config.encryption = Encryption::default();
    config.compression = Compression::default();
    #[cfg(feature = "compress_zstd")]
    {
        config.compression = Compression::Zstd(1);
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

    let (mut mimes, load) = match val {
        Ok(r) => r,
        Err(err) => {
            warn!("Failed to get clipboard contents: {err:?}");
            return false;
        }
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
        thread::spawn(|| {
            compress_and_store_image(pref_data).warn_details("Failed to store clipboard image")
        });
    } else {
        let pref_data = data.get(&pref_mime).expect("Preferred MIME type not found");
        let text = String::from_utf8_lossy(pref_data);
        store_text(&text, config).warn_details("Failed to store clipboard text");
    }
    thread::spawn(move || compress_and_store_map(data, config));
    false
}
