use crate::global::WindowsOverviewData;
use core_lib::WarnWithDetails;
use exec_lib::reset_remain_focused;
use gtk::prelude::*;
use tracing::{debug_span, trace};

pub fn stop_overview(data: &WindowsOverviewData) {
    let _span = debug_span!("stop_overview").entered();
    reset_remain_focused().warn_details("Failed to reset follow mouse");
    for window in data.window_list.keys() {
        trace!("Closing window {:?}", window.id());
        window.close();
    }
}
