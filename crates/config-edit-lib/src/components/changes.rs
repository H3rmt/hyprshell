use crate::flags_csv;
use crate::structs::{Config, Plugins, Switch};
use crate::util::key_combo_label;
use config_lib::actions::ToAction;
use relm4::adw::ActionRow;
use relm4::adw::gtk::SelectionMode;
use relm4::adw::prelude::*;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use std::collections::{BTreeSet, HashSet};
use tracing::trace;

#[derive(Debug)]
pub struct Changes {
    config: Config,
    prev_config: Config,
    list: gtk::ListBox,
    how_to_use: gtk::TextView,
}

#[derive(Debug)]
pub enum ChangesInput {
    SetConfig(Config),
    SetPrevConfig(Config),
}

#[derive(Debug)]
pub struct ChangesInit {
    pub config: Config,
}

#[derive(Debug)]
pub enum ChangesOutput {
    ChangesExist(bool),
}

#[allow(unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for Changes {
    type Init = ChangesInit;
    type Input = ChangesInput;
    type Output = ChangesOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 10,
            set_spacing: 15,
            #[name="list"]
            gtk::ListBox {
                set_show_separators: false,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                set_hexpand: true,
                set_selection_mode: SelectionMode::None,
                set_css_classes: &["items-list", "boxed-list"]
            },
            #[name="how_to_use"]
            gtk::TextView {
                set_editable: false,
                set_sensitive: false,
                set_align: gtk::Align::Fill,
                set_hexpand: true,
                set_vexpand: true,
                set_css_classes: &["changes-text"]
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        #[allow(unused_assignments)]
        let widgets = view_output!();
        let model = Self {
            config: init.config.clone(),
            prev_config: init.config,
            list: widgets.list.clone(),
            how_to_use: widgets.how_to_use.clone(),
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        trace!("changes::update: {message:?}");
        match message {
            ChangesInput::SetConfig(config) => {
                self.config = config;
            }
            ChangesInput::SetPrevConfig(config) => {
                self.prev_config = config;
            }
        }
        let changes = generate_items(&self.list, &self.config, &self.prev_config);

        let text = config_lib::explain(&(self.config.clone().into()), None, false);
        self.how_to_use.buffer().set_text(&text);

        sender
            .output_sender()
            .emit(ChangesOutput::ChangesExist(changes));
    }
}

