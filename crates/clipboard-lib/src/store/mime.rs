use tracing::trace;

static MIME_TYPES_PRIO: &[&str] = &[
    "image/png",
    "image/jpg",
    "image/jpeg",
    "image/webp",
    "image/gif",
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "STRING",
    "TEXT",
    "text/*",
];

static MIME_TYPES_IMAGES_PRIO: &[&str] = &[
    "image/png",
    "image/jpg",
    "image/jpeg",
    "image/webp",
    "image/gif",
];

pub fn get_preferred_mime(mime_types: &[String]) -> Option<String> {
    // Find by priority
    for (index, mime) in MIME_TYPES_PRIO.iter().enumerate() {
        if mime.ends_with("/*") {
            let prefix = &mime[..mime.len() - 1];
            if let Some(mt) = mime_types.iter().find(|x| x.starts_with(prefix)) {
                trace!("Chosen MIME type: {mt:?} from prio({index}), by prefix {prefix}*");
                return Some(mt.clone());
            }
        } else if let Some(mt) = mime_types.iter().find(|x| x.as_str() == *mime) {
            trace!("Chosen MIME type: {mt:?} from prio({index}), by exact match {mime}");
            return Some(mt.clone());
        }
    }
    None
}

pub fn filer_mimes(mime_types: &mut Vec<String>) {
    let count = mime_types.len();

    // remove audio
    mime_types.retain(|mt| !mt.starts_with("audio/"));

    // Retain only one image/ MIME type.
    let image_mime = MIME_TYPES_IMAGES_PRIO
        .iter()
        .find(|preferred| mime_types.contains(&(**preferred).to_string()))
        .map(|&preferred| preferred.to_string())
        .or_else(|| {
            mime_types
                .iter()
                .find(|mt| mt.starts_with("image/"))
                .cloned()
        });

    mime_types.retain(|mt| {
        if mt.starts_with("image/") {
            image_mime.as_ref() == Some(mt)
        } else {
            true
        }
    });

    trace!(
        "Available MIME types: {:?}, removed {} elements",
        mime_types,
        count - mime_types.len()
    );
}
