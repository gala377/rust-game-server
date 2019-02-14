use crate::comm::{
    connection,
    connection::MSG_HEADER_LEN,
    errors::ReadError,
    handlers::{requests, responses, DefaultBuilder},
    MessageId, MessageRaw,
};
use crate::game;

use std::convert::From;
use std::str::FromStr;

pub struct Handler;

// todo WIP 
impl DefaultBuilder for Handler {
    type Request = requests::AddUnit;
    type Response = responses::UnitAdded;

    fn req_id() -> MessageId {
        1000
    }

    fn req_from_raw(raw: &MessageRaw) -> Result<requests::AddUnit, ReadError> {
        let trimmed_msg = &raw[MSG_HEADER_LEN..];
        
        if raw.len() != MSG_HEADER_LEN + 16 + 4 + 4 + 1 {
            return Err(ReadError::from(String::from_str(
                "invalid msg len").unwrap()))
        }

        eprintln!("AddUnit parsing request");
        eprintln!("Getting player id");
        // PlayerID
        let mut player_id_raw: [u8; 16] = [0; 16];
        player_id_raw.copy_from_slice(&trimmed_msg[..16]);
        let player_id = u128::from_le_bytes(player_id_raw);
        eprintln!("Got player id its {}", player_id);

        eprintln!("Getting pos_x");
        let mut pos_x_raw: [u8; 4] = [0; 4];
        pos_x_raw.copy_from_slice(&trimmed_msg[16..20]);
        let pos_x = u32::from_le_bytes(pos_x_raw);
        eprintln!("Got pos x its {}", pos_x);

        eprintln!("Getting pos_y");
        let mut pos_y_raw: [u8; 4] = [0; 4];
        pos_y_raw.copy_from_slice(&trimmed_msg[20..24]);
        let pos_y = u32::from_le_bytes(pos_y_raw);
        eprintln!("Got pos y its {}", pos_y);

        eprintln!("Getting category");
        let category_raw = raw.last().unwrap();
        eprintln!("Got category: {}", category_raw);
        let category = match category_raw {
            0 => game::unit::Category::Cavalry,
            1 => game::unit::Category::Knight,
            2 => game::unit::Category::Pickerman,
            _ => return Err(ReadError::from(String::from_str(
                "invalid unit category").unwrap()))
        };
        eprintln!("Returning AddUnit Request");
        Ok(requests::AddUnit{
            player_id,
            position: (pos_x as usize, pos_y as usize),
            category,
        })
    }

    fn handle_request(
        req: requests::AddUnit,
        ctx: &mut connection::Context,
    ) -> Result<responses::UnitAdded, ReadError> {
        if !ctx.initialized {
            return Err(ReadError::from(String::from_str(
                "Handshake is required to register as a player").unwrap()));
        }
        let id = match ctx.game_agent.write() {
            Ok(mut guard) => {
                match (*guard).add_unit(req.player_id, req.position, req.category) {
                    Some(val) => val,
                    None => {
                        eprintln!(
                            "[{:^15}]: Could not add an unit",
                            "DefaultBuilderHandler");
                        return Err(ReadError::from(format!(
                            "Internal server error")));
                    },
                }
            },
            Err(err) => {
                eprintln!(
                    "[{:^15}]: Could not acquire write lock on a game agent: {}",
                    "DefaultBuilderHandler",
                    err);
                return Err(ReadError::from(format!(
                    "Internal server error: {}",
                    err,
                )));
                
            },
        };
        Ok(responses::UnitAdded{
            unit_id: id,
        })
    }
}