use crate::structs::{GTKApplications, GTKPlugins};
use crate::views::launcher::application_plugin::{
    application_submenu, run_cache_weeks, show_execs,
};
use adw::gdk::Cursor;
use adw::gtk::{Align, Label, Orientation, Switch};
use adw::prelude::{BoxExt, ExpanderRowExt, WidgetExt};
use adw::{ExpanderRow, gtk};

pub fn plugins_rows(plugins: &ExpanderRow) -> GTKPlugins {
    let row_0 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .spacing(16)
        .build();

    let applications = create_plugins_applications_view(&row_0);

    let row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .spacing(16)
        .spacing(16)
        .build();

    let terminal = create_plugins_terminal_view(&row_1);
    let shell = create_plugins_shell_view(&row_1);

    let row_2 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .spacing(16)
        .build();

    let calc = create_plugins_calc_view(&row_2);
    let path = create_plugins_path_view(&row_2);

    let row_3 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .spacing(16)
        .build();
    create_plugins_websearch_view(&row_3);

    let row_4 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .spacing(16)
        .build();
    create_plugins_actions_view(&row_4);

    plugins.add_row(&row_0);
    plugins.add_row(&row_1);
    plugins.add_row(&row_2);
    plugins.add_row(&row_3);
    plugins.add_row(&row_4);

    GTKPlugins {
        row: plugins.clone(),
        terminal,
        shell,
        calc,
        path,
        applications,
    }
}

pub fn create_plugins_terminal_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Run in Terminal")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    switch_box.append(&hide_switch);
    hide_row.append(&switch_box);
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_shell_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Run in Shell (background)")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    switch_box.append(&hide_switch);
    hide_row.append(&switch_box);
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_calc_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Calculator")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    #[cfg(not(feature = "launcher_calc_plugin"))]
    {
        tracing::debug!("calc plugin is disabled");
        hide_row.add_css_class("disabled");
        hide_switch.set_can_focus(false);
        hide_switch.set_sensitive(false);
    }
    switch_box.append(&hide_switch);
    hide_row.append(&switch_box);
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_path_view(row: &gtk::Box) -> Switch {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["bordered"])
        .hexpand(true)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Open Filepath")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);
    let switch_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::End)
        .valign(Align::Center)
        .build();
    let hide_switch = Switch::builder().build();
    switch_box.append(&hide_switch);
    hide_row.append(&switch_box);
    row.append(&hide_row);
    hide_switch
}

pub fn create_plugins_applications_view(row: &gtk::Box) -> GTKApplications {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::Start)
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Applications")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);

    let erow = ExpanderRow::builder()
        .title_selectable(false)
        .show_enable_switch(true)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .build();
    erow.add_prefix(&hide_row);
    row.append(&erow);

    let row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    erow.add_row(&row_1);

    let cache_weeks = run_cache_weeks(&row_1);
    let show_exec = show_execs(&row_1);
    let submenu = application_submenu(&row_1);

    GTKApplications {
        row: erow,
        cache_weeks,
        submenu,
        show_exec,
    }
}

pub fn create_plugins_websearch_view(row: &gtk::Box) {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::Start)
        .css_classes(["disabled"])
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Web search (TODO)")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);

    let erow = ExpanderRow::builder()
        .title_selectable(false)
        .show_enable_switch(false)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .build();
    erow.add_prefix(&hide_row);
    row.append(&erow);

    let row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    erow.add_row(&row_1);

    row_1.append(&Label::new(Some(
        "coming soon (please configure in config file)",
    )));
}

pub fn create_plugins_actions_view(row: &gtk::Box) {
    let hide_row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .halign(Align::Start)
        .css_classes(["disabled"])
        .spacing(10)
        .build();
    hide_row.append(&Label::new(Some("Actions (TODO)")));
    let info_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    info_icon.set_tooltip_text(Some("TODO"));
    info_icon.set_cursor(Cursor::from_name("help", None).as_ref());
    hide_row.append(&info_icon);

    let erow = ExpanderRow::builder()
        .title_selectable(false)
        .show_enable_switch(false)
        .hexpand(true)
        .css_classes(["enable-frame"])
        .build();
    erow.add_prefix(&hide_row);
    row.append(&erow);

    let row_1 = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["frame-row"])
        .spacing(30)
        .build();
    erow.add_row(&row_1);
    row_1.append(&Label::new(Some(
        "coming soon (please configure in config file)",
    )));
}
