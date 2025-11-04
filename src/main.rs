use std::fs::File;
use std::net::{TcpListener, TcpStream};

mod aof;
mod handlers;
mod message;
mod resp;
mod tcp_handler;

use crate::aof::Aof;
use crate::tcp_handler::{callback, handle_client};

fn main() -> std::io::Result<()> {
    let file = match File::options()
        .create(true)
        .append(true)
        .read(true)
        .open("database.aof")
    {
        Ok(f) => f,
        _ => panic!("Could not open or create file"),
    };
    let mut aof = Aof::new(file);
    let _ = aof.read(callback);
    let mut handler = |stream: TcpStream| handle_client(&mut aof, stream);
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    for stream in listener.incoming() {
        handler(stream?);
    }
    Ok(())
}
