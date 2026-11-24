use config_lib::io::Modifier;
use relm4::gtk::gdk::{Cursor, Key, ModifierType};
use relm4::gtk::prelude::*;
use relm4::{adw, gtk};

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
                tracing::warn!("select_row_index: row not found ({index})");
            }
        }
    }
}

pub fn handle_key_and_modifier(
    val: Key,
    state: ModifierType,
) -> Option<(String, Modifier, String)> {
    let key_name = val.name()?;
    let state = state
        & (ModifierType::SHIFT_MASK
            | ModifierType::ALT_MASK
            | ModifierType::CONTROL_MASK
            | ModifierType::SUPER_MASK
            | ModifierType::HYPER_MASK
            | ModifierType::META_MASK);
    let modifier_key = match val {
        Key::Alt_L | Key::Alt_R => Some((Modifier::Alt, ModifierType::ALT_MASK)),
        Key::Control_L | Key::Control_R => Some((Modifier::Ctrl, ModifierType::CONTROL_MASK)),
        Key::Super_L | Key::Super_R => Some((Modifier::Super, ModifierType::SUPER_MASK)),
        _ => None,
    };
    let modifier = if let Some((modifier, mask)) = modifier_key {
        if !(state & !mask).is_empty() {
            return None;
        }
        modifier
    } else if state.is_empty() {
        Modifier::None
    } else if state == ModifierType::ALT_MASK {
        Modifier::Alt
    } else if state == ModifierType::CONTROL_MASK {
        Modifier::Ctrl
    } else if state == ModifierType::SUPER_MASK {
        Modifier::Super
    } else {
        return None;
    };

    let label = mod_key_to_string(modifier, &key_name);

    Some((key_name.to_string(), modifier, label))
}

#[tracing::instrument(level = "trace", ret(level = "trace"))]
pub fn mod_key_to_accelerator(modifier: Modifier, key: &str) -> String {
    // Hyprland physical keycodes are not GTK accelerators; the caller shows
    // them separately rather than passing them to ShortcutLabel's parser.
    if is_keycode(key) {
        return String::new();
    }
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

    match Option::<core_lib::Modifier>::from(modifier) {
        Some(m) => format!("<{m}>{key}"),
        None => key.to_string(),
    }
}

pub fn is_keycode(key: &str) -> bool {
    key.starts_with("code:")
}

pub fn modifier_label(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Alt => "Alt",
        Modifier::Ctrl => "Ctrl",
        Modifier::Super => "Super",
        Modifier::None => "",
    }
}

pub fn mod_key_to_string(modifier: Modifier, key: &str) -> String {
    match Option::<core_lib::Modifier>::from(modifier) {
        Some(m) => format!("{m} + {key}"),
        None => key.to_string(),
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
