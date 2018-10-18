pub mod error;
pub mod unit;

use std::collections::HashSet;
use std::collections::BinaryHeap;

use self::error::GameError;
use self::unit::Unit;

/// Game represents current game state.
/// See documentation for internal logic.
pub struct Game {
    /// Num of players (active and inactive).
    num_of_players: u8,
    /// Number of units currently in play (active).
    num_of_units: usize,
    /// Boundaries of the game board.
    board_size: (usize, usize),
    /// Units currently in play (active).
    units: Vec<Unit>,
}

impl Game {
    /// Creates new Game struct with properties set as provided.
    /// Panics if user tries to create invalid game:
    ///     num_of_players < 2,
    ///     board_size < (1, 1)
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

    /// Returns placeholder unit stats.
    /// Possible extension in future release.
    fn default_unit_stats() -> unit::Stats {
        unit::Stats {
            movement_range: 10,
            vision_range: 10,
            attack_range: 10,
        }
    }

    /// Checks if requested move doesn't violate unit's stats.
    /// todo the same for the Attack state.
    fn assert_unit_move_within_reach(u: &Unit, (x, y): (usize, usize)) -> Result<(), GameError> {
        let pos = &u.position;
        let x_diff = (pos.0 as i32 - x as i32).abs() as usize;
        let y_diff = (pos.1 as i32 - y as i32).abs() as usize;
        if x_diff + y_diff > u.stats.movement_range {
            return Err(GameError::MoveOutsideUnitsReach(x, y));
        }
        Ok(())
    }
}

