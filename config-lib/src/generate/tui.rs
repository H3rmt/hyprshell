use crate::generate::autocomplete::StringAutoCompleter;
use crate::generate::config::ConfigData;
use crate::generate::css::StyleData;
use crate::{Modifier, SearchEngine};
use anyhow::bail;
use inquire::formatter::MultiOptionFormatter;
use inquire::{Confirm, MultiSelect, Select, Text};

pub mod configurable_launcher_plugins {
    pub const APPLICATIONS: &str = "Open Applications";
    pub const SHELL: &str = "Run in shell";
    pub const TERMINAL: &str = "Run in terminal";
    pub const WEB_SEARCH: &str = "Web search";
    pub const CALC: &str = "Calculator";

    pub const ALL: &[&str] = &[
        APPLICATIONS,
        SHELL,
        TERMINAL,
        WEB_SEARCH,
        #[cfg(feature = "launcher_calc_plugin")]
        CALC,
    ];
}

#[allow(clippy::type_complexity)]
pub const WEB_SEARCH_ENGINES: &[(&str, fn() -> SearchEngine)] = &[
    ("Google", || SearchEngine {
        url: "https://www.google.com/search?q={}".into(),
        name: "Google".into(),
        key: 'g',
    }),
    ("DuckDuckGo", || SearchEngine {
        url: "https://duckduckgo.com/?q={}".into(),
        name: "DuckDuckGo".into(),
        key: 'd',
    }),
    ("Bing", || SearchEngine {
        url: "https://www.bing.com/search?q={}".into(),
        name: "Bing".into(),
        key: 'b',
    }),
    ("Wikipedia", || SearchEngine {
        url: "https://en.wikipedia.org/wiki/Special:Search?search={}".into(),
        name: "Wikipedia".into(),
        key: 'w',
    }),
    ("ChatGpt", || SearchEngine {
        url: "https://chatgpt.com/?q={}".into(),
        name: "ChatGpt".into(),
        key: 'c',
    }),
    ("YouTube", || SearchEngine {
        url: "https://www.youtube.com/results?search_query={}".into(),
        name: "YouTube".into(),
        key: 'y',
    }),
    ("Reddit", || SearchEngine {
        url: "https://www.reddit.com/search?q={}".into(),
        name: "Reddit".into(),
        key: 'r',
    }),
];

pub const DEFAULT_COLORS: [(&str, &str); 8] = [
    ("Red", "rgba(239, 9, 9, 0.9)"),
    ("Blue", "rgba(21, 162, 211, 0.9)"),
    ("Green", "rgba(9, 239, 9, 0.9)"),
    ("Yellow", "rgba(239, 239, 9, 0.9)"),
    ("Purple", "rgba(239, 9, 239, 0.9)"),
    ("Pink", "rgba(239, 9, 139, 0.9)"),
    ("Orange", "rgba(239, 139, 9, 0.9)"),
    ("White", "rgba(255, 255, 255, 0.9)"),
];

