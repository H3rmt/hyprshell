#[cfg(test)]
mod tests {
    use crate::{PluginConfig, build, build_plugin, configure, extract};
    use std::path::Path;
    use tracing::info;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn build_plugin() {
        info!("extracting plugin from zip");
        let tmp_dir = Path::new("/tmp/test-build-hyprland");
        std::fs::create_dir_all(tmp_dir).expect("Failed to create temp dir");
        test_plugin_load(tmp_dir);
        std::fs::remove_dir_all(tmp_dir).expect("Failed to remove temp dir");
    }

    fn test_plugin_load(dir: &Path) {
        let test_config = PluginConfig {
            xkb_key_switch_mod: Some(Box::from("XKB_KEY_Alt")),
            xkb_key_switch_key: Some(Box::from("tab")),
            xkb_key_overview_mod: Some(Box::from("XKB_KEY_Super")),
            xkb_key_overview_key: Some(Box::from("tab")),
        };

        extract::extract_plugin(dir).expect("Failed to extract plugin");
        info!("configuring defs file");
        configure::configure(&dir, &test_config).expect("unable to configure defs file");
        info!("building plugin");
        build::build_plugin(&dir).expect("Failed to build plugin");
    }
}
