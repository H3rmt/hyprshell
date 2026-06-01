use crate::plugin::{HighlightedText, MatchedLaunchItem, TextSpan};
use core_lib::default;
use relm4::FactorySender;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent};
use std::path::Path;
use tracing::warn;

#[derive(Debug)]
pub struct LauncherResults {
    pub item: MatchedLaunchItem,
    pub key: String,
    pub has_children: bool,
}

#[derive(Debug)]
pub enum LauncherResultsInput {}

#[derive(Debug, Clone)]
pub struct LauncherResultsInit {
    pub item: MatchedLaunchItem,
    pub key: String,
    pub has_children: bool,
}

#[derive(Debug)]
pub enum LauncherResultsOutput {
    Clicked(DynamicIndex),
}

fn brighten_channel(channel: f32) -> f32 {
    channel + (1.0 - channel) * 0.4
}

fn text_attributes(text: &HighlightedText, base: gtk::gdk::RGBA) -> gtk::pango::AttrList {
    let attrs = gtk::pango::AttrList::new();
    let red = brighten_channel(base.red());
    let green = brighten_channel(base.green());
    let blue = brighten_channel(base.blue());
    let red = (red * 65535.0) as u16;
    let green = (green * 65535.0) as u16;
    let blue = (blue * 65535.0) as u16;
    for TextSpan { start, end } in &text.spans {
        let mut weight = gtk::pango::AttrInt::new_weight(gtk::pango::Weight::Bold);
        weight.set_start_index(*start);
        weight.set_end_index(*end);
        attrs.insert(weight);

        let mut color = gtk::pango::AttrColor::new_foreground(red, green, blue);
        color.set_start_index(*start);
        color.set_end_index(*end);
        attrs.insert(color);
    }
    attrs
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
            set_css_classes: if self.item.item.enabled {&["launcher-item"]} else {&["launcher-item", "monochrome"]},
            set_cursor_from_name: Some("pointer"),
            connect_clicked[sender, index, has_children = self.has_children] => move |_| {
                let _ = has_children;
                sender.output_sender().emit(LauncherResultsOutput::Clicked(index.clone()))
            },
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
                #[name = "name"]
                gtk::Label {
                    set_css_classes: &["launcher-item-name"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_text: &self.item.item.name,
                },
                #[name = "details"]
                gtk::Label {
                    set_css_classes: &["launcher-item-details"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_label: &self.item.item.details,
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
            item: init.item,
            key: init.key,
            has_children: init.has_children,
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
        let name_w: &gtk::Label = &widgets.name;
        let details_w: &gtk::Label = &widgets.details;
        let icon_w: &gtk::Image = &widgets.icon;

        // TODO
        // let base_color = name_w.style_context().color();
        // name_w.set_attributes(Some(&text_attributes(&self.opt.display_name, base_color)));

        if let Some(details_long) = &self.item.item.details_long {
            details_w.set_tooltip_text(Some(details_long));
            details_w.add_css_class("underline");
        }

        if let Some(icon_path) = &self.item.item.icon {
            if icon_path.is_absolute() {
                if let Some(icon_name) = icon_path.file_stem() {
                    if default::theme_has_icon_name(&icon_name.to_string_lossy()) {
                        icon_w.set_icon_name(Some(&icon_name.to_string_lossy()));
                    } else {
                        icon_w.set_from_file(Some(Path::new(&*icon_path.clone())));
                    }
                } else {
                    warn!("invalid icon name: {icon_path:?}");
                }
            } else {
                icon_w.set_icon_name(icon_path.file_name().and_then(|name| name.to_str()));
            }
        }
        widgets
    }
}
