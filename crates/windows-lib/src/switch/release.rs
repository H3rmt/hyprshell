//! Ordering and evidence for committing a switch selection. No I/O runs here.
use exec_lib::switch::ModifierState;

#[derive(Debug, Default)]
pub(super) struct ReleaseState {
    pub open: bool,
    pub unsupported: bool,
    request: u64,
    pending: bool,
    release_event: bool,
    error_reported: bool,
    pub cancelled_at: Option<u32>,
    pub received: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    Ignore,
    Held,
    Commit,
    Unsupported,
    Warn(String),
}

impl ReleaseState {
    // Timestamped bindings carry the original event time. Untagged explicit
    // IPC opens are new requests, including immediately after cancellation.
    pub fn open(&mut self, time: Option<u32>) -> bool {
        if time
            .zip(self.cancelled_at)
            .is_some_and(|(event, cancel)| event.wrapping_sub(cancel).cast_signed() <= 0)
        {
            return false;
        }
        self.open = true;
        self.navigation();
        true
    }

    pub const fn navigation(&mut self) {
        self.release_event = false;
        self.invalidate();
    }

    pub const fn release(&mut self) {
        self.release_event = self.open;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.received.clear();
        self.release_event = false;
        self.invalidate();
    }

    pub fn cancel(&mut self, time: u32) {
        self.cancelled_at = Some(time);
        self.close();
    }

    pub const fn invalidate(&mut self) {
        self.request = self.request.wrapping_add(1);
        self.pending = false;
    }

    pub const fn request(&mut self, refresh: bool) -> Option<u64> {
        if !self.open || self.unsupported || (self.pending && !refresh) {
            return None;
        }
        self.invalidate();
        self.pending = true;
        Some(self.request)
    }

    pub const fn event_request(&mut self) -> Option<u64> {
        if !self.open || !self.unsupported || !self.release_event {
            return None;
        }
        self.invalidate();
        self.pending = true;
        Some(self.request)
    }

    pub const fn is_current(&self, request: u64) -> bool {
        self.open && self.pending && self.request == request
    }

    pub fn result(
        &mut self,
        request: u64,
        result: Result<Option<ModifierState>, String>,
    ) -> Outcome {
        if !self.is_current(request) {
            return Outcome::Ignore;
        }
        self.pending = false;
        match result {
            Ok(Some(ModifierState {
                pressed,
                pending_opens,
            })) if pressed || pending_opens => {
                self.received.clear();
                self.error_reported = false;
                Outcome::Held
            }
            Ok(Some(_)) => {
                self.error_reported = false;
                self.close();
                Outcome::Commit
            }
            Ok(None) => {
                self.received.clear();
                self.unsupported = true;
                Outcome::Unsupported
            }
            Err(error) if !self.error_reported => {
                self.error_reported = true;
                Outcome::Warn(error)
            }
            Err(_) => Outcome::Ignore,
        }
    }

