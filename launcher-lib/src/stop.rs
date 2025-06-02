use crate::LauncherGlobal;
use gtk::prelude::*;
use tracing::{Level, span, trace};

pub fn stop_launcher(global: &LauncherGlobal) {
    let _span = span!(Level::TRACE, "stop_launcher").entered();

    if let Some(data) = &global.data {
        let data1 = data.borrow();
        trace!("Closing window {:?}", data1.window.id());
        data1.window.close();
        drop(data1);
    }
}
