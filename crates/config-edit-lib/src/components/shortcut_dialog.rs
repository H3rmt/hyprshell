use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct KeyboardShortcut {
    key: Option<String>,
    modifier: Option<config_lib::io::Modifier>,
    is_visible: bool,
    dialog: Option<ShortcutDialog>,
    anchor: gtk::Button,
    label: Option<String>,
    require_modifier: bool,
}

#[derive(Debug)]
struct ShortcutDialog {
    dialog: adw::AlertDialog,
    preview: adw::ShortcutLabel,
    error: gtk::Label,
}

impl ShortcutDialog {
    fn set_shortcut(&self, modifier: config_lib::io::Modifier, key: &str) {
        let accelerator = crate::util::mod_key_to_accelerator(modifier, key);
        self.preview.set_visible(!accelerator.is_empty());
        self.preview.set_accelerator(&accelerator);
    }
}

#[derive(Debug)]
pub enum KeyboardShortcutInput {
    ShowKeyboardShortcutDialog(
        Option<(config_lib::io::Modifier, String)>,
        Option<gtk::Widget>,
    ),

    _HideKeyboardShortcut(bool),
    _UpdateShortcut(Option<(config_lib::io::Modifier, String)>),
}
#[derive(Debug)]
pub enum KeyboardShortcutOutput {
    SetKey(config_lib::io::Modifier, String),
    Abort,

    // send out instead of opening because
    // this allows parent to set current config
    OpenRequest,
}

#[derive(Debug)]
pub struct KeyboardShortcutInit {
    pub label: Option<String>,
    pub icon: Option<String>,
    pub require_modifier: bool,
}

