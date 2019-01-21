use crate::comm::{Message, MessageId, Payload};

pub struct Hello;

impl Message for Hello {
    fn id(&self) -> MessageId {
        0
    }

    fn payload(&self) -> Payload {
        Vec::new()
    }
}
