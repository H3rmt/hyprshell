use relm4::adw::prelude::*;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

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
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 10,
            gtk::Label {
                set_label:  "Nix preview (TODO)"
            }
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