impl Game {
    /// Adds new Unit to the game.
    /// Provides id, default stats and sets state to Idle.
    ///
    /// Panics on attempt to add unit to the noexistig player.
    /// Returns error on attempt to add unit utside the board boundaries
    /// or on failure when adding the unit.
    pub fn add_unit<'a>(
        &'a mut self,
        owner_id: u8,
        position: (usize, usize),
        category: unit::Category,
    ) -> Result<&'a Unit, GameError> {
        assert!(owner_id < self.num_of_players);
        self.assert_position_in_board(position)?;
        self.units.push(Unit {
            id: self.num_of_units,
            owner_id,
            position,
            category,
            stats: Game::default_unit_stats(),
            state: unit::State::Idle,
        });
        self.num_of_units += 1;
        match self.units.last() {
            Some(val) => Ok(val),
            None => Err(GameError::NonExistingUnit(self.num_of_units - 1)),
        }
    }

    /// Checks if given position is inside the currents board boundaries.
    /// If true return Ok(()). PositionOutsideTheBoard otherwise.
    fn assert_position_in_board(&self, (x, y): (usize, usize)) -> Result<(), GameError> {
        if (x, y) >= self.board_size {
            return Err(GameError::PositionOutsideTheBoard(x, y));
        }
        Ok(())
    }

    /// Given unit id returns reference to it.
    /// If there is no unit with the given id returns NonExistingUnit
    pub fn get_unit(&self, unit_id: usize) -> Result<&Unit, GameError> {
        for u in &self.units {
            if u.id == unit_id {
                return Ok(u);
            }
        }
        Err(GameError::NonExistingUnit(unit_id))
    }

    /// The same as get_unit but the reference is mutable.
    fn get_unit_mut(&mut self, unit_id: usize) -> Result<&mut Unit, GameError> {
        for u in &mut self.units {
            if u.id == unit_id {
                return Ok(u);
            }
        }
        Err(GameError::NonExistingUnit(unit_id))
    }

    /// Returns vec of references to the units with id's specified
    /// in the ids parameter.
    /// It's important to note, that this method returns Ok only if all of the
    /// provided id's mathched.
    /// If any of the provided ids doesn't map itself to an active unit method
    /// returns Err(NonExistingUnit(unit_id)) where unit_id is the first
    /// nomatching id.
    pub fn get_units(&self, ids: Vec<usize>) -> Result<Vec<&Unit>, GameError> {
        let mut units = Vec::new();
        let mut found_units = HashSet::new();
        let requested_units: HashSet<usize> = ids.iter().cloned().collect();
        for u in &self.units {
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
            };
        }
        Ok(units)
    }

    /// Same as get_units just returns mut refereces.
    pub fn get_units_mut(&mut self, ids: Vec<usize>) -> Result<Vec<&mut Unit>, GameError> {
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
            };
        }
        Ok(units)
    }

    /// After movement assertions changes unit state
    /// to Moving at given postion.
    pub fn move_unit(&mut self, unit_id: usize, (x, y): (usize, usize)) -> Result<(), GameError> {
        self.assert_position_in_board((x, y))?;
        let unit = self.get_unit_mut(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = unit::State::Moving(x, y);
        Ok(())
    }

    /// Gets both units. Sets their state as attack and the position as
    /// average of both of their positions.
    /// If there was an error, no change will be made in both of the units.
    pub fn battle_units(&mut self, u1_id: usize, u2_id: usize) -> Result<(), GameError> {
        let old_state: unit::State;
        let (x, y): (usize, usize);
        {
            let units = self.get_units(vec![u1_id, u2_id])?;

            assert!(units.len() == 2);
            let u1_pos = units[0].position;
            let u2_pos = units[1].position;

            x = (u1_pos.0 + u2_pos.0) / 2;
            y = (u1_pos.1 + u2_pos.1) / 2;

            old_state = units[0].state;
        }
        self.attack_position(u1_id, (x, y))?;
        match self.attack_position(u2_id, (x, y)) {
            ret @ Ok(()) => ret,
            ret @ Err(_) => match self.get_unit_mut(u1_id) {
                Ok(u) => {
                    u.state = old_state;
                    ret
                }
                _ => ret,
            },
        }
    }

    /// After movement assertions changes unit state
    /// to Moving at given postion.
    pub fn attack_position(
        &mut self,
        unit_id: usize,
        (x, y): (usize, usize),
    ) -> Result<(), GameError> {
        self.assert_position_in_board((x, y))?;
        let unit = self.get_unit_mut(unit_id)?;
        Game::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = unit::State::Attack(x, y);
        Ok(())
    }

    // todo test, doc
    pub fn resolve_moves(&mut self) {
        let mut unresolved = self.units_to_be_moved();
        while unresolved.len() > 0 {
            unresolved = self.make_move(unresolved);
        };
        self.resolve_blockades()
    }

    // todo test
    /// Returns vec of ids of the units that require movnig actions.
    fn units_to_be_moved(&self) -> BinaryHeap<unit::MovingWrapper> {
        let mut res = BinaryHeap::new();
        for u in &self.units {
            match u.state {
                unit::State::Moving(..) | unit::State::Attack(..) => {
                    res.push(
                        unit::MovingWrapper::new(u.id));
                }
                _ => (),
            }
        }
        res
    }

    // todo test, doc
    fn make_move(&mut self, mut units: BinaryHeap<unit::MovingWrapper>) -> BinaryHeap<unit::MovingWrapper> {
        let filtered = BinaryHeap::new();
        for u in units.drain() {
            self.resolve_unit(&u);
        }
        filtered
    }

    // todo test, doc
    fn resolve_unit(&mut self, unit: &unit::MovingWrapper) {
        // todo make a match case
        let u_state = self.get_unit(unit.unit_id).unwrap().state;
        match u_state {
            unit::State::Moving(x, y)  => {
                if self.field_empty((x, y)) {
                        let u = self.get_unit_mut(unit.unit_id).unwrap();
                //      if x, y = unit.position change state to idle
                //      if enemy unit in vision change state to idle
                //      create new unit with updated moves made
                //      return it
                }
            },
            unit::State::Attack(x, y) => {
            },
            _ => {},
        }
    }

    // todo test
    /// Checks whether board field is not occupied by
    /// any unit.
    fn field_empty(&self, (x, y): (usize, usize)) -> bool {
        for u in &self.units {
            if u.position == (x, y) {
                return false;
            } 
        }
        true
    }

    // todo test, doc
    fn resolve_blockades(&mut self) {
        // todo body
    }

    // todo test
    /// If the game os over returns Ok with the id of the
    /// player who won. None otherwise.
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

    macro_rules! assert_match_debug {
        ($e:expr, $( $p:pat )+) => {
            assert!(match $e {
                $(
                    $p => true,
                ),*
                actual @ _ => {
                    println!("actual is {:?}", actual);
                    false
                },
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
        g.add_unit(3, (1, 1), unit::Category::Pickerman).unwrap();
    }

    #[test]
    fn add_new_unit_outside_the_board() {
        let mut g = Game::new(2, (10, 50));
        assert_match!(
            g.add_unit(1, (11, 20), unit::Category::Knight),
            Err(GameError::PositionOutsideTheBoard(..))
        );
    }

    #[test]
    fn add_new_unit_for_the_first_player() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (5, 5), unit::Category::Knight).unwrap();
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1);
    }

    #[test]
    fn add_new_unit_for_the_last_player() {
        let mut g = Game::new(10, (100, 100));
        g.add_unit(9, (25, 10), unit::Category::Cavalry).unwrap();
        assert_eq!(g.num_of_units, 1);
        assert_eq!(g.units.len(), 1);
    }

    #[test]
    fn add_multiple_units() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(3, (2, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(2, (3, 1), unit::Category::Cavalry).unwrap();
        assert_eq!(g.num_of_units, 3);
        assert_eq!(g.units.len(), 3);
    }

    #[test]
    fn check_unit_data_after_addition() {
        let mut g = Game::new(10, (10, 10));
        g.add_unit(0, (1, 1), unit::Category::Pickerman).unwrap();
        g.add_unit(3, (2, 1), unit::Category::Cavalry).unwrap();
        assert_match!(
            &g.units[1],
            Unit {
                state: unit::State::Idle,
                category: unit::Category::Cavalry,
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
        g.add_unit(10, (1, 1), unit::Category::Cavalry).unwrap();
    }

    #[test]
    fn test_get_unit() {
        let mut g = Game::new(2, (5, 5));
        g.add_unit(0, (1, 1), unit::Category::Cavalry).unwrap();
        assert_match!(
            g.get_unit(0),
            Ok(Unit {
                state: unit::State::Idle,
                category: unit::Category::Cavalry,
                id: 0,
                owner_id: 0,
                position: (1, 1),
                ..
            })
        );
    }

    #[test]
    fn test_get_units() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (1, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (2, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (3, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (4, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (5, 1), unit::Category::Pickerman).unwrap();
        let units = g.get_units(vec![0, 3, 4]);
        assert_match!(units, Ok(_));
        let units = units.unwrap();
        assert_match!(units[0].category, unit::Category::Cavalry);
        assert_match!(units[1].category, unit::Category::Knight);
        assert_match!(units[2].category, unit::Category::Pickerman);
        assert!(units.len() == 3);
    }

    #[test]
    fn get_units_with_one_being_noexisting() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (1, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (2, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (3, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (4, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (5, 1), unit::Category::Pickerman).unwrap();
        let units = g.get_units(vec![0, 3, 5]);
        assert_match!(units, Err(GameError::NonExistingUnit(5)));
    }

    #[test]
    fn get_few_noexisting_units() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (1, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (2, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (3, 1), unit::Category::Cavalry).unwrap();
        g.add_unit(0, (4, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (5, 1), unit::Category::Pickerman).unwrap();
        let units = g.get_units(vec![0, 3, 6, 7, 8]);
        assert_match!(units, Err(GameError::NonExistingUnit(6)));
    }

    #[test]
    fn get_noexisting_unit() {
        let g = Game::new(2, (5, 5));
        assert_match!(g.get_unit(0), Err(GameError::NonExistingUnit(0)))
    }

    #[test]
    fn move_unit_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), unit::Category::Cavalry).unwrap();
        assert!(match g.move_unit(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(0).unwrap();
                if let unit::State::Moving(4, 4) = u.state {
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
        g.add_unit(0, (2, 2), unit::Category::Cavalry).unwrap();
        assert_match!(g.move_unit(0, (12, 2)), Err(_));
    }

    #[test]
    fn attack_position_inside_boundaries() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (2, 2), unit::Category::Cavalry).unwrap();
        assert!(match g.attack_position(0, (4, 4)) {
            Ok(_) => {
                let u = g.get_unit(0).unwrap();
                if let unit::State::Attack(4, 4) = u.state {
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
        g.add_unit(0, (2, 2), unit::Category::Cavalry).unwrap();
        assert_match!(g.attack_position(0, (11, 10)), Err(_))
    }

    #[test]
    fn move_unit_outside_unit_range() {
        let mut g = Game::new(2, (20, 20));
        g.add_unit(0, (0, 0), unit::Category::Pickerman).unwrap();
        assert_match!(g.move_unit(0, (19, 19)), Err(_));
    }

    #[test]
    fn attack_position_outside_unit_range() {
        let mut g = Game::new(2, (20, 20));
        g.add_unit(0, (0, 0), unit::Category::Knight).unwrap();
        assert_match!(g.attack_position(0, (19, 19)), Err(_));
    }

    #[test]
    fn battle_units() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (0, 0), unit::Category::Knight).unwrap();
        g.add_unit(0, (2, 2), unit::Category::Cavalry).unwrap();
        assert_match!(g.battle_units(0, 1), Ok(()));
        let unit = g.get_unit(0).unwrap();
        assert_match!(
            unit,
            Unit {
                id: 0,
                category: unit::Category::Knight,
                state: unit::State::Attack(1, 1),
                ..
        });
        let unit = g.get_unit(1).unwrap();
        assert_match!(
            unit,
            Unit {
                id: 1,
                category: unit::Category::Cavalry,
                state: unit::State::Attack(1, 1),
                ..
        });
    }

    #[test]
    fn battle_non_existing_unit_second() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (0, 0), unit::Category::Knight).unwrap();
        assert_match!(g.battle_units(0, 1), Err(GameError::NonExistingUnit(1)));
    }

    #[test]
    fn battle_non_existing_unit_first() {
        let mut g = Game::new(2, (10, 10));
        g.add_unit(0, (0, 0), unit::Category::Knight).unwrap();
        assert_match!(g.battle_units(1, 0), Err(GameError::NonExistingUnit(1)));
    }

    #[test]
    fn battle_units_outside_their_reach() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (0, 0), unit::Category::Knight).unwrap();
        g.add_unit(1, (99, 99), unit::Category::Knight).unwrap();
        assert_match!(g.battle_units(0, 1), Err(GameError::MoveOutsideUnitsReach(49, 49)));
        assert_match!(
            g.get_unit(0).unwrap(),
            Unit {
                id: 0,
                category: unit::Category::Knight,
                state: unit::State::Idle,
                position: (0, 0),
                ..
            });
        assert_match!(
            g.get_unit(1).unwrap(),
            Unit {
                id: 1,
                category: unit::Category::Knight,
                state: unit::State::Idle,
                position: (99, 99),
                ..
            });
    }
}   
