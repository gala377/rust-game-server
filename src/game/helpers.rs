use super::error::GameError;
/// definies helper functions for the game module
use std::cmp::Ordering;

use super::unit;
use super::unit::Unit;

pub type Coords = (usize, usize);

/// Returns placeholder unit stats.
/// Possible extension in future release.
pub fn default_unit_stats() -> unit::Stats {
    unit::Stats {
        movement_range: 10,
        vision_range: 10,
        attack_range: 10,
    }
}

/// Checks if requested move doesn't violate unit's stats.
/// todo the same for the Attack state.
pub fn assert_unit_move_within_reach(u: &Unit, (x, y): Coords) -> Result<(), GameError> {
    let pos = &u.position;
    let x_diff = (pos.0 as i32 - x as i32).abs() as usize;
    let y_diff = (pos.1 as i32 - y as i32).abs() as usize;
    if x_diff + y_diff > u.stats.movement_range {
        return Err(GameError::MoveOutsideUnitsReach(x, y));
    }
    Ok(())
}

/// Given current position and the destination
/// returns a tuple denoting next position in path.
pub fn get_next_field_in_path((curr_x, curr_y): Coords, (dest_x, dest_y): Coords) -> Coords {
    let get_next = |curr: usize, dest: usize| match curr.cmp(&dest) {
        Ordering::Less => curr + 1,
        Ordering::Equal => curr,
        Ordering::Greater => curr - 1,
    };
    (get_next(curr_x, dest_x), get_next(curr_y, dest_y))
}

/// Returns copy of units relevant information.
pub fn get_unis_moving_info(unit: &Unit) -> (unit::State, Coords) {
    (unit.state, unit.position)
}
