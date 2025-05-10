use std::net::TcpStream;
use crate::handlers::init_handler_funcs;
use crate::message::Message;
use crate::resp::Resp;
use crate::writer::Writer;

pub fn handle_client(stream: TcpStream) -> () {
    let handlers = init_handler_funcs();
    let mut resp = Resp::new(&stream);
    let mut writer = Writer::new(&stream);
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
                        _ = writer.write(Message::String("".to_string()));
                        continue;
                    }
                };

                let result_msg = handler(args.to_vec());
                _ = writer.write(result_msg);
            }
        } else {
            println!("Invalid request, expected array");
            continue;
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*; // Bring the parent module's items into the test module
//     use std::sync::{Arc, Mutex};
//     use std::collections::HashMap;
//
//     // Mock Resp
//     #[derive(Clone)]
//     pub struct MockResp {
//         pub responses: Arc<Mutex<Vec<Message>>>,
//     }
//
//     impl MockResp {
//         pub fn new() -> Self {
//             MockResp {
//                 responses: Arc::new(Mutex::new(Vec::new())),
//             }
//         }
//
//         // pub fn read(&self) -> Result<Message, String> {
//         //     let mut responses = self.responses.lock().unwrap();
//         //     if let Some(response) = responses.pop() {
//         //         Ok(response)
//         //     } else {
//         //         Err("No more responses".to_string())
//         //     }
//         // }
//     }
//
//     // Mock Writer
//     pub struct MockWriter {
//         pub written: Arc<Mutex<Vec<u8>>>,
//     }
//
//     impl MockWriter {
//         pub fn new() -> Self {
//             MockWriter {
//                 written: Arc::new(Mutex::new(Vec::new())),
//             }
//         }
//
//         // pub fn write(&mut self, message: Message) -> Result<(), io::Error> {
//         //     let mut written = self.written.lock().unwrap();
//         //     written.extend_from_slice(&message.marshal());
//         //     Ok(())
//         // }
//     }
//
//     // Fake handler function
//     fn mock_handler(_args: Vec<Message>) -> Message {
//         Message::String("OK".to_string())
//     }
//
//     #[test]
//     fn test_handle_client_valid_command() {
//         // Mock the handler functions
//         let mut handlers: HashMap<String, Box<dyn Fn(Vec<Message>) -> Message>> = HashMap::new();
//         handlers.insert("PING".to_string(), Box::new(mock_handler));
//
//         // Create mock resp and writer
//         let resp = MockResp::new();
//         let writer = MockWriter::new();
//
//         // Simulate valid message array
//         let array = vec![
//             Message::Bulk("PING".to_string()),
//             Message::String("".to_string()), // Just an empty string argument
//         ];
//
//         // Mock reading the array message from the client
//         resp.responses.lock().unwrap().push(Message::Array(array));
//
//         // Run the handle_client function
//         let stream = TcpStream::connect("localhost:8080").unwrap();
//         handle_client(stream);
//
//         // Assert that the correct message was written
//         let written_data = writer.written.lock().unwrap();
//         assert_eq!(written_data.as_slice(), b"+OK\r\n");
//     }
//
//     #[test]
//     fn test_handle_client_invalid_command() {
//
//         // Create mock resp and writer
//         let resp = MockResp::new();
//         let writer = MockWriter::new();
//
//         // Simulate invalid command message array
//         let array = vec![
//             Message::Bulk("UNKNOWN".to_string()),
//             Message::String("".to_string()),
//         ];
//
//         // Mock reading the array message from the client
//         resp.responses.lock().unwrap().push(Message::Array(array));
//
//         // Run the handle_client function
//         let stream = TcpStream::connect("localhost:8080").unwrap();
//         handle_client(stream);
//
//         // Assert that an empty response was written (since the command was invalid)
//         let written_data = writer.written.lock().unwrap();
//         assert_eq!(written_data.as_slice(), b"");
//     }
//
//     #[test]
//     fn test_handle_client_error_reading() {
//         // Mock the handler functions
//         let mut handlers: HashMap<String, Box<dyn Fn(Vec<Message>) -> Message>> = HashMap::new();
//         handlers.insert("PING".to_string(), Box::new(mock_handler));
//
//         // Create mock resp that will return an error on read
//         let resp = MockResp::new();
//         let writer = MockWriter::new();
//
//         // Simulate error while reading
//         resp.responses.lock().unwrap().push(Message::String("error".to_string()));
//
//         // Simulate an error scenario where read() fails
//         let stream = TcpStream::connect("localhost:8080").unwrap();
//         handle_client(stream);
//
//         // Ensure no data was written (because the read failed)
//         let written_data = writer.written.lock().unwrap();
//         assert_eq!(written_data.len(), 0);
//     }
// }
//
