
pub struct Game {
    num_of_players: u8,
    num_of_units: usize,
    board_size: (usize, usize),
    units: Vec<Unit>,
}

// todo this is not working as intended
pub struct Unit {
    id: u8, 
    owner_id: u8,
    position: (usize, usize), 
    category: UnitType,
    stats: UnitStats,
    state: UnitState<self>, 
}

pub enum UnitType {
    Cavalry,
    Knight, 
    Pickerman,
}

struct UnitStats {
    movement_range: usize,
    attack_range: usize, 
    vision_range: usize, 
}

pub enum UnitState<'a> {
    Idle,
    Moving(usize, usize),
    Blocked,
    Attack(&'a Unit),
}

impl Game {

    fn new(num_of_players: u8, board_size: (usize, usize)) -> Game {
        assert!(board_size.0 > 0 && board_size.1 > 0);
        assert!(num_of_players > 1);
        Game{
            num_of_players,
            num_of_units: 0, 
            board_size,
            units: Vec::new(),    
        }
    }

    fn add_unit(&mut self,
                owner_id: u8,
                position: (usize, usize),
                category: UnitType) {
        
    }
}