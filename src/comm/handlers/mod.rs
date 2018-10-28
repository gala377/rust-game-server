use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};

use super::{
    MessageId,
    MessageRaw,
    Response,
    Request,
    MSG_SKEY_FIELD_LEN,
    MSG_ID_FIELD_LEN,
};
use super::errors::ReadError;

pub mod init;
mod concrete;
mod responses;
mod requests;

pub trait ReqHandler: Fn(MessageRaw) -> Option<Box<dyn Response>> {}
impl<T> ReqHandler for T where T:Fn(MessageRaw) -> Option<Box<dyn Response>> {}

pub type BoxedReqHandler = Box<dyn ReqHandler<Output = Option<Box<dyn Response>>>>;


pub trait Builder {
    fn req_id() -> MessageId;
    fn build_handler() -> BoxedReqHandler;
}


// todo Box<Error> ?
pub trait DefaultBuilder<T: Request, U: Response + 'static> {

    fn req_id() -> MessageId;

    fn req_from_raw(&MessageRaw) -> Result<T, ReadError>;
    fn handle_request(T) -> Result<U, ReadError>;

    fn build_handler() -> BoxedReqHandler {
        Box::new(|raw: MessageRaw| {
            let req = match Self::req_from_raw(&raw) {
                Err(_) => return None,
                Ok(val) => val,
            };
            match Self::handle_request(req) {
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

    pub fn dispatch_from_raw(&self, raw: MessageRaw) -> Result<Box<dyn Response>, ReadError> {
        let id = Self::read_id(&raw);
        match self.handlers.get(&id) {
            None => Err(ReadError{}),
            Some(handler) => {
                return match handler(raw) {
                    None => Err(ReadError{}),
                    Some(resp) => Ok(resp),
                }
            },
        }
    }

    fn read_id(raw: &MessageRaw) -> MessageId {
        LittleEndian::read_u32(&raw[MSG_SKEY_FIELD_LEN..MSG_SKEY_FIELD_LEN+MSG_ID_FIELD_LEN])
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