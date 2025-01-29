use embedded_graphics::{
    prelude::{Point, Size, Transform},
    primitives::Rectangle,
};
use esp_hal::rng::Rng;

pub const BALL_SIZE: Size = Size::new(4, 4);
pub const BALL_SPEED: i32 = 2;

pub struct Ball {
    pub rect: Rectangle,
    pub vel: Point,
    screen_width: i32,
}

impl Ball {
    pub fn new(top_left: Point, rng: &mut Rng, screen_width: i32) -> Self {
        let mut get_random = || ((rng.random() as i32 % 21) - 10).clamp(-1, 1);

        let rand_x = get_random();
        // let rand_y = get_random();
        Self {
            rect: Rectangle::new(top_left, BALL_SIZE),
            vel: Point::new(rand_x, 1),
            screen_width,
        }
    }

    pub fn update(&mut self) {
        self.rect = self
            .rect
            .translate(Point::new(self.vel.x * BALL_SPEED, self.vel.y * BALL_SPEED));

        if self.rect.top_left.x < 0 {
            self.vel.x = 1;
        }
        if self.rect.top_left.x > self.screen_width - self.rect.size.width as i32 {
            self.vel.x = -1;
        }

        if self.rect.top_left.y < 0 {
            self.vel.y = 1;
        }
    }
}
