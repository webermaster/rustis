use std::io::{Error, ErrorKind, Read, Result, Write};

use crate::message::Message;

pub struct Resp<R> {
    read: R
}

impl <R: Read + Write> Resp<R> {
    pub fn new(read: R) -> Resp<R> {
        Resp{read}
    }

    pub fn read(&mut self) -> Result<Message> {
        let read_result = self.read_byte();
        match read_result {
            Ok(b) => {
                match b {
                    b'*' => self.read_array(),
                    b'$' => self.read_bulk(),
                    t => {
                        println!("Unknown type: {}", String::from_utf8_lossy(&[t]));
                        Ok(Message::Null)
                    }
                }
            },
            Err(err) => Err(err)
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];

        match self.read.read(&mut buf) {
            Ok(0) => Err(Error::new(ErrorKind::InvalidInput, "no bytes")),
            Ok(_) => {
                Ok(buf[0])
            },
            Err(err)  => Err(err)
        }
    }

    fn read_array(&mut self) -> Result<Message> {
        let (_, r) = self.read_integer();
        let array_length = r?;
        let mut array = Vec::with_capacity(array_length);
        for _ in 0..array_length {
            let val = match self.read() {
                Ok(msg) => msg,
                Err(error) => Message::Error(error.to_string())
            };
            array.push(val);
        }
        Ok(Message::Array(array))
    }

    fn read_bulk(&mut self) -> Result<Message> {
        let (_, r) = self.read_integer();
        let array_length = r?;
        let mut bulk = vec![0u8; array_length];

        let _ = self.read.read(&mut bulk);
        let _ = self.read_line(&mut vec![0; 2]); // eat trailing CLRF
        let bulk_string = String::from_utf8_lossy(&bulk).to_string();
        Ok(Message::Bulk(bulk_string))
    }

    fn read_integer(&mut self) -> (usize, Result<usize>) {
        let mut buf = vec![];
        let n = match self.read_line(&mut buf){
            Ok(n) => n,
            Err(err) => return (0, Err(err))
        };
        let i = (buf[0] - b'0') as usize;
        (n, Ok(i))
    }

    fn read_line(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut n = 0;
        loop {
            let c = self.read_byte()?;
            n += 1;
            buf.append(&mut vec![c]);
            if buf.len() >= 2 && buf[buf.len() - 2] == b'\r' {
                break
            }
        }
        buf.truncate(n-2);
        Ok(n)
    }
}
