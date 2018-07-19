use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GameError {
    NonExistingUnit,
    PositionOutsideTheBoard(usize, usize),
}

impl Error for GameError {
    fn description(&self) -> &str {
        match self {
            GameError::NonExistingUnit => "Unit doesn't extist!",
            GameError::PositionOutsideTheBoard(x, y) => format!(
                "Position ({}, {}) is outside the board boundaries", x, y).as_str(),
        }
    }
}

impl fmt::Display for GameError {
    // todo does this even work? 
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}