#[allow(clippy::too_many_lines)]
pub fn generate_items(changes: &gtk::ListBox, config: &Config, prev_config: &Config) -> bool {
    while let Some(child) = changes.first_child() {
        changes.remove(&child);
    }

    match (prev_config.windows.enabled, config.windows.enabled) {
        (false, false) => {}
        (true, false) => {
            add_info(changes, "Disabled Windows");
        }
        (_, true) => {
            if !prev_config.windows.enabled {
                add_info(changes, "Enabled Windows");
            }

            #[allow(clippy::cast_sign_loss)]
            if (prev_config.windows.scale - config.windows.scale).abs() > 0.001 {
                add_info_subtitle(
                    changes,
                    "Changed windows scale",
                    format!("{} -> {}", prev_config.windows.scale, config.windows.scale),
                );
            }
            if prev_config.windows.items_per_row != config.windows.items_per_row {
                add_info_subtitle(
                    changes,
                    "Changed windows items per row",
                    format!(
                        "{} -> {}",
                        prev_config.windows.items_per_row, config.windows.items_per_row
                    ),
                );
            }
            match (
                prev_config.windows.overview.enabled,
                config.windows.overview.enabled,
            ) {
                (false, false) => {}
                (true, false) => {
                    add_info(changes, "Disabled Overview");
                }
                (_, true) => {
                    if !prev_config.windows.overview.enabled {
                        add_info(changes, "Enabled Overview");
                    }
                    if prev_config.windows.overview.modifier != config.windows.overview.modifier {
                        add_info_subtitle(
                            changes,
                            "Changed overview modifier",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.modifier,
                                config.windows.overview.modifier
                            ),
                        );
                    }
                    if prev_config.windows.overview.key != config.windows.overview.key {
                        add_info_subtitle(
                            changes,
                            "Changed overview key",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.key, config.windows.overview.key,
                            ),
                        );
                    }
                    if prev_config.windows.overview.same_class != config.windows.overview.same_class
                        || prev_config.windows.overview.current_monitor
                            != config.windows.overview.current_monitor
                        || prev_config.windows.overview.current_workspace
                            != config.windows.overview.current_workspace
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview filter by",
                            format!(
                                "{} -> {}",
                                flags_csv!(
                                    prev_config.windows.overview,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                                flags_csv!(
                                    config.windows.overview,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                            ),
                        );
                    }
                    if prev_config.windows.overview.exclude_workspaces
                        != config.windows.overview.exclude_workspaces
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview exclude workspaces",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.exclude_workspaces,
                                config.windows.overview.exclude_workspaces
                            ),
                        );
                    }
                    if prev_config.windows.overview.launcher.launch_modifier
                        != config.windows.overview.launcher.launch_modifier
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher launch modifier",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.launcher.launch_modifier,
                                config.windows.overview.launcher.launch_modifier
                            ),
                        );
                    }
                    if prev_config.windows.overview.launcher.max_items
                        != config.windows.overview.launcher.max_items
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher max items",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.launcher.max_items,
                                config.windows.overview.launcher.max_items
                            ),
                        );
                    }
                    if prev_config.windows.overview.launcher.show_when_empty
                        != config.windows.overview.launcher.show_when_empty
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher show when empty",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.launcher.show_when_empty,
                                config.windows.overview.launcher.show_when_empty
                            ),
                        );
                    }
                    if prev_config.windows.overview.launcher.width
                        != config.windows.overview.launcher.width
                    {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher width",
                            format!(
                                "{} -> {}",
                                prev_config.windows.overview.launcher.width,
                                config.windows.overview.launcher.width
                            ),
                        );
                    }
                    match (
                        &prev_config.windows.overview.launcher.default_terminal,
                        &config.windows.overview.launcher.default_terminal,
                    ) {
                        (None, None) => {}
                        (Some(_), None) => {
                            add_info(changes, "Disabled overview launcher default terminal");
                        }
                        (None, Some(dt)) => {
                            add_info_subtitle(
                                changes,
                                "Enabled overview launcher default terminal",
                                format!("{dt:?}"),
                            );
                        }
                        (Some(pdt), Some(cdt)) => {
                            if pdt != cdt {
                                add_info_subtitle(
                                    changes,
                                    "Changed overview launcher default terminal",
                                    format!("{pdt:?} -> {cdt:?}"),
                                );
                            }
                        }
                    }
                    add_plugin_changes(
                        changes,
                        &prev_config.windows.overview.launcher.plugins,
                        &config.windows.overview.launcher.plugins,
                    );
                }
            }
            let prev_enabled = prev_config
                .windows
                .switches
                .iter()
                .filter(|s| s.enabled)
                .count();
            let curr_enabled = config
                .windows
                .switches
                .iter()
                .filter(|s| s.enabled)
                .count();
            if prev_enabled != curr_enabled {
                add_info_subtitle(
                    changes,
                    "Changed switch profiles count",
                    format!("{prev_enabled} -> {curr_enabled}"),
                );
            }
            let prev_switches = &prev_config.windows.switches;
            let curr_switches = &config.windows.switches;
            let max_len = prev_switches.len().max(curr_switches.len());
            for idx in 0..max_len {
                add_switch_changes(changes, idx, prev_switches.get(idx), curr_switches.get(idx));
            }
        }
    }

    if changes.first_child().is_none() {
        add_info(changes, "No changes");
        false
    } else {
        true
    }
}

