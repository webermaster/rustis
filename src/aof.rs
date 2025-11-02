
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use std::thread::{sleep, spawn};

use crate::message::Message;

pub struct Aof {
    file: Arc<Mutex<File>>,
    sender: mpsc::Sender<()>
}

impl Aof {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel();
        let aof = match File::options()
            .create(true)
            .append(true)
            .read(true)
            .open(path) {
                Ok(file) => {
                    Aof{file: Arc::new(Mutex::new(file)), sender: sender}
                },
                _ => panic!("Could not open or create file")
            };
        let cloned = Arc::clone(&aof.file);
        spawn(move || {
            loop {
                if let Ok(_) = receiver.try_recv() {
                    println!("Thread received termination signal. Exiting....");
                    break; // Exit the loop
                }
                let f = cloned.lock().unwrap();
                let _ = f.sync_all();
                sleep(Duration::from_secs(1));
            }
        });
        aof
    }

    pub fn write_message(&mut self, value: &Message) -> Result<usize, io::Error> {
        self.write(value.marshal().as_ref())
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
        self.file.lock().unwrap().write(bytes)
    }
}

#[cfg(test)]
mod test {
}
