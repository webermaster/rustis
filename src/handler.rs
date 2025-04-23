use std::net::TcpStream;

use crate::resp::Resp;

pub fn handle_client(stream: &mut TcpStream) -> () {
     loop {

        let mut resp = Resp::new(&mut *stream);

        let read = resp.read();

        let msg  = match read {
            Ok(r) => r,
            Err(err) => {
                println!("error reading form client: {err}");
                break;
            }
        };

        println!("{:?}", msg);
        _ = resp.write(b"+Ok\r\n");
     }
}

