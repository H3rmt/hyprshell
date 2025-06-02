use anyhow::Context;
use core_lib::config::Launcher;
use core_lib::config::{Config, FilterBy, Overview, Reverse, Switch};
use core_lib::transfer::{
    CloseConfig, Direction, OpenOverview, OpenSwitch, SwitchConfig, TransferType,
};
use core_lib::{generate_socat, generate_socat_and_activate_submap, LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE};
use launcher_lib::generate_keybinds;
use tracing::{span, Level};

pub fn create_binds_and_submaps<'a>(config: &Config) -> anyhow::Result<Vec<(&'a str, String)>> {
    let _span = span!(Level::DEBUG, "create_binds_and_submaps").entered();
    let mut keyword_list = Vec::<(&str, String)>::new();

    if config.layerrules {
        keyword_list.push(("layerrule", format!("noanim, {LAUNCHER_NAMESPACE}")));
        keyword_list.push(("layerrule", format!("noanim, {OVERVIEW_NAMESPACE}")));
        keyword_list.push(("layerrule", format!("dimaround, {OVERVIEW_NAMESPACE}")));
        keyword_list.push(("layerrule", format!("ignorezero, {OVERVIEW_NAMESPACE}")));
        keyword_list.push(("layerrule", format!("ignorezero, {LAUNCHER_NAMESPACE}")));
    }

    // TODO allow configure with config
    #[cfg(debug_assertions)]
    keyword_list.push((
        "bind",
        "ctrl, k, exec, pkill hyprshell; hyprctl dispatch submap reset".to_string(),
    ));

    if let Some(windows) = &config.windows {
        if let Some(overview) = &windows.overview {
            let workspaces_per_row = windows.workspaces_per_row;
            let submap_name = "hyprshell-overview";
            generate_overview(
                &mut keyword_list,
                overview,
                submap_name,
                workspaces_per_row,
                &config.launcher,
            )
            .context("Failed to generate overview")?;
        }
        if let Some(switch) = &windows.switch {
            let workspaces_per_row = windows.workspaces_per_row;
            let submap_name = "hyprshell-switch";
            generate_switch(&mut keyword_list, switch, submap_name, workspaces_per_row)
                .context("Failed to generate overview")?;
        }
    }

    Ok(keyword_list)
}

fn generate_exit() -> anyhow::Result<String> {
    let config = TransferType::Exit;
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str))
}

fn generate_return(config: CloseConfig) -> anyhow::Result<String> {
    let config = TransferType::Close(config);
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str))
}

fn generate_switch_press(direction: Direction, workspace: bool) -> anyhow::Result<String> {
    let config = TransferType::Switch(SwitchConfig {
        direction,
        workspace,
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str))
}

fn generate_overview_open(
    submap_name: &str,
    overview: &Overview,
    workspaces_per_row: u8,
) -> anyhow::Result<String> {
    let config = TransferType::OpenOverview(OpenOverview {
        hide_filtered: overview.other.hide_filtered,
        filter_current_workspace: overview
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::CurrentWorkspace),
        filter_current_monitor: overview
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::CurrentMonitor),
        filter_same_class: overview
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::SameClass),
        workspaces_per_row,
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat_and_activate_submap(&config_str, submap_name))
}

fn generate_overview(
    keyword_list: &mut Vec<(&str, String)>,
    overview: &Overview,
    submap_name: &str,
    workspaces_per_row: u8,
    launcher: &Option<Launcher>,
) -> anyhow::Result<()> {
    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {}",
            overview.open.modifier,
            overview.open.key.to_key(),
            generate_overview_open(submap_name, overview, workspaces_per_row)?,
        ),
    ));

    keyword_list.push(("submap", submap_name.to_string()));
    keyword_list.push(("bind", format!(", escape, exec, {}", generate_exit()?)));
    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {}",
            overview.open.modifier,
            overview.open.key.to_key(),
            generate_exit()?
        ),
    ));
    keyword_list.push((
        "bind",
        format!(", return, exec, {}", generate_return(CloseConfig::None)?),
    ));

    if let Some(launcher) = launcher {
        generate_keybinds(keyword_list, launcher);
    }

    keyword_list.push((
        "binden",
        format!(
            ", right, exec, {}",
            generate_switch_press(Direction::Right, true)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            ", left, exec, {}",
            generate_switch_press(Direction::Left, true)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            ", down, exec, {}",
            generate_switch_press(Direction::Down, true)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            ", up, exec, {}",
            generate_switch_press(Direction::Up, true)?
        ),
    ));

    keyword_list.push((
        "binde",
        format!(
            ", {}, exec, {}",
            overview.navigate.forward,
            generate_switch_press(Direction::Right, false)?
        ),
    ));
    match &overview.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "binde",
            format!(
                ", {}, exec, {}",
                key,
                generate_switch_press(Direction::Left, false)?
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "binde",
            format!(
                "{}, {}, exec, {}",
                modk,
                overview.navigate.forward,
                generate_switch_press(Direction::Left, false)?
            ),
        )),
    }

    keyword_list.push((
        "bind",
        "ctrl, k, exec, pkill hyprshell; hyprctl dispatch submap reset".to_string(),
    ));
    keyword_list.push(("submap", "reset".to_string()));
    Ok(())
}

