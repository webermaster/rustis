use std::io::{Result, Write};

use crate::message::Message;

pub struct Writer<W> {
    writer: W
}

impl <W: Write> Writer<W> {
    pub fn new(writer: W) -> Writer<W> {
        Writer{writer}
    }

    pub fn write(&mut self, message: Message) -> Result<usize> {
        let bytes = message.marshal();
        self.writer.write(&bytes)
    }

}