pub fn prompt_config() -> anyhow::Result<(ConfigData, StyleData)> {
    let open_overview = {
        let open_overview = Text::new("Key combination to open the overview and launcher (optional) [similar to gnome's overview]")
            .with_autocomplete(StringAutoCompleter::from(vec![
                "Super",
                "Super + Tab",
                "Ctrl",
                "Ctrl + Tab",
                "Alt",
                "Alt + Tab",
            ]))
            .with_help_message("Shows all apps in a miniature view, allows to switch using arrow keys or tab. Leave blank to disable]\n[Any valid modifier or modifier + key can be typed in]\n[↑↓ to move, tab to autocomplete, enter to submit")
            .prompt()?;
        if open_overview.trim().is_empty() {
            None
        } else {
            match get_mod_and_key(&open_overview) {
                Ok(data) => Some(data),
                Err(err) => {
                    eprintln!("Invalid Modifier: {err:?}");
                    None
                }
            }
        }
    };
    let launcher = if open_overview.is_some() {
        let default_terminal = Text::new("Default Terminal")
                .with_autocomplete(StringAutoCompleter::from(vec!["alacritty", "kitty", "wezterm", "gnome-terminal"]))
                .with_help_message("used to open terminal applications (htop), leave empty to chose from installed terminals]\n[Any valid binary name found in path can be typed in]\n[↑↓ to move, tab to autocomplete, enter to submit")
                .prompt()
                .map_or(None, |term| if term.trim().is_empty() { None } else { Some(term.into_boxed_str()) });

        let formatter: MultiOptionFormatter<'_, &str> = &|a| format!("{} plugins active", a.len());
        let plugins = MultiSelect::new(
            "Plugins for launcher",
            configurable_launcher_plugins::ALL.into(),
        )
        .with_all_selected_by_default()
        .with_formatter(formatter)
        .prompt()
        .map_or(vec![], |selected| {
            selected.into_iter().map(Box::from).collect()
        });

        let engines = if plugins.contains(&Box::from(configurable_launcher_plugins::WEB_SEARCH)) {
            let formatter: MultiOptionFormatter<'_, &str> =
                &|a| format!("{} engines active", a.len());
            MultiSelect::new(
                "SearchEngines for launcher",
                WEB_SEARCH_ENGINES.iter().map(|(name, _)| *name).collect(),
            )
            .with_formatter(formatter)
            .prompt()
            .map_or(vec![], |selected| {
                selected.into_iter().map(Box::from).collect()
            })
        } else {
            vec![]
        };
        Some((default_terminal, plugins, engines))
    } else {
        None
    };

    let switch = {
        let open_switch = Text::new("Modifier to open the switch mode (optional) [similar to windows alt + tab]")
            .with_autocomplete(StringAutoCompleter::from(vec!["Alt", "Ctrl", "Super"]))
            .with_help_message("Shows windows in a list sorted by recently accessed. Please use something different from the Overview modifier. Leave blank to disable]\n[Any valid modifier can be typed in]\n[↑↓ to move, tab to autocomplete, enter to submit")
            .prompt()?;
        if open_switch.trim().is_empty() {
            (None, false)
        } else {
            match get_mod(&open_switch) {
                Ok(r#mod) => {
                    let switch_workspaces =
                        Confirm::new("Switch between workspaces instead of windows in switch mode")
                            .with_default(false)
                            .prompt()?;
                    (Some(r#mod), switch_workspaces)
                }
                Err(err) => {
                    eprintln!("Invalid Modifier: {err:?}");
                    (None, false)
                }
            }
        }
    };

    let default_color = Select::new(
        "Default Focused Color",
        DEFAULT_COLORS.iter().map(|(name, _)| *name).collect(),
    )
    .prompt()
    .map(|color| color.trim().to_string().into_boxed_str())
    .unwrap_or_default();

    Ok((
        ConfigData {
            default_terminal: launcher.as_ref().and_then(|l| l.0.clone()),
            overview: open_overview,
            switch,
            launcher_plugins: launcher.as_ref().map(|l| l.1.clone()).unwrap_or_default(),
            launcher_engines: launcher.map(|l| l.2).unwrap_or_default(),
        },
        StyleData { default_color },
    ))
}

fn get_mod(modifier: &str) -> anyhow::Result<Modifier> {
    match &*modifier.trim().to_ascii_lowercase() {
        "super" => Ok(Modifier::Super),
        "ctrl" => Ok(Modifier::Ctrl),
        "alt" => Ok(Modifier::Alt),
        "shift" => Ok(Modifier::Shift),
        _ => bail!("Unknown modifier: {}", modifier),
    }
}

fn get_mod_and_key(modifier: &str) -> anyhow::Result<(Modifier, Box<str>)> {
    let split = modifier.split('+').collect::<Vec<_>>();
    let r#mod = get_mod(split.first().unwrap_or(&""))?;
    split.get(1).map_or_else(
        || Ok((r#mod, Box::from(r#mod.to_l_key()))),
        |key| Ok((r#mod, Box::from(key.trim()))),
    )
}
