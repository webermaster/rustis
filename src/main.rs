use std::net::TcpListener;

mod message;
mod resp;
mod handlers;
mod tcp_handler;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    for stream in listener.incoming() {
       tcp_handler::handle_client(stream?);
    }
    Ok(())
}
