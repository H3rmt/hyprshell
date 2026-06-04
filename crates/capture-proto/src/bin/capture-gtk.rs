
use std::{cell::RefCell, rc::Rc};
use std::time::Duration;
use std::os::fd::AsRawFd;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk, gio::ApplicationFlags};

use capture_proto::wayland_capture;
use capture_proto::wayland_capture::{CaptureMode, CaptureOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Rc::new(RefCell::new(wayland_capture::CaptureManager::new(CaptureMode::PreferDmabuf)?));

    let app = gtk::Application::new(Some("com.github.hyprshell.CaptureProto"), ApplicationFlags::default());

    let manager_clone = manager.clone();

    app.connect_activate(move |app| {
        let count = manager_clone.borrow().capture_count();

        // One Picture widget per captured window.
        let pictures: Vec<gtk::Picture> = (0..count).map(|_| {
            let pic = gtk::Picture::new();
            pic.set_content_fit(gtk::ContentFit::Contain);
            pic.set_size_request(320, 180);
            pic
        }).collect();

        let flow_box = gtk::FlowBox::builder()
            .homogeneous(true)
            .max_children_per_line(4)
            .min_children_per_line(2)
            .selection_mode(gtk::SelectionMode::None)
            .build();

        for (i, pic) in pictures.iter().enumerate() {
            let mgr  = manager_clone.borrow();
            let wc   = mgr.window(i);
            let label = format!("{} -- {}", wc.app_id.as_deref().unwrap_or("?")
                                         , wc.title.as_deref().unwrap_or("?"));

            let overlay_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let lbl = gtk::Label::new(Some(&label));
            lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            overlay_box.append(pic);
            overlay_box.append(&lbl);

            flow_box.insert(&overlay_box, -1);
        }

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&flow_box)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("GTK4 Multi-Capture prototype")
            .child(&scrolled)
            .default_width(1200)
            .default_height(800)
            .build();

        let display  = gtk4::prelude::RootExt::display(&window);
        let mgr_inner = manager_clone.clone();
        let pics      = pictures.clone();

        gtk::glib::timeout_add_local(Duration::from_millis(100), move || {
            let mut mgr = mgr_inner.borrow_mut();
            let _ = mgr.dispatch_pending();

            for i in 0..pics.len() {
                if !mgr.is_ready(i) { continue; }

                match mgr.take_output(i) {
                    Ok(CaptureOutput::Shm(result)) => {
                        let bytes = gtk::glib::Bytes::from(&result.pixels);
                        let texture = gtk::gdk::MemoryTexture::new( result.width as i32
                                                                  , result.height as i32
                                                                  , gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied
                                                                  , &bytes
                                                                  , result.stride as usize
                                                                  );
                        pics[i].set_paintable(Some(&texture));
                    }
                    Ok(CaptureOutput::Dmabuf(dmabuf)) => {
                        unsafe {
                            let texture = gtk::gdk::DmabufTextureBuilder::new()
                                .set_display(&display)
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
                                Ok(t)  => pics[i].set_paintable(Some(&t)),
                                Err(e) => eprint!("Failed to build DMA-BUF texture: {e}"),
                            }
                        }
                    }
                    Err(e) => eprintln!("capture {i}: take_output failed: {e}"),
                }

                match mgr.capture_next(i) {
                    Ok(_)  => {}
                    Err(e) => eprintln!("capture {i}: capture_next failed: {e}"),
                }
            }
            glib::ControlFlow::Continue
        });

        window.present();
    });

    app.run();
    Ok(())
}
