use gtk::prelude::*;
use gtk::{CheckButton, ComboBoxText, Frame, Label, Orientation};

pub fn generate_switch_config() -> (CheckButton, gtk::Box) {
    let switch_check = CheckButton::with_label("Enable Switch");
    let switch_container = gtk::Box::new(Orientation::Vertical, 8);
    switch_container.set_margin_start(12);
    let switch_frame = Frame::builder().label("Switch").build();
    let switch_box = gtk::Box::new(Orientation::Vertical, 6);
    let switch_mod_row = gtk::Box::new(Orientation::Horizontal, 8);
    switch_mod_row.append(&Label::new(Some("Modifier")));
    let switch_modifier_combo = ComboBoxText::new();
    switch_modifier_combo.append_text("Alt");
    switch_modifier_combo.append_text("Ctrl");
    switch_modifier_combo.append_text("Super");

    // Populate initial values for switch
    {
        // let c = config.borrow();
        // if let Some(w) = &c.windows {
        //     if let Some(sw) = &w.switch {
        //         switch_check.set_active(true);
        //         match sw.modifier {
        //             config_lib::Modifier::Alt => switch_modifier_combo.set_active(Some(0)),
        //             config_lib::Modifier::Ctrl => switch_modifier_combo.set_active(Some(1)),
        //             config_lib::Modifier::Super => switch_modifier_combo.set_active(Some(2)),
        //         }
        //     } else {
        //         switch_check.set_active(false);
        //     }
        // }
    }

    switch_mod_row.append(&switch_modifier_combo);
    switch_box.append(&switch_mod_row);
    switch_frame.set_child(Some(&switch_box));
    switch_container.append(&switch_frame);
    let switch_container_clone = switch_container.clone();
    switch_container_clone.set_visible(switch_check.is_active());
    let scc = switch_container_clone;
    switch_check.connect_toggled(move |btn| {
        scc.set_visible(btn.is_active());
    });

    (switch_check, switch_container)
}
