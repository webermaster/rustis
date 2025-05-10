pub mod get_handler;
pub mod ping_handler;
pub mod set_handler;

use std::collections::HashMap;

use crate::handlers::ping_handler::ping;
use crate::message::Message;

type HandlerFunc = fn(Vec<Message>) -> Message;

pub fn init_handler_funcs() -> HashMap<&'static str, HandlerFunc> {
    let mut m: HashMap<&'static str, HandlerFunc> = HashMap::new();
    m.insert("PING", ping);
    m
}
