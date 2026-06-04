
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
use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_v1;
use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1;
use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1;
use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1;
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

const DRM_FORMAT_MOD_INVALID: u64 = 0x00FFFFFFFFFFFFFF;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub enum CaptureMode { PreferDmabuf
                     , ForceShm
                     }

pub enum BufferMode { Shm
                    , Dmabuf
                    }

struct DmabufSession { gbm_bo:   gbm::BufferObject<()>
                     , _gbm_dev:  gbm::Device<OwnedFd>
                     , fourcc:   u32
                     }

pub struct CaptureSession { connection:     Connection
                          , event_queue:    EventQueue<AppState>
                          , state:          AppState
                          , fd:             OwnedFd
                          , session:        ExtImageCopyCaptureSessionV1
                          , frame:          ExtImageCopyCaptureFrameV1
                          , buffer_mode:    BufferMode
                          , dmabuf_session: Option<DmabufSession>
                          , buffer:         WlBuffer
                          , width:          u32
                          , height:         u32
                          , stride:         u32
                          , size:           u32
                          }

pub enum CaptureOutput<'a> { Shm(ShmResult)
                           , Dmabuf(DmabufResult<'a>)
                           }

pub struct DmabufResult<'a> { pub fd:       BorrowedFd<'a>
                            , pub fourcc:   u32
                            , pub modifier: u64
                            , pub width:    u32
                            , pub height:   u32
                            , pub stride:   u32
                            }

/// Result of a window capture.
/// Pixels are in BGRA format (native Wayland ARGB8888 byte order on little-endian).
pub struct ShmResult { pub pixels: Vec<u8>
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
                , dmabuf_device:        Option<u64>
                , source_manager:       Option<ExtForeignToplevelImageCaptureSourceManagerV1>
                , copy_capture_manager: Option<ExtImageCopyCaptureManagerV1>
                // capture constraints
                , buffer_geometry:      Option<(u32, u32)>
                , shm_format:           Option<wl_shm::Format>
                , dmabuf_formats:       Vec<(u32, Vec<u64>)> // (fourcc, formats)
                , linux_dmabuf:         Option<ZwpLinuxDmabufV1>
                , session_done:         bool
                , ready:                bool
                , failed:               bool
                , buffer_released:      bool
                }

