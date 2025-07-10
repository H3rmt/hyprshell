use std::io::Read;
use wl_clipboard_rs::paste::{MimeType, Seat, get_contents_channel};

pub fn test() {
    let rx = get_contents_channel(Seat::Unspecified, MimeType::Any).unwrap();
    loop {
        match rx.recv() {
            Ok(Ok((mut pipe, mime_type))) => {
                println!("Got data of the {} MIME type", &mime_type);
                let mut contents = vec![];
                let _ = pipe.read_to_end(&mut contents);
                println!(
                    "Read {} bytes of data: {}",
                    contents.len(),
                    String::from_utf8_lossy(&contents)
                );
            }
            _ => (),
        };
    }
}
