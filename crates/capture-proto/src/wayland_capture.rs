
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

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub enum CaptureMode { PreferDmabuf
                     , ForceShm
                     }

pub enum BufferMode { Shm
                    , Dmabuf
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

/// Result of a window capture via shared memory.
/// Pixels are in BGRA format (native Wayland ARGB8888 byte order on
/// little-endian).
pub struct ShmResult { pub pixels: Vec<u8>
                     , pub width:  u32
                     , pub height: u32
                     , pub stride: u32
                     }

/// Per-window capture handle stored in CaptureManager.
pub struct WindowCapture { pub title:       Option<String>
                         , pub app_id:      Option<String>
                         // wayland objects
                         , session:         ExtImageCopyCaptureSessionV1
                         , frame:           Option<ExtImageCopyCaptureFrameV1>
                         , buffer:          WlBuffer
                         , fd:              OwnedFd
                         , buffer_mode:     BufferMode
                         , _dmabuf_bo:      Option<gbm::BufferObject<()>>
                         , fourcc:          Option<u32>
                         , width:           u32
                         , height:          u32
                         , stride:          u32
                         , size:            u32
                         }

/// Manages capture sessions for all toplevel windows sharing a single
/// Wayland connection and event queue.
pub struct CaptureManager { connection:  Connection
                          , event_queue: EventQueue<AppState>
                          , state:       AppState
                          , captures:    Vec<WindowCapture>
                          , _gbm_dev:    Option<gbm::Device<OwnedFd>>
                          }

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Per-capture event state, stored inside AppState so Dispatch handlers
/// can route events by capture index.
struct PerCaptureState { buffer_geometry:     Option<(u32, u32)>
                       , shm_format:         Option<wl_shm::Format>
                       , dmabuf_formats:     Vec<(u32, Vec<u64>)>
                       , dmabuf_device:      Option<u64>
                       , session_done:       bool
                       , ready:              bool
                       , failed:             bool
                       , size_changed:       bool
                       , dmabuf_buf_failed:  bool
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
               , linux_dmabuf:         Option<ZwpLinuxDmabufV1>
               // per-capture state (indexed by capture id)
               , captures:             Vec<PerCaptureState>
               }

// ---------------------------------------------------------------------------
// Dispatch implementations
// ---------------------------------------------------------------------------

// Session and frame events are routed by capture index (usize user data).

impl Dispatch<ExtImageCopyCaptureSessionV1, usize> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureSessionV1
            , _event:   ext_image_copy_capture_session_v1::Event
            , id:       &usize
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        let cs = &mut state.captures[*id];
        match _event {
            ext_image_copy_capture_session_v1::Event::DmabufDevice { device } => {
                let dev = u64::from_ne_bytes(device[..8].try_into().unwrap());
                cs.dmabuf_device = Some(dev);
            }
            ext_image_copy_capture_session_v1::Event::DmabufFormat { format, modifiers } => {
                let mods: Vec<u64> = modifiers.chunks_exact(8)
                                              .map(|chunk| u64::from_ne_bytes(chunk.try_into().unwrap()))
                                              .collect();
                cs.dmabuf_formats.push((format, mods));
            }
            ext_image_copy_capture_session_v1::Event::ShmFormat { format } => {
                if let WEnum::Value(fmt) = format {
                    cs.shm_format = Some(fmt);
                }
            }
            ext_image_copy_capture_session_v1::Event::BufferSize { width, height } => {
                let new_geom = (width, height);
                if cs.buffer_geometry.is_some() && cs.buffer_geometry != Some(new_geom) {
                    cs.size_changed = true;
                }
                cs.buffer_geometry = Some(new_geom);
            }
            ext_image_copy_capture_session_v1::Event::Done => {
                cs.session_done = true;
            }
            _ => { }
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, usize> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureFrameV1
            , _event:   ext_image_copy_capture_frame_v1::Event
            , id:       &usize
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        let cs = &mut state.captures[*id];
        match _event {
            ext_image_copy_capture_frame_v1::Event::Ready => {
                cs.ready = true;
            }
            ext_image_copy_capture_frame_v1::Event::Failed { reason } => {
                eprintln!("capture {}: frame failed: {:?}", id, reason);
                cs.failed = true;
            }
            _ => { }
        }
    }
}

