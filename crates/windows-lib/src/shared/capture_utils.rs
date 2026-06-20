use std::collections::HashMap;
use std::os::fd::AsRawFd;

use relm4::adw::prelude::*;
use relm4::adw::{glib, gtk};
use tracing::{debug, trace, warn};

use core_lib::ClientId;
use exec_lib::wayland_capture::{CaptureManager, CaptureOutput, ObjectId};

pub fn refresh_captures(
    mgr: &mut CaptureManager,
    display: &gtk::gdk::Display,
    continuous: bool,
) -> HashMap<ClientId, gtk::gdk::Texture> {
    if let Err(e) = mgr.dispatch_pending() {
        warn!("Failed to dispatch pending Wayland events: {e}");
        return HashMap::new();
    }

    mgr.drain_closed();

    if let Err(e) = mgr.drain_new() {
        warn!("Failed to drain new captures: {e}");
    }

    let capture_map: HashMap<ClientId, ObjectId> = mgr
        .capture_ids()
        .into_iter()
        .filter_map(|oid| mgr.client_id(&oid).map(|cid| (cid, oid)))
        .collect();

    let mut textures = HashMap::new();
    let tick_start = std::time::Instant::now();
    let mut ready_count: u32 = 0;
    let mut no_damage_count: u32 = 0;
    let mut total_texture_time = std::time::Duration::ZERO;
    let mut max_latency = std::time::Duration::ZERO;
    let mut total_damage_area: u64 = 0;
    let mut total_buffer_area: u64 = 0;

    for (client_id, obj_id) in &capture_map {
        if mgr.is_failed(obj_id) && continuous {
            trace!("Capture failed for client {client_id}, retrying");
            if let Err(e) = mgr.capture_next(obj_id) {
                warn!("Failed to restart capture for client {client_id}: {e}");
            }
        }

        if !mgr.is_ready(obj_id) {
            continue;
        }

        ready_count += 1;
        if let Some(stats) = mgr.frame_stats(obj_id) {
            if stats.damage_count == 0 {
                no_damage_count += 1;
            }
            total_damage_area += stats.damage_area;
            total_buffer_area += stats.buffer_area;
            if let Some(lat) = stats.latency {
                max_latency = max_latency.max(lat);
            }
        }

        let t0 = std::time::Instant::now();
        let texture = match mgr.take_output(obj_id) {
            Ok(output) => match create_texture(output, display) {
                Ok(tex) => Some(tex),
                Err(e) => {
                    warn!("Failed to create texture for client {client_id}: {e}");
                    None
                }
            },
            Err(e) => {
                warn!("Failed to take capture output for client {client_id}: {e}");
                None
            }
        };
        total_texture_time += t0.elapsed();

        if let Some(texture) = texture {
            textures.insert(*client_id, texture);
            if continuous && let Err(e) = mgr.capture_next(obj_id) {
                warn!("Failed to schedule next capture for client {client_id}: {e}");
            }
        }
    }

    debug!("tick: {:?} total, {} ready, {} textures ({:?} texture_time), {} no_damage, {:?} max_latency, damage_ratio={:.1}%",
           tick_start.elapsed(), ready_count, textures.len(), total_texture_time, no_damage_count, max_latency,
           if total_buffer_area > 0 { (total_damage_area as f64 / total_buffer_area as f64) * 100.0 } else { 0.0 });

    textures
}

/// Convert a [`CaptureOutput`] into a [`gtk::gdk::Texture`].
///
/// For `Dmabuf` output, builds a `DmabufTexture` via the GDK DMA-BUF
/// importer (zero-copy, GPU-side).
/// For `Shm` output, wraps the pixel buffer in a `MemoryTexture`
/// (CPU-side copy).
fn create_texture(
    output: CaptureOutput,
    display: &gtk::gdk::Display,
) -> Result<gtk::gdk::Texture, glib::Error> {
    match output {
        CaptureOutput::Dmabuf(dmabuf) => unsafe {
            gtk::gdk::DmabufTextureBuilder::new()
                .set_display(display)
                .set_width(dmabuf.width)
                .set_height(dmabuf.height)
                .set_fourcc(dmabuf.fourcc)
                .set_modifier(dmabuf.modifier)
                .set_n_planes(1)
                .set_fd(0, dmabuf.fd.as_raw_fd())
                .set_stride(0, dmabuf.stride)
                .set_offset(0, 0)
                .build()
        },
        CaptureOutput::Shm(shm_result) => {
            let bytes = gtk::glib::Bytes::from(&shm_result.pixels);
            Ok(gtk::gdk::MemoryTexture::new(
                shm_result.width.cast_signed(),
                shm_result.height.cast_signed(),
                gtk::gdk::MemoryFormat::B8g8r8a8Premultiplied,
                &bytes,
                shm_result.stride as usize,
            )
            .upcast())
        }
    }
}
