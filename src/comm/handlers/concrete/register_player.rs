use crate::comm::{
    connection,
    connection::MSG_HEADER_LEN,
    errors::ReadError,
    handlers::{requests, responses, DefaultBuilder},
    MessageId, MessageRaw,
};
use std::convert::From;
use std::str::FromStr;

pub struct Handler;

impl DefaultBuilder for Handler {
    type Request = requests::RegisterPlayer;
    type Response = responses::PlayerRegistered;

    fn req_id() -> MessageId {
        1
    }

    fn req_from_raw(raw: &MessageRaw) -> Result<requests::RegisterPlayer, ReadError> {
        if raw.len() != MSG_HEADER_LEN {
            Err(ReadError::from(format!(
                "Message len is incorrect. Expected: {}. Actual: {}.",
                MSG_HEADER_LEN,
                raw.len()
            )))
        } else {
            Ok(requests::RegisterPlayer {})
        }
    }

    fn handle_request(
        _req: requests::RegisterPlayer,
        ctx: &mut connection::Context,
    ) -> Result<responses::PlayerRegistered, ReadError> {
        if !ctx.initialized {
            return Err(ReadError::from(String::from_str(
                "Handshake is required to register as a player").unwrap()));
        }
        let player_id = match ctx.game_agent.write() {
            Ok(mut guard) => {
                (*guard).register_player()
            },
            Err(err) => {
                eprintln!(
                    "[{:^15}]: Could not acquire write lock on a game agent: {}",
                    "RegisterPlayerHandler",
                    err);
                return Err(ReadError::from(format!(
                    "Internal server error: {}",
                    err,
                )));
                
            },
        };
        match player_id {
            None => Err(ReadError::from(
                String::from_str("All players already registered!").unwrap())),
            Some(val) => {
                eprintln!("Got player id as: {}", val);
                Ok(responses::PlayerRegistered{
                    player_id: val,
                })
            },
        }
    }
}
