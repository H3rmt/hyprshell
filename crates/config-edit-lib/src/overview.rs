use gtk::prelude::*;
use gtk::{CheckButton, ComboBoxText, Frame, Label, Orientation};

pub fn generate_overview_config() -> (CheckButton, gtk::Box) {
    let overview_check = CheckButton::with_label("Enable Overview");
    let overview_container = gtk::Box::new(Orientation::Vertical, 8);
    overview_container.set_margin_start(12);

    // Overview content
    let overview_frame = Frame::builder().label("Overview").build();
    let overview_box = gtk::Box::new(Orientation::Vertical, 6);

    // Overview.key
    let key_row = gtk::Box::new(Orientation::Horizontal, 8);
    key_row.append(&Label::new(Some("Key")));
    let key_entry = gtk::Entry::new();
    // Modifier enum as ComboBoxText
    let mod_row = gtk::Box::new(Orientation::Horizontal, 8);
    mod_row.append(&Label::new(Some("Modifier")));
    let modifier_combo = ComboBoxText::new();
    modifier_combo.append_text("Alt");
    modifier_combo.append_text("Ctrl");
    modifier_combo.append_text("Super");

    // hide_filtered
    let hide_filtered_check = CheckButton::with_label("Hide filtered windows");

    // Populate initial values
    {
        // let c = config.borrow();
        // if let Some(w) = &c.windows {
        //     if let Some(ov) = &w.overview {
        //         overview_check.set_active(true);
        //         key_entry.set_text(&ov.key);
        //         match ov.modifier {
        //             config_lib::Modifier::Alt => modifier_combo.set_active(Some(0)),
        //             config_lib::Modifier::Ctrl => modifier_combo.set_active(Some(1)),
        //             config_lib::Modifier::Super => modifier_combo.set_active(Some(2)),
        //         }
        //         hide_filtered_check.set_active(ov.hide_filtered);
        //     } else {
        //         overview_check.set_active(false);
        //     }
        // }
    }

    key_row.append(&key_entry);
    overview_box.append(&key_row);
    mod_row.append(&modifier_combo);
    overview_box.append(&mod_row);
    overview_box.append(&hide_filtered_check);
    overview_frame.set_child(Some(&overview_box));

    overview_container.append(&overview_frame);

    // Toggle visibility based on checkbox
    let overview_container_clone = overview_container.clone();
    overview_container_clone.set_visible(overview_check.is_active());
    let occ = overview_container_clone;
    overview_check.connect_toggled(move |btn| {
        occ.set_visible(btn.is_active());
    });
    (overview_check, overview_container)
}
