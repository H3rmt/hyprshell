
use std::os::fd::BorrowedFd;
use std::os::fd::{AsFd, OwnedFd};

use rustix::fs::MemfdFlags;
use rustix::mm::{MapFlags, ProtFlags};

use wayland_client::WEnum;
use wayland_client::backend::WaylandError;
use wayland_client::{Connection, Dispatch, EventQueue};
use wayland_client::protocol::{wl_registry, wl_shm};
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_buffer;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm_pool;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1;
use wayland_protocols::ext::image_capture_source::v1::client::ext_image_capture_source_v1;
use wayland_protocols::ext::image_capture_source::v1::client::ext_image_capture_source_v1::ExtImageCaptureSourceV1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_manager_v1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_manager_v1::ExtImageCopyCaptureManagerV1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_session_v1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_session_v1::ExtImageCopyCaptureSessionV1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_frame_v1;
use wayland_protocols::ext::image_copy_capture::v1::client::ext_image_copy_capture_frame_v1::ExtImageCopyCaptureFrameV1;
use wayland_protocols::ext::image_capture_source::v1::client::ext_foreign_toplevel_image_capture_source_manager_v1;
use wayland_protocols::ext::image_capture_source::v1::client::ext_foreign_toplevel_image_capture_source_manager_v1::ExtForeignToplevelImageCaptureSourceManagerV1;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct CaptureSession { connection:  Connection
                          , event_queue: EventQueue<AppState>
                          , state:       AppState
                          , fd:          OwnedFd
                          , session:     ExtImageCopyCaptureSessionV1
                          , frame:       ExtImageCopyCaptureFrameV1
                          , buffer:      WlBuffer
                          , width:       u32
                          , height:      u32
                          , stride:      u32
                          , size:        u32
                          }

/// Result of a window capture.
/// Pixels are in BGRA format (native Wayland ARGB8888 byte order on little-endian).
pub struct CaptureResult { pub pixels: Vec<u8>
                         , pub width:  u32
                         , pub height: u32
                         , pub stride: u32
                         }

#[derive(Debug)]
#[allow(dead_code)]
struct TopLevelInfo { handle: ExtForeignToplevelHandleV1
                    , title:  Option<String>
                    , app_id: Option<String>
                    }

struct AppState { toplevels:            Vec<TopLevelInfo>
                , pending_title:        Option<String>
                , pending_app_id:       Option<String>
                // globals
                , wl_shm:               Option<WlShm>
                , source_manager:       Option<ExtForeignToplevelImageCaptureSourceManagerV1>
                , copy_capture_manager: Option<ExtImageCopyCaptureManagerV1>
                // capture constraints
                , buffer_geometry:      Option<(u32, u32)>
                , shm_format:           Option<wl_shm::Format>
                , session_done:         bool
                , ready:                bool
                , failed:               bool
                , buffer_released:      bool
                }

impl Dispatch<WlBuffer, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &WlBuffer
            , _event:   wl_buffer::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            wl_buffer::Event::Release => {
                println!("wl_buffer::Event::Release");
                _state.buffer_released = true;
            }
            _ => { }
        }
    }
}

impl Dispatch<WlShmPool, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &WlShmPool
            , _event:   wl_shm_pool::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<WlShm, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &WlShm
            , _event:   wl_shm::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            wl_shm::Event::Format { format } => {
                println!("wl_shm::Event::Format: {:?}", format);
            }
            _ => { }
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureFrameV1
            , _event:   ext_image_copy_capture_frame_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
            match _event {
                ext_image_copy_capture_frame_v1::Event::Transform { transform }  => {
                    println!("ext_image_copy_capture_frame_v1::Event::Transform: {:?}", transform);
                }
                ext_image_copy_capture_frame_v1::Event::Damage { x, y, width, height }  => {
                    println!("ext_image_copy_capture_frame_v1::Event::Damage: x={}, y={}, width={}, height={}", x, y, width, height);
                }
                ext_image_copy_capture_frame_v1::Event::PresentationTime { tv_sec_hi, tv_sec_lo, tv_nsec }  => {
                    let presentation_time = ((tv_sec_hi as u64) << 32) | (tv_sec_lo as u64);
                    println!("ext_image_copy_capture_frame_v1::Event::PresentationTime: {}.{} seconds", presentation_time, tv_nsec);
                }
                ext_image_copy_capture_frame_v1::Event::Ready  => {
                    _state.ready = true;
                }
                ext_image_copy_capture_frame_v1::Event::Failed { reason }  => {
                    println!("ext_image_copy_capture_frame_v1::Event::Failed: reason={:?}", reason);
                    _state.failed = true;
                }
                _ => { }
            }
    }
}

