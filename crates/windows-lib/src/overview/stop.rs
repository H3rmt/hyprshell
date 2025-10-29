use crate::global::WindowsOverviewData;
use adw::gtk::prelude::*;
use core_lib::WarnWithDetails;
use exec_lib::reset_no_follow_mouse;
use tracing::{debug_span, trace};

pub fn stop_overview(data: &WindowsOverviewData) {
    let _span = debug_span!("stop_overview").entered();
    reset_no_follow_mouse().warn_details("Failed to reset follow mouse");
    for window in data.window_list.keys() {
        trace!("Closing window {:?}", window.id());
        window.close();
    }
}
