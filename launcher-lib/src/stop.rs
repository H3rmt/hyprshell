use crate::LauncherData;
use gtk::prelude::*;
use tracing::{debug_span, trace};

pub fn stop_launcher(data: &LauncherData) {
    let _span = debug_span!("stop_launcher").entered();

    trace!("Closing window {:?}", data.window.id());
    data.window.close();
}
