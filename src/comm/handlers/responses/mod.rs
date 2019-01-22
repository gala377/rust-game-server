use crate::agent;
use crate::comm::{Message, MessageId, Payload};

pub struct Welcome;

impl Message for Welcome {
    fn id(&self) -> MessageId {
        1
    }

    fn payload(&self) -> Payload {
        Vec::new()
    }
}

pub struct PlayerRegistered {
    pub player_id: agent::PlayerID,
}

impl Message for PlayerRegistered {
    fn id(&self) -> MessageId {
        2
    }

    fn payload(&self) -> Payload {
        self.player_id.to_le_bytes().to_vec()
    }
}