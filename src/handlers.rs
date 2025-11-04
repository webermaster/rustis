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
    m.insert("HGETALL", Box::new(|args| hgetall(args, &HSETS)));
    m.insert("PING", Box::new(|args| ping(args, &SETS)));
    m.insert("SET", Box::new(|args| set(args, &SETS)));
    m.insert("HSET", Box::new(|args| hset(args, &HSETS)));
    m
});

pub static SETS: LazyLock<SetMap> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static HSETS: LazyLock<HSetMap> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn ping(args: Vec<Message>, _sets: &SetMap) -> Message {
    match args.as_slice() {
        [] => Message::simple("PONG"),
        [Bulk(arg), _rest @ ..] => Message::simple(str::from_utf8(arg).expect("Invalid UTF-8")),
        _ => Message::error("Protocol error: expected Bulk string"),
    }
}

pub fn set(args: Vec<Message>, sets: &SetMap) -> Message {
    match args.as_slice() {
        [Bulk(key), Bulk(value)] => {
            let mut sets = sets.lock().unwrap();
            sets.insert(key.clone(), value.clone());
            Message::simple("OK")
        }
        _ => Message::error("ERR wrong number of arguments for 'set' command"),
    }
}

pub fn hset(args: Vec<Message>, hsets: &HSetMap) -> Message {
    match args.as_slice() {
        [Bulk(hash_key), Bulk(key), Bulk(value)] => {
            let mut hsets = hsets.lock().unwrap();
            if hsets.get(&hash_key.clone()).is_none() {
                hsets.insert(hash_key.to_vec(), HashMap::new());
            }
            let hset = hsets.entry(hash_key.clone()).or_default();
            hset.insert(key.to_vec(), value.clone());
            Message::simple("OK")
        }
        _ => Message::error("ERR wrong number of arguments for 'hset' command"),
    }
}

pub fn get(args: Vec<Message>, sets: &SetMap) -> Message {
    match args.as_slice() {
        [Bulk(key)] => {
            let sets = sets.lock().unwrap();
            match sets.get(&key.clone()) {
                Some(value) => Message::bulk(value.clone()),
                _ => Message::Null,
            }
        }
        _ => Message::error("ERR wrong number of arguments for 'get' command"),
    }
}

pub fn hget(args: Vec<Message>, hsets: &HSetMap) -> Message {
    match args.as_slice() {
        [Bulk(hash_key), Bulk(key)] => {
            let hsets = hsets.lock().unwrap();
            match hsets.get(&hash_key.clone()) {
                Some(hash) => match hash.get(&key.clone()) {
                    Some(value) => Message::bulk(value.clone()),
                    _ => Message::Null,
                },
                _ => Message::Null,
            }
        }
        _ => Message::error("ERR wrong number of arguments for 'hget' command"),
    }
}

