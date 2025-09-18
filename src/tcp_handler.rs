use std::io::{ Read, Write };

use crate::handlers::HANDLERS;
use crate::message::Message::*;
use crate::message::Message;
use crate::resp::Resp;

pub fn handle_client<R: Read + Write>(stream: R) {
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

        if let Array(array) = &msg {
            if array.is_empty() {
                println!("Invalid request, expected array length > 0");
                continue;
            }
            if let Bulk(command) = &array[0] {
                if let Ok(cmd_str) = std::str::from_utf8(command) {
                    let cmd = cmd_str.to_uppercase();
                    let args = &array[1..];

                    match HANDLERS.get(cmd.as_str()) {
                        Some(f) => {
                            let result_msg = f(args.to_vec());
                            _ = resp.write(result_msg);
                        },
                        None => {
                            _ = resp.write(Message::simple(""));
                            continue;
                        }
                    }
                } else {
                    _ = resp.write(Message::error("Commands must be valid UTF-8"));
                    continue;
                }
            }
        } else {
            _ = resp.write(Message::error("Protocol error: expected '*'"));
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
    fn test_handle_client_non_array_message() {
        // Simulate a malformed message: $5\r\nhello\r\n
        let input = b"$5\r\nhello\r\n".to_vec();
        let mut mock_stream = MockStream::new(input);

        handle_client(&mut mock_stream);

        // The expected response is an empty string: +\r\n
        let expected_output = b"-Protocol error: expected '*'\r\n";
        assert_eq!(&mock_stream.write_data, expected_output);
    }

    #[test]
    fn test_handle_invalid_utf_8_command() {
        // Simulate a malformed message: $5\r\nhello\r\n
        let input = b"*1\r\n$1\r\n\xFF\r\n".to_vec();
        let mut mock_stream = MockStream::new(input);

        handle_client(&mut mock_stream);

        // The expected response is an empty string: +\r\n
        let expected_output = b"-Commands must be valid UTF-8\r\n";
        assert_eq!(&mock_stream.write_data, expected_output);
    }
}

