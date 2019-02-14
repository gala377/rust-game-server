use crate::comm::{Message, MessageId, Payload};
use crate::agent;
use crate::game;

pub struct Hello;

impl Message for Hello {
    fn id(&self) -> MessageId {
        0
    }

    fn payload(&self) -> Payload {
        Vec::new()
    }
}

pub struct RegisterPlayer;

impl Message for RegisterPlayer {
    
    fn id(&self) -> MessageId {
        1
    }

    fn payload(&self) -> Payload {
        Vec::new()
    }
}

pub struct AddUnit {
    pub player_id: agent::PlayerID,
    pub position: (usize, usize),
    pub category: game::unit::Category,
}

impl Message for AddUnit {
    fn id(&self) -> MessageId {
        1000
    }

    fn payload(&self) -> Payload {
        let mut payload = Vec::new();
        payload.extend(self.player_id.to_le_bytes().to_vec());
        payload.extend(self.position.0.to_le_bytes().to_vec());
        payload.extend(self.position.1.to_le_bytes().to_vec());
        payload
    }
}

pub struct GetUnits {
    pub player_id: agent::PlayerID,
}

impl Message for GetUnits {
    fn id(&self) -> MessageId {
        1001
    }

    fn payload(&self) -> Payload {
        let mut payload = Vec::new();
        payload.extend(self.player_id.to_le_bytes().to_vec());
        payload
    }
}
