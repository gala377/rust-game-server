// todo this is not working as intended
pub struct Unit {
    pub id: usize,
    pub owner_id: u8,
    pub position: (usize, usize),
    pub category: Category,
    pub stats: Stats,
    pub state: State,
}

pub enum Category {
    Cavalry,
    Knight,
    Pickerman,
}

pub struct Stats {
    pub movement_range: usize,
    pub attack_range: usize,
    pub vision_range: usize,
}

/// Represents current Unit state
pub enum State {
    /// Default or no action to perform
    Idle,
    /// Unit is moving to the specified location trying to avoid collidin with other units
    Moving(usize, usize),
    /// Unit is stuck in a blokade and will be as long as blockade is in play
    Blocked,
    /// Same as moving, except collision with enemy Unit will start a battle
    Attack(usize, usize),
}