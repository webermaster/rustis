use std::io::{Error, ErrorKind, Read, Result, Write};

use crate::message::Message;

pub struct Resp<R> {
    rw: R
}

impl <R: Read + Write> Resp<R> {
    pub fn new(rw: R) -> Resp<R> {
        Resp{rw}
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

    pub fn write(&mut self, message: Message) -> Result<usize> {
        let bytes = message.marshal();
        self.rw.write(&bytes)
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];

        match self.rw.read(&mut buf) {
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

        let _ = self.rw.read(&mut bulk);
        let _ = self.read_line(&mut vec![0; 2]); // eat trailing CLRF
        Ok(Message::Bulk(bulk))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::message::Message;

    #[test]
    fn test_write_bulk_message() {
        let msg = Message::Bulk("hello".into());
        let mut buffer = Cursor::new(Vec::new());
        let mut resp = Resp::new(&mut buffer);

        let bytes_written = resp.write(msg).unwrap();
        assert!(bytes_written > 0);

        let result = buffer.get_ref();
        assert_eq!(result, b"$5\r\nhello\r\n"); // Adjust if marshal differs
    }

    #[test]
    fn test_read_bulk_message() {
        let input = b"$5\r\nhello\r\n";
        let cursor = Cursor::new(input.to_vec());
        let mut resp = Resp::new(cursor);

        let message = resp.read().unwrap();
        assert_eq!(message, Message::Bulk("hello".into()));
    }

    #[test]
    fn test_read_array_message() {
        let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let cursor = Cursor::new(input.to_vec());
        let mut resp = Resp::new(cursor);

        let message = resp.read().unwrap();
        assert_eq!(
            message,
            Message::Array(vec![
                Message::Bulk("foo".into()),
                Message::Bulk("bar".into())
            ])
        );
    }

    #[test]
    fn test_read_unknown_type() {
        let input = b"!\r\n";
        let cursor = Cursor::new(input.to_vec());
        let mut resp = Resp::new(cursor);

        let message = resp.read().unwrap();
        assert_eq!(message, Message::Null);
    }

    #[test]
    fn test_read_byte_empty() {
        let cursor = Cursor::new(Vec::new());
        let mut resp = Resp::new(cursor);
        let result = resp.read_byte();
        assert!(result.is_err());
    }
}

