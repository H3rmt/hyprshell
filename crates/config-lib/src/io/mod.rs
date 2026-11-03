mod convert;
mod structs;

pub mod load;
pub mod save;

pub use structs::*;

#[cfg(feature = "expose_raw_io")]
pub use raw::RawIOConfig;

pub(crate) mod raw {
    use crate::io::{Config, ConfigFile};

    pub trait RawIOConfig {
        fn get_raw_config_mut(&mut self) -> &mut Config;
        fn get_raw_config(&self) -> &Config;
    }

    impl RawIOConfig for ConfigFile {
        fn get_raw_config_mut(&mut self) -> &mut Config {
            return &mut self.config;
        }
        fn get_raw_config(&self) -> &Config {
            return &self.config;
        }
    }
}
