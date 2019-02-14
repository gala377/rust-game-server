

pub struct Player {
    pub id: u8,
    /// Number of units that can be placed on board
    pub available_units: u32,
    /// Ids of units belonging to a player
    pub units: Vec<usize>,
}


impl Player {
    /// Creates a new player with the given id
    pub fn new(id: u8) -> Player {
        Player {
            id,
            // todo make it a game setting
            available_units: 10,
            units: Vec::with_capacity(10),
        }
    }
}