    // Only a real release event plus a known seat mask establishes release.
    // Missing focus/seat state is unknown, never equivalent to both keys up.
    pub fn event_result(&mut self, request: u64, held: Option<bool>) -> Outcome {
        if !self.is_current(request) || !self.release_event || !self.unsupported {
            return Outcome::Ignore;
        }
        self.pending = false;
        if held == Some(false) {
            self.close();
            Outcome::Commit
        } else {
            Outcome::Held
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ModifierState, Outcome, ReleaseState};

    fn snapshot(pressed: bool) -> ModifierState {
        ModifierState {
            pressed,
            pending_opens: false,
        }
    }

    fn query(state: &mut ReleaseState, held: bool) -> Outcome {
        let id = state.request(true).expect("open selection can query");
        state.result(id, Ok(Some(snapshot(held))))
    }

    #[test]
    fn release_before_open_and_missed_event_commit() {
        let mut state = ReleaseState::default();
        state.release(); // close-before-open IPC
        assert!(state.open(None));
        assert_eq!(query(&mut state, false), Outcome::Commit);
        assert!(!state.open);
        assert!(state.open(Some(10)));
        assert_eq!(query(&mut state, true), Outcome::Held);
        // No release event arrives during the GTK focus handoff.
        assert_eq!(query(&mut state, false), Outcome::Commit);
    }

    #[test]
    fn held_state_rejects_stale_close_and_both_side_handoffs() {
        let mut state = ReleaseState::default();
        state.open(None);
        for (left, right) in [
            (true, false),
            (true, true),
            (false, true),
            (true, true),
            (true, false),
        ] {
            state.release();
            assert_eq!(query(&mut state, left || right), Outcome::Held);
            assert!(state.open);
        }
        assert_eq!(query(&mut state, false), Outcome::Commit);
    }

    #[test]
    fn queued_navigation_invalidates_older_commit() {
        let mut state = ReleaseState::default();
        state.open(None);
        let old = state.request(true).expect("first request");
        // A navigation already ahead of the result in the component queue wins.
        state.navigation();
        let new = state.request(true).expect("replacement request");
        assert_eq!(
            state.result(old, Ok(Some(snapshot(false)))),
            Outcome::Ignore
        );
        assert!(state.is_current(new));
        assert_eq!(
            state.result(new, Ok(Some(snapshot(false)))),
            Outcome::Commit
        );
    }

    #[test]
    fn issued_navigation_must_arrive_before_release_commits() {
        let mut state = ReleaseState::default();
        state.open(Some(10));
        let id = state.request(true).expect("query");
        assert_eq!(
            state.result(
                id,
                Ok(Some(ModifierState {
                    pressed: false,
                    pending_opens: true
                }))
            ),
            Outcome::Held
        );
        assert!(
            state.open,
            "keep original snapshot until issued navigation arrives"
        );
        state.open(Some(11));
        let id = state.request(true).expect("query after navigation");
        assert_eq!(
            state.result(
                id,
                Ok(Some(ModifierState {
                    pressed: false,
                    pending_opens: false
                }))
            ),
            Outcome::Commit
        );
    }

    #[test]
    fn cancellation_delayed_open_and_next_quick_chord() {
        let mut state = ReleaseState::default();
        state.open(Some(99));
        let old = state.request(true).expect("query");
        state.cancel(100);
        assert!(!state.open(Some(99)));
        assert!(!state.open(Some(100)));
        assert_eq!(
            state.result(old, Ok(Some(snapshot(false)))),
            Outcome::Ignore
        );
        assert!(state.open(Some(101)));
        assert_eq!(query(&mut state, false), Outcome::Commit);
        state.cancel(102);
        assert!(state.open(None)); // Explicit IPC is a new request.
        assert_eq!(query(&mut state, false), Outcome::Commit);
    }

    #[test]
    fn wraparound_and_close_reopen_reject_old_results() {
        let mut state = ReleaseState::default();
        state.cancel(2);
        assert!(!state.open(Some(u32::MAX - 2)));
        state.cancel(u32::MAX - 2);
        assert!(state.open(Some(2)));
        let old = state.request(true).expect("query");
        state.close();
        assert!(state.request(false).is_none());
        state.open(None);
        let new = state.request(true).expect("reopened query");
        assert_eq!(state.result(old, Ok(None)), Outcome::Ignore);
        assert!(state.is_current(new));
        assert_eq!(state.result(new, Ok(Some(snapshot(true)))), Outcome::Held);
    }

    #[test]
    fn malformed_and_timeout_are_not_release_and_warn_once_until_success() {
        let mut state = ReleaseState::default();
        state.open(None);
        for error in ["malformed", "timeout", "timeout"] {
            let id = state.request(false).expect("retry");
            assert!(state.request(false).is_none(), "one in-flight query");
            let outcome = state.result(id, Err(error.into()));
            assert_eq!(
                outcome,
                if error == "malformed" {
                    Outcome::Warn(error.into())
                } else {
                    Outcome::Ignore
                }
            );
            assert!(state.open);
        }
        assert_eq!(query(&mut state, true), Outcome::Held);
        let id = state.request(false).expect("query");
        assert_eq!(
            state.result(id, Err("timeout".into())),
            Outcome::Warn("timeout".into())
        );
    }

    #[test]
    fn unsupported_needs_release_evidence_and_known_seat_state() {
        let mut state = ReleaseState::default();
        state.open(None);
        let id = state.request(true).expect("capability probe");
        assert_eq!(state.result(id, Ok(None)), Outcome::Unsupported);
        assert!(
            state.request(false).is_none(),
            "stop polling unsupported IPC"
        );
        assert!(
            state.event_request().is_none(),
            "Shift/navigation is not release evidence"
        );
        for held in [Some(true), None] {
            state.release();
            let id = state.event_request().expect("release event");
            assert_eq!(state.event_result(id, held), Outcome::Held);
            assert!(state.open);
        }
        state.release();
        let old = state.event_request().expect("event");
        state.navigation();
        assert_eq!(state.event_result(old, Some(false)), Outcome::Ignore);
        state.release();
        let id = state.event_request().expect("final side released");
        assert_eq!(state.event_result(id, Some(false)), Outcome::Commit);
    }

    #[test]
    fn release_during_capability_probe_is_preserved_but_cancel_removes_it() {
        let mut state = ReleaseState::default();
        state.open(None);
        let id = state.request(true).expect("probe");
        state.release();
        assert_eq!(state.result(id, Ok(None)), Outcome::Unsupported);
        let old = state.event_request().expect("pending release");
        state.cancel(10);
        state.open(Some(11));
        assert_eq!(state.event_result(old, Some(false)), Outcome::Ignore);
        assert!(state.open);
    }
}
