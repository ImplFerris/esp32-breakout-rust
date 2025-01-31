use core::sync::atomic::Ordering;

use atomic_enum::atomic_enum;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};

use crate::game::DisplayType;

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
const PLAYER_LIVES: u8 = 3;

pub struct Player {
    pub rect: Rectangle,
    pub direction: PlayerDirection,
    pub lives: u8,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            rect: Rectangle::new(Point::new(x, y), PLAYER_SIZE),
            direction: PlayerDirection::Idle,
            lives: PLAYER_LIVES,
        }
    }

    pub fn draw(&self, display: &mut DisplayType) {
        let style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::On)
            .build();

        self.rect.into_styled(style).draw(display).unwrap();
    }

    pub fn update(&mut self, display: &mut DisplayType) {
        let direction = PLAYER_DIRECTION.load(Ordering::Relaxed);

        match direction {
            PlayerDirection::Idle => {}
            PlayerDirection::Left => {
                // println!("Moving Left");
                let new_x = (self.rect.top_left.x - PLAYER_VELOCITY).max(0);

                self.rect = Rectangle::new(Point::new(new_x, self.rect.top_left.y), PLAYER_SIZE);
            }
            PlayerDirection::Right => {
                // println!("Moving Right");
                let right_edge = display.dimensions().0 as i32 - PLAYER_SIZE.width as i32;
                let new_x = (self.rect.top_left.x + PLAYER_VELOCITY).min(right_edge);

                self.rect = Rectangle::new(Point::new(new_x, self.rect.top_left.y), PLAYER_SIZE);
            }
        };
    }
}
