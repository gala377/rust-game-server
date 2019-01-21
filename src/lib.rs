// Deserialization crates for config module
extern crate serde;
extern crate serde_derive;

extern crate fast_from_derive;

// GameServer configuration
pub mod config;

// Game object
pub mod game;

// Net communcation
pub mod comm;

// private module with helper functions
mod helpers;