#[relm4::component(pub)]
impl SimpleComponent for KeyboardShortcut {
    type Init = KeyboardShortcutInit;
    type Input = KeyboardShortcutInput;
    type Output = KeyboardShortcutOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            key: None,
            modifier: None,
            is_visible: false,
            anchor: root.clone(),
            dialog: None,
            label: init.label.clone(),
            require_modifier: init.require_modifier,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        gtk::Button {
            set_icon_name?: &init.icon,
            #[watch]
            set_label?: &model.label,
            #[watch]
            set_css_classes: if model.is_visible { &["active"] } else { &["not-active"] },
            connect_clicked[sender] => move |_b| { sender.output_sender().emit(KeyboardShortcutOutput::OpenRequest); },
        },
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        tracing::trace!("shortcut_dialog::update: {message:?}");
        match message {
            KeyboardShortcutInput::_UpdateShortcut(shortcut) => {
                let Some(dialog) = &self.dialog else {
                    return;
                };
                let shortcut = shortcut.filter(|(modifier, _)| {
                    !self.require_modifier || *modifier != config_lib::io::Modifier::None
                });
                self.key = shortcut.as_ref().map(|(_, key)| key.clone());
                self.modifier = shortcut.as_ref().map(|(modifier, _)| *modifier);
                dialog
                    .dialog
                    .set_response_enabled("use", shortcut.is_some());
                dialog.error.set_visible(shortcut.is_none());
                if let Some((modifier, key)) = shortcut {
                    dialog.set_shortcut(modifier, &key);
                } else {
                    dialog.preview.set_visible(false);
                }
            }
            KeyboardShortcutInput::_HideKeyboardShortcut(confirm) => {
                self.is_visible = false;
                self.dialog = None;
                if confirm {
                    if let (Some(key), Some(r#mod)) = (&self.key, &self.modifier) {
                        sender
                            .output_sender()
                            .emit(KeyboardShortcutOutput::SetKey(*r#mod, key.clone()));
                    } else {
                        tracing::warn!(
                            "Tried to hide keyboard shortcut dialog without key or modifier"
                        );
                        sender.output_sender().emit(KeyboardShortcutOutput::Abort);
                    }
                } else {
                    sender.output_sender().emit(KeyboardShortcutOutput::Abort);
                }
            }
            KeyboardShortcutInput::ShowKeyboardShortcutDialog(initial, root) => {
                if self.is_visible {
                    return;
                }
                self.is_visible = true;
                self.key = initial.as_ref().map(|(_, key)| key.clone());
                self.modifier = initial.map(|(r#mod, _)| r#mod);
                let dialog = create_shortcut_dialog(self.require_modifier, &sender);
                dialog.dialog.set_response_enabled(
                    "use",
                    self.key.is_some()
                        && self.modifier.is_some_and(|modifier| {
                            !self.require_modifier || modifier != config_lib::io::Modifier::None
                        }),
                );
                if let (Some(r#mod), Some(key)) = (&self.modifier, &self.key) {
                    dialog.set_shortcut(*r#mod, key);
                }
                dialog
                    .dialog
                    .present(Some(root.as_ref().unwrap_or(self.anchor.upcast_ref())));
                self.dialog = Some(dialog);
            }
        }
    }
}

fn create_shortcut_dialog(
    require_modifier: bool,
    sender: &ComponentSender<KeyboardShortcut>,
) -> ShortcutDialog {
    let preview = adw::ShortcutLabel::new("");
    preview.set_halign(gtk::Align::Center);
    preview.add_css_class("shortcut-preview");
    let error = gtk::Label::new(Some("Unsupported shortcut"));
    error.add_css_class("error");
    error.set_visible(false);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.append(&preview);
    content.append(&error);

    let dialog = adw::AlertDialog::new(
        Some("Set Keyboard Shortcut"),
        Some(if require_modifier {
            "Press a key with Alt, Ctrl, or Super, then confirm."
        } else {
            "Press a key, optionally with Alt, Ctrl, or Super, then confirm."
        }),
    );
    dialog.set_extra_child(Some(&content));
    dialog.add_responses(&[("cancel", "Cancel"), ("use", "Use Shortcut")]);
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("use", adw::ResponseAppearance::Suggested);
    let send = sender.clone();
    dialog.connect_response(None, move |_, response| {
        send.input(KeyboardShortcutInput::_HideKeyboardShortcut(
            response == "use",
        ));
    });

    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    let dialog_weak = dialog.downgrade();
    let send = sender.clone();
    key_controller.connect_key_pressed(move |_, val, _, state| {
        tracing::trace!("Raw key event - val: {}, state: {:?}", val, state);
        let modifiers = state
            & (gtk::gdk::ModifierType::SHIFT_MASK
                | gtk::gdk::ModifierType::ALT_MASK
                | gtk::gdk::ModifierType::CONTROL_MASK
                | gtk::gdk::ModifierType::SUPER_MASK
                | gtk::gdk::ModifierType::HYPER_MASK
                | gtk::gdk::ModifierType::META_MASK);
        // Keep the confirmation buttons keyboard-accessible without intercepting
        // modifier+Tab shortcuts such as Alt+Tab.
        if matches!(val, gtk::gdk::Key::Tab | gtk::gdk::Key::ISO_Left_Tab)
            && (modifiers.is_empty() || modifiers == gtk::gdk::ModifierType::SHIFT_MASK)
        {
            return gtk::glib::Propagation::Proceed;
        }
        if matches!(
            val,
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter | gtk::gdk::Key::space
        ) && modifiers.is_empty()
            && dialog_weak
                .upgrade()
                .and_then(|dialog| adw::prelude::AdwDialogExt::focus(&dialog))
                .is_some_and(|focus| focus.is::<gtk::Button>())
        {
            return gtk::glib::Propagation::Proceed;
        }
        let shortcut = crate::util::handle_key_and_modifier(val, state)
            .map(|(key, modifier, _)| (modifier, key));
        send.input(KeyboardShortcutInput::_UpdateShortcut(shortcut));
        gtk::glib::Propagation::Stop
    });
    dialog.add_controller(key_controller);

    ShortcutDialog {
        dialog,
        preview,
        error,
    }
}
