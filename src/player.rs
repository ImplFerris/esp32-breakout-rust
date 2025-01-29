use atomic_enum::atomic_enum;
use embedded_graphics::{
    prelude::{Point, Size},
    primitives::Rectangle,
};

#[atomic_enum]
#[derive(PartialEq)]
pub enum PlayerDirection {
    Left,
    Right,
    Idle,
}

pub const PLAYER_SIZE: Size = Size::new(40, 5);
pub const PLAYER_VELOCITY: i32 = 5;
pub static PLAYER_DIRECTION: AtomicPlayerDirection =
    AtomicPlayerDirection::new(PlayerDirection::Idle);

pub struct Player {
    pub rect: Rectangle,
    pub direction: PlayerDirection,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            rect: Rectangle::new(Point::new(x, y), PLAYER_SIZE),
            direction: PlayerDirection::Idle,
        }
    }
}
