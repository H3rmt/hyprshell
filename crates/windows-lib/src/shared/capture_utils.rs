
use std::os::fd::AsRawFd;
use std::collections::HashMap;

use relm4::adw::{glib, gtk};
use relm4::adw::prelude::*;

use core_lib::ClientId;
use exec_lib::wayland_capture::{CaptureManager, CaptureOutput, ObjectId};

pub fn refresh_captures(mgr: &mut CaptureManager, display: &gtk::gdk::Display) -> HashMap<ClientId, gtk::gdk::Texture> {
    let _ = mgr.dispatch_pending();

    mgr.drain_closed();

    let _ = mgr.drain_new();

    let capture_map: HashMap<ClientId, ObjectId> = mgr.capture_ids()
                                                      .into_iter()
                                                      .filter_map(|oid| mgr.client_id(&oid).map(|cid| (cid, oid)))
                                                      .collect();

    let mut textures = HashMap::new();

    for (client_id, obj_id) in &capture_map {
        if mgr.is_failed(&obj_id) {
            // TODO: gérer l'erreur
            let _ = mgr.capture_next(&obj_id);
        }

        if !mgr.is_ready(&obj_id) { continue; }

        // TODO: gérer les erreurs
        let texture = match mgr.take_output(&obj_id) {
            Ok(output) => create_texture(output, display).ok(),
            Err(_)     => None,
        };

        if let Some(texture) = texture {
            textures.insert(*client_id, texture);
            let _ = mgr.capture_next(&obj_id);
        }

    }

    textures
}

fn create_texture(output: CaptureOutput, display: &gtk::gdk::Display) -> Result<gtk::gdk::Texture, glib::Error> {
    match output {
        CaptureOutput::Dmabuf(dmabuf) => {
            unsafe {
                gtk::gdk::DmabufTextureBuilder::new().set_display(display)
                                                     .set_width(dmabuf.width)
                                                     .set_height(dmabuf.height)
                                                     .set_fourcc(dmabuf.fourcc)
                                                     .set_modifier(dmabuf.modifier)
                                                     .set_n_planes(1)
                                                     .set_fd(0, dmabuf.fd.as_raw_fd())
                                                     .set_stride(0, dmabuf.stride)
                                                     .set_offset(0, 0)
                                                     .build()
            }
        }
        CaptureOutput::Shm(shm_result) => {
            let bytes = gtk::glib::Bytes::from(&shm_result.pixels);
            Ok(gtk::gdk::MemoryTexture::new( shm_result.width as i32
                                           , shm_result.height as i32
                                           , gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied
                                           , &bytes
                                           , shm_result.stride as usize
                                           ).upcast())
        }
    }
}

