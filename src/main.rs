use std::fs::File;
use std::net::TcpListener;

mod aof;
mod message;
mod resp;
mod handlers;
mod tcp_handler;

use crate::aof::{ Aof };
use crate::tcp_handler::{ TcpHandler, callback };

fn main() -> std::io::Result<()> {

    let file = match File::options()
        .create(true)
        .append(true)
        .read(true)
        .open("database.aof") {
            Ok(f) => f,
            _ => panic!("Could not open or create file")
        };
    let aof = Aof::new(file);
    let _ = aof.read(callback);
    let mut handler = TcpHandler::new(aof);
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    for stream in listener.incoming() {
       handler.handle_client(stream?);
    }
    Ok(())
}
