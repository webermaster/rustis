use std::io::{ Read, Write };

use crate::handlers::init_handler_funcs;
use crate::message::Message;
use crate::resp::Resp;

pub fn handle_client<R: Read + Write>(stream: R) -> () {
    let handlers = init_handler_funcs();
    let mut resp = Resp::new(stream);
    loop {
        let read = resp.read();
        let msg = match read {
            Ok(r) => r,
            Err(err) => {
                println!("error reading from client: {err}");
                break;
            }
        };

        if let Message::Array(array) = &msg {
            if array.len() == 0 {
                println!("Invalid request, expected array length > 0");
                continue;
            }
            if let Message::Bulk(command) = &array[0] {
                let cmd = command.to_uppercase();
                let args = &array[1..];

                let handler = match handlers.get(cmd.as_str()) {
                    Some(f) => f,
                    None => {
                        println!("Invalid command: {}", cmd);
                        _ = resp.write(Message::String("".to_string()));
                        continue;
                    }
                };

                let result_msg = handler(args.to_vec());
                _ = resp.write(result_msg);
            }
        } else {
            println!("Invalid request, expected array");
            continue;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read, Write};

    /// A mock stream to simulate client-server communication.
    pub struct MockStream {
        pub read_data: Vec<u8>,
        pub write_data: Vec<u8>,
        pub position: usize,
    }

    impl MockStream {
        /// Creates a new MockStream with the given input data.
        pub fn new(read_data: Vec<u8>) -> Self {
            Self {
                read_data,
                write_data: Vec::new(),
                position: 0,
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.position >= self.read_data.len() {
                return Ok(0); // Simulate EOF
            }
            let bytes_to_read = std::cmp::min(buf.len(), self.read_data.len() - self.position);
            buf[..bytes_to_read]
                .copy_from_slice(&self.read_data[self.position..self.position + bytes_to_read]);
            self.position += bytes_to_read;
            Ok(bytes_to_read)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_handle_client_ping() {
        // Simulate a PING command: *1\r\n$4\r\nPING\r\n
        let input = b"*1\r\n$4\r\nPING\r\n".to_vec();
        let mut mock_stream = MockStream::new(input);

        handle_client(&mut mock_stream);

        // The expected response is: +PONG\r\n
        let expected_output = b"+PONG\r\n";
        assert_eq!(&mock_stream.write_data, expected_output);
    }

    #[test]
    fn test_handle_client_invalid_command() {
        // Simulate an invalid command: *1\r\n$7\r\nUNKNOWN\r\n
        let input = b"*1\r\n$7\r\nUNKNOWN\r\n".to_vec();
        let mut mock_stream = MockStream::new(input);

        handle_client(&mut mock_stream);

        // The expected response is an empty string: +\r\n
        let expected_output = b"+\r\n";
        assert_eq!(&mock_stream.write_data, expected_output);
    }

    #[test]
    fn test_handle_client_malformed_message() {
        // Simulate a malformed message: $5\r\nhello\r\n
        let input = b"$5\r\nhello\r\n".to_vec();
        let mut mock_stream = MockStream::new(input);

        handle_client(&mut mock_stream);

        // The expected response is an empty string: +\r\n
        let expected_output = b"+\r\n";
        assert_eq!(&mock_stream.write_data, expected_output);
    }
}

