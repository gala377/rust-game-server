// Deserialization crates for config module
#[macro_use]
extern crate serde_derive;
extern crate serde;

// #[macro_use]
// extern crate log;

extern crate byteorder;

// GameServer configuration
pub mod config;

// Game object
pub mod game;

// Net communcation
pub mod comm;

// private module with helper functions
mod helpers;
