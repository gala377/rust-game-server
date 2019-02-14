use rand;

use crate::game::Game;

pub type PlayerID = u128;

pub struct Agent {
    game_model: Game,

    assigned_players: u8,
    // todo make it a HashMap maybe? for faster access...?
    player_ids: Vec<PlayerID>,
}

impl Agent {

    /// Creates new agent
    pub fn new(game_model: Game) -> Agent {
        Agent {
            game_model,
            assigned_players: 0,
            player_ids: Vec::new(),
        }
    }
    
    // todo now its an Option. Rewrite it later to Result
    // with descriptive error message 
    // todo make it some identifier mapped to id
    pub fn register_player(&mut self) -> Option<PlayerID> {
        if self.assigned_players == self.game_model.num_of_players() {
            None 
        } else {
            self.assigned_players += 1;
            let id = Self::gen_player_id();
            self.player_ids.push(id.clone());
            Some(id)
        }
    }    

    fn gen_player_id() -> PlayerID {
        rand::random()
    }
}
