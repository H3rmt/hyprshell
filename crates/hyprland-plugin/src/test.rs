#[cfg(test)]
mod tests {
    use crate::{PluginConfig, SwitchBindConfig, build, configure, extract};
    use core_lib::binds::generate_transfer;
    use core_lib::transfer::{HoldMod, OpenSwitch, TransferType};
    use tracing::info;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn build_plugin() {
        let test_config = PluginConfig {
            switch_binds: vec![SwitchBindConfig {
                key: Box::from("Tab"),
                mod_mask: 1,
                hold_mask: 1,
                command: generate_transfer(&TransferType::OpenSwitch(OpenSwitch {
                    reverse: false,
                    profile: 0,
                    hold_mods: vec![HoldMod::Alt],
                }))
                .into_boxed_str(),
            }],
            xkb_key_overview_mod: Some(Box::from("Super")),
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
