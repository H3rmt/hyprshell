use anyhow::bail;

#[allow(clippy::wildcard_imports)]
use crate::workarrounds::*;

use crate::workarrounds::io::structs as io;

impl TryFrom<io::Workarrounds> for Workarrounds {
    type Error = anyhow::Error;

    fn try_from(value: io::Workarrounds) -> Result<Self, Self::Error> {
        Ok(Self {
            class_to_icon: value
                .class_to_icon
                .into_iter()
                .map(ClassToIcon::try_from)
                .flatten()
                .collect(),
        })
    }
}

impl TryFrom<io::ClassToIcon> for ClassToIcon {
    type Error = anyhow::Error;

    fn try_from(value: io::ClassToIcon) -> Result<Self, Self::Error> {
        if value.enabled {
            bail!("Not enabled")
        }
        Ok(Self {
            class: value.class,
            icon: value.icon,
        })
    }
}
