/// Defines Error types for the game struct to use.
use std::error::Error;
use std::fmt;

// todo refactor to structs and From<> trait

/// Returned if there was an error
/// violating Games logic.
#[derive(Debug)]
pub enum GameError {
    /// Provided id doesn't correspond with
    /// any unit in play.
    NonExistingUnit(usize),
    /// Provided coordinates are outside the
    /// board boundaries.
    PositionOutsideTheBoard(usize, usize),
    /// Requested move cannot be done due
    /// to units stats.
    MoveOutsideUnitsReach(usize, usize),
    /// Player cannot place more units
    AllUnitSlotsUsed,
}

impl Error for GameError {}

impl fmt::Display for GameError {
    // todo does this even work?
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}
