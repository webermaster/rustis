use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::message::Message;
use crate::message::Message::*;

type HandlerFunc = fn(Vec<Message>) -> Message;

pub static HANDLERS: LazyLock<HashMap<&'static str, HandlerFunc>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, HandlerFunc> = HashMap::new();
    m.insert("PING", ping);
    m.insert("SET", set);
    m
});

pub fn ping(args: Vec<Message>) -> Message {
    match args.as_slice() {
        [] => Message::simple("PONG"),
        [Bulk(arg), _rest @ ..] => Message::simple(str::from_utf8(arg).expect("Invalid UTF-8")),
        _ => Message::error("Protocol error: expected Bulk string")
    }
}

static SETS: LazyLock<Mutex<HashMap<Vec<u8>, Vec<u8>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn set(args: Vec<Message>) -> Message {
    match args.as_slice() {
        [Bulk(key), Bulk(value)] => {
           let mut sets = SETS.lock().unwrap();
            sets.insert(key.to_vec(), value.to_vec());
            Message::simple("OK")
        },
        _ => Message::error("ERR wrong number of arguments for 'set' command")
    }
}

// func get(args []Value) Value {
// 	if len(args) != 1 {
// 		return Value{typ: "error", str: "ERR wrong number of arguments for 'get' command"}
// 	}
//
// 	key := args[0].bulk
//
// 	SETsMu.RLock()
// 	value, ok := SETs[key]
// 	SETsMu.RUnlock()
//
// 	if !ok {
// 		return Value{typ: "null"}
// 	}
//
// 	return Value{typ: "bulk", bulk: value}
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_handler_funcs_contains_ping() {
        assert!(HANDLERS.contains_key("PING"));
    }

    #[test]
    fn test_init_handler_funcs_contains_set() {
        assert!(HANDLERS.contains_key("SET"));
    }

    #[test]
    fn test_ping() {
        let result = ping(vec![]);
        assert_eq!(result, Message::simple("PONG"));
    }

    #[test]
    fn test_ping_with_args() {
        let pong = b"foo".to_vec();
        let result = ping(vec![Message::bulk(pong.clone())]);
        assert_eq!(result, Message::simple(str::from_utf8(&pong).expect("Invalid UTF-8")));
    }

    #[test]
    fn test_ping_protocol_error() {
        let result = ping(vec![Message::simple("foo")]);
        assert_eq!(result, Message::error("Protocol error: expected Bulk string"));
    }

    #[test]
    fn test_set() {
        let key = b"foo".to_vec();
        let value = b"bar".into();
        let result = set(vec![Message::bulk(key.clone()),Message::bulk(value)]);
        let sets = SETS.lock().unwrap();
        let in_set = sets.get(&key);
        assert_eq!(result, Message::simple("OK"));
        assert_eq!(in_set, Some(b"bar".to_vec()).as_ref());
    }

    #[test]
    fn test_set_too_many_args() {
        let result = set(
            vec![Message::bulk(b"foo".into()),
                 Message::bulk(b"bar".into()),
                 Message::bulk(b"baz".into())]);
        assert_eq!(result, Message::error("ERR wrong number of arguments for 'set' command"));
    }
}

