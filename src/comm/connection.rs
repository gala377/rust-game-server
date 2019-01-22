use std::{
    convert::From,
    io::{Read, Write},
    iter::FromIterator,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::agent;
use crate::comm::{errors, handlers, MessageRaw, Response};

pub const SKEY: &[u8; MSG_SKEY_FIELD_LEN] = b"RG";
pub const MSG_BATCH_LEN: usize = 512;
pub const MSG_LEN_FIELD_LEN: usize = 4;
pub const MSG_SKEY_FIELD_LEN: usize = 2;
pub const MSG_ID_FIELD_LEN: usize = 4;
pub const MSG_HEADER_LEN: usize = MSG_SKEY_FIELD_LEN + MSG_LEN_FIELD_LEN + MSG_ID_FIELD_LEN;

#[derive(Clone)]
pub struct Context {
    pub id: usize,
    pub initialized: bool,

    pub game_agent: Arc<RwLock<agent::Agent>>,
}

impl Context {
    pub fn new(conn_id: usize, game_agent: Arc<RwLock<agent::Agent>>) -> Context {
        Context {
            id: conn_id,
            initialized: false,
            game_agent,
        }
    }
}

pub struct Handler {
    context: Context,
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
}

impl Handler {
    /// Initializes new connection handler. 
    /// Where context is initial connection context.
    /// And req_handlers is a reader mutex on request dispatcher so
    /// its not cloned each new connection and not blocked 
    /// as its only used as a const reference.
    pub fn new(context: Context, req_handlers: Arc<RwLock<handlers::Dispatcher>>) -> Handler {
        Handler {
            context,
            req_handlers,
        }
    }

    // todo:
    // has no meanings of stopping.
    // communication channel on which we can check to see if we should close?
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut ctx = self.context.clone();
        loop {
            eprintln!(
                "[{:^12}[{}]]: Trying to build message!",
                "ConnHandler", &self.context.id
            );
            let raw = match self.try_mess_read(&mut stream) {
                Some(val) => val,
                None => return,
            };
            match self.req_handlers.read() {
                Ok(guard) => {
                    match self.handle_request(raw, &(*guard), &mut ctx) {
                        Some(resp) => self.write_response(resp, &mut stream),
                        None => return,
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

    fn try_mess_read(&self, stream: &mut TcpStream) -> Option<MessageRaw> {
        eprintln!(
            "[{:^12}[{}]]: Trying to build message!",
            "ConnHandler", &self.context.id
        );
        let raw = match self.read_mess(stream) {
            Ok(buffer) => buffer,
            Err(err) => {
                eprintln!(
                    "[{:^12}[{}]]: Error while building message: \"{}\". Aborting...",
                    "ConnHandler", &self.context.id, err,
                );
                return None;
            }
        };
        eprintln!(
            "[{:^12}[{}]]: Message assembled. Request parsing!",
            "ConnHandler", &self.context.id
        );
        return Some(raw);
    }

    fn handle_request(&self, raw: MessageRaw, req_dispatcher: &handlers::Dispatcher, ctx: &mut Context) -> Option<Box<dyn Response>> {
        match req_dispatcher.dispatch_from_raw(raw, ctx) {
            // todo method to return error response from error
            Err(err) => {
                eprintln!(
                    "[{:^12}[{}]]: Error while handling request {:?}",
                    "ConnHandler", &self.context.id, err
                );
                None
            }
            Ok(resp) => {
                eprintln!(
                    "[{:^12}[{}]]: Got response!",
                    "ConnHandler", &self.context.id
                );
                Some(resp)
            }
        }
    }

    fn write_response(&self, resp: Box<dyn Response>, stream: &mut TcpStream) {
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

    fn read_mess(&self, stream: &mut TcpStream) -> Result<MessageRaw, errors::BadRequestError> {
        let mut raw = Vec::with_capacity(MSG_HEADER_LEN);

        let mut header_parsed = false;
        let mut full_msg_len = 0;
        loop {
            self.extend_raw_mess(&mut raw, stream)?;
            if raw.len() >= MSG_HEADER_LEN && !header_parsed {
                full_msg_len = self.read_header(&mut raw)?;
                header_parsed = true;
            }
            if raw.len() == full_msg_len as usize && header_parsed {
                eprintln!(
                    "[{:^12}[{}]]: Read all the payload bytes.",
                    "ConnHandler", &self.context.id
                );
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

    fn extend_raw_mess(
        &self,
        raw: &mut MessageRaw,
        stream: &mut TcpStream,
    ) -> Result<(), errors::BadRequestError> {
        let mut buffer = [0; MSG_BATCH_LEN];
        match stream.read(&mut buffer) {
            Ok(n) => match n {
                0 => {
                    eprintln!(
                        "[{:^12}[{}]]: Connection severed!",
                        "ConnHandler", &self.context.id
                    );
                    Err(errors::BadRequestError::from(errors::ConnectionSevered {}))
                }
                _ => {
                    eprintln!(
                        "[{:^12}[{}]]: Read {} bytes. Proceeding.",
                        "ConnHandler", &self.context.id, n
                    );
                    raw.extend_from_slice(&buffer[0..n]);
                    Ok(())
                }
            },
            Err(err) => Err(errors::BadRequestError::from(errors::ReadError::from(
                err.to_string(),
            ))),
        }
    }

    fn read_header(&self, raw: &MessageRaw) -> Result<u32, errors::HeaderValidationError> {
        eprintln!(
            "[{:^12}[{}]]: Read sufficient number of bytes to parse header.",
            "ConnHandler", &self.context.id
        );
        let mut full_msg_len = Self::parse_header(&raw[..])?;
        full_msg_len += MSG_HEADER_LEN as u32;
        eprintln!(
            "[{:^12}[{}]]: Full msg is {} bytes. {} more bytes to read",
            "ConnHandler",
            &self.context.id,
            full_msg_len,
            full_msg_len as usize - raw.len()
        );
        Ok(full_msg_len)
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
        let mut payload_len: [u8; 4] = [0; 4];
        payload_len.copy_from_slice(&header[beggining..beggining + MSG_LEN_FIELD_LEN]);
        Ok(u32::from_le_bytes(payload_len))
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
