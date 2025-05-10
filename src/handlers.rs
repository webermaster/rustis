use std::collections::HashMap;

use crate::message::Message;

type HandlerFunc = fn(Vec<Message>) -> Message;

pub fn init_handler_funcs() -> HashMap<&'static str, HandlerFunc> {
    let mut m: HashMap<&'static str, HandlerFunc> = HashMap::new();
    m.insert("PING", ping);
    m
}

pub fn ping(_: Vec<Message>) -> Message {
    Message::String("PONG".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn test_ping_handler() {
        let result = ping(vec![]);
        assert_eq!(result, Message::String("PONG".to_string()));
    }

    #[test]
    fn test_init_handler_funcs_contains_ping() {
        let handlers = init_handler_funcs();
        assert!(handlers.contains_key("PING"));

        let handler = handlers.get("PING").unwrap();
        let result = handler(vec![]);
        assert_eq!(result, Message::String("PONG".to_string()));
    }
}

