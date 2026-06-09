
use std::collections::{HashMap, VecDeque};
use std::os::fd::BorrowedFd;
use std::os::fd::{AsFd, OwnedFd};

use rustix::fs::MemfdFlags;
use rustix::mm::{MapFlags, ProtFlags};

use wayland_client::{Proxy, WEnum};
use wayland_client::backend::{ObjectId, WaylandError};
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
                         , _retired_bos:    VecDeque<(gbm::BufferObject<()>, OwnedFd)>
                         , fourcc:          Option<u32>
                         , width:           u32
                         , height:          u32
                         , stride:          u32
                         , size:            u32
                         }

/// Manages capture sessions for all toplevel windows sharing a single
/// Wayland connection and event queue.
pub struct CaptureManager { connection:       Connection
                          , event_queue:      EventQueue<AppState>
                          , captures:         HashMap<ObjectId, WindowCapture>
                          , state:            AppState // state needs to be released after captures to avoid invalid gbm device pointer
                          , pending_sessions: HashMap<ObjectId, ExtImageCopyCaptureSessionV1>
                          , use_dmabuf:       bool
                          }

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Per-capture event state, stored inside AppState so Dispatch handlers
/// can route events by capture index.
struct PerCaptureState { buffer_geometry:    Option<(u32, u32)>
                       , shm_format:         Option<wl_shm::Format>
                       , dmabuf_formats:     Vec<(u32, Vec<u64>)>
                       , session_done:       bool
                       , ready:              bool
                       , failed:             bool
                       , size_changed_at:    Option<std::time::Instant>
                       , dmabuf_buf_failed:  bool
                       }

#[derive(Default)]
struct PendingToplevelProps { title:  Option<String>
                            , app_id: Option<String>
                            }

struct TopLevelInfo { handle: ExtForeignToplevelHandleV1
                    , title:  Option<String>
                    , app_id: Option<String>
                    }

struct AppState { toplevels:            Vec<TopLevelInfo>
                , pending_props:        HashMap<ObjectId, PendingToplevelProps>
                // globals
                , wl_shm:               Option<WlShm>
                , source_manager:       Option<ExtForeignToplevelImageCaptureSourceManagerV1>
                , copy_capture_manager: Option<ExtImageCopyCaptureManagerV1>
                , linux_dmabuf:         Option<ZwpLinuxDmabufV1>
                // per-capture state (indexed by capture id)
                , captures:             HashMap<ObjectId, PerCaptureState>
                , closed_ids:           Vec<ObjectId>
                , gbm_dev:              Option<gbm::Device<OwnedFd>>
                }

// ---------------------------------------------------------------------------
// Dispatch implementations
// ---------------------------------------------------------------------------

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

