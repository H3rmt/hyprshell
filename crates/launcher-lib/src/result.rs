use crate::plugins::SortedLaunchOption;
use core_lib::default;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::{FactorySender, SimpleComponent};
use std::path::Path;
use tracing::warn;

#[derive(Debug)]
pub struct LauncherResults {
    opt: SortedLaunchOption,
    key: String,
}

#[derive(Debug)]
pub enum LauncherResultsInput {}

#[derive(Debug)]
pub struct LauncherResultsInit {
    pub opt: SortedLaunchOption,
    pub key: String,
}

#[derive(Debug)]
pub enum LauncherResultsOutput {
    Clicked(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for LauncherResults {
    type Init = LauncherResultsInit;
    type Input = LauncherResultsInput;
    type Output = LauncherResultsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            set_css_classes: if self.opt.enabled {&["launcher-item"]} else {&["launcher-item", "monochrome"]},
            set_cursor_from_name: Some("pointer"),
            connect_clicked[sender, index] => move |_| sender.output_sender().emit(LauncherResultsOutput::Clicked(index.clone())),
            gtk::Box {
                set_css_classes: &["launcher-item-inner"],
                set_orientation: gtk::Orientation::Horizontal,
                set_height_request: 45,
                set_spacing: 8,
                set_hexpand: true,
                set_vexpand: true,
                #[name = "icon"]
                gtk::Image {
                    set_css_classes: &["launcher-item-image"],
                    set_icon_size: gtk::IconSize::Large,
                },
                gtk::Label {
                    set_css_classes: &["launcher-item-name"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                    set_label: &self.opt.name,
                },
                #[name = "details"]
                gtk::Label {
                    set_css_classes: &["launcher-item-details"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_label: &self.opt.details,
                },
                gtk::Label {
                    set_css_classes: &["launcher-key"],
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,
                    set_label: &self.key
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            opt: init.opt,
            key: init.key,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {}
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        if let Some(details_long) = &self.opt.details_long {
            widgets.details.set_tooltip_text(Some(details_long));
            widgets.details.add_css_class("underline");
        }
        if let Some(icon_path) = &self.opt.icon {
            if icon_path.is_absolute() {
                if let Some(icon_name) = icon_path.file_stem() {
                    if default::theme_has_icon_name(&icon_name.to_string_lossy()) {
                        widgets
                            .icon
                            .set_icon_name(Some(&icon_name.to_string_lossy()));
                    } else {
                        widgets
                            .icon
                            .set_from_file(Some(Path::new(&*icon_path.clone())));
                    }
                } else {
                    warn!("invalid icon name: {icon_path:?}");
                }
            } else {
                // use filename as some files are named org.gnome.file
                // trace!(
                //     "using name: {:?}",
                //     icon_path.file_name().and_then(|name| name.to_str())
                // );
                widgets
                    .icon
                    .set_icon_name(icon_path.file_name().and_then(|name| name.to_str()));
            }
        }

        widgets
    }
}
