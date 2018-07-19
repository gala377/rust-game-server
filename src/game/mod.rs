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

    fn assert_position_in_board(&self, x: usize, y: usize) -> Result<(), GameError> {
        if x >= self.board_size.0 || y >= self.board_size.1 {
            return Err(GameError::PositionOutsideTheBoard(x, y))
        }
        Ok(())
    }

    fn assert_unit_move_within_reach(u: &Unit, (x, y): (usize, usize)) -> Result<(), GameError> {
        let pos = &u.position;
        let x_diff = (pos.0 as i32 - x as i32).abs() as usize;
        let y_diff = (pos.1 as i32 - y as i32).abs() as usize;
        if x_diff + y_diff > u.stats.movement_range {
            return Err(GameError::MoveOutsideUnitsReach(x, y))
        }
        Ok(())
    }

    // todo - all methods below need to be tested actually

    pub fn move_unit(&mut self, unit_id: usize, (x, y): (usize, usize)) -> Result<(), GameError> {
        self.assert_position_in_board(x, y)?;
        let unit = self.unit_by_id(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = UnitState::Moving(x, y);
        Ok(())
    }

    pub fn battle_units(&mut self, u1_id: usize, u2_id: usize) -> Result<(), GameError> {
        let mut units = self.units_by_id(vec![u1_id, u2_id])?;
        
        assert!(units.len() == 2);
        let u1_pos = units[0].position;
        let u2_pos = units[1].position;

        let (x, y) = (
            (u1_pos.0 + u2_pos.0)/2,
            (u1_pos.1 + u2_pos.1)/2);

        units[0].state = UnitState::Attack(x, y);
        units[1].state = UnitState::Attack(x, y);

        Ok(())
    }
   
    fn attack_position(&mut self, unit_id: usize, (x, y): (usize, usize)) -> 
        Result<(), GameError> {
        
        self.assert_position_in_board(x, y)?;
        let unit = self.unit_by_id(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = UnitState::Attack(x, y);
        Ok(())
    }

    fn unit_by_id(&mut self, id: usize) -> Result<&mut Unit, GameError> {
        for u in &mut self.units {
            if u.id == id {
                return Ok(u)
            }
        }
        Err(GameError::NonExistingUnit)
    }

    fn units_by_id(&mut self, ids: Vec<usize>) -> Result<Vec<&mut Unit>, GameError> {
        let mut units = Vec::new();
        for u in &mut self.units {
            if ids.contains(&u.id) {
                units.push(u);
            }
        }
        if units.len() == 0 {
            return Err(GameError::NonExistingUnit)
        } 
        Ok(units)
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
                    _ => false,
                }
            }
            _ => false,
        })
    }

    #[test]
    fn move_unit_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry);
        assert!(match g.move_unit(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(2, 2).unwrap();
                if let UnitState::Moving(4, 4) = u.state {
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        });
    }

    #[test]
    fn move_unit_outside_boundaries() {
        let mut g = Game::new(3, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry);
        assert!(match g.move_unit(0, (12, 2)) {
            Ok(_) => false,
            Err(_) => true,
        });
    }

    #[test]
    fn attack_position_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry);
        assert!(match g.attack_position(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(2, 2).unwrap();
                if let UnitState::Attack(4, 4) = u.state {
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        });
    }

    #[test]
    fn attack_position_outside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry);
        assert!(match g.attack_position(0, (11, 10)) {
            Ok(_) => false,
            Err(_) => true,
        });
    }

    // todo move and attack outside units range

}