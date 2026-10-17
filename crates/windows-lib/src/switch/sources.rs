use relm4::gtk::glib;

/// Own every release-related callback, including the deferred focus dispatch.
#[derive(Debug, Default)]
pub(super) struct Sources {
    pub check: Option<glib::SourceId>,
    pub request: Option<glib::JoinHandle<()>>,
    pub commit: Option<glib::JoinHandle<()>>,
}

impl Sources {
    pub fn cancel_request(&mut self) {
        if let Some(request) = self.request.take() {
            request.abort();
        }
    }

    pub fn cancel_commit(&mut self) {
        if let Some(commit) = self.commit.take() {
            commit.abort();
        }
    }

    pub fn stop_check(&mut self) {
        if let Some(check) = self.check.take() {
            check.remove();
        }
        self.cancel_request();
    }
}

impl Drop for Sources {
    fn drop(&mut self) {
        self.stop_check();
        self.cancel_commit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    #[test]
    fn close_cancel_and_destroy_remove_real_glib_work() {
        let context = glib::MainContext::default();
        let _guard = context.acquire().expect("own test GLib context");
        let ticks = Rc::new(Cell::new(0));
        let completed = Rc::new(Cell::new(0));
        let mut sources = Sources::default();
        let count = ticks.clone();
        sources.check = Some(glib::timeout_add_local(
            Duration::from_millis(1),
            move || {
                count.set(count.get() + 1);
                glib::ControlFlow::Continue
            },
        ));
        let count = completed.clone();
        sources.request = Some(glib::spawn_future_local(async move {
            glib::timeout_future(Duration::from_millis(10)).await;
            count.set(count.get() + 1);
        }));
        context.block_on(glib::timeout_future(Duration::from_millis(3)));
        assert!(ticks.get() > 0, "timer really ran while open");
        sources.stop_check();
        let stopped_at = ticks.get();
        context.block_on(glib::timeout_future(Duration::from_millis(15)));
        assert_eq!(ticks.get(), stopped_at, "close removed periodic work");
        assert_eq!(completed.get(), 0, "close cancelled in-flight completion");

        // Escape/reopen cancels an already scheduled focus change.
        let count = completed.clone();
        sources.commit = Some(glib::spawn_future_local(async move {
            glib::timeout_future(Duration::ZERO).await;
            count.set(count.get() + 1);
        }));
        sources.cancel_commit();
        context.block_on(glib::timeout_future(Duration::from_millis(3)));
        assert_eq!(completed.get(), 0);

        // Component destruction applies the same cleanup, even while waiting.
        let count = completed.clone();
        sources.request = Some(glib::spawn_future_local(async move {
            glib::timeout_future(Duration::from_millis(3)).await;
            count.set(count.get() + 1);
        }));
        drop(sources);
        context.block_on(glib::timeout_future(Duration::from_millis(6)));
        assert_eq!(
            completed.get(),
            0,
            "destroyed component cannot receive completion"
        );
    }
}
