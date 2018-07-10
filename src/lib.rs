// Deserialization crates for config module
#[macro_use]
extern crate serde_derive;
extern crate serde;

// GameServer configuration
pub mod config;


// Game object
pub mod game;

// private module with helper functions
mod helpers;
