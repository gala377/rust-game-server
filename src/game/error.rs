use std::error::Error;
use std::fmt;

// todo refactor to structs and From<> trait

#[derive(Debug)]
pub enum GameError {
    NonExistingUnit(usize),
    PositionOutsideTheBoard(usize, usize),
    MoveOutsideUnitsReach(usize, usize),
}

impl Error for GameError {
    fn description(&self) -> &str {
        match self {
            GameError::NonExistingUnit(_) => "Unit doesn't extist!",
            GameError::PositionOutsideTheBoard(_, _) => "Position is outside the board boundaries",
            GameError::MoveOutsideUnitsReach(_, _) => "Positions not within units reach"
        }
    }
}

impl fmt::Display for GameError {
    // todo does this even work? 
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}
