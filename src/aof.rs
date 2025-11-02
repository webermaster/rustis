
use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::thread::{sleep, spawn};

use crate::message::Message;
use crate::resp::Resp;

pub type CB = fn(msg: Message);

pub struct Aof {
    file: File,
    sender: Sender<()>
}

impl Aof {
    pub fn new(file: File) -> Self {
        let (sender, receiver) = channel();
        let aof = Aof{file, sender};
        let cloned = aof.file.try_clone().unwrap();
        spawn(move || {
            loop {
                if let Ok(_) = receiver.try_recv() {
                    println!("Thread received termination signal. Exiting....");
                    break; // Exit the loop
                }
                let _ = cloned.sync_all();
                sleep(Duration::from_secs(1));
            }
        });
        aof
    }

    pub fn write_message(&mut self, value: &Message) -> Result<usize, io::Error> {
        self.write(value.marshal().as_ref())
    }

    pub fn read(&self, callback: CB) -> Result<(), io::Error> {
        let mut resp = Resp::new(&self.file);
        loop {
            let msg = resp.read();
            if let Ok(Message::Null) = msg {
                return Ok(());
            };
            callback(msg?);
        }
    }

    pub fn close(&mut self) -> Result<(), io::Error> {
        self.flush()
    }
}

impl Drop for Aof {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl Write for Aof {
    fn flush(&mut self) -> Result<(), io::Error> {
        match self.sender.send(()) {
            Ok(a) => Ok(a),
            Err(msg) => Err(io::Error::new(io::ErrorKind::Other, msg))
        }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<usize, io::Error> {
        self.file.write(bytes)
    }
}

#[cfg(test)]
mod test {
}
