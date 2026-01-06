#[cfg(test)]
mod tests {
    use crate::{PluginConfig, build, configure, extract};
    use tracing::info;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn build_plugin() {
        let test_config = PluginConfig {
            xkb_key_switch_mod: Some(Box::from("XKB_KEY_Alt")),
            xkb_key_switch_key: Some(Box::from("tab")),
            xkb_key_overview_mod: Some(Box::from("XKB_KEY_Super")),
            xkb_key_overview_key: Some(Box::from("tab")),
        };

        info!("extracting plugin from zip");
        let dir = extract::extract_plugin().expect("Failed to extract plugin");
        info!("configuring defs file");
        configure::configure(&dir, &test_config).expect("unable to configure defs file");
        info!("building plugin");
        build::build(&dir).expect("Failed to build plugin");
    }
}
