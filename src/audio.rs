use std::io::Cursor;

use rodio::{Decoder, OutputStream, Sink};

use crate::paragraph::Paragraph;

pub fn play(p: Paragraph) {
    println!("original text: \n{}", p.original_text);
    println!("cleaned text: \n{}", p.cleaned_text);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    if let Some(a) = p.audio {
        let source = Decoder::new(Cursor::new(a)).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    } else {
        println!("no audio found");
    }
}