impl Dispatch<ExtImageCopyCaptureSessionV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureSessionV1
            , _event:   ext_image_copy_capture_session_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
            match _event {
                ext_image_copy_capture_session_v1::Event::ShmFormat { format }  => {
                    if let WEnum::Value(fmt) = format {
                        _state.shm_format = Some(fmt);
                    }
                }
                ext_image_copy_capture_session_v1::Event::BufferSize { width, height } => {
                    _state.buffer_geometry = Some((width, height));
                }
                ext_image_copy_capture_session_v1::Event::Done  => {
                    _state.session_done = true;
                }
                _ => { }
            }
    }
}

impl Dispatch<ExtImageCaptureSourceV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtImageCaptureSourceV1
            , _event:   ext_image_capture_source_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<ExtForeignToplevelImageCaptureSourceManagerV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtForeignToplevelImageCaptureSourceManagerV1
            , _event:   ext_foreign_toplevel_image_capture_source_manager_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<ExtImageCopyCaptureManagerV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureManagerV1
            , _event:   ext_image_copy_capture_manager_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<ExtForeignToplevelHandleV1, ()> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ExtForeignToplevelHandleV1
            , _event:   ext_foreign_toplevel_handle_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            ext_foreign_toplevel_handle_v1::Event::Title { title }  => {
                println!("ext_foreign_toplevel_handle_v1::Event::Title: {}", title);
                state.pending_title = Some(title);
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id }  => {
                println!("ext_foreign_toplevel_handle_v1::Event::AppId: {}", app_id);
                state.pending_app_id = Some(app_id);
            }
            ext_foreign_toplevel_handle_v1::Event::Done  => {
                println!("ext_foreign_toplevel_handle_v1::Event::Done");
                if state.pending_title.is_none() {
                    println!("Warning: toplevel handle done event received without title");
                }
                if state.pending_app_id.is_none() {
                    println!("Warning: toplevel handle done event received without app_id");
                }
                state.toplevels.push(TopLevelInfo { handle: _proxy.clone()
                                                  , title:  state.pending_title.clone()
                                                  , app_id: state.pending_app_id.clone()
                                                  });
                state.pending_title = None;
                state.pending_app_id = None;
            }
            _ => { }
        }
    }
}