fn add_switch_changes(
    changes: &gtk::ListBox,
    index: usize,
    prev: Option<&Switch>,
    current: Option<&Switch>,
) {
    match (prev, current) {
        (None, None) => {}
        (None, Some(current)) => {
            add_info_subtitle(changes, "Added switch with", switch_summary(index, current));
        }
        (Some(prev), None) => {
            add_info_subtitle(
                changes,
                &format!("Removed switch#{index}"),
                switch_summary(index, prev),
            );
        }
        (Some(prev), Some(current)) => {
            if prev.enabled != current.enabled {
                if prev.enabled {
                    add_info(changes, &format!("Disabled switch#{index}"));
                } else {
                    add_info(changes, &format!("Enabled switch#{index}"));
                }
            }
            if prev.forward_binds != current.forward_binds {
                let (added, removed) =
                    diff_keybinds(&prev.forward_binds, &current.forward_binds);
                if !added.is_empty() {
                    add_info_subtitle(
                        changes,
                        &format!("Added switch#{index} forward keybindings"),
                        added.join(", "),
                    );
                }
                if !removed.is_empty() {
                    add_info_subtitle(
                        changes,
                        &format!("Removed switch#{index} forward keybindings"),
                        removed.join(", "),
                    );
                }
            }
            if prev.reverse_binds != current.reverse_binds {
                let (added, removed) =
                    diff_keybinds(&prev.reverse_binds, &current.reverse_binds);
                if !added.is_empty() {
                    add_info_subtitle(
                        changes,
                        &format!("Added switch#{index} reverse keybindings"),
                        added.join(", "),
                    );
                }
                if !removed.is_empty() {
                    add_info_subtitle(
                        changes,
                        &format!("Removed switch#{index} reverse keybindings"),
                        removed.join(", "),
                    );
                }
            }
            if prev.same_class != current.same_class
                || prev.current_monitor != current.current_monitor
                || prev.current_workspace != current.current_workspace
            {
                add_info_subtitle(
                    changes,
                    &format!("Changed switch#{index} filter by"),
                    format!(
                        "{} -> {}",
                        switch_filters_label(prev),
                        switch_filters_label(current)
                    ),
                );
            }
            if prev.switch_workspaces != current.switch_workspaces {
                add_info_subtitle(
                    changes,
                    &format!("Changed switch#{index} switch workspaces"),
                    format!("{} -> {}", prev.switch_workspaces, current.switch_workspaces),
                );
            }
            if prev.exclude_workspaces != current.exclude_workspaces {
                add_info_subtitle(
                    changes,
                    &format!("Changed switch#{index} exclude workspaces"),
                    format!(
                        "{} -> {}",
                        prev.exclude_workspaces, current.exclude_workspaces
                    ),
                );
            }
            if prev.kill_key != current.kill_key {
                add_info_subtitle(
                    changes,
                    &format!("Changed switch#{index} kill key"),
                    format!("{} -> {}", prev.kill_key, current.kill_key),
                );
            }
        }
    }
}

fn switch_summary(index: usize, switch: &Switch) -> String {
    format!(
        "switch#{index}: enabled={}, forward=[{}], reverse=[{}], filter_by={}, switch_workspaces={}, exclude_workspaces=\"{}\", kill_key={}",
        switch.enabled,
        key_combo_list_label(&switch.forward_binds),
        key_combo_list_label(&switch.reverse_binds),
        switch_filters_label(switch),
        switch.switch_workspaces,
        switch.exclude_workspaces,
        switch.kill_key
    )
}

fn switch_filters_label(switch: &Switch) -> String {
    let filters = flags_csv!(switch, same_class, current_monitor, current_workspace);
    if filters.is_empty() {
        "none".to_string()
    } else {
        filters
    }
}

fn key_combo_list_label(combos: &[config_lib::KeyCombo]) -> String {
    if combos.is_empty() {
        return "none".to_string();
    }
    combos
        .iter()
        .map(key_combo_details)
        .collect::<Vec<_>>()
        .join(", ")
}

fn key_combo_details(combo: &config_lib::KeyCombo) -> String {
    let base = key_combo_label(combo);
    match &combo.hold_mods {
        None => base,
        Some(mods) => format!("{base} (hold: {})", key_mods_label(mods)),
    }
}

fn key_mods_label(mods: &[config_lib::KeyMod]) -> String {
    if mods.is_empty() {
        return "none".to_string();
    }
    mods.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(" + ")
}

fn diff_keybinds(
    prev: &[config_lib::KeyCombo],
    current: &[config_lib::KeyCombo],
) -> (Vec<String>, Vec<String>) {
    let prev_set = key_combo_set(prev);
    let curr_set = key_combo_set(current);
    let added = curr_set.difference(&prev_set).cloned().collect::<Vec<_>>();
    let removed = prev_set.difference(&curr_set).cloned().collect::<Vec<_>>();
    (added, removed)
}

fn key_combo_set(combos: &[config_lib::KeyCombo]) -> BTreeSet<String> {
    combos.iter().map(key_combo_details).collect()
}

