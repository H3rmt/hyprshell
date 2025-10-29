use crate::Config;
use std::fmt::Write;
use std::path::Path;

const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

#[must_use]
pub fn explain(
    config: &Config,
    config_path: &Path,
    enable_color: bool,
    enable_header: bool,
) -> String {
    let (bold, italic, blue, green, reset) = if enable_color {
        (BOLD, ITALIC, BLUE, GREEN, RESET)
    } else {
        ("", "", "", "", "")
    };

    let config_path_display = config_path.display();
    let mut builder = if enable_header {
        format!(
            "{bold}{green}Config is valid{reset} ({config_path_display})\n{bold}Explanation{reset} ({blue}blue{reset} are keys, {bold}{blue}bold blue{reset} keys can be configured in config):{reset}\n",
        )
    } else {
        String::new()
    };

    if let Some(windows) = &config.windows {
        if let Some(overview) = &windows.overview {
            let _ = builder.write_str(&format!(
                "Use {bold}{blue}{}{reset} + {bold}{blue}{}{reset} to open the Overview. Use {blue}tab{reset} and {blue}grave{reset} / {blue}shift{reset} + {blue}tab{reset} to select a different window, press {blue}return{reset} to switch\n\
                You can also use the {blue}arrow keys{reset} or {bold}{blue}{}{reset} + vim keys to navigate the workspaces. Use {blue}Esc{reset} to close the overview.\n",
                overview.modifier,
                overview.key,
                overview.launcher.launch_modifier
            ));
            let _ = builder.write_str(&format!(
                "After opening the Overview the {bold}Launcher{reset} is available:\n"
            ));
            let mut any_plugin = false;
            if let Some(_applications) = overview.launcher.plugins.applications.as_ref() {
                any_plugin = true;
                let _ = builder.write_str(&format!("\t- Start typing to search through applications (sorted by how often they were opened). Press {blue}return{reset} to launch the first app, use {blue}Ctrl{reset} + {blue}1{reset}/{blue}2{reset}/{blue}3{reset}/... to open the second, third, etc.\n"));
            }
            if overview.launcher.plugins.terminal.is_some() {
                any_plugin = true;
                let _ = builder.write_str(&format!(
                    "\t- Press {blue}Ctrl{reset} + {blue}t{reset} to run the typed command in a terminal.\n"
                ));
            }
            if overview.launcher.plugins.shell.is_some() {
                any_plugin = true;
                let _ = builder.write_str(&format!(
                    "\t- Press {blue}Ctrl{reset} + {blue}r{reset} to run the typed command in the background.\n",
                ));
            }
            if let Some(engines) = &overview.launcher.plugins.websearch {
                any_plugin = true;
                let _ =    builder.write_str(&format!("\t- Press {blue}Ctrl{reset} + {bold}{blue}<key>{reset} to search the typed text in any of the configured SearchEngines: {}.\n",
                                                      engines.engines.iter().map(|e| e.name.to_string()).collect::<Vec<_>>().join(", ")));
            }
            if overview.launcher.plugins.calc.is_some() {
                any_plugin = true;
                let _ =   builder.write_str(
                    "\t- Typing a mathematical expression will calculate it and display the result in the launcher.\n",
                );
            }
            if overview.launcher.plugins.path.is_some() {
                any_plugin = true;
                let _ = builder.write_str(
                    "\t- Paths (starting with ~ or /) can be open in default file-manager.\n",
                );
            }
            if overview.launcher.plugins.actions.is_some() {
                any_plugin = true;
                let _ = builder.write_str(
                    "\t- Type Reboot/Shutdown/etc. to run corresponding commands. Type `actions` to see all available ones.\n",
                );
            }
            if !any_plugin {
                let _ = builder.write_str(&format!(
                    "{italic}\t<No plugins enabled in launcher>{reset}\n"
                ));
            }
        } else {
            let _ = builder.write_str(&format!("{italic}<Overview disabled>{reset}\n"));
        }
        builder.push('\n');

        if let Some(switch) = &windows.switch {
            let _ = builder.write_str(&format!(
                "Press {bold}{blue}{}{reset} + {blue}tab{reset} and hold {bold}{blue}{}{reset} to view recently used applications. Press {blue}tab{reset} and {blue}grave{reset} / {blue}shift{reset} + {blue}tab{reset} to select a different window, release {bold}{blue}{}{reset} to close the window.\n",
                switch.modifier,
                switch.modifier,
                switch.modifier,
            ));
        } else {
            let _ = builder.write_str(&format!("{italic}<Switch mode disabled>{reset}\n"));
        }
    } else {
        let _ = builder.write_str(&format!("{italic}<Windows disabled>{reset}\n"));
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::*;
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        Config {
            windows: Some(Windows {
                overview: Some(Overview::default()),
                switch: Some(Switch::default()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_explain_with_overview() {
        const CONFIG: &str = r"Config is valid (/test/config.ron)
Explanation (blue are keys, bold blue keys can be configured in config):
Use Super + super_l to open the Overview. Use tab and grave / shift + tab to select a different window, press return to switch
You can also use the arrow keys or Ctrl + vim keys to navigate the workspaces. Use Esc to close the overview.
After opening the Overview the Launcher is available:
	- Start typing to search through applications (sorted by how often they were opened). Press return to launch the first app, use Ctrl + 1/2/3/... to open the second, third, etc.
	- Press Ctrl + t to run the typed command in a terminal.
	- Press Ctrl + <key> to search the typed text in any of the configured SearchEngines: Google, Wikipedia.
	- Typing a mathematical expression will calculate it and display the result in the launcher.
	- Paths (starting with ~ or /) can be open in default file-manager.
	- Type Reboot/Shutdown/etc. to run corresponding commands. Type `actions` to see all available ones.

Press Alt + tab and hold Alt to view recently used applications. Press tab and grave / shift + tab to select a different window, release Alt to close the window.
";
        let config = create_test_config();
        let path = PathBuf::from("/test/config.ron");
        let result = explain(&config, &path, false, true);
        assert_eq!(result, CONFIG);
    }

    #[test]
    fn test_explain_without_overview() {
        const CONFIG: &str = r"Config is valid (/test/config.ron)
Explanation (blue are keys, bold blue keys can be configured in config):
<Overview disabled>

Press Alt + tab and hold Alt to view recently used applications. Press tab and grave / shift + tab to select a different window, release Alt to close the window.
";
        let mut config = create_test_config();
        config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview = None;
        let path = PathBuf::from("/test/config.ron");
        let result = explain(&config, &path, false, true);
        assert_eq!(result, CONFIG);
    }

    #[test]
    fn test_explain_without_switch() {
        const CONFIG: &str = r"Config is valid (/test/config.ron)
Explanation (blue are keys, bold blue keys can be configured in config):
Use Super + super_l to open the Overview. Use tab and grave / shift + tab to select a different window, press return to switch
You can also use the arrow keys or Ctrl + vim keys to navigate the workspaces. Use Esc to close the overview.
After opening the Overview the Launcher is available:
	- Start typing to search through applications (sorted by how often they were opened). Press return to launch the first app, use Ctrl + 1/2/3/... to open the second, third, etc.
	- Press Ctrl + t to run the typed command in a terminal.
	- Press Ctrl + <key> to search the typed text in any of the configured SearchEngines: Google, Wikipedia.
	- Typing a mathematical expression will calculate it and display the result in the launcher.
	- Paths (starting with ~ or /) can be open in default file-manager.
	- Type Reboot/Shutdown/etc. to run corresponding commands. Type `actions` to see all available ones.

<Switch mode disabled>
";
        let mut config = create_test_config();
        config
            .windows
            .as_mut()
            .expect("config option missing")
            .switch = None;
        let path = PathBuf::from("/test/config.ron");
        let result = explain(&config, &path, false, true);
        assert_eq!(result, CONFIG);
    }

    #[test]
    fn test_explain_without_plugins() {
        const CONFIG: &str = r"Use Super + super_l to open the Overview. Use tab and grave / shift + tab to select a different window, press return to switch
You can also use the arrow keys or Ctrl + vim keys to navigate the workspaces. Use Esc to close the overview.
After opening the Overview the Launcher is available:
	<No plugins enabled in launcher>

Press Alt + tab and hold Alt to view recently used applications. Press tab and grave / shift + tab to select a different window, release Alt to close the window.
";
        let mut config = create_test_config();
        config
            .windows
            .as_mut()
            .expect("config option missing")
            .overview
            .as_mut()
            .expect("config option missing")
            .launcher
            .plugins = Plugins {
            applications: None,
            terminal: None,
            shell: None,
            websearch: None,
            calc: None,
            path: None,
            actions: None,
        };
        let path = PathBuf::from("/test/config.ron");
        let result = explain(&config, &path, false, false);
        assert_eq!(result, CONFIG);
    }
}
