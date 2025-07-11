use std::fmt;
use tracing::{debug, warn};

pub trait GetFirstOrLast: Iterator + Sized {
    fn get_first_or_last(self, last: bool) -> Option<Self::Item>;
}
impl<I: Iterator> GetFirstOrLast for I {
    fn get_first_or_last(mut self, last: bool) -> Option<Self::Item> {
        if last { self.last() } else { self.next() }
    }
}

pub trait GetNextOrPrev: Iterator + Sized {
    fn get_next_or_prev(self, last: bool, len: usize) -> Option<Self::Item>;
}
impl<I: Iterator> GetNextOrPrev for I {
    fn get_next_or_prev(mut self, last: bool, len: usize) -> Option<Self::Item> {
        if last {
            if len == 0 {
                None
            } else {
                // skip to the last element
                self.nth(len - 1)
            }
        } else {
            self.next()
        }
    }
}

pub trait RevIf<'a>: Iterator + Sized + 'a {
    fn rev_if(self, cond: bool) -> Box<dyn Iterator<Item = Self::Item> + 'a>;
}

impl<'a, I: DoubleEndedIterator + 'a> RevIf<'a> for I {
    fn rev_if(self, cond: bool) -> Box<dyn Iterator<Item = Self::Item> + 'a> {
        if cond {
            Box::new(self.rev())
        } else {
            Box::new(self)
        }
    }
}

pub trait WarnWithDetails<A> {
    fn warn_details(self, msg: &str) -> Option<A>;
}

pub trait Warn<A> {
    fn warn(self) -> Option<A>;
}

impl<A> WarnWithDetails<A> for Option<A> {
    fn warn_details(self, msg: &str) -> Option<A> {
        match self {
            Some(o) => Some(o),
            None => {
                warn!("{}", msg);
                None
            }
        }
    }
}

impl<A, E: fmt::Debug + fmt::Display> WarnWithDetails<A> for Result<A, E> {
    fn warn_details(self, msg: &str) -> Option<A> {
        match self {
            Ok(o) => Some(o),
            Err(e) => {
                warn!("{}: {}", msg, e);
                debug!("Error: {e:?}");
                None
            }
        }
    }
}

impl<A, E: fmt::Debug + fmt::Display> Warn<A> for Result<A, E> {
    fn warn(self) -> Option<A> {
        match self {
            Ok(o) => Some(o),
            Err(e) => {
                warn!("{}", e);
                debug!("Error: {e:?}");
                None
            }
        }
    }
}
