use relm4::adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw};

#[derive(Debug)]
pub struct NixPreview {}

#[derive(Debug)]
pub enum NixPreviewInput {}

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
        adw::StatusPage {
            set_icon_name: Some("document-properties-symbolic"),
            set_title: "Nix Preview Unavailable",
            set_description: Some("Nix preview is still in progress and is not available in this alpha."),
            set_hexpand: true,
            set_vexpand: true,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {}
}
