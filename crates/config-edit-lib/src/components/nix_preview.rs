use relm4::adw::prelude::*;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
pub struct NixPreview {
    text: String,
}

#[derive(Debug)]
pub enum NixPreviewInput {
    SetConfig(crate::Config),
}

#[derive(Debug)]
pub struct NixPreviewInit {}

#[derive(Debug)]
pub enum NixPreviewOutput {}

#[relm4::component(pub)]
impl SimpleComponent for NixPreview {
    type Init = NixPreviewInit;
    type Input = NixPreviewInput;
    type Output = NixPreviewOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 10,
            gtk::Label {
                set_label: "Switch shortcuts (Nix configuration snippet)",
            },
            gtk::Label {
                set_selectable: true,
                set_xalign: 0.0,
                set_css_classes: &["monospace"],
                #[watch]
                set_label: &model.text,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            text: String::new(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            NixPreviewInput::SetConfig(config) => {
                self.text = [
                    preview("switch", &config.windows.switch),
                    preview("switch_2", &config.windows.switch_2),
                ]
                .join("\n\n");
            }
        }
    }
}

fn nix_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace("${", "\\${")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

fn nix_keys(keys: &[String]) -> String {
    format!(
        "[ {} ]",
        keys.iter()
            .map(|key| nix_string(key))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn preview(name: &str, switch: &crate::Switch) -> String {
    let keys = switch
        .keys
        .as_ref()
        .map_or_else(|| "null".to_string(), |keys| nix_keys(keys));
    format!(
        "programs.hyprshell.settings.windows.{name} = {{\n  enable = {};\n  modifier = {};\n  key = {};\n  keys = {keys};\n  reverse_keys = {};\n}};",
        switch.enabled,
        nix_string(&switch.modifier.to_string().to_ascii_lowercase()),
        nix_string(&switch.key),
        nix_keys(&switch.reverse_keys)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_preserves_lists_and_empty_reverse_keys() {
        let mut switch = crate::Switch::from(Some(config_lib::Switch::default()));
        switch.keys = Some(vec!["Tab".into(), "F6".into()]);
        switch.reverse_keys.clear();
        let text = preview("switch", &switch);
        assert!(text.contains("keys = [ \"Tab\" \"F6\" ];"));
        assert!(text.contains("reverse_keys = [  ];"));
        switch.reverse_keys = vec!["grave".into(), "F7".into()];
        assert!(preview("switch", &switch).contains("reverse_keys = [ \"grave\" \"F7\" ];"));
        switch.keys = None;
        assert!(preview("switch", &switch).contains("keys = null;"));
    }

    #[test]
    fn nix_strings_cannot_interpolate_user_input() {
        assert_eq!(nix_string("${value}"), "\"\\${value}\"");
    }
}
