
use std::{cell::RefCell, rc::Rc};
use std::time::Duration;
use std::os::fd::AsRawFd;

use gtk4::glib;
use gtk4::prelude::RootExt;
use gtk4::{self as gtk, gio::{ApplicationFlags, prelude::{ApplicationExt, ApplicationExtManual}}, prelude::GtkWindowExt};

use capture_proto::wayland_capture;
use capture_proto::wayland_capture::{CaptureMode, CaptureOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wl_capture_session = Rc::new(RefCell::new(wayland_capture::CaptureSession::new(CaptureMode::PreferDmabuf)?));

    let app = gtk::Application::new(Some("com.github.hyprshell.CaptureProto"), ApplicationFlags::default());

    let wl_capture_session_clone = wl_capture_session.clone();

    app.connect_activate(move |app| {
        let picture = gtk::Picture::new();
        let window  = gtk::ApplicationWindow::builder().application(app)
                                                       .title("GTK4 Capture prototype")
                                                       .child(&picture)
                                                       .build();

        let wl_cacpture_session_inner = wl_capture_session_clone.clone();
        let picture_inner             = picture.clone();
        let display                   = window.display();

        gtk::glib::timeout_add_local(Duration::from_millis(16), move || {
            let mut s = wl_cacpture_session_inner.borrow_mut();
            let _ = s.dispatch_pending();
            if s.is_ready() {
                match s.take_output() {
                    Ok(CaptureOutput::Shm(result)) => {
                        let bytes = gtk::glib::Bytes::from(&result.pixels);
                        let texture = gtk::gdk::MemoryTexture::new( result.width as i32
                                                                  , result.height as i32
                                                                  , gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied
                                                                  , &bytes
                                                                  , result.stride as usize
                                                                  );
                        picture_inner.set_paintable(Some(&texture));
                    }
                    Ok(CaptureOutput::Dmabuf(dmabuf)) => {
                        unsafe {
                            let texture = gtk::gdk::DmabufTextureBuilder::new().set_display(&display)
                                                                               .set_width(dmabuf.width)
                                                                               .set_height(dmabuf.height)
                                                                               .set_fourcc(dmabuf.fourcc)
                                                                               .set_modifier(dmabuf.modifier)
                                                                               .set_n_planes(1)
                                                                               .set_fd(0, dmabuf.fd.as_raw_fd())
                                                                               .set_stride(0, dmabuf.stride)
                                                                               .set_offset(0, 0)
                                                                               .build();
                            match texture {
                                Ok(t) => picture_inner.set_paintable(Some(&t)),
                                Err(e) => eprint!("Failed to build DMA-BUF texture: {e}"),
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to take output: {e}"),
                }

                match s.capture_next() {
                    Ok(_)  => {}
                    Err(e) => eprintln!("Failed to capture next frame: {e}"),
                }
            }
            glib::ControlFlow::Continue
        });

        window.present();

    });

    app.run();
    Ok(())
}
