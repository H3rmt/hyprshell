use crate::plugins::StaticLaunchOption;
use relm4::FactorySender;
use relm4::adw::gtk;
use relm4::adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent};

#[derive(Debug)]
pub struct LauncherPlugins {
    opt: StaticLaunchOption,
    launch_modifier: config_lib::Modifier,
}

#[derive(Debug)]
pub enum LauncherPluginsInput {}

#[derive(Debug)]
pub struct LauncherPluginsInit {
    pub opt: StaticLaunchOption,
    pub launch_modifier: config_lib::Modifier,
}

#[derive(Debug)]
pub enum LauncherPluginsOutput {
    Clicked(char),
}

#[relm4::factory(pub)]
impl FactoryComponent for LauncherPlugins {
    type Init = LauncherPluginsInit;
    type Input = LauncherPluginsInput;
    type Output = LauncherPluginsOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            set_css_classes: if self.opt.enabled {&["launcher-plugin"]} else {&["launcher-plugin", "monochrome"]},
            set_cursor_from_name: Some("pointer"),
            connect_clicked[sender, ch = self.opt.key] => move |_| sender.output_sender().emit(LauncherPluginsOutput::Clicked(ch)),
            gtk::Box {
                set_css_classes: &["launcher-plugin-inner"],
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                #[name = "icon"]
                gtk::Image {
                    set_css_classes: &["launcher-plugin-image"],
                    set_icon_size: gtk::IconSize::Large,
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    gtk::Label {
                        set_css_classes: &["underline"],
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Start,
                        set_label: &self.opt.text,
                    },
                    gtk::Label {
                        set_css_classes: &["launcher-plugin-key"],
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::End,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &format!("{} + {}", self.launch_modifier, self.opt.key),
                    },
                },
            },
        },
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            opt: init.opt,
            launch_modifier: init.launch_modifier,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {}
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        if let Some(icon_name) = &self.opt.icon {
            widgets
                .icon
                .set_icon_name(Some(&icon_name.to_string_lossy()));
        }
        widgets
    }
}
