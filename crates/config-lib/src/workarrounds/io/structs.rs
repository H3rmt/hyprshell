use anyhow::Context;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use smart_default::SmartDefault;
use std::path::Path;

use crate::edit_sync::{sync_array, sync_document};

#[derive(Debug)]
pub struct WorkarroundsFile {
    backing_document: toml_edit::DocumentMut,
    config: Workarrounds,
}

impl TryFrom<toml_edit::DocumentMut> for WorkarroundsFile {
    type Error = anyhow::Error;
    fn try_from(doc: toml_edit::DocumentMut) -> Result<Self, Self::Error> {
        let config = Workarrounds::deserialize(doc.clone().into_deserializer())
            .context("Failed to deserialize Workarrounds from TOML document");
        Ok(Self {
            backing_document: doc,
            config: config?,
        })
    }
}

impl WorkarroundsFile {
    pub fn new_without_file(config: Workarrounds) -> Self {
        Self {
            backing_document: toml_edit::DocumentMut::new(),
            config,
        }
    }

    pub fn to_string_pretty(&mut self) -> anyhow::Result<String> {
        self.sync_config_to_document()
            .context("failed to sync config in document")?;
        Ok(self.backing_document.to_string())
    }

    fn sync_config_to_document(&mut self) -> anyhow::Result<()> {
        let generated =
            toml_edit::ser::to_document(&self.config).expect("config should always serialize");

        sync_document(&mut self.backing_document, &generated);
        sync_array(
            self.backing_document
                .get_mut("class_to_icon")
                .expect("class_to_icon exists in struct"),
            generated
                .get("class_to_icon")
                .expect("class_to_icon exists in struct"),
            |i| i.get("class").map(|i| i.to_string()),
        )
        .context("failed to sync config into original toml config")?;
        Ok(())
    }
}

impl TryInto<crate::workarrounds::Workarrounds> for WorkarroundsFile {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<crate::workarrounds::Workarrounds, Self::Error> {
        self.config.try_into()
    }
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "ci_no_default_config_values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Workarrounds {
    #[default(crate::CURRENT_WORKARROUND_VERSION)]
    pub version: u64,
    #[default(Vec::new())]
    pub class_to_icon: Vec<ClassToIcon>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassToIcon {
    pub enabled: bool,
    pub class: Box<str>,
    pub icon: Box<Path>,
}
