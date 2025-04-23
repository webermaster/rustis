
#[derive(Debug)]
pub enum Message {
    String(String),
    Error(String),
    Number(u64),
    Bulk(String),
    Array(Vec<Message>),
    Null
}

 impl Message {
    pub fn marshal(&self) -> Vec<u8> {
        match self {
            array @ Message::Array(_) => array.marshal_array(),
            bulk @ Message::Bulk(_) => bulk.marshal_bulk(),
            string @ Message::String(_) => string.marshal_string(),
            error @ Message::Error(_) => error.marshal_error(),
            null @ Message::Null => null.marshal_null(),
            _ => Vec::new()
        }
    }

    fn marshal_array(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'*');
        if let Message::Array(array) = self {
            let l = array.len();
            bytes.append(&mut vec![l as u8]);
            bytes.append(&mut vec![b'\r', b'\n']);
            for item in array {
                bytes.append(&mut item.marshal());
            }
        }
        bytes
    }

    fn marshal_bulk(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'$');
        if let Message::Bulk(string) = self {
            bytes.append(&mut vec![string.len() as u8]);
            bytes.append(&mut vec![b'\r', b'\n']);
            bytes.append(&mut string.clone().into_bytes());
        }
        bytes.append(&mut vec![b'\r', b'\n']);
        bytes
    }

    fn marshal_string(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'+');
        if let Message::String(string) = self {
            bytes.append(&mut string.clone().into_bytes());
        }
        bytes.append(&mut vec![b'\r', b'\n']);
        bytes
    }

    fn marshal_error(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'-');
        if let Message::Error(error) = self {
            bytes.append(&mut error.clone().into_bytes());
        }
        bytes.append(&mut vec![b'\r', b'\n']);
        bytes
    }

    fn marshal_null(&self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }

}
