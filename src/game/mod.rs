// All of this module is considered WIP

pub mod error;
mod helpers;
pub mod unit;

use std::collections::BinaryHeap;
use std::collections::HashSet;

use self::error::GameError;
use self::helpers::Coords;
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
    //todo Rewrite to generiational index. RustConf ECS
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
}

impl Game {
    /// Returns value od the field num_of_players 
    pub fn get_num_of_players(&self) -> u8 {
        self.num_of_players
    }

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
            stats: helpers::default_unit_stats(),
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
    fn assert_position_in_board(&self, (x, y): Coords) -> Result<(), GameError> {
        if (x, y) >= self.board_size {
            return Err(GameError::PositionOutsideTheBoard(x, y));
        }
        Ok(())
    }

    /// Given unit id returns reference to it.
    /// If there is no unit with the given id returns NonExistingUnit
    pub fn get_unit(&self, unit_id: usize) -> Result<&Unit, GameError> {
        match self.units.iter().find(|unit| unit.id == unit_id) {
            Some(val) => Ok(val),
            None => Err(GameError::NonExistingUnit(unit_id)),
        }
    }

    /// The same as get_unit but the reference is mutable.
    fn get_unit_mut(&mut self, unit_id: usize) -> Result<&mut Unit, GameError> {
        match self.units.iter_mut().find(|unit| unit.id == unit_id) {
            Some(val) => Ok(val),
            None => Err(GameError::NonExistingUnit(unit_id)),
        }
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
    pub fn move_unit(&mut self, unit_id: usize, (x, y): Coords) -> Result<(), GameError> {
        self.assert_position_in_board((x, y))?;
        let unit = self.get_unit_mut(unit_id)?;
        helpers::assert_unit_move_within_reach(&unit, (x, y))?;
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
        helpers::assert_unit_move_within_reach(&unit, (x, y))?;
        unit.state = unit::State::Attack(x, y);
        Ok(())
    }

    // todo test
    /// Takes all actions queued on units and executes them.
    pub fn resolve_moves(&mut self) {
        let mut unresolved = self.units_to_be_moved();
        while unresolved.len() > 0 {
            unresolved = self.make_move(unresolved);
        }
        self.resolve_blockades()
    }

    /// Returns queue of ids of the units that require moving actions.
    fn units_to_be_moved(&self) -> BinaryHeap<unit::MovingWrapper> {
        self.units
            .iter()
            .filter(|&unit| match unit.state {
                unit::State::Moving(..) | unit::State::Attack(..) => true,
                _ => false,
            })
            .map(|unit| unit::MovingWrapper::new(unit.id))
            .collect()
    }

    // todo test
    /// Makes a single move for each unit in units argument.
    /// Returns filtered queue. With units that still need to be moved.
    fn make_move(
        &mut self,
        mut units: BinaryHeap<unit::MovingWrapper>,
    ) -> BinaryHeap<unit::MovingWrapper> {
        let mut filtered = BinaryHeap::new();
        for u in units.drain() {
            if let Some(val) = self.resolve_unit(&u) {
                filtered.push(val);
            }
        }
        filtered
    }

    /// Moves unit to it's next position and returns its updated MovingWrapper.
    /// If the move was completed or the unit entered a blockade resolve_unit
    /// changes units state approprietly.
    fn resolve_unit(&mut self, wrapper: &unit::MovingWrapper) -> Option<unit::MovingWrapper> {
        // Because of rusts weird pattern matching it has to be done that way
        let (state, pos) = helpers::get_unis_moving_info(self.get_unit(wrapper.unit_id).unwrap());

        match state {
            unit::State::Moving(x, y) => {
                let next_pos = helpers::get_next_field_in_path(pos, (x, y));
                if self.field_empty(next_pos) {
                    let u = self.get_unit_mut(wrapper.unit_id).unwrap();
                    u.position = next_pos;
                    if u.position == (x, y) {
                        u.state = unit::State::Idle;
                        return None;
                    }
                    // todo if enemy unit in vision change state to idle
                    return Some(unit::MovingWrapper {
                        moves_made: wrapper.moves_made + 1,
                        unit_id: wrapper.unit_id,
                    });
                } else {
                    // todo <- resolve it somehow (?)
                    // but how do we resolve situations as
                    //     a <- b
                    //     v    ^
                    //     c -> d
                    // where we cant move any unit at all
                    let u = self.get_unit_mut(wrapper.unit_id).unwrap();
                    // for now let's just stop moving.
                    // maybe later make max number of repetitions to resolve turn ?
                    u.state = unit::State::Idle;
                }
            }
            unit::State::Attack(_x, _y) => {
                // todo implement
            }
            _ => {}
        };
        None
    }

    /// Checks whether board field is not occupied by
    /// any unit.
    fn field_empty(&self, (x, y): Coords) -> bool {
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
        // todo implement
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

    #[allow(dead_code)]
    macro_rules! assert_match_debug {
        ($e:expr, $( $p:pat )+) => {
            assert!(match $e {
                $(
                    $p => true,
                ),*
                actual @ _ => {
                    eprintln!("actual is {:?}", actual);
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
        // todo test it some more <- result changed after implementation change
        assert_match!(units, Err(GameError::NonExistingUnit(6...8)));
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
        assert_match!(
            g.battle_units(0, 1),
            Err(GameError::MoveOutsideUnitsReach(49, 49))
        );
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

    #[test]
    fn empty_heap_when_no_units_should_move() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 2), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 3), unit::Category::Knight).unwrap();
        assert!(g.units_to_be_moved().len() == 0);
    }

    #[test]
    fn moving_units_are_considered_as_the_one_to_move() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 2), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 3), unit::Category::Knight).unwrap();
        g.move_unit(0, (4, 4)).unwrap();
        g.move_unit(2, (5, 5)).unwrap();
        let mut res = g.units_to_be_moved();
        assert!(res.len() == 2);
        let u = res.pop().unwrap();
        assert!(u.unit_id == 0 || u.unit_id == 2);
        let u = res.pop().unwrap();
        assert!(u.unit_id == 0 || u.unit_id == 2);
    }

    #[test]
    fn attacking_units_are_considered_as_the_one_to_move() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 2), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 3), unit::Category::Knight).unwrap();
        g.attack_position(1, (5, 5)).unwrap();
        g.attack_position(2, (3, 3)).unwrap();
        let mut res = g.units_to_be_moved();
        assert!(res.len() == 2);
        let u = res.pop().unwrap();
        assert!(u.unit_id == 1 || u.unit_id == 2);
        let u = res.pop().unwrap();
        assert!(u.unit_id == 1 || u.unit_id == 2);
    }

    #[test]
    fn field_empty_returns_true_for_empty_field() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 2), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 3), unit::Category::Knight).unwrap();
        assert!(g.field_empty((2, 2)));
    }

    #[test]
    fn field_empty_returns_false_for_occupied_space() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 2), unit::Category::Knight).unwrap();
        g.add_unit(0, (1, 3), unit::Category::Knight).unwrap();
        assert!(!g.field_empty((1, 2)));
    }

    #[test]
    fn resolve_unit_returns_proper_new_moving_wrapper() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (3, 3)).unwrap();
        let mut wrap = unit::MovingWrapper::new(0);
        wrap = g.resolve_unit(&wrap).unwrap();
        assert!(wrap.moves_made == 1);
        assert!(wrap.unit_id == 0);
    }

    #[test]
    fn resolve_unit_moves_units_the_proper_way_in_straight_line_on_y() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (1, 3)).unwrap();
        let mut wrap = unit::MovingWrapper::new(0);
        wrap = g.resolve_unit(&wrap).unwrap();
        let u = g.get_unit(wrap.unit_id).unwrap();
        assert!(u.position == (1, 2));
    }

    #[test]
    fn resolve_unit_moves_units_the_proper_way_in_straight_line_on_x() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (3, 1)).unwrap();
        let mut wrap = unit::MovingWrapper::new(0);
        wrap = g.resolve_unit(&wrap).unwrap();
        let u = g.get_unit(wrap.unit_id).unwrap();
        assert!(u.position == (2, 1));
    }

    #[test]
    fn resolve_unit_moves_units_the_proper_way_diagonaly() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (3, 3)).unwrap();
        let mut wrap = unit::MovingWrapper::new(0);
        wrap = g.resolve_unit(&wrap).unwrap();
        let u = g.get_unit(wrap.unit_id).unwrap();
        assert!(u.position == (2, 2));
    }

    #[test]
    fn resolve_stops_unit_after_reaching_destination() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (2, 2)).unwrap();
        let wrap = unit::MovingWrapper::new(0);
        g.resolve_unit(&wrap);
        let u = g.get_unit(0).unwrap();
        assert_match!(u.state, unit::State::Idle);
    }

    #[test]
    fn resolve_returns_none_after_reaching_destination() {
        let mut g = Game::new(2, (100, 100));
        g.add_unit(0, (1, 1), unit::Category::Knight).unwrap();
        g.move_unit(0, (2, 2)).unwrap();
        let wrap = unit::MovingWrapper::new(0);
        assert_match!(g.resolve_unit(&wrap), None);
    }
}
