use crate::util::crypt::generate_new_key;
use anyhow::Context;
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::warn;

fn get_secret_service() -> Option<&'static SecretService<'static>> {
    static SERVICE: OnceLock<Option<SecretService>> = OnceLock::new();
    SERVICE
        .get_or_init(|| {
            SecretService::connect(EncryptionType::Dh).map_or_else(
                |e| {
                    warn!("Failed to connect to Secret Service: {e}");
                    None
                },
                Some,
            )
        })
        .as_ref()
}

pub fn get_hyprshell_key() -> anyhow::Result<Vec<u8>> {
    let service =
        get_secret_service().ok_or_else(|| anyhow::anyhow!("Secret Service not available"))?;
    let collection = service
        .get_default_collection()
        .context("Failed to get default collection")?;
    let items =
        collection.search_items(HashMap::from([("application", "Hyprshell Clipboard Key")]))?;
    let key = if items.is_empty() {
        // instead generate a new key and insert it into the collection
        let key = generate_new_key().context("Failed to generate new encryption key")?;
        collection
            .create_item(
                "hyprshell",
                HashMap::from([("application", "Hyprshell Clipboard Key")]),
                &key,
                true,
                "application/octet-stream",
            )
            .context("Failed to create new secret service item for hyprshell key")?;
        key
    } else {
        items[0]
            .get_secret()
            .context("Failed to get hyprshell key from secret service")?
    };
    Ok(key)
}
