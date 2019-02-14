use rand;

use crate::game::{
    Game,
    unit,
};

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

    // todo make it a result, option is just
    // to have some mvp
    pub fn add_unit(
        &mut self,
        player: PlayerID,
        position: (usize, usize),
        category: unit::Category) -> Option<usize> {
        let owner_id = match self.player_id_to_inner(player) {
            None => return None,
            Some(val) => val,
        };
        match self.game_model.add_unit(owner_id as u8, position, category) {
            Ok(unit) => Some(unit.id),
            Err(_) => None,
        }
    }

    pub fn get_units(&self, player: PlayerID) -> Option<Vec<&unit::Unit>> {
        let owner_id = match self.player_id_to_inner(player) {
            None => return None,
            Some(val) => val as u8,
        };
        let player = match self.game_model.get_player(owner_id) {
            None => return None,
            Some(val) => val,
        };
        match self.game_model.get_units(player.units.clone()) {
            Err(_) => None,
            Ok(res) => Some(res),
        }
    }

    fn gen_player_id() -> PlayerID {
        rand::random()
    }

    fn player_id_to_inner(&self, id: PlayerID) -> Option<usize> {
        for i in 0..self.player_ids.len() {
            if self.player_ids[i] == id {
                return Some(i);
            }
        }
        None
    }
}
