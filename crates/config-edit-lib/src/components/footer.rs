use relm4::adw::prelude::*;
use relm4::gtk::Orientation;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};
use std::path::Path;
use tracing::trace;

#[derive(Debug)]
pub struct Footer {
    config_file: Box<Path>,
    changes: bool,
    generate: bool,
}

#[derive(Debug)]
pub enum FooterInput {
    ChangesExist(bool),
    GenerateMode(bool),
}

#[derive(Debug)]
pub struct FooterInit {
    pub config_file: Box<Path>,
}

#[derive(Debug)]
pub enum FooterOutput {
    Close,
    Save,
    Reset,
    Reload,
    Abort,
}

#[relm4::component(pub)]
impl SimpleComponent for Footer {
    type Init = FooterInit;
    type Input = FooterInput;
    type Output = FooterOutput;

    view! {
        gtk::ActionBar  {
            #[wrap(Some)]
            set_center_widget = &gtk::Box {
                set_spacing: 20,
                set_hexpand: true,
                set_css_classes: &["footer"],
                set_orientation: Orientation::Horizontal,
                gtk::LinkButton {
                    set_label: &format!("Hyprshell v{}", env!("CARGO_PKG_VERSION")),
                    set_uri: &format!("https://github.com/H3rmt/hyprshell/tree/v{}", env!("CARGO_PKG_VERSION")),
                },
                gtk::Box {
                    set_spacing: 10,
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                    set_orientation: Orientation::Horizontal,
                    gtk::Button {
                        set_label: "Reload from Disk",
                        #[watch]
                        set_visible: !model.generate,
                        set_sensitive: true,
                        set_css_classes: &["destructive-action"],
                        connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Reload),
                    },
                    gtk::Button {
                        set_label: "Reset",
                        #[watch]
                        set_visible: !model.generate,
                        #[watch]
                        set_sensitive: model.changes,
                        set_css_classes: &["destructive-action"],
                        connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Reset),
                    },
                    gtk::Button {
                        set_label: "Save Changes",
                        #[watch]
                        set_visible: !model.generate,
                        #[watch]
                        set_sensitive: model.changes,
                        set_css_classes: &["suggested-action"],
                        set_tooltip_text: Some(&format!("Config file: {}", model.config_file.display())),
                        connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Save),
                    },
                    gtk::Button {
                        set_label: "Abort Generate",
                        #[watch]
                        set_visible: model.generate,
                        set_css_classes: &["destructive-action"],
                        connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Abort),
                    },
                    gtk::Button {
                        set_label: "Close",
                        set_css_classes: &["destructive-action"],
                        connect_clicked[sender] => move |_| sender.output_sender().emit(FooterOutput::Close),
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            config_file: init.config_file,
            changes: false,
            generate: false,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        trace!("footer::update: {message:?}");
        match message {
            FooterInput::ChangesExist(changes) => {
                self.changes = changes;
            }
            FooterInput::GenerateMode(generate) => {
                self.generate = generate;
            }
        }
    }
}
