use std::collections::HashMap;

use rand;

use crate::game::Game;

pub type PlayerID = String;

pub struct Agent<'a> {
    game_model: &'a mut Game,

    assigned_players: u8,
    player_ids: Vec<String>,
}

impl Agent<'_> {

    /// Creates new agent
    pub fn new(game_model: &mut Game) -> Agent {
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
        if self.assigned_players == self.game_model.get_num_of_players() {
            None 
        } else {
            self.assigned_players += 1;
            let id = Self::gen_player_id();
            self.player_ids.push(id.clone());
            Some(id)
        }
    }    

    fn gen_player_id() -> PlayerID {
        let mut id = String::new();
        for _i in 0..16 {
            id.push(rand::random())
        }
        id
    }
}
