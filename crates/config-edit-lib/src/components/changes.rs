use crate::flags_csv;
use crate::structs::{Config, Plugins};
use relm4::adw::ActionRow;
use relm4::adw::gtk::SelectionMode;
use relm4::adw::prelude::*;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use std::collections::HashSet;
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
            match (
                &prev_config.windows.switch.enabled,
                &config.windows.switch.enabled,
            ) {
                (false, false) => {}
                (true, false) => {
                    add_info(changes, "Disabled Switch view");
                }
                (_, true) => {
                    if !prev_config.windows.switch.enabled {
                        add_info(changes, "Enabled Switch view");
                    }

                    if prev_config.windows.switch.key != config.windows.switch.key {
                        add_info_subtitle(
                            changes,
                            "Changed switch key",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch.key, config.windows.switch.key
                            ),
                        );
                    }
                    if prev_config.windows.switch.modifier != config.windows.switch.modifier {
                        add_info_subtitle(
                            changes,
                            "Changed switch modifier",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch.modifier, config.windows.switch.modifier
                            ),
                        );
                    }
                    if prev_config.windows.switch.same_class != config.windows.switch.same_class
                        || prev_config.windows.switch.current_monitor
                            != config.windows.switch.current_monitor
                        || prev_config.windows.switch.current_workspace
                            != config.windows.switch.current_workspace
                    {
                        add_info_subtitle(
                            changes,
                            "Changed switch filter by",
                            format!(
                                "{} -> {}",
                                flags_csv!(
                                    prev_config.windows.switch,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                                flags_csv!(
                                    config.windows.switch,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                            ),
                        );
                    }
                    if prev_config.windows.switch.switch_workspaces
                        != config.windows.switch.switch_workspaces
                    {
                        add_info_subtitle(
                            changes,
                            "Changed switch switch workspaces",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch.switch_workspaces,
                                config.windows.switch.switch_workspaces
                            ),
                        );
                    }
                }
            }
            match (
                &prev_config.windows.switch_2.enabled,
                &config.windows.switch_2.enabled,
            ) {
                (false, false) => {}
                (true, false) => {
                    add_info(changes, "Disabled Switch 2 view");
                }
                (_, true) => {
                    if !prev_config.windows.switch_2.enabled {
                        add_info(changes, "Enabled Switch 2 view");
                    }

                    if prev_config.windows.switch_2.key != config.windows.switch_2.key {
                        add_info_subtitle(
                            changes,
                            "Changed switch 2 key",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch_2.key, config.windows.switch_2.key
                            ),
                        );
                    }
                    if prev_config.windows.switch_2.modifier != config.windows.switch_2.modifier {
                        add_info_subtitle(
                            changes,
                            "Changed switch 2 modifier",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch_2.modifier,
                                config.windows.switch_2.modifier
                            ),
                        );
                    }
                    if prev_config.windows.switch_2.same_class != config.windows.switch_2.same_class
                        || prev_config.windows.switch_2.current_monitor
                            != config.windows.switch_2.current_monitor
                        || prev_config.windows.switch_2.current_workspace
                            != config.windows.switch_2.current_workspace
                    {
                        add_info_subtitle(
                            changes,
                            "Changed switch 2 filter by",
                            format!(
                                "{} -> {}",
                                flags_csv!(
                                    prev_config.windows.switch_2,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                                flags_csv!(
                                    config.windows.switch_2,
                                    same_class,
                                    current_monitor,
                                    current_workspace
                                ),
                            ),
                        );
                    }
                    if prev_config.windows.switch_2.switch_workspaces
                        != config.windows.switch_2.switch_workspaces
                    {
                        add_info_subtitle(
                            changes,
                            "Changed switch 2 switch workspaces",
                            format!(
                                "{} -> {}",
                                prev_config.windows.switch_2.switch_workspaces,
                                config.windows.switch_2.switch_workspaces
                            ),
                        );
                    }
                }
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
}

fn add_info(changes: &gtk::ListBox, text: &str) {
    let label = ActionRow::builder().title(text).build();
    changes.append(&label);
}

fn add_info_subtitle(changes: &gtk::ListBox, text: &str, subtitle: String) {
    let label = ActionRow::builder().title(text).subtitle(subtitle).build();
    changes.append(&label);
}
