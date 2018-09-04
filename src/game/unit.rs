/// Defines Unit datatype and any related structs.

use std::cmp::Ordering;

/// Unit represents single soldier entity
/// inside the game.
pub struct Unit {
    /// Units identifier.
    pub id: usize,
    /// Identifier of Units owner.
    pub owner_id: u8,
    /// Position at which the Unit is currently.
    pub position: (usize, usize),
    /// Unit's category - see Category.
    pub category: Category,
    /// Unit's stats - see Stats.
    pub stats: Stats,
    /// Unit's curretn state - see State.
    pub state: State,
}

/// Wraps Unit during it's moving process
/// and counts number of moves already made.
pub struct MovingWrapper {
    pub moves_made: usize,
    pub unit_id: usize,
}

impl MovingWrapper {
    pub fn new(unit_id: usize) -> MovingWrapper {
        MovingWrapper {
            moves_made: 0,
            unit_id,
        }
    }
}

impl Ord for MovingWrapper {
    fn cmp(&self, other: &MovingWrapper) -> Ordering {
        self.moves_made.cmp(&other.moves_made)
    }
}

impl PartialOrd for MovingWrapper {
    fn partial_cmp(&self, other: &MovingWrapper) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MovingWrapper {
    fn eq(&self, other: &MovingWrapper) -> bool {
        self.moves_made == other.moves_made
    }
}

impl Eq for MovingWrapper {}


/// Category of the Unit.
/// As in Rock-Scissor-Paper each category
/// has one other as it's weakness and
/// another one as it's advantage.
#[derive(Copy, Clone)]
pub enum Category {
    /// Beats Knight, loses to Pickerman.
    Cavalry,
    /// Beats Pickerman, loses to Cavalry.
    Knight,
    /// Beats Cavalry, loses to Knight.
    Pickerman,
}

/// Unit statistics determinig it's ability to move.
#[derive(Copy, Clone)]
pub struct Stats {
    /// Number of tiles Unit can be moved during one turn while in Moving state.
    pub movement_range: usize,
    /// Number of tiles Unit can be moved during one turn while in Attack state.
    pub attack_range: usize,
    /// Number of tiles determining distance at which the Unit will see enemu Units.
    pub vision_range: usize,
}

/// Represents current Unit state.
#[derive(Copy, Clone)]
pub enum State {
    /// Default or no action to perform.
    Idle,
    /// Unit is moving to the specified location trying to avoid collision with other units.
    Moving(usize, usize),
    /// Unit is stuck in a blokade and will be as long as blockade is in play.
    Blocked,
    /// Same as moving, except collision with enemy Unit will start a battle.
    Attack(usize, usize),
}
