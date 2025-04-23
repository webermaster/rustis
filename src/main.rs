use std::net::TcpListener;

mod handler;
mod message;
mod writer;
mod resp;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    for stream in listener.incoming() {
       handler::handle_client(stream?);
    }
    Ok(())
}
