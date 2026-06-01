use crate::plugin::{HighlightElement, HighlightedText, MatchedLaunchItem, TextSpan};
use core_lib::default;
use relm4::FactorySender;
use relm4::adw::prelude::*;
use relm4::adw::{glib, gtk};
use relm4::factory::{DynamicIndex, FactoryComponent};
use std::path::Path;
use tracing::warn;

#[derive(Debug)]
pub struct LauncherResults {
    item: MatchedLaunchItem,
    key: String,
    keyword: Option<Box<str>>,
}

#[derive(Debug)]
pub enum LauncherResultsInput {}

#[derive(Debug, Clone)]
pub struct LauncherResultsInit {
    pub item: MatchedLaunchItem,
    pub key: String,
}

#[derive(Debug)]
pub enum LauncherResultsOutput {
    Clicked(DynamicIndex),
}

#[allow(clippy::cast_sign_loss)]
fn text_attributes(text: &HighlightedText, base: gtk::gdk::RGBA) -> gtk::pango::AttrList {
    let attrs = gtk::pango::AttrList::new();
    let red = (base.red() * 65535.0) as u16;
    let green = (base.green() * 65535.0) as u16;
    let blue = (base.blue() * 65535.0) as u16;
    for TextSpan { start, end } in &text.spans {
        let mut weight = gtk::pango::AttrInt::new_underline(gtk::pango::Underline::Single);
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

#[allow(clippy::cast_sign_loss)]
fn markup_text(text: &HighlightedText, base: gtk::gdk::RGBA) -> String {
    let red = (base.red() * 255.0) as u8;
    let green = (base.green() * 255.0) as u8;
    let blue = (base.blue() * 255.0) as u8;
    let color_hex = format!("#{red:02x}{green:02x}{blue:02x}");

    let mut result = String::new();
    let text_bytes = text.text.as_bytes();
    let mut last_end = 0u32;

    for TextSpan { start, end } in &text.spans {
        // Add unformatted text before this span
        if last_end < *start {
            result.push_str(&glib::markup_escape_text(
                std::str::from_utf8(&text_bytes[last_end as usize..*start as usize]).unwrap_or(""),
            ));
        }

        // Add formatted span
        let span_text =
            std::str::from_utf8(&text_bytes[*start as usize..*end as usize]).unwrap_or("");

        result.push_str(&format!(
            "<span underline='single' foreground='{}'>{}</span>",
            color_hex,
            glib::markup_escape_text(span_text)
        ));

        last_end = *end;
    }

    // Add remaining unformatted text
    if (last_end as usize) < text_bytes.len() {
        result.push_str(&glib::markup_escape_text(
            std::str::from_utf8(&text_bytes[last_end as usize..]).unwrap_or(""),
        ));
    }

    result
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
            connect_clicked[sender, index] => move |_| {
                sender.output_sender().emit(LauncherResultsOutput::Clicked(index.clone()));
            },
            #[name = "probe"]
            gtk::Label {
                set_css_classes: &["launcher-item-inner-color-probe-element"],
                set_hexpand: false,
                set_hexpand_set: true,
                set_vexpand: false,
                set_vexpand_set: true,
            },
            gtk::Box {
                set_css_classes: &["launcher-item-inner"],
                set_orientation: gtk::Orientation::Horizontal,
                set_height_request: 45,
                set_spacing: 12,
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
                if self.keyword.is_some() {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 0,
                        #[name = "keyword"]
                        gtk::Label {
                            set_css_classes: &["launcher-item-details"],
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                            set_hexpand: true,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            #[watch]
                            set_label: &self.keyword.clone().unwrap_or_default(),
                        },
                        #[name = "details"]
                        gtk::Label {
                            set_css_classes: &["launcher-item-details"],
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                            set_hexpand: true,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_label: &self.item.item.details,
                        }
                    }
                } else {
                    #[name = "details_single"]
                    gtk::Label {
                        set_css_classes: &["launcher-item-details"],
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        set_hexpand: true,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &self.item.item.details,
                    }
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
            keyword: match init.item.highlight {
                HighlightElement::Keyword(ref h) => Some(h.text.clone()),
                _ => None,
            },
            item: init.item,
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
        let name_w: &gtk::Label = &widgets.name;
        let details_w: &gtk::Label = &widgets.details;
        let details_s_w: &gtk::Label = &widgets.details_single;
        let keyword_w: &gtk::Label = &widgets.keyword;
        let probe_w: &gtk::Label = &widgets.probe;
        let icon_w: &gtk::Image = &widgets.icon;

        if let Some(details_long) = &self.item.item.details_long {
            details_w.set_tooltip_text(Some(details_long));
            details_w.add_css_class("underline");
            details_s_w.set_tooltip_text(Some(details_long));
            details_s_w.add_css_class("underline");
        }

        self.keyword = None;
        match self.item.highlight {
            HighlightElement::Name(ref h) => {
                name_w.set_attributes(Some(&text_attributes(h, probe_w.color())));
            }
            HighlightElement::Keyword(ref h) => {
                self.keyword = Some(h.text.clone());
                keyword_w.set_attributes(Some(&text_attributes(h, probe_w.color())));
            }
            HighlightElement::Details(ref h) => {
                details_w.set_attributes(Some(&text_attributes(h, probe_w.color())));
                details_s_w.set_attributes(Some(&text_attributes(h, probe_w.color())));
            }
            HighlightElement::DetailsLong(ref h) => {
                details_w.set_tooltip_markup(Some(&markup_text(h, probe_w.color())));
                details_s_w.set_tooltip_markup(Some(&markup_text(h, probe_w.color())));
            }
            HighlightElement::None => {}
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
