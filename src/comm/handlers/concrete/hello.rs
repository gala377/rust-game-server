use super::super::super::{errors::ReadError, MessageId, MessageRaw, MSG_HEADER_LEN};
use super::super::requests;
use super::super::responses;
use super::super::DefaultBuilder;

pub struct Handler;

impl DefaultBuilder<requests::Hello, responses::Welcome> for Handler {
    fn req_id() -> MessageId {
        0
    }

    fn req_from_raw(raw: &MessageRaw) -> Result<requests::Hello, ReadError> {
        println!("HelloHandler: Are you hello?");
        if raw.len() != MSG_HEADER_LEN {
            println!("HelloHandler: You are not");
            Err(ReadError::from(
                format!("Message len is incorrect. Expected: {}. Actual: {}.", MSG_HEADER_LEN, raw.len())))
        } else {
            println!("HelloHandler: Yes you are");
            Ok(requests::Hello {})
        }
    }

    fn handle_request(_req: requests::Hello) -> Result<responses::Welcome, ReadError> {
        println!("HelloHandler: Welcome");
        Ok(responses::Welcome{})
    }
}
