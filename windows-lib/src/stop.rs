use crate::WindowsGlobal;
use core_lib::Warn;
use exec_lib::activate_submap;
use gtk::prelude::*;
use tracing::{Level, span, trace};

pub fn stop_overview(global: &WindowsGlobal) {
    let _span = span!(Level::TRACE, "stop_overview").entered();
    activate_submap("reset").warn("Failed to reset submap");
    let data = global.data.borrow();
    for (window, _) in data.monitor_list.iter() {
        trace!("Closing window {:?}", window.id());
        window.close();
    }
    drop(data);
}
