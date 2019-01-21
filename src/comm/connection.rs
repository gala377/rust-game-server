use std::{
    convert::From,
    io::{Read, Write},
    iter::FromIterator,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::comm::{
    errors,
    handlers,
    Response,
    MessageRaw,
};

pub const SKEY: &[u8; MSG_SKEY_FIELD_LEN] = b"RG";
pub const MSG_BATCH_LEN: usize = 512;
pub const MSG_LEN_FIELD_LEN: usize = 4;
pub const MSG_SKEY_FIELD_LEN: usize = 2;
pub const MSG_ID_FIELD_LEN: usize = 4;
pub const MSG_HEADER_LEN: usize = MSG_SKEY_FIELD_LEN + MSG_LEN_FIELD_LEN + MSG_ID_FIELD_LEN;

pub struct Context {
    pub id: usize,
    pub initialized: bool,
}

impl Context {
    pub fn new(conn_id: usize) -> Context {
        Context{
            id: conn_id,
            initialized: false,
        }
    } 
}

pub struct Handler {
    context: Context,
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
}

// todo: refactor
// read_mess and handle_connection are too looong
impl Handler {
    pub fn new(context: Context, req_handlers: Arc<RwLock<handlers::Dispatcher>>) -> Handler {
        Handler{ 
            context,
            req_handlers
        }
    }

    // todo:
    // has no meanings of stopping.
    // communication channel on which we can check to see if we should close?
    //
    // whats more it needs to be stateful.
    // maybe &mut self and sending reference to connection
    // to dispatch from raw?
    // Or some kind of reference to a context struct being passed along? 
    pub fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            eprintln!("[{:^12}[{}]]: Trying to build message!", "ConnHandler", &self.context.id);
            let raw = match self.read_mess(&mut stream) {
                Ok(buffer) => buffer,
                Err(err) => {
                    eprintln!(
                        "[{:^12}[{}]]: Error while building message: \"{}\". Aborting...",
                        "ConnHandler", &self.context.id, err,
                    );
                    return;
                }
            };
            eprintln!(
                "[{:^12}[{}]]: Message assembled. Request parsing!",
                "ConnHandler", &self.context.id
            );

            match self.req_handlers.read() {
                Ok(guard) => {
                    // todo pass reference to context
                    match (*guard).dispatch_from_raw(raw) {
                        // todo method to return error response from error
                        Err(err) => eprintln!(
                            "[{:^12}[{}]]: Error while handling request {:?}",
                            "ConnHandler", &self.context.id, err
                        ),
                        Ok(resp) => {
                            eprintln!("[{:^12}[{}]]: Got response!", "ConnHandler", &self.context.id);
                            match stream.write_all(&Self::response_as_bytes(resp)[..]) {
                                Ok(_) => eprintln!(
                                    "[{:^12}[{}]]: Message sent successfully!",
                                    "ConnHandler", &self.context.id
                                ),
                                Err(err) => eprintln!(
                                    "[{:^12}[{}]]: Error while sending the response \"{}\"",
                                    "ConnHandler", &self.context.id, err
                                ),
                            }
                            stream.flush().unwrap();
                        }
                    };
                }
                Err(err) => {
                    eprintln!(
                        "[{:^12}[{}]]: Error while getting a lock! {}",
                        "ConnHandler", &self.context.id, err
                    );
                    return;
                }
            }
        }
    }

    fn read_mess(&self, stream: &mut TcpStream) -> Result<MessageRaw, errors::BadRequestError> {
        let mut buffer = [0; MSG_BATCH_LEN];
        let mut raw = Vec::with_capacity(MSG_HEADER_LEN);

        let mut header_parsed = false;
        let mut full_msg_len = 0;
        loop {
            match stream.read(&mut buffer) {
                Ok(n) => match n {
                    0 => {
                        eprintln!("[{:^12}[{}]]: Connection severed!", "ConnHandler", &self.context.id);
                        return Err(errors::BadRequestError::from(errors::ConnectionSevered {}));
                    }
                    _ => {
                        eprintln!("[{:^12}[{}]]: Read {} bytes. Proceeding.", "ConnHandler", &self.context.id, n);
                        raw.extend_from_slice(&buffer[0..n]);
                    }
                },
                Err(err) => {
                    return Err(errors::BadRequestError::from(errors::ReadError::from(
                        err.to_string(),
                    )))
                }
            }
            if raw.len() >= MSG_HEADER_LEN && !header_parsed {
                eprintln!(
                    "[{:^12}[{}]]: Read sufficient number of bytes to parse header.",
                    "ConnHandler", &self.context.id
                );
                full_msg_len = Self::parse_header(&raw[..])?;
                full_msg_len += MSG_HEADER_LEN as u32;
                eprintln!(
                    "[{:^12}[{}]]: Full msg is {} bytes. {} more bytes to read",
                    "ConnHandler",
                    &self.context.id,
                    full_msg_len,
                    full_msg_len as usize - raw.len()
                );
                header_parsed = true;
            }
            if raw.len() == full_msg_len as usize && header_parsed {
                eprintln!("[{:^12}[{}]]: Read all the payload bytes.", "ConnHandler", &self.context.id);
                break;
            } else if raw.len() > full_msg_len as usize && header_parsed {
                eprintln!(
                    "[{:^12}[{}]]: Read more than specified in payload len. Aborting...",
                    "ConnHandler", &self.context.id
                );
                return Err(errors::BadRequestError::from(errors::ReadError::from(
                    format!(
                        "read more bytes than specified in mess len. 
                            Expected: {}, Read: {}",
                        full_msg_len,
                        raw.len(),
                    ),
                )));
            }
        }
        Ok(raw)
    }

    fn parse_header(header: &[u8]) -> Result<u32, errors::HeaderValidationError> {
        for i in 0..MSG_SKEY_FIELD_LEN {
            if SKEY[i] != header[i] {
                return Err(errors::HeaderValidationError {
                    expected: Vec::from_iter(SKEY.iter().cloned()),
                    actual: Vec::from_iter(header.iter().cloned()),
                });
            }
        }
        let beggining = MSG_SKEY_FIELD_LEN + MSG_ID_FIELD_LEN;
        let mut id_as_bytes: [u8; 4] = [0; 4];
        id_as_bytes.copy_from_slice(&header[beggining..beggining + MSG_ID_FIELD_LEN]);
        Ok(u32::from_le_bytes(id_as_bytes))
    }

    fn response_as_bytes(resp: Box<dyn Response>) -> Vec<u8> {
        let mut as_bytes = Vec::with_capacity(MSG_HEADER_LEN);
        for &ch in SKEY.iter() {
            as_bytes.push(ch);
        }
        as_bytes.extend(&resp.id().to_le_bytes());

        let payload = resp.payload();
        as_bytes.extend(&(payload.len() as u32).to_le_bytes());
        as_bytes.reserve(payload.len());
        as_bytes.extend_from_slice(&payload[..]);

        as_bytes
    }
}