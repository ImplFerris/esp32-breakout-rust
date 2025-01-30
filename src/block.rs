use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};

use crate::game::DisplayType;

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

    pub fn draw(&self, display: &mut DisplayType) {
        let style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::On)
            .build();

        self.rect.into_styled(style).draw(display).unwrap();
    }
}