impl Dispatch<WlBuffer, ()> for AppState {
    fn event( _state:   &mut AppState
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
    fn event( _state:   &mut AppState
            , _proxy:   &WlShmPool
            , _event:   wl_shm_pool::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<WlShm, ()> for AppState {
    fn event( _state:   &mut AppState
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

impl Dispatch<ZwpLinuxBufferParamsV1, ()> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &ZwpLinuxBufferParamsV1
            , _event:   zwp_linux_buffer_params_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            zwp_linux_buffer_params_v1::Event::Created { buffer } => {
                println!("zwp_linux_buffer_params_v1::Event::Created: {:?}", buffer);
            }
            zwp_linux_buffer_params_v1::Event::Failed => {
                println!("zwp_linux_buffer_params_v1::Event::Failed");
            }
            _ => { }
        }
    }
}

impl Dispatch<ZwpLinuxDmabufV1, ()> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &ZwpLinuxDmabufV1
            , _event:   zwp_linux_dmabuf_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            zwp_linux_dmabuf_v1::Event::Modifier { format, modifier_hi, modifier_lo }  => {
                println!("zwp_linux_dmabuf_v1::Event::Modifier: format={:?}, modifier={:#x}", format, ((modifier_hi as u64) << 32) | (modifier_lo as u64));
            }
            zwp_linux_dmabuf_v1::Event::Format { format } => {
                println!("wl_shm::Event::Format: {:?}", format);
            }
            _ => { }
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, ()> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &ExtImageCopyCaptureFrameV1
            , _event:   ext_image_copy_capture_frame_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
            match _event {
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
    fn event( _state:   &mut AppState
            , _proxy:   &ExtImageCopyCaptureSessionV1
            , _event:   ext_image_copy_capture_session_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
            match _event {
                ext_image_copy_capture_session_v1::Event::DmabufDevice { device }   => {
                    println!("ext_image_copy_capture_session_v1::Event::DmabufDevice: device={:?}", device);
                    let dev = u64::from_ne_bytes(device[..8].try_into().unwrap());
                    _state.dmabuf_device = Some(dev);
                }
                ext_image_copy_capture_session_v1::Event::DmabufFormat { format, modifiers }   => {
                    println!("ext_image_copy_capture_session_v1::Event::DmabufFormat: format={:?}, modifiers={:?}", format, modifiers);
                    let mods: Vec<u64> = modifiers.chunks_exact(8)
                                                  .map(|chunk| u64::from_ne_bytes(chunk.try_into().unwrap()))
                                                  .collect();
                    _state.dmabuf_formats.push((format, mods));
                }
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
    fn event( _state:   &mut AppState
            , _proxy:   &ExtImageCaptureSourceV1
            , _event:   ext_image_capture_source_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<ExtForeignToplevelImageCaptureSourceManagerV1, ()> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &ExtForeignToplevelImageCaptureSourceManagerV1
            , _event:   ext_foreign_toplevel_image_capture_source_manager_v1::Event
            , _udata:   &()
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            ) { }
}

impl Dispatch<ExtImageCopyCaptureManagerV1, ()> for AppState {
    fn event( _state:   &mut AppState
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
    fn event( _state:   &mut AppState
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
                state.wl_shm = Some(_proxy.bind::<WlShm, _, _>(name, version.min(1), _qhandle, _udata.clone()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "zwp_linux_dmabuf_v1" => {
                state.linux_dmabuf = Some(_proxy.bind::<ZwpLinuxDmabufV1, _, _>(name, version.min(4), _qhandle, _udata.clone()))
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_foreign_toplevel_image_capture_source_manager_v1" => {
                state.source_manager = Some(_proxy.bind::<ExtForeignToplevelImageCaptureSourceManagerV1, _, _>(name, version.min(1), _qhandle, _udata.clone()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_image_copy_capture_manager_v1" => {
                state.copy_capture_manager = Some(_proxy.bind::<ExtImageCopyCaptureManagerV1, _, _>(name, version.min(1), _qhandle, _udata.clone()));
            }
            wl_registry::Event::Global { name, interface, version } if interface == "ext_foreign_toplevel_list_v1" => {
                _proxy.bind::<ExtForeignToplevelListV1, _, _>(name, version.min(1), _qhandle, _udata.clone());
            }
            _ => { }
        }
    }
}

impl CaptureSession {

    pub fn new(mode: CaptureMode) -> Result<Self> {
        let connection                            = Connection::connect_to_env()?;
        let mut event_queue: EventQueue<AppState> = connection.new_event_queue();
        let mut state                             = AppState { toplevels:            Vec::new()
                                                             , pending_title:        None
                                                             , pending_app_id:       None
                                                             , wl_shm:               None
                                                             , dmabuf_device:        None
                                                             , source_manager:       None
                                                             , copy_capture_manager: None
                                                             , buffer_geometry:      None
                                                             , shm_format:           None
                                                             , linux_dmabuf:         None
                                                             , dmabuf_formats:       Vec::new()
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

        let (buffer_mode, dmabuf_session, fd, buffer, stride, size) = match mode {
            CaptureMode::PreferDmabuf if let Some(dev) = state.dmabuf_device => {
                let (fourcc, modifiers) = state.dmabuf_formats.first().ok_or("No dmabuf format received")?;
                let drm_fd              = Self::find_drm_node(dev)?;
                let gbm_dev             = gbm::Device::new(drm_fd)?;
                let gbm_bo              = gbm_dev.create_buffer_object_with_modifiers::<()>( width
                                                                                           , height
                                                                                           , gbm::Format::try_from(*fourcc)?
                                                                                           , modifiers.iter()
                                                                                                      .filter(|&&m| m != DRM_FORMAT_MOD_INVALID && m == 0)
                                                                                                      .map(|&m| gbm::Modifier::from(m))
                                                                                           )?;

                let dmabuf_fd    = gbm_bo.fd()?;
                let stride       = gbm_bo.stride();
                let modifier     = gbm_bo.modifier();
                let linux_dmabuf = state.linux_dmabuf.as_ref().ok_or("No zwp_linux_dmabuf_v1")?;
                let params       = linux_dmabuf.create_params(&event_queue.handle(), ());

                let mod_val: u64 = modifier.into();
                let modifier_hi = (mod_val >> 32) as u32;
                let modifier_lo = (mod_val & 0xFFFF_FFFF) as u32;

                params.add(dmabuf_fd.as_fd(), 0, 0, stride, modifier_hi, modifier_lo);

                let buffer = params.create_immed(width as i32, height as i32, *fourcc, zwp_linux_buffer_params_v1::Flags::empty(), &event_queue.handle(), ());
                let size   = stride * height;

                (BufferMode::Dmabuf, Some(DmabufSession { gbm_bo, gbm_dev, fourcc: *fourcc }), dmabuf_fd, buffer, stride, size)
            }
            _ => {
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

                (BufferMode::Shm, None, fd, buffer, stride, size)
            }
        };

        let frame = session.create_frame(&event_queue.handle(), ());
        frame.attach_buffer(&buffer);
        frame.capture();

        event_queue.flush()?;

        Ok(CaptureSession { connection
                          , event_queue
                          , state
                          , fd
                          , session
                          , frame
                          , buffer_mode
                          , dmabuf_session
                          , buffer
                          , width
                          , height
                          , stride
                          , size
                          })
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

    pub fn take_output(&self) -> Result<CaptureOutput<'_>> {

        match &self.buffer_mode {
            BufferMode::Shm => {
                let pixels = unsafe {
                    let ptr  = rustix::mm::mmap(std::ptr::null_mut(), self.size as usize, ProtFlags::READ, MapFlags::SHARED, &self.fd, 0)?;
                    let data = std::slice::from_raw_parts(ptr as *const u8, self.size as usize);
                    data.to_vec()
                };

                Ok(CaptureOutput::Shm(ShmResult { pixels, width: self.width, height: self.height, stride: self.stride }))
            }
            BufferMode::Dmabuf => Ok(CaptureOutput::Dmabuf(DmabufResult { fd:       self.fd.as_fd()
                                                                        , fourcc:   self.dmabuf_session.as_ref().unwrap().fourcc
                                                                        , modifier: self.dmabuf_session.as_ref().unwrap().gbm_bo.modifier().into()
                                                                        , width:    self.width
                                                                        , height:   self.height
                                                                        , stride:   self.stride
                                                                        }))
        }
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

    fn find_drm_node(device: u64) -> Result<OwnedFd> {
        for entry in std::fs::read_dir("/dev/dri")? {
            let entry = entry?;
            let name  = entry.file_name();
            if name.to_str().map_or(false, |n| n.starts_with("renderD")) {
                let stat = rustix::fs::stat(&entry.path())?;
                if stat.st_rdev == device {
                    return Ok(rustix::fs::open(&entry.path(), rustix::fs::OFlags::RDWR, rustix::fs::Mode::empty())?);
                }
            }
        }
        Err("No matching DRM render node found".into())
    }
}

/// Capture the first available toplevel window.
/// Returns raw pixels in BGRA format (native Wayland ARGB8888 byte order on little-endian).
pub fn capture() -> Result<ShmResult> {
    let mut session = CaptureSession::new(CaptureMode::ForceShm)?;

    // Wait for the capture to complete (Ready or Failed)
    while !session.is_ready() && !session.is_failed() {
        session.event_queue.blocking_dispatch(&mut session.state)?;
    }

    if session.state.failed {
        return Err("Capture failed: compositor returned an error".into());
    }

    println!("Capture successful");
    match session.take_output()? {
        CaptureOutput::Shm(result) => Ok(result),
        CaptureOutput::Dmabuf(_)   => unreachable!("ForceShm mode should never produce Dmabuf")
    }
}