fn generate_switch_open(
    submap_name: &str,
    switch: &Switch,
    workspaces_per_row: u8,
    direction: Direction,
) -> anyhow::Result<String> {
    let config = TransferType::OpenSwitch(OpenSwitch {
        hide_filtered: switch.other.hide_filtered,
        filter_current_workspace: switch
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::CurrentWorkspace),
        filter_current_monitor: switch
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::CurrentMonitor),
        filter_same_class: switch
            .other
            .filter_by
            .iter()
            .any(|f| f == &FilterBy::SameClass),
        workspaces_per_row,
        direction,
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat_and_activate_submap(&config_str, submap_name))
}

fn generate_switch(
    keyword_list: &mut Vec<(&str, String)>,
    switch: &Switch,
    submap_name: &str,
    workspaces_per_row: u8,
) -> anyhow::Result<()> {
    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {}",
            switch.open.modifier,
            switch.navigate.forward,
            generate_switch_open(submap_name, switch, workspaces_per_row, Direction::Right,)?,
        ),
    ));
    match &switch.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {}",
                switch.open.modifier,
                key,
                generate_switch_open(submap_name, switch, workspaces_per_row, Direction::Left,)?,
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{} {}, {}, exec, {}",
                switch.open.modifier,
                modk,
                switch.navigate.forward,
                generate_switch_open(submap_name, switch, workspaces_per_row, Direction::Left,)?,
            ),
        )),
    }

    keyword_list.push(("submap", submap_name.to_string()));
    keyword_list.push(("bind", format!(", escape, exec, {}", generate_exit()?)));

    // register left and right release
    for mods in switch.open.modifier.mod_to_keys() {
        keyword_list.push((
            "bindrt",
            format!(
                "{}, {}, exec, {}",
                switch.open.modifier,
                mods,
                generate_return(CloseConfig::None)?
            ),
        ));
    }
    // second keybinding to close of mod + reverse mod is released
    if let Reverse::Mod(modk) = &switch.navigate.reverse {
        for mods in switch.open.modifier.mod_to_keys() {
            keyword_list.push((
                "bindrt",
                format!(
                    "{} {}, {}, exec, {}",
                    switch.open.modifier,
                    modk,
                    mods,
                    generate_return(CloseConfig::None)?,
                ),
            ));
        }
    }

    keyword_list.push((
        "binden",
        format!(
            "{}, right, exec, {}",
            switch.open.modifier,
            generate_switch_press(Direction::Right, false)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            "{}, left, exec, {}",
            switch.open.modifier,
            generate_switch_press(Direction::Left, false)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            "{}, down, exec, {}",
            switch.open.modifier,
            generate_switch_press(Direction::Down, false)?
        ),
    ));
    keyword_list.push((
        "binden",
        format!(
            "{}, up, exec, {}",
            switch.open.modifier,
            generate_switch_press(Direction::Up, false)?
        ),
    ));

    keyword_list.push((
        "binde",
        format!(
            "{}, {}, exec, {}",
            switch.open.modifier,
            switch.navigate.forward,
            generate_switch_press(Direction::Right, false)?
        ),
    ));
    match &switch.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "binde",
            format!(
                "{}, {}, exec, {}",
                switch.open.modifier,
                key,
                generate_switch_press(Direction::Left, false)?
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "binde",
            format!(
                "{} {}, {}, exec, {}",
                switch.open.modifier,
                modk,
                switch.navigate.forward,
                generate_switch_press(Direction::Left, false)?
            ),
        )),
    }

    keyword_list.push((
        "bind",
        "ctrl, k, exec, pkill hyprshell; hyprctl dispatch submap reset".to_string(),
    ));

    keyword_list.push(("submap", "reset".to_string()));
    Ok(())
}
