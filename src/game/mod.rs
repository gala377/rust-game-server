pub mod error;
pub mod unit;

use std::collections::HashSet;

use self::error::GameError;
use self::unit::{
    Unit,
    UnitType,
    UnitState,
    UnitStats,
};




// todo document
pub struct Game {
    num_of_players: u8,
    num_of_units: usize,
    board_size: (usize, usize),
    units: Vec<Unit>,
}


// todo document
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


    fn default_unit_stats() -> UnitStats {
        UnitStats {
            movement_range: 10,
            vision_range: 10,
            attack_range: 10,     
        }
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

}


// todo document
impl Game {
    

    pub fn add_unit<'a>(&'a mut self,
                owner_id: u8,
                position: (usize, usize),
                category: UnitType) -> Result<&'a Unit, GameError> {
        assert!(owner_id < self.num_of_players);
        self.assert_position_in_board(position)?;
        self.units.push(Unit{
                id: self.num_of_units,
                owner_id,
                position,
                category,
                stats: Game::default_unit_stats(),
                state: UnitState::Idle,
        });
        self.num_of_units += 1;
        match self.units.last() {
            Some(val) => Ok(val),
            None => Err(GameError::NonExistingUnit(self.num_of_units-1)),
    }
    }

    fn assert_position_in_board(&self, (x, y): (usize, usize)) -> Result<(), GameError> {
        if (x, y) >= self.board_size {
            return Err(GameError::PositionOutsideTheBoard(x, y))
        }
        Ok(())
    }


    pub fn get_unit(&self, unit_id: usize) -> Result<&Unit, GameError> {
        for u in &self.units {
            if u.id == unit_id {
                return Ok(u)
            }
        }
        Err(GameError::NonExistingUnit(unit_id))
    }


    fn get_unit_mut(&mut self, unit_id: usize) -> Result<&mut Unit, GameError> {
        for u in &mut self.units {
            if u.id == unit_id {
                return Ok(u)
        }
    }
        Err(GameError::NonExistingUnit(unit_id))
    }

    // todo tests
    fn get_units_mut(&mut self, ids: Vec<usize>) -> Result<Vec<&mut Unit>, GameError> {
        let mut units = Vec::new();
        let mut found_units = HashSet::new();
        let requested_units: HashSet<usize> = ids.iter().cloned().collect();
        for u in &mut self.units {
            if ids.contains(&u.id) {
                found_units.insert(u.id);
                units.push(u);
        }
    }
        if requested_units != found_units {
            let mut diff = requested_units.difference(&found_units);
            return match diff.next() {
                Some(&val) => Err(GameError::NonExistingUnit(val)),
                None => panic!("This sets difference should never be empty!"),
        }
    }
        Ok(units)
    }


    pub fn move_unit(&mut self, unit_id: usize, (x, y): (usize, usize)) -> Result<(), GameError> {
        self.assert_position_in_board((x, y))?;
        let unit = self.get_unit_mut(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = UnitState::Moving(x, y);
        Ok(())
    }


    // todo tests
    pub fn battle_units(&mut self, u1_id: usize, u2_id: usize) -> Result<(), GameError> {
        let mut units = self.get_units_mut(vec![u1_id, u2_id])?;
        
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
   

    pub fn attack_position(&mut self, unit_id: usize, (x, y): (usize, usize)) -> 
        Result<(), GameError> {
        
        self.assert_position_in_board((x, y))?;
        let unit = self.get_unit_mut(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = UnitState::Attack(x, y);
        Ok(())
    }


    // todo test
    pub fn resolve_moves(&mut self) {
        {   
            let mut unresolved = self.units_to_be_moved();
            while unresolved.len() > 0 {
                // todo body
            }
        }
        self.resolve_blockades();
    }


    // todo test
    fn units_to_be_moved(&mut self) -> Vec<&mut Unit> {
        let mut res = Vec::new();
        for u in &mut self.units {
            match u.state {
                UnitState::Moving(..) | UnitState::Attack(..) => {
                    res.push(u);
                }
                _ => (), 
            }
        }
        res
    }


    // todo test
    fn resolve_blockades(&mut self) {
        // todo body
    }


    // todo test
    pub fn game_over(&self) -> Option<usize> {
        // todo body
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! assert_match {
        ($e:expr, $( $p:pat )+) => {
            assert!(match $e {
                $(
                    $p => true,
                ),*
                _ => false,
            });
        }
    }

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
    fn game_struct_creation() {
        let g = Game::new(4, (10, 10));
        assert_eq!(g.units.len(), 0);
        assert_match!(
            g,
            Game {
                num_of_players: 4,
                board_size: (10, 10),
                num_of_units: 0,
                ..
            });
    }

    #[test]
    #[should_panic]
    fn add_new_unit_for_non_existing_player() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(3, (1, 1), UnitType::Pickerman).unwrap();
    }

    #[test]
    fn add_new_unit_outside_the_board() {
        let mut g = Game::new(2, (10, 50));
        assert_match!(
            g.add_unit(1, (11, 20), UnitType::Knight),
            Err(GameError::PositionOutsideTheBoard(..)));
    }

    #[test]
    fn add_new_unit_for_the_first_player() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (5, 5), UnitType::Knight).unwrap();
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1);
    }

    #[test]
    fn add_new_unit_for_the_last_player() {
        let mut g = Game::new(10, (100, 100));
        g.add_unit(9, (25, 10), UnitType::Cavalry).unwrap();
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1); 
    }

    #[test]
    fn add_multiple_units() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), UnitType::Cavalry).unwrap();
        g.add_unit(3, (2, 1), UnitType::Cavalry).unwrap();
        g.add_unit(2, (3, 1), UnitType::Cavalry).unwrap();
        assert_eq!(g.num_of_units, 3);
        assert_eq!(g.units.len(), 3); 
    }

    #[test]
    fn check_unit_data_after_addition() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), UnitType::Pickerman).unwrap();
        g.add_unit(3, (2, 1), UnitType::Cavalry).unwrap();  
        assert_match!(
            &g.units[1],
            Unit {
                state: UnitState::Idle,
                category: UnitType::Cavalry,
                id: 1,
                owner_id: 3,
                position: (2, 1),
                ..
            });
    }


    #[test]
    #[should_panic]
    fn add_new_unit_after_the_last_player() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(10, (1, 1), UnitType::Cavalry).unwrap();
    }

    #[test]
    fn get_unit() {
        let mut g = Game::new(2, (5, 5));
        g.add_unit(0, (1, 1), UnitType::Cavalry).unwrap();
        assert_match!(
            g.get_unit(0),
            Ok(Unit{
                state: UnitState::Idle,
                category: UnitType::Cavalry,
                id: 0,
                owner_id: 0,
                position: (1, 1),
                ..
            })
        );
    }

    #[test]
    fn get_noexisting_unit() {
        let g = Game::new(2, (5,5));
        assert_match!(g.get_unit(0), Err(GameError::NonExistingUnit(0)))
    }

    #[test]
    fn move_unit_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry).unwrap();
        assert!(match g.move_unit(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(0).unwrap();
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
        g.add_unit(0, (2, 2), UnitType::Cavalry).unwrap();
        assert_match!(g.move_unit(0, (12, 2)), Err(_));
    }

    #[test]
    fn attack_position_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), UnitType::Cavalry).unwrap();
        assert!(match g.attack_position(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(0).unwrap();
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
        g.add_unit(0, (2, 2), UnitType::Cavalry).unwrap();
        assert_match!(g.attack_position(0, (11, 10)), Err(_))
    }

    #[test]
    fn move_unit_outside_unit_range() {
        let mut g = Game::new(2, (20,20));
        g.add_unit(0, (0, 0), UnitType::Pickerman).unwrap();
        assert_match!(g.move_unit(0, (19, 19)), Err(_));
    }

    #[test]
    fn attack_position_outside_unit_range() {
        let mut g = Game::new(2, (20,20));
        g.add_unit(0, (0, 0), UnitType::Knight).unwrap();
        assert_match!(g.attack_position(0, (19, 19)), Err(_));
    }

}