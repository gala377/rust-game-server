use super::super::{Message, MessageId, Payload};

pub struct Welcome;

impl Message for Welcome {
    fn id(&self) -> MessageId {
        1
    }

    fn payload(&self) -> Payload {
        Vec::new()
    }
}
