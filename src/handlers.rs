use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::message::Message;
use crate::message::Message::*;

pub type HandlerFunc = Box<dyn Handler + Sync + Send>;

type MapMutex<K, V> = Mutex<HashMap<K, V>>;
type SetMap = MapMutex<Vec<u8>, Vec<u8>>;
type HSetMap = MapMutex<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>;
type HandlerMap = LazyLock<HashMap<&'static str, HandlerFunc>>;

pub trait Handler {
    fn call(&self, args: Vec<Message>) -> Message;
}

impl<F> Handler for F
where
    F: Fn(Vec<Message>) -> Message + Send + Sync + 'static,
{
    fn call(&self, args: Vec<Message>) -> Message {
        (self)(args)
    }
}


pub static HANDLERS: HandlerMap = LazyLock::new(|| {
   let mut m: HashMap<&'static str, HandlerFunc> = HashMap::new();
    m.insert("GET", Box::new(|args| get(args, &SETS)));
    m.insert("HGET", Box::new(|args| hget(args, &HSETS)));
    m.insert("PING", Box::new(|args| ping(args, &SETS)));
    m.insert("SET", Box::new(|args| set(args, &SETS)));
    m.insert("HSET", Box::new(|args| hset(args, &HSETS)));
    m
});

pub static SETS: LazyLock<SetMap> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub static HSETS: LazyLock<HSetMap> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn ping(args: Vec<Message>, _sets: &SetMap) -> Message {
    match args.as_slice() {
        [] => Message::simple("PONG"),
        [Bulk(arg), _rest @ ..] => Message::simple(str::from_utf8(arg).expect("Invalid UTF-8")),
        _ => Message::error("Protocol error: expected Bulk string")
    }
}

pub fn set(args: Vec<Message>, sets: &SetMap) -> Message {
    match args.as_slice() {
        [Bulk(key), Bulk(value)] => {
           let mut sets = sets.lock().unwrap();
            sets.insert(key.to_vec(), value.to_vec());
            Message::simple("OK")
        },
        _ => Message::error("ERR wrong number of arguments for 'set' command")
    }
}

pub fn hset(_args: Vec<Message>, _sets: &HSetMap) -> Message {
    todo!()
}

pub fn get(args: Vec<Message>, sets: &SetMap) -> Message {
    match args.as_slice() {
        [Bulk(key)] => {
            let sets = sets.lock().unwrap();
            match sets.get(&key.to_vec()) {
                Some(value) => {
                    Message::bulk(value.to_vec())
                },
                _ => {
                    Message::Null
                }
            }
        },
        _ => Message::error("ERR wrong number of arguments for 'get' command")
    }
}

pub fn hget(_args: Vec<Message>, _sets: &HSetMap) -> Message {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_handler_funcs_contains_ping() {
        assert!((*HANDLERS).contains_key("PING"));
    }

    #[test]
    fn test_init_handler_funcs_contains_set() {
        assert!((*HANDLERS).contains_key("SET"));
    }

    #[test]
    fn test_init_handler_funcs_contains_get() {
        assert!((*HANDLERS).contains_key("GET"));
    }

    #[test]
    fn test_ping() {
        let result = ping(vec![], &Mutex::new(HashMap::new()));
        assert_eq!(result, Message::simple("PONG"));
    }

    #[test]
    fn test_ping_with_args() {
        let pong = b"foo".to_vec();
        let result = ping(vec![Message::bulk(pong.clone())], &Mutex::new(HashMap::new()));
        assert_eq!(result, Message::simple(str::from_utf8(&pong).expect("Invalid UTF-8")));
    }

    #[test]
    fn test_ping_protocol_error() {
        let result = ping(vec![Message::simple("foo")], &Mutex::new(HashMap::new()));
        assert_eq!(result, Message::error("Protocol error: expected Bulk string"));
    }

    #[test]
    fn test_set() {
        let key = b"foo".to_vec();
        let value = b"bar".into();
        let sets = Mutex::new(HashMap::new());
        let result = set(vec![Message::bulk(key.clone()),Message::bulk(value)], &sets);
        let in_set = {
            let guard = sets.lock().unwrap();
            guard.get(&key).cloned()
        };
        assert_eq!(result, Message::simple("OK"));
        assert_eq!(in_set, Some(b"bar".to_vec()));

        sets.lock().unwrap().remove(&key);
    }

    #[test]
    fn test_set_too_many_args() {
        let result = set(
            vec![Message::bulk(b"foo".into()),
                 Message::bulk(b"bar".into()),
                 Message::bulk(b"baz".into())], &Mutex::new(HashMap::new()));
        assert_eq!(result, Message::error("ERR wrong number of arguments for 'set' command"));
    }


    #[test]
    fn test_get() {
        let key = b"foo".to_vec();
        let value = b"bar".to_vec();
        let sets = Mutex::new(HashMap::new());
        {
            sets.lock().unwrap().insert(key.clone(), value.clone());
        }
        let result = get(vec![Message::bulk(key.clone())], &sets);
        let in_set = {
            let guard = sets.lock().unwrap();
            guard.get(&key).cloned()
        };
        assert_eq!(result, Message::bulk(in_set.unwrap().clone()));

        sets.lock().unwrap().remove(&key);
    }
}
