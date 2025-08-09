use crate::global::WindowsSwitchData;
use core_lib::WarnWithDetails;
use exec_lib::reset_remain_focused;
use gtk::prelude::*;
use tracing::{debug_span, trace};

pub fn stop_switch(data: &WindowsSwitchData) {
    let _span = debug_span!("stop_switch").entered();
    reset_remain_focused().warn_details("Failed to reset follow mouse");
    trace!("Closing window {:?}", data.window.id());
    data.window.close();
}