impl Dispatch<ExtForeignToplevelListV1, ()> for AppState {
    fn event( _state:    &mut AppState
            , _proxy:   &ExtForeignToplevelListV1
            , _event:   ext_foreign_toplevel_list_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } => {
                println!("ext_foreign_toplevel_list_v1::Event::Toplevel: {:?}", toplevel);
            }
            ext_foreign_toplevel_list_v1::Event::Finished  => {
                println!("ext_foreign_toplevel_list_v1::Event::Finished");
            }
            _ => { }
        }
    }

    fn event_created_child( _opcode: u16
                          , _qhandle: &wayland_client::QueueHandle<Self>
                          ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData>
    {
        _qhandle.make_data::<ExtForeignToplevelHandleV1, _>(())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &wl_registry::WlRegistry
            , _event:   wl_registry::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            wl_registry::Event::Global { name, interface, version } if interface == "wl_shm" => {
                state.wl_shm = Some(_proxy.bind::<WlShm, _, _>(name, version.min(1), _qhandle, ()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_foreign_toplevel_image_capture_source_manager_v1" => {
                state.source_manager = Some(_proxy.bind::<ExtForeignToplevelImageCaptureSourceManagerV1, _, _>(name, version.min(1), _qhandle, ()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_image_copy_capture_manager_v1" => {
                state.copy_capture_manager = Some(_proxy.bind::<ExtImageCopyCaptureManagerV1, _, _>(name, version.min(1), _qhandle, ()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_foreign_toplevel_list_v1" => {
                _proxy.bind::<ExtForeignToplevelListV1, _, _>(name, version.min(1), _qhandle, ());
            }
            _ => { }
        }
    }
}

impl CaptureSession {

    pub fn new() -> Result<Self> {
        let connection                            = Connection::connect_to_env()?;
        let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
        let mut state                             = AppState { toplevels:            Vec::new()
                                                             , pending_title:        None
                                                             , pending_app_id:       None
                                                             , wl_shm:               None
                                                             , source_manager:       None
                                                             , copy_capture_manager: None
                                                             , buffer_geometry:      None
                                                             , shm_format:           None
                                                             , session_done:         false
                                                             , ready:                false
                                                             , failed:               false
                                                             , buffer_released:      false
                                                             };

        connection.display().get_registry(&event_queue.handle(), ());

        event_queue.roundtrip(&mut state)?;
        event_queue.roundtrip(&mut state)?;

        let toplevel = state.toplevels.first()
            .ok_or("No toplevels found")?;
        println!("First toplevel: {:?}", toplevel);

        let source_manager = state.source_manager.as_ref()
            .ok_or("No source manager found")?;
        let source = source_manager.create_source(&toplevel.handle, &event_queue.handle(), ());

        let copy_capture_manager = state.copy_capture_manager.as_ref()
            .ok_or("No copy capture manager found")?;
        let session = copy_capture_manager.create_session(&source, ext_image_copy_capture_manager_v1::Options::empty(), &event_queue.handle(), ());

        event_queue.roundtrip(&mut state)?;

        let (width, height) = state.buffer_geometry
            .ok_or("No buffer geometry received")?;
        let shm_format = state.shm_format
            .ok_or("No shm format received")?;
        let stride = width * 4;
        let size   = stride * height;

        let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
        rustix::fs::ftruncate(&fd, size.into())?;

        let wlshm = state.wl_shm.as_ref()
            .ok_or("No wl_shm found")?;
        let pool   = wlshm.create_pool(fd.as_fd(), size as i32, &event_queue.handle(), ());
        let buffer = pool.create_buffer(0, width as i32, height as i32, stride as i32, shm_format, &event_queue.handle(), ());

        let frame = session.create_frame(&event_queue.handle(), ());
        frame.attach_buffer(&buffer);
        frame.capture();

        event_queue.flush()?;

        Ok(CaptureSession { connection, event_queue, state, fd, session, frame, buffer, width, height, stride, size })
    }

    pub fn connection_fd(&self) -> BorrowedFd<'_> { self.connection.as_fd() }

    pub fn dispatch_pending(&mut self) -> Result<()> {
        if let Some(guard) = self.event_queue.prepare_read() {
            match guard.read() {
                Ok(_) => { }
                Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => { }
                Err(e) => return Err(e.into()),
            }
        }
        let _ = self.event_queue.dispatch_pending(&mut self.state)?;
        Ok(())
    }

    pub fn is_ready(&self) -> bool { self.state.ready }

    pub fn is_failed(&self) -> bool { self.state.failed }

    pub fn take_pixels(&self) -> Result<CaptureResult> {
        let pixels = unsafe {
            let ptr  = rustix::mm::mmap(std::ptr::null_mut(), self.size as usize, ProtFlags::READ, MapFlags::SHARED, &self.fd, 0)?;
            let data = std::slice::from_raw_parts(ptr as *const u8, self.size as usize);
            data.to_vec()
        };

        Ok(CaptureResult { pixels, width: self.width, height: self.height, stride: self.stride })
    }

    pub fn capture_next(&mut self) -> Result<()> {
        self.state.ready = false;
        self.state.failed = false;

        self.frame.destroy();

        let frame = self.session.create_frame(&self.event_queue.handle(), ());
        frame.attach_buffer(&self.buffer);
        frame.capture();

        self.frame = frame;

        self.event_queue.flush()?;

        Ok(())
    }
}

/// Capture the first available toplevel window.
/// Returns raw pixels in BGRA format (native Wayland ARGB8888 byte order on little-endian).
pub fn capture() -> Result<CaptureResult> {
    let mut session = CaptureSession::new()?;

    // Wait for the capture to complete (Ready or Failed)
    while !session.is_ready() && !session.is_failed() {
        session.event_queue.blocking_dispatch(&mut session.state)?;
    }

    if session.state.failed {
        return Err("Capture failed: compositor returned an error".into());
    }

    println!("Capture successful");
    session.take_pixels()
}
