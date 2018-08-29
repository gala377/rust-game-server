use std::error::Error;
use std::fmt;

// todo refactor to structs and From<> trait

#[derive(Debug)]
pub enum GameError {
    NonExistingUnit(usize),
    PositionOutsideTheBoard(usize, usize),
    MoveOutsideUnitsReach(usize, usize),
}

impl Error for GameError {}

impl fmt::Display for GameError {
    // todo does this even work? 
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}
