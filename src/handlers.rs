use std::collections::HashMap;

use crate::message::Message;

type HandlerFunc = fn(Vec<Message>) -> Message;

pub fn init_handler_funcs() -> HashMap<&'static str, HandlerFunc> {
    let mut m: HashMap<&'static str, HandlerFunc> = HashMap::new();
    m.insert("PING", ping);
    m
}

pub fn ping(args: Vec<Message>) -> Message {
    match args.as_slice() {
        [] => Message::String("PONG".to_string()),
        [Message::Bulk(arg), _rest @ ..] => Message::String(String::from_utf8(arg.clone()).expect("Invalid UTF-8")),
        _ => Message::Error("Protocol error: expected Bulk string".to_string())
    }
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

    #[test]
    fn test_ping_handler_with_args() {
        let pong = b"foo".to_vec();
        let result = ping(vec![Message::Bulk(pong.clone())]);
        assert_eq!(result, Message::String(String::from_utf8(pong).expect("Invalid UTF-8")));
    }
}