impl Dispatch<ExtImageCopyCaptureSessionV1, ObjectId> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureSessionV1
            , _event:   ext_image_copy_capture_session_v1::Event
            , id:       &ObjectId
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        let Some(cs) = state.captures.get_mut(id) else { return; };
        match _event {
            ext_image_copy_capture_session_v1::Event::DmabufDevice { device } => {
                if state.gbm_dev.is_none() {
                    let dev = u64::from_ne_bytes(device[..8].try_into().unwrap());
                    state.gbm_dev = match find_drm_node(dev) {
                        Ok(drm_fd) => {
                            match gbm::Device::new(drm_fd) {
                                Ok(d)  => Some(d),
                                Err(e) => {
                                    eprintln!("failed to create gbm device: {e}");
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("failed to find drm node: {e}");
                            None
                        }
                    }
                }
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
                    cs.size_changed_at = Some(std::time::Instant::now());
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

impl Dispatch<ExtImageCopyCaptureFrameV1, ObjectId> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ExtImageCopyCaptureFrameV1
            , _event:   ext_image_copy_capture_frame_v1::Event
            , id:       &ObjectId
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        let Some(cs) = state.captures.get_mut(id) else { return; };
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

impl Dispatch<WlBuffer, ObjectId> for AppState {
    fn event( _state:   &mut AppState
            , _proxy:   &WlBuffer
            , _event:   wl_buffer::Event
            , _id:      &ObjectId
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

impl Dispatch<ZwpLinuxBufferParamsV1, ObjectId> for AppState {
    fn event( state:    &mut AppState
            , _proxy:   &ZwpLinuxBufferParamsV1
            , _event:   zwp_linux_buffer_params_v1::Event
            , id:       &ObjectId
            , _conn:    &wayland_client::Connection
            , _qhandle: &wayland_client::QueueHandle<AppState>
            )
    {
        match _event {
            zwp_linux_buffer_params_v1::Event::Failed => {
                eprintln!("capture {id}: dmabuf buffer creation failed, will fall back to shm");
                state.captures.get_mut(id).map(|c| c.dmabuf_buf_failed = true);
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
                state.pending_props.entry(_proxy.id()).or_default().title = Some(title);
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                state.pending_props.entry(_proxy.id()).or_default().app_id = Some(app_id);
            }
            ext_foreign_toplevel_handle_v1::Event::Done => {
                let id    = _proxy.id();
                let props = state.pending_props.remove(&id).unwrap_or_default();
                if let Some(existing) = state.toplevels.iter_mut().find(|tl| tl.handle.id() == id) {
                    if let Some(title) = props.title {
                        existing.title = Some(title);
                    }
                    if let Some(app_id) = props.app_id {
                        existing.app_id = Some(app_id);
                    }
                } else {
                    state.toplevels.push(TopLevelInfo { handle: _proxy.clone()
                                                      , title:  props.title
                                                      , app_id: props.app_id
                                                      });
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Closed => {
                let id = _proxy.id();
                state.captures.remove(&id);
                state.toplevels.retain(|tl| tl.handle.id() != id);
                state.closed_ids.push(id);
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
        let connection      = Connection::connect_to_env()?;
        let mut event_queue = connection.new_event_queue::<AppState>();
        let mut state       = AppState { toplevels:            Vec::new()
                                       , pending_props:        HashMap::new()
                                       , wl_shm:               None
                                       , source_manager:       None
                                       , copy_capture_manager: None
                                       , linux_dmabuf:         None
                                       , captures:             HashMap::new()
                                       , closed_ids:           Vec::new()
                                       , gbm_dev:              None
                                       };

        connection.display().get_registry(&event_queue.handle(), ());
        event_queue.roundtrip(&mut state)?;
        event_queue.roundtrip(&mut state)?;

        // Determine DMA-BUF support.
        let use_dmabuf = matches!(mode, CaptureMode::PreferDmabuf);

        Ok(CaptureManager { connection
                          , event_queue
                          , state
                          , captures:         HashMap::new()
                          , use_dmabuf
                          , pending_sessions: HashMap::new()
                          })
    }

    pub fn connection_fd(&self) -> BorrowedFd<'_> { self.connection.as_fd() }

    pub fn capture_count(&self) -> usize { self.captures.len() }

    pub fn capture_ids(&self) -> Vec<ObjectId> {
        self.captures.keys().cloned().collect()
    }

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

    pub fn has_capture(&self, index: &ObjectId) -> bool {
        self.captures.contains_key(index) && self.state.captures.contains_key(index)
    }

    pub fn is_ready(&self, index: &ObjectId) -> bool {
        self.state.captures.get(index).is_some_and(|cs| cs.ready)
    }

    pub fn is_failed(&self, index: &ObjectId) -> bool {
        self.state.captures.get(index).is_some_and(|cs| cs.failed)
    }

    pub fn window(&self, index: &ObjectId) -> &WindowCapture {
        &self.captures[index]
    }

    pub fn take_output(&self, index: &ObjectId) -> Result<CaptureOutput<'_>> {
        let wc = &self.captures[index];

        match &wc.buffer_mode {
            BufferMode::Shm => {
                let (mmapped_ptr, pixels) = unsafe {
                    let mmapped_ptr  = rustix::mm::mmap( std::ptr::null_mut()
                                                       , wc.size as usize
                                                       , ProtFlags::READ
                                                       , MapFlags::SHARED
                                                       , &wc.fd, 0
                                                       )?;
                    let data = std::slice::from_raw_parts(mmapped_ptr as *const u8, wc.size as usize);
                    (mmapped_ptr, data.to_vec())
                };
                unsafe {
                    rustix::mm::munmap(mmapped_ptr, wc.size as usize)?; // release previously allocated bac
                }
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

    pub fn capture_next(&mut self, index: &ObjectId) -> Result<()> {
        let realloc_spec = {
            let cs = self.state.captures.get_mut(index).unwrap();
            // If the window was resized, reallocate the buffer.
            if let Some(t) = cs.size_changed_at {
                if t.elapsed() > std::time::Duration::from_millis(200) {
                    cs.size_changed_at = None;
                    Some(cs.buffer_geometry.unwrap())
                } else {
                    return Ok(())
                }
            } else {
                None
            }
        };

        if let Some((new_w, new_h)) = realloc_spec {
            self.reallocate_buffer(index, new_w, new_h)?;
        }

        let cs    = self.state.captures.get_mut(index).unwrap();
        cs.ready  = false;
        cs.failed = false;

        let wc = self.captures.get_mut(index).unwrap();
        if let Some(old_frame) = wc.frame.take() {
            old_frame.destroy();
        }

        let frame = wc.session.create_frame(&self.event_queue.handle(), index.clone());
        frame.attach_buffer(&wc.buffer);
        frame.capture();

        wc.frame = Some(frame);

        self.event_queue.flush()?;
        Ok(())
    }

    pub fn drain_new(&mut self) -> Result<Vec<ObjectId>> {
        self.pending_sessions.extend(Self::create_sessions(&mut self.state, &mut self.event_queue)?);

        let ready_ids: Vec<ObjectId> = self.pending_sessions.keys().filter(|id| self.state.captures[id].session_done).cloned().collect();

        let mut ready_sessions: HashMap<ObjectId, ExtImageCopyCaptureSessionV1> = HashMap::new();
        for id in ready_ids {
            if let Some(s) = self.pending_sessions.remove(&id) {
                ready_sessions.insert(id, s);
            }
        }

        let mut new_captures = Self::allocate_capture(&self.state, &mut self.event_queue, ready_sessions.clone(), self.use_dmabuf, &self.state.gbm_dev)?;

        for (id, s) in ready_sessions {
            if !new_captures.contains_key(&id) {
                self.pending_sessions.insert(id, s);
            }
        }

        Self::start_frames(&mut new_captures, &mut self.event_queue)?;

        let new_ids: Vec<ObjectId> = new_captures.keys().cloned().collect();

        self.captures.extend(new_captures);

        Ok(new_ids)
    }

    pub fn drain_closed(&mut self) -> Vec<ObjectId> {
        let ids: Vec<ObjectId> = self.state.closed_ids.drain(..).collect();
        for id in &ids {
            if let Some(wc) = self.captures.remove(id) {
                wc.session.destroy();
                if let Some(frame) = wc.frame {
                    frame.destroy();
                }
                wc.buffer.destroy();
            }
        }
        ids
    }

    /// Block until capture `index` is ready or has failed.
    pub fn blocking_dispatch_until_ready(&mut self, index: &ObjectId) -> Result<()> {
        loop {
            self.event_queue.blocking_dispatch(&mut self.state)?;
            if self.state.captures[index].ready  { return Ok(()); }
            if self.state.captures[index].failed {
                return Err(format!("capture {index}: compositor returned an error").into());
            }
        }
    }

    fn create_sessions( state:       &mut AppState
                      , event_queue: &mut EventQueue<AppState>
                      ) -> Result<HashMap<ObjectId, ExtImageCopyCaptureSessionV1>>
    {
        let mut pending_sessions: HashMap<ObjectId, ExtImageCopyCaptureSessionV1> = HashMap::new();
        let source_manager                                                        = state.source_manager.as_ref().ok_or("No source manager found")?;
        let ccm                                                                   = state.copy_capture_manager.as_ref().ok_or("No copy capture manager found")?;

        let toplevel_ids: Vec<(ObjectId, ExtForeignToplevelHandleV1)> = state.toplevels.iter()
                                                                                       .filter(|tl| state.captures.get(&tl.handle.id()).is_none())
                                                                                       .map(|tl| (tl.handle.id(), tl.handle.clone()))
                                                                                       .collect();

        for (id, handle) in &toplevel_ids {
            state.captures.insert(id.clone(), PerCaptureState { buffer_geometry:   None
                                                              , shm_format:        None
                                                              , dmabuf_formats:    Vec::new()
                                                              , session_done:      false
                                                              , ready:             false
                                                              , failed:            false
                                                              , size_changed_at:   None
                                                              , dmabuf_buf_failed: false
                                                              });
            let source  = source_manager.create_source(handle, &event_queue.handle(), ());
            let session = ccm.create_session( &source
                                            , ext_image_copy_capture_manager_v1::Options::empty()
                                            , &event_queue.handle()
                                            , id.clone()
                                            );
            pending_sessions.insert(id.clone(), session);
        }

        Ok(pending_sessions)
    }

    fn allocate_capture( state:       &AppState
                       , event_queue: &mut EventQueue<AppState>
                       , sessions:    HashMap<ObjectId, ExtImageCopyCaptureSessionV1>
                       , use_dmabuf:  bool
                       , gbm_dev:     &Option<gbm::Device<OwnedFd>>
                       ) -> Result<HashMap<ObjectId, WindowCapture>>
    {
        let mut window_captures: HashMap<ObjectId, WindowCapture> = HashMap::new();

        // Build a quick lookup for title/app_id by ObjectId.
        let toplevel_info: HashMap<ObjectId, (Option<String>, Option<String>)> = state.toplevels.iter()
                                                                                                .map(|tl| (tl.handle.id(), (tl.title.clone(), tl.app_id.clone())))
                                                                                                .collect();

        for (id, session) in sessions {
            let cs = &state.captures[&id];
            let (width, height) = cs.buffer_geometry
                .ok_or_else(|| format!("capture {}: no buffer geometry", id))?;

            let (buffer_mode, dmabuf_bo, fourcc, fd, buffer, stride, size) = if use_dmabuf  && !cs.dmabuf_formats.is_empty()
            {

                // If gbm_dev is not set, it means, we still need to wait for dispatchers to initialize the GPU device.
                if !state.gbm_dev.is_some() { continue; }

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
                let params = linux_dmabuf.create_params(&event_queue.handle(), id.clone());

                let mod_val: u64 = modifier.into();
                params.add( dmabuf_fd.as_fd(), 0, 0, stride
                          , (mod_val >> 32) as u32
                          , (mod_val & 0xFFFF_FFFF) as u32
                          );
                let buffer = params.create_immed( width as i32, height as i32
                                                , *chosen_fmt
                                                , zwp_linux_buffer_params_v1::Flags::empty()
                                                , &event_queue.handle(), id.clone()
                                                );
                let size = stride * height;

                (BufferMode::Dmabuf, Some(gbm_bo), Some(*chosen_fmt), dmabuf_fd, buffer, stride, size)
            } else {
                let shm_format = cs.shm_format
                    .ok_or_else(|| format!("capture {}: no shm format", id))?;
                let stride = width * 4;
                let size   = stride * height;

                let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
                rustix::fs::ftruncate(&fd, size.into())?;

                let wlshm  = state.wl_shm.as_ref().ok_or("No wl_shm")?;
                let pool   = wlshm.create_pool(fd.as_fd(), size as i32, &event_queue.handle(), ());
                let buffer = pool.create_buffer( 0, width as i32, height as i32
                                               , stride as i32, shm_format
                                               , &event_queue.handle(), id.clone()
                                               );
                (BufferMode::Shm, None, None, fd, buffer, stride, size)
            };

            let (title, app_id) = toplevel_info.get(&id)
                                               .map(|(t, a)| (t.clone(), a.clone()))
                                               .unwrap_or((None, None));

            window_captures.insert(id, WindowCapture { title
                                                     , app_id
                                                     , session
                                                     , frame:        None
                                                     , buffer
                                                     , fd
                                                     , buffer_mode
                                                     , _dmabuf_bo:   dmabuf_bo
                                                     , _retired_bos: VecDeque::new()
                                                     , fourcc
                                                     , width
                                                     , height
                                                     , stride
                                                     , size
                                                     });
        }

        Ok(window_captures)
    }

    fn start_frames( window_captures: &mut HashMap<ObjectId, WindowCapture>
                   , event_queue:     &mut EventQueue<AppState>
                   ) -> Result<()>
    {
        // Start the first capture for each window.
        for (id, wc) in window_captures.iter_mut() {
            let frame = wc.session.create_frame(&event_queue.handle(), id.clone());
            frame.attach_buffer(&wc.buffer);
            frame.capture();
            wc.frame = Some(frame);
        }

        event_queue.flush()?;
        Ok(())
    }

    fn reallocate_buffer(&mut self, index: &ObjectId, width: u32, height: u32) -> Result<()> {
        let wc = &mut self.captures.get_mut(index).unwrap();
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

                let dev = self.state.gbm_dev.as_ref().unwrap();
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
                let params = linux_dmabuf.create_params(&self.event_queue.handle(), index.clone());

                let mod_val: u64 = modifier.into();
                params.add( dmabuf_fd.as_fd(), 0, 0, stride
                          , (mod_val >> 32) as u32
                          , (mod_val & 0xFFFF_FFFF) as u32
                          );
                let buffer = params.create_immed( width as i32, height as i32
                                                , *chosen_fmt
                                                , zwp_linux_buffer_params_v1::Flags::empty()
                                                , &self.event_queue.handle(), index.clone()
                                                );
                if let Some(old_bo) = wc._dmabuf_bo.take() {
                    let old_fd = std::mem::replace(&mut wc.fd, dmabuf_fd);
                    wc._retired_bos.push_back((old_bo, old_fd));
                    while wc._retired_bos.len() > 3 {
                        wc._retired_bos.pop_front();
                    }
                }

                wc._dmabuf_bo = Some(gbm_bo);
                wc.buffer     = buffer;
                wc.fourcc     = Some(*chosen_fmt);
                wc.width      = width;
                wc.height     = height;
                wc.stride     = stride;
                wc.size       = stride * height;
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
                                               , &self.event_queue.handle(), index.clone()
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

}

/// Capture the first available toplevel window (one-shot, shm only).
/// Returns raw pixels in BGRA format.
pub fn capture() -> Result<ShmResult> {
    let mut mgr = CaptureManager::new(CaptureMode::ForceShm)?;

    let id = mgr.state.toplevels.first().unwrap().handle.id();
    mgr.blocking_dispatch_until_ready(&id)?;

    match mgr.take_output(&id)? {
        CaptureOutput::Shm(result) => Ok(result),
        CaptureOutput::Dmabuf(_)   => unreachable!("ForceShm mode should never produce Dmabuf")
    }
}
