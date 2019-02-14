use crate::agent;
use crate::comm::{Message, MessageId, Payload};
use crate::game;

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

// todo WIP
pub struct UnitAdded {
    pub unit_id: usize,
}

impl Message for UnitAdded {
    fn id(&self) -> MessageId {
        1000
    }

    fn payload(&self) -> Payload {
        self.unit_id.to_le_bytes().to_vec()
    }
}

// todo WIP
pub struct UnitsList<'a> {
    pub units: Vec<&'a game::unit::Unit>,
}

impl Message for UnitsList<'_> {
    fn id(&self) -> MessageId {
        1001
    }

    fn payload(&self) -> Payload {
        let mut payload = Vec::new();
        payload.extend(self.units.len().to_le_bytes().to_vec());
        for unit in &self.units {
            payload.extend(Self::unit_to_bytes(unit));   
        }
        payload 
    }
}

impl UnitsList<'_> {
    fn unit_to_bytes(unit: &game::unit::Unit) -> Vec<u8> {
        let mut as_bytes = Vec::<u8>::new();
        as_bytes.extend(unit.id.to_le_bytes().to_vec());
        as_bytes.extend(unit.position.0.to_le_bytes().to_vec());
        as_bytes.extend(unit.position.1.to_le_bytes().to_vec());
        as_bytes
    }
}