impl Dispatch<WlBuffer, usize> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &WlBuffer
            , _event:   wl_buffer::Event
            , _id:      &usize
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        // wl_buffer.release is unused by ext-image-copy-capture
    }
}

// Globals and toplevel discovery use () user data (no per-capture routing).

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
            ) { }
}

impl Dispatch<ZwpLinuxBufferParamsV1, usize> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ZwpLinuxBufferParamsV1
            , _event:   zwp_linux_buffer_params_v1::Event
            , id:       &usize
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            zwp_linux_buffer_params_v1::Event::Failed => {
                eprintln!("capture {id}: dmabuf buffer creation failed, will fall back to shm");
                state.captures[*id].dmabuf_buf_failed = true;
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
            ) { }
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
            ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                state.pending_title = Some(title);
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                state.pending_app_id = Some(app_id);
            }
            ext_foreign_toplevel_handle_v1::Event::Done => {
                state.toplevels.push(TopLevelInfo { handle: _proxy.clone()
                                                   , title:  state.pending_title.take()
                                                   , app_id: state.pending_app_id.take()
                                                   });
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
            ) { }

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
            wl_registry::Event::Global { name, interface, version } if interface == "zwp_linux_dmabuf_v1" => {
                state.linux_dmabuf = Some(_proxy.bind::<ZwpLinuxDmabufV1, _, _>(name, version.min(4), _qhandle, ()));
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

// ---------------------------------------------------------------------------
// CaptureManager implementation
// ---------------------------------------------------------------------------

impl CaptureManager {

    /// Connect to the Wayland compositor, discover all toplevel windows
    /// and start capturing each one.
    pub fn new(mode: CaptureMode) -> Result<Self> {
        let connection  = Connection::connect_to_env()?;
        let mut eq      = connection.new_event_queue::<AppState>();
        let mut state   = AppState { toplevels:            Vec::new()
                                   , pending_title:        None
                                   , pending_app_id:       None
                                   , wl_shm:               None
                                   , source_manager:       None
                                   , copy_capture_manager: None
                                   , linux_dmabuf:         None
                                   , captures:             Vec::new()
                                   };

        connection.display().get_registry(&eq.handle(), ());
        eq.roundtrip(&mut state)?;
        eq.roundtrip(&mut state)?;

        if state.toplevels.is_empty() {
            return Err("No toplevels found".into());
        }

        let source_manager = state.source_manager.as_ref()
            .ok_or("No source manager found")?;
        let ccm = state.copy_capture_manager.as_ref()
            .ok_or("No copy capture manager found")?;

        // First pass: create sessions and keep proxy handles.
        let mut sessions: Vec<ExtImageCopyCaptureSessionV1> = Vec::new();
        let toplevel_count = state.toplevels.len();
        for i in 0..toplevel_count {
            state.captures.push(PerCaptureState { buffer_geometry:    None
                                                , shm_format:        None
                                                , dmabuf_formats:    Vec::new()
                                                , dmabuf_device:     None
                                                , session_done:      false
                                                , ready:             false
                                                , failed:            false
                                                , size_changed:      false
                                                , dmabuf_buf_failed: false
                                                });
            let source  = source_manager.create_source(&state.toplevels[i].handle, &eq.handle(), ());
            let session = ccm.create_session( &source
                                            , ext_image_copy_capture_manager_v1::Options::empty()
                                            , &eq.handle()
                                            , i
                                            );
            sessions.push(session);
        }

        // Receive buffer constraints for all sessions.
        // One roundtrip may not suffice for all sessions to send Done.
        loop {
            eq.roundtrip(&mut state)?;
            if state.captures.iter().all(|cs| cs.session_done) { break; }
        }

        // Determine DMA-BUF support.
        let use_dmabuf = matches!(mode, CaptureMode::PreferDmabuf)
                      && state.captures.iter().any(|cs| cs.dmabuf_device.is_some());

        let gbm_dev = if use_dmabuf {
            let dev_t = state.captures.iter()
                .find_map(|cs| cs.dmabuf_device)
                .unwrap();
            let drm_fd = Self::find_drm_node(dev_t)?;
            Some(gbm::Device::new(drm_fd)?)
        } else {
            None
        };

        // Second pass: allocate buffers, create first frame, start capture.
        let mut captures: Vec<WindowCapture> = Vec::new();

        for i in 0..toplevel_count {
            let cs = &state.captures[i];
            let (width, height) = cs.buffer_geometry
                .ok_or_else(|| format!("capture {}: no buffer geometry", i))?;

            let (buffer_mode, dmabuf_bo, fourcc, fd, buffer, stride, size)
                = if use_dmabuf && cs.dmabuf_device.is_some() && !cs.dmabuf_formats.is_empty()
            {
                const DRM_FORMAT_MOD_LINEAR: u64 = 0;

                let (chosen_fmt, modifiers) = &cs.dmabuf_formats[0];
                let mut filtered: Vec<u64> = modifiers.iter()
                    .filter(|&&m| m != DRM_FORMAT_MOD_INVALID)
                    .copied()
                    .collect();

                // Prefer LINEAR modifier if available.
                if filtered.contains(&DRM_FORMAT_MOD_LINEAR) {
                    filtered = vec![DRM_FORMAT_MOD_LINEAR];
                }

                let dev    = gbm_dev.as_ref().unwrap();
                let gbm_bo = dev.create_buffer_object_with_modifiers::<()>( width
                                                                          , height
                                                                          , gbm::Format::try_from(*chosen_fmt)?
                                                                          , filtered.iter()
                                                                                    .map(|&m| gbm::Modifier::from(m))
                                                                          )?;
                let dmabuf_fd  = gbm_bo.fd()?;
                let stride     = gbm_bo.stride();
                let modifier   = gbm_bo.modifier();

                let linux_dmabuf = state.linux_dmabuf.as_ref()
                    .ok_or("No zwp_linux_dmabuf_v1")?;
                let params = linux_dmabuf.create_params(&eq.handle(), i);

                let mod_val: u64 = modifier.into();
                params.add( dmabuf_fd.as_fd(), 0, 0, stride
                          , (mod_val >> 32) as u32
                          , (mod_val & 0xFFFF_FFFF) as u32
                          );
                let buffer = params.create_immed( width as i32, height as i32
                                                , *chosen_fmt
                                                , zwp_linux_buffer_params_v1::Flags::empty()
                                                , &eq.handle(), i
                                                );
                let size = stride * height;

                (BufferMode::Dmabuf, Some(gbm_bo), Some(*chosen_fmt), dmabuf_fd, buffer, stride, size)
            } else {
                let shm_format = cs.shm_format
                    .ok_or_else(|| format!("capture {}: no shm format", i))?;
                let stride = width * 4;
                let size   = stride * height;

                let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
                rustix::fs::ftruncate(&fd, size.into())?;

                let wlshm  = state.wl_shm.as_ref().ok_or("No wl_shm")?;
                let pool   = wlshm.create_pool(fd.as_fd(), size as i32, &eq.handle(), ());
                let buffer = pool.create_buffer( 0, width as i32, height as i32
                                               , stride as i32, shm_format
                                               , &eq.handle(), i
                                               );
                (BufferMode::Shm, None, None, fd, buffer, stride, size)
            };

            let session = sessions.remove(0);

            captures.push(WindowCapture { title:       state.toplevels[i].title.clone()
                                        , app_id:      state.toplevels[i].app_id.clone()
                                        , session
                                        , frame:       None
                                        , buffer
                                        , fd
                                        , buffer_mode
                                        , _dmabuf_bo:  dmabuf_bo
                                        , fourcc
                                        , width
                                        , height
                                        , stride
                                        , size
                                        });
        }

        // Ensure buffers are registered by the compositor before using them.
        // This roundtrip also delivers Failed events for bad dmabuf buffers.
        eq.roundtrip(&mut state)?;

        // Fall back to shm for any capture whose dmabuf buffer was rejected.
        for i in 0..captures.len() {
            if state.captures[i].dmabuf_buf_failed {
                let cs = &state.captures[i];
                let wc = &mut captures[i];

                // The dmabuf buffer proxy is dead (create_immed failed).
                // Do NOT call wc.buffer.destroy() -- the compositor never
                // created this object so sending destroy would be a
                // protocol error. Just drop the proxy silently.
                wc._dmabuf_bo = None;

                let shm_format = cs.shm_format
                    .ok_or_else(|| format!("capture {i}: dmabuf failed and no shm format"))?;
                let stride = wc.width * 4;
                let size   = stride * wc.height;

                let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
                rustix::fs::ftruncate(&fd, size.into())?;

                let wlshm  = state.wl_shm.as_ref().ok_or("No wl_shm")?;
                let pool   = wlshm.create_pool(fd.as_fd(), size as i32, &eq.handle(), ());
                let buffer = pool.create_buffer( 0, wc.width as i32, wc.height as i32
                                               , stride as i32, shm_format
                                               , &eq.handle(), i
                                               );

                wc.fd          = fd;
                wc.buffer      = buffer;
                wc.buffer_mode = BufferMode::Shm;
                wc.fourcc      = None;
                wc.stride      = stride;
                wc.size        = size;

                eprintln!("capture {i}: fell back to shm ({}x{})", wc.width, wc.height);
            }
        }

        // Start the first capture for each window.
        for i in 0..captures.len() {
            let wc    = &mut captures[i];
            let frame = wc.session.create_frame(&eq.handle(), i);
            frame.attach_buffer(&wc.buffer);
            frame.capture();
            wc.frame = Some(frame);
        }

        eq.flush()?;

        Ok(CaptureManager { connection, event_queue: eq, state, captures, _gbm_dev: gbm_dev })
    }

    pub fn connection_fd(&self) -> BorrowedFd<'_> { self.connection.as_fd() }

    pub fn capture_count(&self) -> usize { self.captures.len() }

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

    pub fn is_ready(&self, index: usize) -> bool {
        self.state.captures[index].ready
    }

    pub fn is_failed(&self, index: usize) -> bool {
        self.state.captures[index].failed
    }

    pub fn window(&self, index: usize) -> &WindowCapture {
        &self.captures[index]
    }

    pub fn take_output(&self, index: usize) -> Result<CaptureOutput<'_>> {
        let wc = &self.captures[index];

        match &wc.buffer_mode {
            BufferMode::Shm => {
                let pixels = unsafe {
                    let ptr  = rustix::mm::mmap( std::ptr::null_mut()
                                               , wc.size as usize
                                               , ProtFlags::READ
                                               , MapFlags::SHARED
                                               , &wc.fd, 0
                                               )?;
                    let data = std::slice::from_raw_parts(ptr as *const u8, wc.size as usize);
                    data.to_vec()
                };
                Ok(CaptureOutput::Shm(ShmResult { pixels
                                                , width:  wc.width
                                                , height: wc.height
                                                , stride: wc.stride
                                                }))
            }
            BufferMode::Dmabuf => {
                let bo = wc._dmabuf_bo.as_ref().unwrap();
                Ok(CaptureOutput::Dmabuf(DmabufResult { fd:       wc.fd.as_fd()
                                                      , fourcc:   wc.fourcc.unwrap()
                                                      , modifier: bo.modifier().into()
                                                      , width:    wc.width
                                                      , height:   wc.height
                                                      , stride:   wc.stride
                                                      }))
            }
        }
    }

    pub fn capture_next(&mut self, index: usize) -> Result<()> {
        let cs = &mut self.state.captures[index];
        cs.ready  = false;
        cs.failed = false;

        // If the window was resized, reallocate the buffer.
        if cs.size_changed {
            cs.size_changed = false;
            let (new_w, new_h) = cs.buffer_geometry.unwrap();
            self.reallocate_buffer(index, new_w, new_h)?;
        }

        let wc = &mut self.captures[index];
        if let Some(old_frame) = wc.frame.take() {
            old_frame.destroy();
        }

        let frame = wc.session.create_frame(&self.event_queue.handle(), index);
        frame.attach_buffer(&wc.buffer);
        frame.capture();

        wc.frame = Some(frame);

        self.event_queue.flush()?;
        Ok(())
    }

    fn reallocate_buffer(&mut self, index: usize, width: u32, height: u32) -> Result<()> {
        let wc = &mut self.captures[index];
        let cs = &self.state.captures[index];

        // Destroy old buffer.
        wc.buffer.destroy();

        match &wc.buffer_mode {
            BufferMode::Dmabuf => {
                const DRM_FORMAT_MOD_LINEAR: u64 = 0;

                let (chosen_fmt, modifiers) = &cs.dmabuf_formats[0];
                let mut filtered: Vec<u64> = modifiers.iter()
                    .filter(|&&m| m != DRM_FORMAT_MOD_INVALID)
                    .copied()
                    .collect();
                if filtered.contains(&DRM_FORMAT_MOD_LINEAR) {
                    filtered = vec![DRM_FORMAT_MOD_LINEAR];
                }

                let dev = self._gbm_dev.as_ref().unwrap();
                let gbm_bo = dev.create_buffer_object_with_modifiers::<()>( width
                                                                          , height
                                                                          , gbm::Format::try_from(*chosen_fmt)?
                                                                          , filtered.iter()
                                                                                    .map(|&m| gbm::Modifier::from(m))
                                                                          )?;
                let dmabuf_fd = gbm_bo.fd()?;
                let stride    = gbm_bo.stride();
                let modifier  = gbm_bo.modifier();

                let linux_dmabuf = self.state.linux_dmabuf.as_ref()
                    .ok_or("No zwp_linux_dmabuf_v1")?;
                let params = linux_dmabuf.create_params(&self.event_queue.handle(), index);

                let mod_val: u64 = modifier.into();
                params.add( dmabuf_fd.as_fd(), 0, 0, stride
                          , (mod_val >> 32) as u32
                          , (mod_val & 0xFFFF_FFFF) as u32
                          );
                let buffer = params.create_immed( width as i32, height as i32
                                                , *chosen_fmt
                                                , zwp_linux_buffer_params_v1::Flags::empty()
                                                , &self.event_queue.handle(), index
                                                );
                let size = stride * height;

                wc.fd         = dmabuf_fd;
                wc.buffer     = buffer;
                wc._dmabuf_bo = Some(gbm_bo);
                wc.fourcc     = Some(*chosen_fmt);
                wc.width      = width;
                wc.height     = height;
                wc.stride     = stride;
                wc.size       = size;
            }
            BufferMode::Shm => {
                let shm_format = cs.shm_format
                    .ok_or("no shm format for reallocation")?;
                let stride = width * 4;
                let size   = stride * height;

                let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
                rustix::fs::ftruncate(&fd, size.into())?;

                let wlshm  = self.state.wl_shm.as_ref().ok_or("No wl_shm")?;
                let pool   = wlshm.create_pool(fd.as_fd(), size as i32, &self.event_queue.handle(), ());
                let buffer = pool.create_buffer( 0, width as i32, height as i32
                                               , stride as i32, shm_format
                                               , &self.event_queue.handle(), index
                                               );

                wc.fd     = fd;
                wc.buffer = buffer;
                wc.width  = width;
                wc.height = height;
                wc.stride = stride;
                wc.size   = size;
            }
        }

        eprintln!("capture {index}: buffer reallocated ({width}x{height})");
        Ok(())
    }

    /// Block until capture `index` is ready or has failed.
    pub fn blocking_dispatch_until_ready(&mut self, index: usize) -> Result<()> {
        loop {
            self.event_queue.blocking_dispatch(&mut self.state)?;
            if self.state.captures[index].ready  { return Ok(()); }
            if self.state.captures[index].failed {
                return Err(format!("capture {index}: compositor returned an error").into());
            }
        }
    }

    fn find_drm_node(device: u64) -> Result<OwnedFd> {
        for entry in std::fs::read_dir("/dev/dri")? {
            let entry = entry?;
            let name  = entry.file_name();
            if name.to_str().map_or(false, |n| n.starts_with("renderD")) {
                let stat = rustix::fs::stat(&entry.path())?;
                if stat.st_rdev == device {
                    return Ok(rustix::fs::open( &entry.path()
                                              , rustix::fs::OFlags::RDWR
                                              , rustix::fs::Mode::empty()
                                              )?);
                }
            }
        }
        Err("No matching DRM render node found".into())
    }
}

/// Capture the first available toplevel window (one-shot, shm only).
/// Returns raw pixels in BGRA format.
pub fn capture() -> Result<ShmResult> {
    let mut mgr = CaptureManager::new(CaptureMode::ForceShm)?;

    mgr.blocking_dispatch_until_ready(0)?;

    match mgr.take_output(0)? {
        CaptureOutput::Shm(result) => Ok(result),
        CaptureOutput::Dmabuf(_)   => unreachable!("ForceShm mode should never produce Dmabuf")
    }
}
