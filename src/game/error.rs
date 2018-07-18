use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GameError {
    NonExistingUnit,
}

impl Error for GameError {
    fn description(&self) -> &str {
        match self {
            GameError::NonExistingUnit => "Unit doesn't extist!",
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::NonExistingUnit => write!(f, "{:?}", &self),
        }
    }
}
