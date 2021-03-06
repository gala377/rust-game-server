use std::collections::HashMap;
use std::error::Error;

use crate::comm::{
    connection,
    connection::{MSG_ID_FIELD_LEN, MSG_SKEY_FIELD_LEN},
    errors::{BadRequestError, InternalServerError, ReadError},
    MessageId, MessageRaw, Request, Response,
};

mod concrete;
mod requests;
mod responses;

pub mod init;

pub trait ReqHandler: Fn(MessageRaw, &mut connection::Context) -> Option<Box<dyn Response>> {}
impl<T> ReqHandler for T where T: Fn(MessageRaw, &mut connection::Context) -> Option<Box<dyn Response>> {}

pub type BoxedReqHandler = Box<dyn ReqHandler<Output = Option<Box<dyn Response>>> + Sync + Send>;

pub trait Builder {
    fn req_id() -> MessageId;
    fn build_handler() -> BoxedReqHandler;
}

// todo Box<Error> ?
pub trait DefaultBuilder<T: Request, U: Response + 'static> {
    fn req_id() -> MessageId;

    fn req_from_raw(raw: &MessageRaw) -> Result<T, ReadError>;
    fn handle_request(req: T, ctx: &mut connection::Context) -> Result<U, ReadError>;

    fn build_handler() -> BoxedReqHandler {
        Box::new(|raw: MessageRaw, ctx: &mut connection::Context| {
            let req = match Self::req_from_raw(&raw) {
                Err(_) => return None,
                Ok(val) => val,
            };
            match Self::handle_request(req, ctx) {
                Ok(resp) => Some(Box::new(resp)),
                Err(_) => None,
            }
        })
    }
}

pub struct Dispatcher {
    handlers: HashMap<MessageId, BoxedReqHandler>,
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {
            handlers: HashMap::new(),
        }
    }

    pub fn dispatch_from_raw(&self, raw: MessageRaw, ctx: &mut connection::Context) -> Result<Box<dyn Response>, Box<dyn Error>> {
        let id = Self::read_id(&raw);
        match self.handlers.get(&id) {
            None => Err(Box::new(BadRequestError::from(ReadError::from(format!(
                "Mess id ({}) doesn't match any of registered ones.",
                id
            ))))),
            Some(handler) => {
                return match handler(raw, ctx) {
                    None => Err(Box::new(InternalServerError(Box::new(ReadError::from(
                        format!("Req handler for message with id {} returned None", id),
                    ))))),
                    Some(resp) => Ok(resp),
                };
            }
        }
    }

    fn read_id(raw: &MessageRaw) -> MessageId {
        let mut id_bytes: [u8; 4] = [0; 4];
        id_bytes.copy_from_slice(&raw[MSG_SKEY_FIELD_LEN..MSG_SKEY_FIELD_LEN + MSG_ID_FIELD_LEN]);
        u32::from_le_bytes(id_bytes)
    }

    pub fn register(&mut self, id: MessageId, builder: BoxedReqHandler) -> bool {
        if self.handlers.contains_key(&id) {
            false
        } else {
            self.handlers.insert(id, builder);
            true
        }
    }
}
