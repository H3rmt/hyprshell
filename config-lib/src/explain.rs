use crate::Config;
use std::fmt::Write;
use std::path::Path;

const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

pub fn explain(config: &Config, config_path: &Path) -> String {
    let config_path_display = config_path.display();
    let mut builder = format!(
        "{BOLD}{GREEN}Config is valid{RESET} ({config_path_display})\n{BOLD}Explanation{RESET} ({BLUE}blue{RESET} are keys, {BOLD}{BLUE}bold blue{RESET} keys can be configured in config):{RESET}\n",
    );

    if let Some(windows) = &config.windows {
        if let Some(overview) = &windows.overview {
            let _ = builder.write_str(&format!(
                "Use {BOLD}{BLUE}{}{RESET} + {BOLD}{BLUE}{}{RESET} to open the Overview. Use {BLUE}tab{RESET} and {BLUE}grave{RESET} / {BLUE}shift{RESET} + {BLUE}tab{RESET} to select a different window, press {BLUE}return{RESET} to switch\n\
                You can also use the {BLUE}arrow keys{RESET} to navigate the workspaces. Use {BLUE}Esc{RESET} to close the overview.\n",
                overview.modifier,
                overview.key,
            ));
            let _ = builder.write_str(&format!(
                "After opening the Overview the {BOLD}Launcher{RESET} is available:\n"
            ));
            if let Some(_applications) = overview.launcher.plugins.applications.as_ref() {
                let _ = builder.write_str(&format!("\t- Start typing to search through applications (sorted by how often they were opened). Press {BLUE}return{RESET} to launch the first app, use {BLUE}Ctrl{RESET} + {BLUE}1{RESET}/{BLUE}2{RESET}/{BLUE}3{RESET}/... to open the second, third, etc.\n"));
            }
            if overview.launcher.plugins.terminal.is_some() {
                let _ = builder.write_str(&format!(
                    "\t- Press {BLUE}Ctrl{RESET} + {BLUE}t{RESET} to run the typed command in a terminal.\n"
                ));
            }
            if overview.launcher.plugins.shell.is_some() {
                let _ = builder.write_str(&format!(
                    "\t- Press {BLUE}Ctrl{RESET} + {BLUE}r{RESET} to run the typed command in the background.\n",
                ));
            }
            if let Some(engines) = &overview.launcher.plugins.websearch {
                let _ =    builder.write_str(&format!("\t- Press {BLUE}Ctrl{RESET} + {BOLD}{BLUE}<key>{RESET} to search the typed text in any of the configured SearchEngines: {}.\n",
                                          engines.engines.iter().map(|e| e.name.to_string()).collect::<Vec<_>>().join(", ")));
            }
            if overview.launcher.plugins.calc.is_some() {
                let _ =   builder.write_str(
                    "\t- Typing a mathematical expression will calculate it and display the result in the launcher.\n",
                );
            }
            if overview.launcher.plugins.path.is_some() {
                let _ = builder.write_str(
                    "\t- Paths (starting with ~ or /) can be open in default file-manager.\n",
                );
            }
        } else {
            let _ = builder.write_str(&format!("{ITALIC}<Overview move disabled>{RESET}\n"));
        }
    }

    builder.push('\n');

    if let Some(windows) = &config.windows {
        if let Some(switch) = &windows.switch {
            let _ = builder.write_str(&format!(
                "Press {BOLD}{BLUE}{}{RESET} + {BLUE}tab{RESET} and hold {BOLD}{BLUE}{}{RESET} to view recently used applications. Press {BLUE}tab{RESET} and {BLUE}grave{RESET} / {BLUE}shift{RESET} + {BLUE}tab{RESET} to select a different window, release {BOLD}{BLUE}{}{RESET} to close the window.\n",
                switch.modifier,
                switch.modifier,
                switch.modifier,
            ));
        } else {
            let _ = builder.write_str(&format!("{ITALIC}<Switch mode disabled>{RESET}\n"));
        }
    }

    builder
}
