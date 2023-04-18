use std::io::Cursor;

use bytes::Bytes;
use rodio::{Decoder, OutputStream, Sink};

use crate::paragraph::Paragraph;

pub fn play(audio: Bytes) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Decoder::new(Cursor::new(audio)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
}
