use crate::plugins::get_static_options_chars;
use core::str::FromStr;
use core_lib::config::Launcher;
use core_lib::generate_socat;
use core_lib::transfer::{CloseConfig, TransferType};

fn generate_launcher_return(config: CloseConfig) -> String {
    let config = TransferType::Close(config);
    let config_str = serde_json::to_string(&config).expect("Failed to serialize config");
    generate_socat(&config_str)
}

pub fn generate_keybinds(keyword_list: &mut Vec<(&str, String)>, launcher: &Launcher) {
    // add index keys launcher run
    for i in 1..=9 {
        keyword_list.push((
            "bind",
            format!(
                "ctrl, {}, exec, {}",
                i,
                generate_launcher_return(CloseConfig::LauncherPress(
                    char::from_str(&i.to_string()).expect("Failed to convert u32 to char")
                ))
            ),
        ));
    }

    for char in get_static_options_chars(&launcher.plugins) {
        keyword_list.push((
            "bind",
            format!(
                "ctrl, {}, exec, {}",
                char,
                generate_launcher_return(CloseConfig::LauncherPress(char))
            ),
        ));
    }
}
