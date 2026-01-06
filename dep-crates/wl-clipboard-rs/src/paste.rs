//! Getting the offered MIME types and the clipboard contents.

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::io::Read;
use std::os::fd::AsFd;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{io, thread};

use os_pipe::{pipe, PipeReader};
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{
    delegate_dispatch, event_created_child, ConnectError, Dispatch, DispatchError, EventQueue,
};

use crate::common::{self, initialize};
use crate::data_control::{
    self, impl_dispatch_device, impl_dispatch_manager, impl_dispatch_offer, Offer,
};
use crate::seat_data::SeatData;
use crate::utils::is_text;

/// The clipboard to operate on.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum ClipboardType {
    /// The regular clipboard.
    #[default]
    Regular,
    /// The "primary" clipboard.
    ///
    /// Working with the "primary" clipboard requires the compositor to support ext-data-control,
    /// or wlr-data-control version 2 or above.
    Primary,
}

/// MIME types that can be requested from the clipboard.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum MimeType<'a> {
    /// Request any available MIME type.
    ///
    /// If multiple MIME types are offered, the requested MIME type is unspecified and depends on
    /// the order they are received from the Wayland compositor. However, plain text formats are
    /// prioritized, so if a plain text format is available among others then it will be requested.
    Any,
    /// Request a plain text MIME type.
    ///
    /// This will request one of the multiple common plain text MIME types. It will prioritize MIME
    /// types known to return UTF-8 text.
    Text,
    /// Request the given MIME type, and if it's not available fall back to `MimeType::Text`.
    ///
    /// Example use-case: pasting `text/html` should try `text/html` first, but if it's not
    /// available, any other plain text format will do fine too.
    TextWithPriority(&'a str),
    /// Request a specific MIME type.
    Specific(&'a str),
}

/// Seat to operate on.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord, Default)]
pub enum Seat<'a> {
    /// Operate on one of the existing seats depending on the order returned by the compositor.
    ///
    /// This is perfectly fine when only a single seat is present, so for most configurations.
    #[default]
    Unspecified,
    /// Operate on a seat with the given name.
    Specific(&'a str),
}

struct State {
    common: common::State,
    // The value is the set of MIME types in the offer.
    // TODO: We never remove offers from here, even if we don't use them or after destroying them.
    offers: HashMap<Offer, HashSet<String>>,
    got_primary_selection: bool,
}

delegate_dispatch!(State: [WlSeat: ()] => common::State);

impl AsMut<common::State> for State {
    fn as_mut(&mut self) -> &mut common::State {
        &mut self.common
    }
}

/// Errors that can occur for pasting and listing MIME types.
///
/// You may want to ignore some of these errors (rather than show an error message), like
/// `NoSeats`, `ClipboardEmpty` or `NoMimeType` as they are essentially equivalent to an empty
/// clipboard.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("There are no seats")]
    NoSeats,

    #[error("The clipboard of the requested seat is empty")]
    ClipboardEmpty,

    #[error("No suitable type of content copied")]
    NoMimeType,

    #[error("Couldn't open the provided Wayland socket")]
    SocketOpenError(#[source] io::Error),

    #[error("Couldn't connect to the Wayland compositor")]
    WaylandConnection(#[source] ConnectError),

    #[error("Wayland compositor communication error")]
    WaylandCommunication(#[source] DispatchError),

    #[error(
        "A required Wayland protocol ({} version {}) is not supported by the compositor",
        name,
        version
    )]
    MissingProtocol { name: &'static str, version: u32 },

    #[error("The compositor does not support primary selection")]
    PrimarySelectionUnsupported,

    #[error("The requested seat was not found")]
    SeatNotFound,

    #[error("Couldn't create a pipe for content transfer")]
    PipeCreation(#[source] io::Error),
}

impl From<common::Error> for Error {
    fn from(x: common::Error) -> Self {
        use common::Error::*;

        match x {
            SocketOpenError(err) => Error::SocketOpenError(err),
            WaylandConnection(err) => Error::WaylandConnection(err),
            WaylandCommunication(err) => Error::WaylandCommunication(err.into()),
            MissingProtocol { name, version } => Error::MissingProtocol { name, version },
        }
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl_dispatch_manager!(State);

impl_dispatch_device!(State, WlSeat, |state: &mut Self, event, seat| {
    match event {
        Event::DataOffer { id } => {
            let offer = data_control::Offer::from(id);
            state.offers.insert(offer, HashSet::new());
        }
        Event::Selection { id } => {
            let offer = id.map(data_control::Offer::from);
            let seat = state.common.seats.get_mut(seat).unwrap();
            seat.set_offer(offer);
        }
        Event::Finished => {
            // Destroy the device stored in the seat as it's no longer valid.
            let seat = state.common.seats.get_mut(seat).unwrap();
            seat.set_device(None);
        }
        Event::PrimarySelection { id } => {
            let offer = id.map(data_control::Offer::from);
            state.got_primary_selection = true;
            let seat = state.common.seats.get_mut(seat).unwrap();
            seat.set_primary_offer(offer);
        }
        _ => (),
    }
});

impl_dispatch_offer!(State, |state: &mut Self, offer: Offer, event| {
    if let Event::Offer { mime_type } = event {
        state.offers.get_mut(&offer).unwrap().insert(mime_type);
    }
});

fn get_offer(
    primary: bool,
    seat: Seat<'_>,
    socket_name: Option<OsString>,
) -> Result<(EventQueue<State>, State, Offer), Error> {
    let (mut queue, mut common) = initialize(primary, socket_name)?;

    // Check if there are no seats.
    if common.seats.is_empty() {
        return Err(Error::NoSeats);
    }

    // Go through the seats and get their data devices.
    for (seat, data) in &mut common.seats {
        let device = common
            .clipboard_manager
            .get_data_device(seat, &queue.handle(), seat.clone());
        data.set_device(Some(device));
    }

    let mut state = State {
        common,
        offers: HashMap::new(),
        got_primary_selection: false,
    };

    // Retrieve all seat names and offers.
    queue
        .roundtrip(&mut state)
        .map_err(Error::WaylandCommunication)?;

    // Check if the compositor supports primary selection.
    if primary && !state.got_primary_selection {
        return Err(Error::PrimarySelectionUnsupported);
    }

    // Figure out which offer we're interested in.
    let data = get_seat(&mut state, seat);

    let Some(data) = data else {
        return Err(Error::SeatNotFound);
    };

    let offer = if primary {
        &data.primary_offer
    } else {
        &data.offer
    };

    // Check if we found anything.
    match offer.clone() {
        Some(offer) => Ok((queue, state, offer)),
        None => Err(Error::ClipboardEmpty),
    }
}

/// Retrieves the offered MIME types.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::paste::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::{paste::{get_mime_types, ClipboardType, Seat}};
///
/// let mime_types = get_mime_types(ClipboardType::Regular, Seat::Unspecified)?;
/// for mime_type in mime_types {
///     println!("{}", mime_type);
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn get_mime_types(clipboard: ClipboardType, seat: Seat<'_>) -> Result<HashSet<String>, Error> {
    get_mime_types_internal(clipboard, seat, None)
}

// The internal function accepts the socket name, used for tests.
pub(crate) fn get_mime_types_internal(
    clipboard: ClipboardType,
    seat: Seat<'_>,
    socket_name: Option<OsString>,
) -> Result<HashSet<String>, Error> {
    let primary = clipboard == ClipboardType::Primary;
    let (_, mut state, offer) = get_offer(primary, seat, socket_name)?;
    Ok(state.offers.remove(&offer).unwrap())
}

/// Retrieves the clipboard contents.
///
/// This function returns a tuple of the reading end of a pipe containing the clipboard contents
/// and the actual MIME type of the contents.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use std::io::Read;
/// use wl_clipboard_rs::{paste::{get_contents, ClipboardType, Error, MimeType, Seat}};
///
/// let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Any);
/// match result {
///     Ok((mut pipe, mime_type)) => {
///         println!("Got data of the {} MIME type", &mime_type);
///
///         let mut contents = vec![];
///         pipe.read_to_end(&mut contents)?;
///         println!("Read {} bytes of data", contents.len());
///     }
///
///     Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
///         // The clipboard is empty, nothing to worry about.
///     }
///
///     Err(err) => Err(err)?
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn get_contents(
    clipboard: ClipboardType,
    seat: Seat<'_>,
    mime_type: MimeType<'_>,
) -> Result<(PipeReader, String), Error> {
    get_contents_internal(clipboard, seat, mime_type, None)
}

// The internal function accepts the socket name, used for tests.
pub(crate) fn get_contents_internal(
    clipboard: ClipboardType,
    seat: Seat<'_>,
    mime_type: MimeType<'_>,
    socket_name: Option<OsString>,
) -> Result<(PipeReader, String), Error> {
    let primary = clipboard == ClipboardType::Primary;
    let (mut queue, mut state, offer) = get_offer(primary, seat, socket_name)?;

    let mime_types = state.offers.remove(&offer).unwrap();

    // Find the desired MIME type.
    let mime_type = check_mime_type(mime_types, mime_type);

    // Check if a suitable MIME type is copied.
    if mime_type.is_none() {
        return Err(Error::NoMimeType);
    }

    let mime_type = mime_type.unwrap();

    // Create a pipe for content transfer.
    let (read, write) = pipe().map_err(Error::PipeCreation)?;

    // Start the transfer.
    offer.receive(mime_type.clone(), write.as_fd());
    drop(write);

    // A flush() is not enough here, it will result in sometimes pasting empty contents. I suspect this is due to a
    // race between the compositor reacting to the receive request, and the compositor reacting to wl-paste
    // disconnecting after queue is dropped. The roundtrip solves that race.
    queue
        .roundtrip(&mut state)
        .map_err(Error::WaylandCommunication)?;

    Ok((read, mime_type))
}

/// Asynchronously sends clipboard contents through mpsc::channel.
///
/// This function returns a mpsc::Receiver containing the reading end of pipes containing the clipboard contents
/// and the actual MIME type of the content.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use std::io::Read;
/// use wl_clipboard_rs::paste::{get_contents_channel, Error, MimeType, Seat};
///
/// let result = get_contents_channel(Seat::Unspecified, MimeType::Any);
/// match result {
///     Ok(rx) => {
///         loop {
///             match rx.recv() {
///                 Ok(Ok((mut pipe, mime_type))) => {
///                     println!("Got data of the {} MIME type", &mime_type);
///                     let mut contents = vec![];
///                     pipe.read_to_end(&mut contents)?;
///                     println!("Read {} bytes of data", contents.len());
///                 }
///                 Ok(Err(Error::NoSeats)) | Ok(Err(Error::ClipboardEmpty)) | Ok(Err(Error::NoMimeType)) => {
///                     // The clipboard is empty, nothing to worry about.
///                 }
///                 Ok(Err(err)) => Err(err)?, // other error getting clipboard data
///                 Err(err) => Err(err)? // error receiving data
///             }
///         }
///     }
///
///     Err(Error::WaylandConnection) | Err(Error::WaylandCommunication) | Err(Error::MissingProtocol) => {
///         // Error setting up channel
///     }
///
///     Err(err) => Err(err)?
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn get_contents_channel(
    seat: Seat<'static>,
    mime_type: MimeType<'static>,
) -> Result<mpsc::Receiver<Result<(PipeReader, String), Error>>, Error> {
    get_contents_channel_internal(seat, mime_type, None)
}

pub(crate) fn get_contents_channel_internal(
    seat: Seat<'static>,
    mime_type: MimeType<'static>,
    socket_name: Option<OsString>,
) -> Result<mpsc::Receiver<Result<(PipeReader, String), Error>>, Error> {
    let (sender, receiver) = mpsc::channel();
    let (queue, mut common) = initialize(false, socket_name)?;
    for (seat, data) in &mut common.seats {
        let device = common
            .clipboard_manager
            .get_data_device(seat, &queue.handle(), seat.clone());
        data.set_device(Some(device));
    }
    let state = State {
        common,
        offers: HashMap::new(),
        got_primary_selection: false,
    };
    thread::spawn(move || run_dispatch_loop(queue, state, seat, mime_type, sender));
    Ok(receiver)
}

fn run_dispatch_loop(
    mut queue: EventQueue<State>,
    mut state: State,
    seat: Seat<'static>,
    mime_type: MimeType<'static>,
    sender: mpsc::Sender<Result<(PipeReader, String), Error>>,
) {
    loop {
        if let Err(err) = queue.blocking_dispatch(&mut state) {
            if sender.send(Err(Error::WaylandCommunication(err))).is_err() {
                return; // receiver closed
            }
            continue;
        }

        let Some(seat_data) = get_seat(&mut state, seat) else {
            if sender.send(Err(Error::NoSeats)).is_err() {
                return; // receiver closed
            }
            continue;
        };

        // should also not happen as new data should be put into offers
        let Some(offer) = seat_data.offer.take() else {
            continue;
        };

        // shouldn't happen
        let Some(mime_types) = state.offers.remove(&offer) else {
            continue;
        };

        let Some(mime_type) = check_mime_type(mime_types, mime_type) else {
            if sender.send(Err(Error::NoMimeType)).is_err() {
                return; // receiver closed
            }
            continue;
        };

        let Ok((read, write)) = pipe() else {
            if sender
                .send(Err(Error::PipeCreation(io::Error::last_os_error())))
                .is_err()
            {
                return; // receiver closed
            }
            continue;
        };

        offer.receive(mime_type.clone(), write.as_fd());
        drop(write);

        if let Err(err) = queue.roundtrip(&mut state) {
            if sender.send(Err(Error::WaylandCommunication(err))).is_err() {
                return; // receiver closed
            }
            continue;
        }

        if sender.send(Ok((read, mime_type))).is_err() {
            return; // receiver closed
        }
    }
}

/// Asynchronously handle all clipboard contents with a callback.
///
/// This function returns a JoinHandle to the background thread and accepts a callback that either receives an Ok variant containing a HashMap of all offered MIME types
/// and a function to load the contents of a specific MIME type or Error if something went wrong.
/// If the function returns true, the thread will exit.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// use wl_clipboard_rs::paste::get_all_contents_channel;
///
/// fn handle_values(
///     data: wl_clipboard_rs::paste::Data
/// ) -> bool {
///   let Ok((mut mimes, mut load)) = data else {
///     return false;
///   };
///   if mimes.contains("text/html") {
///     let data = load("text/html".to_string()).unwrap();
///     println!("Got HTML data: {}", String::from_utf8_lossy(&data));
///   }
///   false
/// }
///
/// fn foo() -> Result<(), Box<dyn std::error::Error>> {
///   let result = get_all_contents_channel(Seat::Unspecified, Box::new(filter_mime));
///   match result {
///     Ok(handle) => {
///         handle.join()
///     }
///     Err(Error::WaylandConnection) | Err(Error::WaylandCommunication) | Err(Error::MissingProtocol) => {
///         // Error setting up listener
///     }
///     Err(err) => Err(err)?
///   }
///   Ok(())
/// }
/// ```
#[inline]
pub fn get_all_contents_callback(
    seat: Seat<'static>,
    callback: Box<dyn Fn(CallbackData) -> bool + Send + Sync + 'static>,
) -> Result<JoinHandle<()>, Error> {
    get_all_contents_callback_internal(seat, callback, None)
}

pub type CallbackData<'a> = Result<
    (
        HashSet<String>,
        &'a mut (dyn FnMut(String) -> Result<Vec<u8>, Error> + Send),
    ),
    Error,
>;

pub(crate) fn get_all_contents_callback_internal(
    seat: Seat<'static>,
    callback: Box<dyn Fn(CallbackData) -> bool + Send + Sync + 'static>,
    socket_name: Option<OsString>,
) -> Result<JoinHandle<()>, Error> {
    let (queue, mut common) = initialize(false, socket_name)?;
    for (seat, data) in &mut common.seats {
        let device = common
            .clipboard_manager
            .get_data_device(seat, &queue.handle(), seat.clone());
        data.set_device(Some(device));
    }
    let state = State {
        common,
        offers: HashMap::new(),
        got_primary_selection: false,
    };
    let handle = thread::spawn(move || run_callback_dispatch_loop(queue, state, seat, callback));
    Ok(handle)
}

fn run_callback_dispatch_loop(
    mut queue: EventQueue<State>,
    mut state: State,
    seat: Seat<'static>,
    callback: Box<dyn Fn(CallbackData) -> bool + Send + Sync + 'static>,
) {
    loop {
        if let Err(err) = queue.blocking_dispatch(&mut state) {
            if callback(Err(Error::WaylandCommunication(err))) {
                return;
            }
            continue;
        }

        let Some(seat_data) = get_seat(&mut state, seat) else {
            if callback(Err(Error::NoSeats)) {
                return;
            }
            continue;
        };

        // should also not happen as new data should be put into offers
        let Some(offer) = seat_data.offer.take() else {
            continue;
        };

        // shouldn't happen
        let Some(mime_types) = state.offers.remove(&offer) else {
            continue;
        };

        {
            let mut load = create_load_mime_fn(offer.clone(), &mut queue, &mut state);
            if callback(Ok((mime_types, &mut load))) {
                return;
            }
        }
    }
}

fn create_load_mime_fn<'a>(
    offer: Offer,
    queue: &'a mut EventQueue<State>,
    state: &'a mut State,
) -> impl FnMut(String) -> Result<Vec<u8>, Error> + use<'a> {
    move |mime: String| {
        let Ok((mut read, write)) = pipe() else {
            return Err(Error::PipeCreation(io::Error::last_os_error()));
        };
        offer.receive(mime, write.as_fd());
        drop(write);
        let mut contents = Vec::new();
        let _ = queue.roundtrip(state);
        let _ = read.read_to_end(&mut contents);
        Ok(contents)
    }
}

fn get_seat<'a>(state: &'a mut State, seat: Seat) -> Option<&'a mut SeatData> {
    match seat {
        Seat::Unspecified => state.common.seats.values_mut().next(),
        Seat::Specific(name) => state
            .common
            .seats
            .values_mut()
            .find(|data| data.name.as_deref() == Some(name)),
    }
}

fn check_mime_type(mut mime_types: HashSet<String>, mime_type: MimeType) -> Option<String> {
    match mime_type {
        MimeType::Any => mime_types
            .take("text/plain;charset=utf-8")
            .or_else(|| mime_types.take("UTF8_STRING"))
            .or_else(|| mime_types.iter().find(|x| is_text(x)).cloned())
            .or_else(|| mime_types.drain().next()),
        MimeType::Text => mime_types
            .take("text/plain;charset=utf-8")
            .or_else(|| mime_types.take("UTF8_STRING"))
            .or_else(|| mime_types.drain().find(|x| is_text(x))),
        MimeType::TextWithPriority(priority) => mime_types
            .take(priority)
            .or_else(|| mime_types.take("text/plain;charset=utf-8"))
            .or_else(|| mime_types.take("UTF8_STRING"))
            .or_else(|| mime_types.drain().find(|x| is_text(x))),
        MimeType::Specific(mime_type) => mime_types.take(mime_type),
    }
}
