
use std::{cell::RefCell, rc::Rc};
use std::time::Duration;

use gtk4::glib;
use gtk4::{self as gtk, gio::{ApplicationFlags, prelude::{ApplicationExt, ApplicationExtManual}}, prelude::GtkWindowExt};

use capture_proto::wayland_capture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wl_capture_session = Rc::new(RefCell::new(wayland_capture::CaptureSession::new()?));

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

        gtk::glib::timeout_add_local(Duration::from_millis(16), move || {
            let mut s = wl_cacpture_session_inner.borrow_mut();
            let _ = s.dispatch_pending();
            if s.is_ready() {
                if let Ok(result) = s.take_pixels() {
                    let bytes = gtk::glib::Bytes::from(&result.pixels);
                    let texture = gtk::gdk::MemoryTexture::new( result.width as i32
                                                              , result.height as i32
                                                              , gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied
                                                              , &bytes
                                                              , result.stride as usize
                                                              );
                    picture_inner.set_paintable(Some(&texture));
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
