use std::net::TcpStream;

use crate::message::Message;
use crate::resp::Resp;
use crate::writer::Writer;

pub fn handle_client(stream: TcpStream) -> () {
     let mut resp = Resp::new(&stream);
     loop {
        let read = resp.read();
        let msg  = match read {
            Ok(r) => r,
            Err(err) => {
                println!("error reading from client: {err}");
                break;
            }
        };

        println!("{:?}", msg);
        let mut writer = Writer::new(&stream);
        _ = writer.write(Message::String("OK".to_string()));
        // _ = resp.write(b"+OK\r\n");
     }
}

