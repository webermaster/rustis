
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Simple(String),
    Error(String),
    // Number(u64),
    Bulk(Vec<u8>),
    Array(Vec<Message>),
    Null,
}

impl Message {

    pub fn simple<S: Into<String>>(s: S) -> Self {
        Message::Simple(s.into())
    }

    pub fn error<S: Into<String>>(s: S) -> Self {
        Message::Error(s.into())
    }

    pub fn bulk(v: Vec<u8>) -> Self {
        Message::Bulk(v)
    }

    pub fn array(v: Vec<Message>) -> Self {
        Message::Array(v)
    }

    pub fn marshal(&self) -> Vec<u8> {
        match self {
            array @ Message::Array(_) => array.marshal_array(),
            bulk @ Message::Bulk(_) => bulk.marshal_bulk(),
            string @ Message::Simple(_) => string.marshal_string(),
            error @ Message::Error(_) => error.marshal_error(),
            null @ Message::Null => null.marshal_null(),
            // _ => Vec::new(),
        }
    }

    fn marshal_array(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'*');
        if let Message::Array(array) = self {
            let l = array.len();
            bytes.extend_from_slice(l.to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");
            for item in array {
                bytes.extend(item.marshal());
            }
        }
        bytes
    }

    fn marshal_bulk(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'$');
        if let Message::Bulk(string) = self {
            bytes.extend_from_slice(string.len().to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");
            bytes.extend_from_slice(string);
        }
        bytes.extend_from_slice(b"\r\n");
        bytes
    }

    fn marshal_string(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'+');
        if let Message::Simple(string) = self {
            bytes.extend_from_slice(string.as_bytes());
        }
        bytes.extend_from_slice(b"\r\n");
        bytes
    }

    fn marshal_error(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'-');
        if let Message::Error(error) = self {
            bytes.extend_from_slice(error.as_bytes());
        }
        bytes.extend_from_slice(b"\r\n");
        bytes
    }

    fn marshal_null(&self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_string() {
        let msg = Message::simple("OK");
        assert_eq!(msg.marshal(), b"+OK\r\n");
    }

    #[test]
    fn test_marshal_error() {
        let msg = Message::error("ERR something went wrong");
        assert_eq!(msg.marshal(), b"-ERR something went wrong\r\n");
    }

    #[test]
    fn test_marshal_bulk() {
        let msg = Message::bulk(b"foobar".to_vec());
        assert_eq!(msg.marshal(), b"$6\r\nfoobar\r\n");
    }

    #[test]
    fn test_marshal_null() {
        let msg = Message::Null;
        assert_eq!(msg.marshal(), b"$-1\r\n");
    }

    #[test]
    fn test_marshal_array() {
        let msg = Message::array(vec![
            Message::bulk("foo".into()),
            Message::bulk("bar".into())
        ]);
        assert_eq!(msg.marshal(), b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    }
}

