use crate::message::Message;

pub fn ping(_: Vec<Message>) -> Message {
    Message::String("PONG".to_string())
}
