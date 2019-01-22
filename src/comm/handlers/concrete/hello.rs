use crate::comm::{
    connection, 
    connection::MSG_HEADER_LEN,
    errors::ReadError, 
    MessageId, MessageRaw, 
    handlers::{
        requests,
        responses,
        DefaultBuilder
    },
};

pub struct Handler;

impl DefaultBuilder<requests::Hello, responses::Welcome> for Handler {
    fn req_id() -> MessageId {
        0
    }

    fn req_from_raw(raw: &MessageRaw) -> Result<requests::Hello, ReadError> {
        eprintln!("[{:^15}]: Are you hello?", "HelloHandler");
        if raw.len() != MSG_HEADER_LEN {
            eprintln!("[{:^15}]: You are not", "HelloHandler");
            Err(ReadError::from(format!(
                "Message len is incorrect. Expected: {}. Actual: {}.",
                MSG_HEADER_LEN,
                raw.len()
            )))
        } else {
            eprintln!("[{:^15}]: Yes you are", "HelloHandler");
            Ok(requests::Hello {})
        }
    }

    fn handle_request(_req: requests::Hello, ctx: &mut connection::Context) -> Result<responses::Welcome, ReadError> {
        eprintln!("[{:^15}]: Welcome", "HelloHandler");
        ctx.initialized = true;
        Ok(responses::Welcome {})
    }
}
