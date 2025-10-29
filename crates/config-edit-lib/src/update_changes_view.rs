#![allow(clippy::too_many_lines)]

use adw::ActionRow;
use adw::gtk::{ListBox, TextView};
use adw::prelude::{TextBufferExt, TextViewExt, WidgetExt};
use config_lib::{Config, Plugins};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

static PREV_CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
fn get_previous_config<'a>() -> MutexGuard<'a, Config> {
    PREV_CONFIG
        .get()
        .expect("Failed to get PREV_CONFIG lock")
        .lock()
        .expect("Failed to lock PREV_CONFIG")
}

pub fn set_previous_config(config: Config) {
    if PREV_CONFIG.get().is_none() {
        PREV_CONFIG
            .set(Mutex::new(config))
            .expect("Failed to set PREV_CONFIG");
        return;
    }

    let mut c = PREV_CONFIG
        .get()
        .expect("Failed to get PREV_CONFIG")
        .lock()
        .expect("Failed to lock PREV_CONFIG");
    *c = config;
}

pub fn update_changes_view(
    changes: &ListBox,
    how_to_use: &TextView,
    config: &Config,
    path: &Path,
) -> bool {
    let previous_config = get_previous_config();
    while let Some(child) = changes.first_child() {
        changes.remove(&child);
    }

    match (&previous_config.windows, &config.windows) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled Windows");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled Windows");
        }
        (Some(pw), Some(cw)) => {
            #[allow(clippy::cast_sign_loss)]
            if (pw.scale - cw.scale).abs() > 0.001 {
                add_info_subtitle(
                    changes,
                    "Changed windows scale",
                    format!("{} -> {}", pw.scale, cw.scale),
                );
            }
            if pw.items_per_row != cw.items_per_row {
                add_info_subtitle(
                    changes,
                    "Changed windows items per row",
                    format!("{} -> {}", pw.items_per_row, cw.items_per_row),
                );
            }
            match (&pw.overview, &cw.overview) {
                (None, None) => {}
                (Some(_), None) => {
                    add_info(changes, "Disabled Overview");
                }
                (None, Some(_)) => {
                    add_info(changes, "Enabled Overview");
                }
                (Some(po), Some(co)) => {
                    if po.modifier != co.modifier {
                        add_info_subtitle(
                            changes,
                            "Changed overview modifier",
                            format!("{} -> {}", po.modifier, co.modifier),
                        );
                    }
                    if po.key != co.key {
                        add_info_subtitle(
                            changes,
                            "Changed overview key",
                            format!("{} -> {}", po.key, co.key),
                        );
                    }
                    if po.hide_filtered != co.hide_filtered {
                        add_info_subtitle(
                            changes,
                            "Changed overview hide filtered",
                            format!("{} -> {}", po.hide_filtered, co.hide_filtered),
                        );
                    }
                    if po.filter_by != co.filter_by {
                        add_info_subtitle(
                            changes,
                            "Changed overview filter by",
                            format!("{:?} -> {:?}", po.filter_by, co.filter_by),
                        );
                    }
                    if po.launcher.launch_modifier != co.launcher.launch_modifier {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher launch modifier",
                            format!(
                                "{} -> {}",
                                po.launcher.launch_modifier, co.launcher.launch_modifier
                            ),
                        );
                    }
                    if po.launcher.max_items != co.launcher.max_items {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher max items",
                            format!("{} -> {}", po.launcher.max_items, co.launcher.max_items),
                        );
                    }
                    if po.launcher.show_when_empty != co.launcher.show_when_empty {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher show when empty",
                            format!(
                                "{} -> {}",
                                po.launcher.show_when_empty, co.launcher.show_when_empty
                            ),
                        );
                    }
                    if po.launcher.width != co.launcher.width {
                        add_info_subtitle(
                            changes,
                            "Changed overview launcher width",
                            format!("{} -> {}", po.launcher.width, co.launcher.width),
                        );
                    }
                    match (&po.launcher.default_terminal, &co.launcher.default_terminal) {
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
                    add_plugin_changes(changes, &po.launcher.plugins, &co.launcher.plugins);
                }
            }
            match (&pw.switch, &cw.switch) {
                (None, None) => {}
                (Some(_), None) => {
                    add_info(changes, "Disabled Switch view");
                }
                (None, Some(_)) => {
                    add_info(changes, "Enabled Switch view");
                }
                (Some(ps), Some(cs)) => {
                    if ps.modifier != cs.modifier {
                        add_info_subtitle(
                            changes,
                            "Changed switch modifier",
                            format!("{} -> {}", ps.modifier, cs.modifier),
                        );
                    }
                    if ps.filter_by != cs.filter_by {
                        add_info_subtitle(
                            changes,
                            "Changed switch filter by",
                            format!("{:?} -> {:?}", ps.filter_by, cs.filter_by),
                        );
                    }
                    if ps.switch_workspaces != cs.switch_workspaces {
                        add_info_subtitle(
                            changes,
                            "Changed switch switch workspaces",
                            format!("{} -> {}", ps.switch_workspaces, cs.switch_workspaces),
                        );
                    }
                }
            }
        }
    }
    drop(previous_config);

    let changes_exist = if changes.first_child().is_none() {
        add_info(changes, "No changes");
        false
    } else {
        true
    };

    let text = config_lib::explain(config, path, false, false);
    how_to_use.buffer().set_text(&text);

    changes_exist
}

fn add_plugin_changes(changes: &ListBox, pp: &Plugins, cp: &Plugins) {
    match (&pp.applications, &cp.applications) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled applications launcher plugin");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled applications launcher plugin");
        }
        (Some(ppc), Some(cpc)) => {
            if ppc.run_cache_weeks != cpc.run_cache_weeks {
                add_info_subtitle(
                    changes,
                    "Changed applications launcher run cache weeks",
                    format!("{} -> {}", ppc.run_cache_weeks, cpc.run_cache_weeks),
                );
            }
            if ppc.show_execs != cpc.show_execs {
                add_info_subtitle(
                    changes,
                    "Changed applications launcher show execs",
                    format!("{} -> {}", ppc.show_execs, cpc.show_execs),
                );
            }
            if ppc.show_actions_submenu != cpc.show_actions_submenu {
                add_info_subtitle(
                    changes,
                    "Changed applications launcher show actions submenu",
                    format!(
                        "{} -> {}",
                        ppc.show_actions_submenu, cpc.show_actions_submenu
                    ),
                );
            }
        }
    }
    match (&pp.terminal, &cp.terminal) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled terminal launcher plugin");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled terminal launcher plugin");
        }
        (Some(_pt), Some(_ct)) => {}
    }
    match (&pp.shell, &cp.shell) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled shell launcher plugin");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled shell launcher plugin");
        }
        (Some(_pt), Some(_ct)) => {}
    }
    match (&pp.calc, &cp.calc) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled calc launcher plugin");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled calc launcher plugin");
        }
        (Some(_pt), Some(_ct)) => {}
    }
    match (&pp.path, &cp.path) {
        (None, None) => {}
        (Some(_), None) => {
            add_info(changes, "Disabled path launcher plugin");
        }
        (None, Some(_)) => {
            add_info(changes, "Enabled path launcher plugin");
        }
        (Some(_pt), Some(_ct)) => {}
    }
}

fn add_info(changes: &ListBox, text: &str) {
    let label = ActionRow::builder().title(text).build();
    changes.append(&label);
}

fn add_info_subtitle(changes: &ListBox, text: &str, subtitle: String) {
    let label = ActionRow::builder().title(text).subtitle(subtitle).build();
    changes.append(&label);
}
