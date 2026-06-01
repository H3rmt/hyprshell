
use gtk4::{self as gtk, gio::{ApplicationFlags, prelude::{ApplicationExt, ApplicationExtManual}}, prelude::GtkWindowExt};

use capture_proto::wayland_capture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = wayland_capture::capture()?;

    let app = gtk::Application::new(Some("com.github.hyprshell.CaptureProto"), ApplicationFlags::default());

    app.connect_activate(move |app| {
        let bytes = gtk::glib::Bytes::from(&result.pixels);
        let texture = gtk::gdk::MemoryTexture::new( result.width as i32
                                                  , result.height as i32
                                                  , gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied
                                                  , &bytes
                                                  , result.stride as usize
                                                  );
        let picture = gtk::Picture::for_paintable(&texture);
        let window  = gtk::ApplicationWindow::builder().application(app)
                                                       .title("GTK4 Capture prototype")
                                                       .child(&picture)
                                                       .build();
        window.present();
    });

    app.run();

    Ok(())
}
