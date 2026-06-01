
use std::os::fd::AsFd;

use rustix::fs::MemfdFlags;
use rustix::mm::{MapFlags, ProtFlags};

use wayland_client::WEnum;
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

#[derive(Debug)]
struct TopLevelInfo { handle: ExtForeignToplevelHandleV1
                    , title:  Option<String>
                    , app_id: Option<String>
                    }

struct AppState { toplevels:            Vec<TopLevelInfo>
                , pending_title:        Option<String>
                , pending_app_id:       Option<String>
                // Globals
                , wl_shm:               Option<WlShm>
                , source_manager:       Option<ExtForeignToplevelImageCaptureSourceManagerV1>
                , copy_capture_manager: Option<ExtImageCopyCaptureManagerV1>
                // capture constraints
                , buffer_geometry:      Option<(u32, u32)>
                , shm_format:           Option<wl_shm::Format>
                , session_done:         bool
                , ready:                bool
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
            // wl_registry::Event::GlobalRemove { name } => {
            // }
            _ => { }

        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection                       = Connection::connect_to_env()?;
    let mut equeue: EventQueue<AppState> = connection.new_event_queue();
    let mut state                        = AppState { toplevels:            Vec::new()
                                                    , pending_title:        None
                                                    , pending_app_id:       None
                                                    , wl_shm:               None
                                                    , source_manager:       None
                                                    , copy_capture_manager: None
                                                    , buffer_geometry:      None
                                                    , shm_format:           None
                                                    , session_done:         false
                                                    , ready:                false
                                                    , buffer_released:      false
                                                    };

    connection.display().get_registry(&equeue.handle(), ());

    equeue.roundtrip(&mut state)?;
    equeue.roundtrip(&mut state)?;

    let mut source  = None;
    let mut session = None;
    let mut buffer  = None;

    if let Some(toplevel) = state.toplevels.first() {
        println!("First toplevel: {:?}", toplevel);
        if let Some(source_manager) = &state.source_manager {
            source = Some(source_manager.create_source(&toplevel.handle, &equeue.handle(), ()));
        } else {
            println!("No source manager found");
        }
    } else {
        println!("No toplevels found");
    }

    if let Some(copy_capture_manager) = &state.copy_capture_manager {
        if let Some(src) = &source {
            session = Some(copy_capture_manager.create_session(&src, ext_image_copy_capture_manager_v1::Options::empty(), &equeue.handle(), ()));
        } else {
            println!("No source found");
        }
    } else {
        println!("No copy capture manager found");
    }

    equeue.roundtrip(&mut state)?;

    if let Some(s) = &session {
        let fd = rustix::fs::memfd_create("capture", MemfdFlags::CLOEXEC)?;
        let mut size = 0;
        let mut width = 0;
        let mut height = 0;

        if let Some((w, h)) = state.buffer_geometry {
            width = w;
            height = h;
            let stride = width * 4;
            size = stride * height;
            rustix::fs::ftruncate(&fd, size.into())?;
            if let Some(wlshm) = &state.wl_shm {
                let pool = wlshm.create_pool(fd.as_fd(), size as i32, &equeue.handle(), ());
                if state.shm_format.is_some() {
                    buffer = Some(pool.create_buffer(0, width as i32, height as i32, stride as i32, state.shm_format.unwrap(), &equeue.handle(), ()));
                } else {
                    println!("No shm format received");
                }
            } else {
                println!("No wl_shm found");
            }
        } else {
            println!("No buffer geometry received");
        }

        let frame = s.create_frame(&equeue.handle(), ());
        if buffer.is_some() {
            frame.attach_buffer(&buffer.unwrap());
            frame.capture();
        }

        equeue.roundtrip(&mut state)?;
        equeue.roundtrip(&mut state)?;

        if state.ready {
            println!("Capture successful");
            unsafe {
                let ptr             = rustix::mm::mmap(std::ptr::null_mut(), size as usize, ProtFlags::READ, MapFlags::SHARED, fd, 0)?;
                let data            = std::slice::from_raw_parts(ptr as *const u8, size as usize);
                let pixels: Vec<u8> = data.chunks_exact(4).flat_map(|b| [b[2], b[1], b[0], b[3]]).collect();
                image::RgbaImage::from_raw(width, height, pixels).unwrap().save("/tmp/capture-proto-test.png")?;
            }
        } else {
            println!("Capture failed...");
        }

    } else {
        println!("No session created");
    }

    Ok(())
}
