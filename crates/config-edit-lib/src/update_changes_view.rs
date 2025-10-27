use adw::ActionRow;
use adw::gtk::{ListBox, TextView};
use adw::prelude::{BoxExt, TextBufferExt, TextViewExt, WidgetExt};
use config_lib::Config;
use std::path::Path;
use std::sync::OnceLock;

static PREV_CONFIG: OnceLock<Config> = OnceLock::new();
fn get_previous_config() -> &'static Config {
    PREV_CONFIG.get().expect("Failed to get PREV_CONFIG lock")
}

pub fn set_previous_config(config: Config) {
    let _ = PREV_CONFIG.set(config);
}
pub fn update_changes_view(changes: &ListBox, how_to_use: &TextView, config: &Config, path: &Path) {
    let previous_config = get_previous_config();
    while let Some(child) = changes.first_child() {
        changes.remove(&child)
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
            if pw.scale != cw.scale {
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
                        )
                    }
                    // TODO add launcher
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
                        )
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

    if changes.first_child().is_none() {
        add_info(changes, "No changes");
    }

    let text = config_lib::explain(config, path, false, false);
    how_to_use.buffer().set_text(&text);
}

fn add_info(changes: &ListBox, text: &str) {
    let label = ActionRow::builder().title(text).build();
    changes.append(&label);
}

fn add_info_subtitle(changes: &ListBox, text: &str, subtitle: String) {
    let label = ActionRow::builder().title(text).subtitle(subtitle).build();
    changes.append(&label);
}
