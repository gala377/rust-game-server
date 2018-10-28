use super::super::super::{
    MessageId,
    MessageRaw,
    MSG_HEADER_LEN,
    errors::{
        ReadError,
    }
};
use super::super::DefaultBuilder;
use super::super::requests;
use super::super::responses;

pub struct Handler;

impl DefaultBuilder<requests::Hello, responses::Welcome> for Handler {

    fn req_id() -> MessageId {
        0
    }

    fn req_from_raw(raw: &MessageRaw) -> Result<requests::Hello, ReadError> {
        println!("HelloHandler: Are you hello?");
        if raw.len() != MSG_HEADER_LEN {
            println!("HelloHandler: You are not");
            Err(ReadError)
        } else {
            println!("HelloHandler: Yes you are");
            Ok(requests::Hello{})
        }
    }

    fn handle_request(_req: requests::Hello) -> Result<responses::Welcome, ReadError> {
        println!("HelloHandler: Welcome");
        Ok(responses::Welcome{})
    } 
}
