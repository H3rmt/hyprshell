use crate::structs::ConfigModifier;
use relm4::gtk::gdk::{Cursor, Key, ModifierType};
use relm4::gtk::prelude::{Cast, EditableExt, WidgetExt};
use relm4::{adw, gtk};
// use relm4::tokio::time::sleep;
use tracing::{instrument, warn};

pub trait SetTextIfDifferent {
    fn set_text_if_different(&self, text: &str);
}

impl SetTextIfDifferent for gtk::Entry {
    fn set_text_if_different(&self, text: &str) {
        use relm4::adw::prelude::EditableExt;
        if self.text() != text {
            self.set_text(text);
        }
    }
}

impl SetTextIfDifferent for adw::EntryRow {
    fn set_text_if_different(&self, text: &str) {
        if self.text() != text {
            self.set_text(text);
        }
    }
}

pub trait SetCursor {
    fn set_cursor_by_name(&self, name: &str);
}

impl SetCursor for gtk::Image {
    fn set_cursor_by_name(&self, name: &str) {
        use relm4::adw::prelude::WidgetExt;
        self.set_cursor(Cursor::from_name(name, None).as_ref());
    }
}

pub trait ScrollToPosition {
    fn scroll_to_pos(&self, pos: usize, animate: bool);
}

impl ScrollToPosition for adw::Carousel {
    fn scroll_to_pos(&self, pos: usize, animate: bool) {
        if let Some(wdg) = self.observe_children().into_iter().flatten().nth(pos)
            && let Ok(widget) = wdg.downcast::<gtk::Widget>()
        {
            let s2 = self.clone();
            // scuffed method to select a new widget (else it doesn't work on the first render)
            gtk::glib::idle_add_local(move || {
                s2.scroll_to(&widget, animate);
                #[allow(clippy::cast_sign_loss)]
                if s2.position() as usize == pos {
                    gtk::glib::ControlFlow::Break
                } else {
                    gtk::glib::ControlFlow::Continue
                }
            });
        }
    }
}

#[allow(dead_code)]
pub trait SelectRow {
    fn select_row_index_opt(&self, index: Option<i32>);
    fn select_row_index(&self, index: i32);
}

impl SelectRow for gtk::ListBox {
    fn select_row_index_opt(&self, index: Option<i32>) {
        self.unselect_all();
        if let Some(index) = index {
            self.select_row_index(index);
        }
    }

    fn select_row_index(&self, index: i32) {
        self.unselect_all();
        if self.selected_row() != self.row_at_index(index) {
            if let Some(row) = self.row_at_index(index) {
                self.select_row(Some(&row));
            } else {
                warn!("select_row_index: row not found ({index})");
            }
        }
    }
}

pub fn handle_key(val: Key, state: ModifierType) -> Option<(String, ConfigModifier, String)> {
    let key_name = val.name()?;
    let modifier = match val {
        Key::Alt_L | Key::Alt_R => ConfigModifier::Alt,
        Key::Control_L | Key::Control_R => ConfigModifier::Ctrl,
        Key::Super_L | Key::Super_R => ConfigModifier::Super,
        _ => match state {
            ModifierType::NO_MODIFIER_MASK => ConfigModifier::None,
            ModifierType::ALT_MASK => ConfigModifier::Alt,
            ModifierType::CONTROL_MASK => ConfigModifier::Ctrl,
            ModifierType::SUPER_MASK => ConfigModifier::Super,
            _ => return None,
        },
    };

    let label = if modifier == ConfigModifier::None {
        key_name.to_string()
    } else {
        format!("{modifier} + {key_name}")
    };

    Some((key_name.to_string(), modifier, label))
}

pub fn default_config() -> config_lib::Config {
    config_lib::Config {
        windows: Some(config_lib::Windows::default()),
        ..Default::default()
    }
}

#[instrument(level = "trace", ret(level = "trace"))]
pub fn mod_key_to_accelerator(modifier: ConfigModifier, key: &str) -> String {
    // correct some keys that can sometimes have wrong capitalization
    let key = match &*key.to_lowercase() {
        "super_l" => "Super_L",
        "super_r" => "Super_R",
        "alt_l" => "Alt_L",
        "alt_r" => "Alt_R",
        "control_l" => "Control_L",
        "control_r" => "Control_R",
        _ => key,
    };

    if modifier == ConfigModifier::None {
        key.to_string()
    } else {
        format!("<{modifier}>{key}")
    }
}

#[instrument(level = "trace", ret(level = "trace"))]
pub fn mod_key_to_string(modifier: ConfigModifier, key: &str) -> String {
    if modifier == ConfigModifier::None {
        key.to_string()
    } else {
        format!("{modifier} + {key}")
    }
}

#[macro_export]
macro_rules! flags_csv {
    ($s:expr, $($field:ident),+ $(,)?) => {{
        [$( (stringify!($field), $s.$field) ),+]
            .into_iter()
            .filter(|(_, v)| *v)
            .map(|(k, _)| k)
            .collect::<Vec<&str>>()
            .join(", ")
    }};
}