#[allow(clippy::too_many_lines)]
fn add_plugin_changes(changes: &gtk::ListBox, prev: &Plugins, current: &Plugins) {
    match (&prev.applications.enabled, &current.applications.enabled) {
        (false, false) => {}
        (true, false) => {
            add_info(changes, "Disabled Application Plugin");
        }
        (_, true) => {
            if !prev.applications.enabled {
                add_info(changes, "Enabled Application Plugin");
            }

            if prev.applications.show_execs != current.applications.show_execs {
                add_info_subtitle(
                    changes,
                    "Changed application plugin show execs",
                    format!(
                        "{} -> {}",
                        prev.applications.show_execs, current.applications.show_execs
                    ),
                );
            }

            if prev.applications.show_actions_submenu != current.applications.show_actions_submenu {
                add_info_subtitle(
                    changes,
                    "Changed application plugin show actions",
                    format!(
                        "{} -> {}",
                        prev.applications.show_actions_submenu,
                        current.applications.show_actions_submenu
                    ),
                );
            }

            if prev.applications.run_cache_weeks != current.applications.run_cache_weeks {
                add_info_subtitle(
                    changes,
                    "Changed application plugin run cache weeks",
                    format!(
                        "{} -> {}",
                        prev.applications.run_cache_weeks, current.applications.run_cache_weeks
                    ),
                );
            }
        }
    }

    match (&prev.terminal.enabled, &current.terminal.enabled) {
        (true, false) => {
            add_info(changes, "Disabled Terminal Plugin");
        }
        (false, true) => {
            add_info(changes, "Enabled Terminal Plugin");
        }
        _ => {}
    }

    match (&prev.shell.enabled, &current.shell.enabled) {
        (true, false) => {
            add_info(changes, "Disabled Shell Plugin");
        }
        (false, true) => {
            add_info(changes, "Enabled Shell Plugin");
        }
        _ => {}
    }

    match (&prev.calc.enabled, &current.calc.enabled) {
        (true, false) => {
            add_info(changes, "Disabled Calculator Plugin");
        }
        (false, true) => {
            add_info(changes, "Enabled Calculator Plugin");
        }
        _ => {}
    }

    match (&prev.path.enabled, &current.path.enabled) {
        (true, false) => {
            add_info(changes, "Disabled Path Plugin");
        }
        (false, true) => {
            add_info(changes, "Enabled Path Plugin");
        }
        _ => {}
    }

    match (&prev.websearch.enabled, &current.websearch.enabled) {
        (false, false) => {}
        (true, false) => {
            add_info(changes, "Disabled Websearch Plugin");
        }
        (_, true) => {
            if !prev.websearch.enabled {
                add_info(changes, "Enabled Websearch Plugin");
            }

            let prev_engines = &prev.websearch.engines;
            let cur_engines = &current.websearch.engines;

            let prev_keys: HashSet<_> = prev_engines.iter().map(|e| e.key).collect();
            let cur_keys: HashSet<_> = cur_engines.iter().map(|e| e.key).collect();

            for e in cur_engines.iter().filter(|e| !prev_keys.contains(&e.key)) {
                add_info_subtitle(
                    changes,
                    "Added Websearch engine",
                    format!("{} ({})", e.name, e.key),
                );
            }

            for e in prev_engines.iter().filter(|e| !cur_keys.contains(&e.key)) {
                add_info_subtitle(
                    changes,
                    "Removed Websearch engine",
                    format!("{} ({})", e.name, e.key),
                );
            }
        }
    }

    match (&prev.actions.enabled, &current.actions.enabled) {
        (false, false) => {}
        (true, false) => {
            add_info(changes, "Disabled Actions Plugin");
        }
        (_, true) => {
            if !prev.actions.enabled {
                add_info(changes, "Enabled Actions Plugin");
            }

            let prev_engines = &prev.actions.actions;
            let cur_engines = &current.actions.actions;

            let prev_keys: HashSet<_> = prev_engines
                .iter()
                .map(|e| e.clone().to_action().command)
                .collect();
            let cur_keys: HashSet<_> = cur_engines
                .iter()
                .map(|e| e.clone().to_action().command)
                .collect();

            for e in cur_engines
                .iter()
                .filter(|e| !prev_keys.contains(&(*e).clone().to_action().command))
            {
                let a = e.clone().to_action();
                add_info_subtitle(
                    changes,
                    "Added Action",
                    format!(
                        "{} ({})",
                        a.names.first().cloned().unwrap_or_default(),
                        a.details
                    ),
                );
            }

            for e in prev_engines
                .iter()
                .filter(|e| !cur_keys.contains(&(*e).clone().to_action().command))
            {
                let a = e.clone().to_action();
                add_info_subtitle(
                    changes,
                    "Removed Action",
                    format!(
                        "{} ({})",
                        a.names.first().cloned().unwrap_or_default(),
                        a.details
                    ),
                );
            }
        }
    }
}

fn add_info(changes: &gtk::ListBox, text: &str) {
    let label = ActionRow::builder().title(text).build();
    changes.append(&label);
}

fn add_info_subtitle(changes: &gtk::ListBox, text: &str, subtitle: String) {
    let label = ActionRow::builder().title(text).subtitle(subtitle).build();
    changes.append(&label);
}
