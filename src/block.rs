use embedded_graphics::{prelude::*, primitives::Rectangle};

pub const BLOCK_SIZE: Size = Size::new(20, 3);

pub struct Block {
    pub rect: Rectangle,
    pub lives: i32,
}

impl Block {
    pub fn new(point: Point) -> Self {
        Self {
            rect: Rectangle::new(point, BLOCK_SIZE),
            lives: 2,
        }
    }
}
