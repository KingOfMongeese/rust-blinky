#![no_std]

#[derive(Clone, Copy)]
pub enum LedDirection {
    Up,
    Down,
}

pub fn toggle_direction(current: LedDirection) -> LedDirection {
    match current {
        LedDirection::Down => LedDirection::Up,
        LedDirection::Up => LedDirection::Down,
    }
}
