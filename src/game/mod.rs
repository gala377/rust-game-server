pub mod error;
pub mod unit;

use self::error::GameError;
use self::unit::{
    Unit,
    UnitType,
    UnitState,
    UnitStats,
};

pub struct Game {
    num_of_players: u8,
    num_of_units: usize,
    board_size: (usize, usize),
    units: Vec<Unit>,
}

impl Game {
    
    pub fn new(num_of_players: u8, board_size: (usize, usize)) -> Game {
        assert!(board_size.0 > 0 && board_size.1 > 0);
        assert!(num_of_players > 1);
        Game {
            num_of_players,
            num_of_units: 0,
            board_size,
            units: Vec::new(),
        }
    }

    // future versions
    // maybe return result so the player will know 
    // which units where set outside the board
    pub fn add_unit(&mut self,
                owner_id: u8,
                position: (usize, usize),
                category: UnitType) {
        assert!(owner_id < self.num_of_players);
        assert!(position < self.board_size);
        self.units.push(
            Unit{
                id: self.num_of_units,
                owner_id,
                position,
                category,
                stats: Game::default_unit_stats(),
                state: UnitState::Idle,
            });
        self.num_of_units += 1;
    }

    pub fn get_unit(&self, x: usize, y: usize) -> Result<&Unit, GameError> {
       for u in &self.units {
           if u.position == (x, y) {
               return Ok(&u)
           }
       }    
       Err(GameError::NonExistingUnit) 
    }

    fn default_unit_stats() -> UnitStats {
        UnitStats {
            movement_range: 10,
            vision_range: 10,
            attack_range: 10,     
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn game_struct_creation_with_0_player() {
        Game::new(0, (5, 5));
    }

    #[test]
    #[should_panic]
    fn game_struct_creation_with_1_player() {
        Game::new(1, (5, 5));
    }

    #[test]
    #[should_panic]
    fn game_struct_creation_with_0_x_board() {
        Game::new(2, (0, 1));
    }

    #[test]
    #[should_panic]
    fn game_struct_creation_with_0_y_board() {
        Game::new(2, (1, 0));
    }

    #[test]
    fn game_struct_cretaion() {
        let g = Game::new(4, (10, 10));
        assert_eq!(g.num_of_players, 4);
        assert_eq!(g.board_size, (10, 10));
        assert_eq!(g.num_of_units, 0);
        assert_eq!(g.units.len(), 0);
    }

    #[test]
    #[should_panic]
    fn add_new_unit_for_non_existing_player() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(3, (1, 1), UnitType::Pickerman);
    }

    #[test]
    #[should_panic]
    fn add_new_unit_outside_the_board() {
        let mut g = Game::new(2, (10, 50));
        g.add_unit(1, (11, 20), UnitType::Knight);
    }

    #[test]
    fn add_new_unit_for_the_first_player() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (5, 5), UnitType::Knight);
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1);
    }

    #[test]
    fn add_new_unit_for_the_last_player() {
        let mut g = Game::new(10, (100, 100));
        g.add_unit(9, (25, 10), UnitType::Cavalry);
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1); 
    }

    #[test]
    fn add_multiple_units() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), UnitType::Cavalry);
        g.add_unit(3, (2, 1), UnitType::Cavalry);
        g.add_unit(2, (3, 1), UnitType::Cavalry);
        assert_eq!(g.num_of_units, 3);
        assert_eq!(g.units.len(), 3); 
    }

    #[test]
    fn check_unit_data_after_addition() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), UnitType::Pickerman);
        g.add_unit(3, (2, 1), UnitType::Cavalry);
        
        let u: &Unit = &g.units[1];
        
        assert_eq!(u.id, 1);
        assert_eq!(u.owner_id, 3);
        assert_eq!(u.position, (2, 1));
        assert!(match u.state {
             UnitState::Idle => true,
             _ => false,
        });
        assert!(match u.category {
             UnitType::Cavalry => true,
             _ => false,
        });

    }


    #[test]
    #[should_panic]
    fn add_new_unit_after_the_last_player() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(10, (1, 1), UnitType::Cavalry);
    }

    #[test]
    fn get_unit_by_position() {
        let mut g = Game::new(2, (5, 5));
        g.add_unit(0, (1, 1), UnitType::Cavalry);
        let u: &Unit = g.get_unit(1, 1).unwrap();

        assert_eq!(u.id, 0);
        assert_eq!(u.owner_id, 0);
        assert_eq!(u.position, (1, 1));
        assert!(match u.state {
             UnitState::Idle => true,
             _ => false,
        });
        assert!(match u.category {
             UnitType::Cavalry => true,
             _ => false,
        });
    }

    #[test]
    fn get_nonexisting_unit() {
        let g = Game::new(2, (5,5));
        let res = g.get_unit(4, 4);
        assert!(match res {
            Err(err) => {
                match err {
                    GameError::NonExistingUnit => true,
                }
            }
            _ => false,
        })
    }

}