pub fn hgetall(args: Vec<Message>, hsets: &HSetMap) -> Message {
    match args.as_slice() {
        [Bulk(hash_key)] => {
            let hsets = hsets.lock().unwrap();
            match hsets.get(&hash_key.clone()) {
                Some(hash) => Message::Array(
                    hash.iter()
                        .flat_map(|(key, value)| {
                            vec![Message::Bulk(key.to_vec()), Message::Bulk(value.to_vec())]
                        })
                        .collect::<Vec<Message>>(),
                ),
                _ => Message::Null,
            }
        }
        _ => Message::error("ERR wrong number of arguments for 'hgetall' command"),
    }
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
    fn test_init_handler_funcs_contains_hset() {
        assert!((*HANDLERS).contains_key("HSET"));
    }

    #[test]
    fn test_init_handler_funcs_contains_hget() {
        assert!((*HANDLERS).contains_key("HGET"));
    }

    #[test]
    fn test_ping() {
        let result = ping(vec![], &Mutex::new(HashMap::new()));
        assert_eq!(result, Message::simple("PONG"));
    }

    #[test]
    fn test_ping_with_args() {
        let pong = b"foo".to_vec();
        let result = ping(
            vec![Message::bulk(pong.clone())],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::simple(str::from_utf8(&pong).expect("Invalid UTF-8"))
        );
    }

    #[test]
    fn test_ping_protocol_error() {
        let result = ping(vec![Message::simple("foo")], &Mutex::new(HashMap::new()));
        assert_eq!(
            result,
            Message::error("Protocol error: expected Bulk string")
        );
    }

    #[test]
    fn test_set() {
        let key = b"foo".to_vec();
        let value = b"bar".into();
        let sets = Mutex::new(HashMap::new());
        let result = set(
            vec![Message::bulk(key.clone()), Message::bulk(value)],
            &sets,
        );
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
            vec![
                Message::bulk(b"foo".into()),
                Message::bulk(b"bar".into()),
                Message::bulk(b"baz".into()),
            ],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::error("ERR wrong number of arguments for 'set' command")
        );
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
    }

    #[test]
    fn test_get_too_many_args() {
        let result = get(
            vec![Message::bulk(b"foo".into()), Message::bulk(b"bar".into())],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::error("ERR wrong number of arguments for 'get' command")
        );
    }

    #[test]
    fn test_hset() {
        let hash_key = b"baz".to_vec();
        let key = b"foo".to_vec();
        let value = b"bar".into();
        let hsets = Mutex::new(HashMap::new());
        let result = hset(
            vec![
                Message::bulk(hash_key.clone()),
                Message::bulk(key.clone()),
                Message::bulk(value),
            ],
            &hsets,
        );
        let in_set = {
            let guard = hsets.lock().unwrap();
            guard.get(&hash_key).unwrap().get(&key).cloned()
        };
        assert_eq!(result, Message::simple("OK"));
        assert_eq!(in_set, Some(b"bar".to_vec()));
    }

    #[test]
    fn test_hset_reset() {
        let hash_key = b"baz".to_vec();
        let key = b"foo".to_vec();
        let value = b"bar".to_vec();
        let hsets: Mutex<HashMap<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>> = Mutex::new(HashMap::new());
        let mut set: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        set.insert(key.clone(), b"quax".to_vec());
        {
            hsets.lock().unwrap().insert(hash_key.clone(), set);
        }
        let result = hset(
            vec![
                Message::bulk(hash_key.clone()),
                Message::bulk(key.clone()),
                Message::bulk(value),
            ],
            &hsets,
        );
        let in_set = {
            let guard = hsets.lock().unwrap();
            guard.get(&hash_key).unwrap().get(&key).cloned()
        };
        assert_eq!(result, Message::simple("OK"));
        assert_eq!(in_set, Some(b"bar".to_vec()));
    }

    #[test]
    fn test_hset_too_many_args() {
        let result = hset(
            vec![
                Message::bulk(b"foo".into()),
                Message::bulk(b"bar".into()),
                Message::bulk(b"quax".into()),
                Message::bulk(b"baz".into()),
            ],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::error("ERR wrong number of arguments for 'hset' command")
        );
    }

    #[test]
    fn test_hget() {
        let hash_key = b"baz".to_vec();
        let key = b"foo".to_vec();
        let value = b"bar".to_vec();
        let hsets: Mutex<HashMap<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>> = Mutex::new(HashMap::new());
        let mut set: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        set.insert(key.clone(), value.clone());
        {
            hsets.lock().unwrap().insert(hash_key.clone(), set);
        }
        let result = hget(
            vec![Message::bulk(hash_key.clone()), Message::bulk(key.clone())],
            &hsets,
        );
        let in_set = {
            hsets
                .lock()
                .unwrap()
                .get(&hash_key)
                .unwrap()
                .get(&key)
                .cloned()
        };
        assert_eq!(result, Message::bulk(in_set.unwrap().clone()));
    }

    #[test]
    fn test_hget_too_many_args() {
        let result = hget(
            vec![
                Message::bulk(b"foo".into()),
                Message::bulk(b"baz".into()),
                Message::bulk(b"bar".into()),
            ],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::error("ERR wrong number of arguments for 'hget' command")
        );
    }

    #[test]
    fn test_hgetall() {
        let hash_key = b"baz".to_vec();
        let entries = vec![
            (b"foo".to_vec(), b"bar".to_vec()),
            (b"quax".to_vec(), b"quoo".to_vec()),
        ];
        let hsets: Mutex<HashMap<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>> = Mutex::new(HashMap::new());
        let mut set: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        for (k, v) in entries.into_iter() {
            set.insert(k.clone(), v.clone());
        }
        {
            hsets.lock().unwrap().insert(hash_key.clone(), set);
        }
        let result = hgetall(vec![Message::bulk(hash_key.clone())], &hsets);
        let expected = {
            Message::Array(
                hsets
                    .lock()
                    .unwrap()
                    .get(&hash_key)
                    .unwrap()
                    .iter()
                    .flat_map(|(key, value)| {
                        vec![Message::Bulk(key.to_vec()), Message::Bulk(value.to_vec())]
                    })
                    .collect::<Vec<Message>>(),
            )
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hgetall_too_many_args() {
        let result = hgetall(
            vec![Message::bulk(b"foo".into()), Message::bulk(b"bar".into())],
            &Mutex::new(HashMap::new()),
        );
        assert_eq!(
            result,
            Message::error("ERR wrong number of arguments for 'hgetall' command")
        );
    }
}
