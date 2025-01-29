use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text},
};
use esp_hal::{i2c::master::I2c, rng::Rng};
use heapless::{String, Vec};
use ssd1306::{
    mode::BufferedGraphicsModeAsync, prelude::I2CInterface, size::DisplaySize128x64, Ssd1306Async,
};

use crate::{
    ball::Ball,
    block::{Block, BLOCK_SIZE},
    player::{Player, PlayerDirection, PLAYER_DIRECTION, PLAYER_SIZE, PLAYER_VELOCITY},
};

const MAX_BALLS: usize = 5;
const MAX_BLOCKS: usize = 50;
const PLAYER_LIVES: u8 = 3;
pub type DisplayType<'a> = Ssd1306Async<
    I2CInterface<I2c<'a, esp_hal::Async>>,
    DisplaySize128x64,
    BufferedGraphicsModeAsync<DisplaySize128x64>,
>;

pub static RESET_GAME: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    LevelCompleted,
    Dead,
}

pub struct Game<'a> {
    state: GameState,
    score: u32,
    player_lives: u8,
    player: Player,
    blocks: Vec<Block, MAX_BLOCKS>,
    balls: Vec<Ball, MAX_BALLS>,
    display: DisplayType<'a>,
    rng: Rng,
}

impl<'a> Game<'a> {
    pub fn new(display: DisplayType<'a>, mut rng: Rng) -> Self {
        let screen_dims = display.dimensions();
        Self {
            state: GameState::Menu,
            score: 0,
            player_lives: PLAYER_LIVES,
            player: Game::spawn_player(&display),
            blocks: Game::init_blocks(),
            balls: Game::init_balls(&mut rng, screen_dims.0 as i32, screen_dims.1 as i32),
            display,
            rng,
        }
    }

    pub fn reset_game(&mut self) {
        self.player_lives = PLAYER_LIVES;
        self.score = 0;
        self.player = Game::spawn_player(&self.display);

        let screen_dims = self.display.dimensions();

        self.balls = Game::init_balls(&mut self.rng, screen_dims.0 as i32, screen_dims.1 as i32);
        self.blocks = Game::init_blocks();
    }

    pub fn init_balls(
        rng: &mut Rng,
        screen_width: i32,
        screen_height: i32,
    ) -> Vec<Ball, MAX_BALLS> {
        let ball_pos = Point::new(screen_width / 2, screen_height / 2);
        let mut balls = Vec::new();
        balls
            .push(Ball::new(ball_pos, rng, screen_width))
            .map_err(|_| "Failed to add Ball")
            .unwrap();
        balls
    }

    fn init_blocks() -> Vec<Block, MAX_BLOCKS> {
        let mut blocks = Vec::new();
        let (cols, rows) = (6, 5);
        let padding = 1;
        let total_block_size = BLOCK_SIZE + Size::new(padding, padding);
        let board_start_pos = Point::new(1, 1);
        for i in 0..cols * rows {
            let block_x = (i % cols) * total_block_size.width as i32;
            let block_y = (i / cols) * total_block_size.height as i32;
            blocks
                .push(Block::new(board_start_pos + Point::new(block_x, block_y)))
                .map_err(|_| "block push failed")
                .unwrap();
        }

        blocks
    }

    pub fn spawn_player(display: &DisplayType) -> Player {
        let screen_dims = display.dimensions();
        Player::new(
            // To center: Half of the screen - half the player size(mid point of player)
            screen_dims.0 as i32 / 2 - PLAYER_SIZE.width as i32 / 2,
            // Position player just above the bottom:
            screen_dims.1 as i32 - PLAYER_SIZE.height as i32,
        )
    }

    pub async fn clear_display(&mut self) {
        self.display.clear_buffer();
        self.display.clear(BinaryColor::Off).unwrap();
        // self.display.flush().await.unwrap();
    }

    async fn draw_game(&mut self) {
        self.clear_display().await;

        self.draw_player().await;
        self.draw_blocks().await;
        self.draw_balls().await;
        // self.print_score();
        // self.print_lives();
    }

    pub async fn start(&mut self) {
        self.clear_display().await;
        self.display.flush().await.unwrap();
        let mut title_buff: String<64> = String::new();

        loop {
            title_buff.clear();
            match self.state {
                GameState::Menu => {
                    if RESET_GAME.swap(false, Ordering::Relaxed) {
                        self.reset_game();
                        self.state = GameState::Playing;
                    }
                }
                GameState::Playing => {
                    self.move_player().await;
                    self.move_balls().await;

                    self.collison_handle();
                    self.remove_balls();

                    self.blocks.retain(|block| block.lives > 0);
                    if self.blocks.is_empty() {
                        self.state = GameState::LevelCompleted;
                    }
                }
                _ => {
                    if RESET_GAME.swap(false, Ordering::Relaxed) {
                        self.state = GameState::Menu;
                    }
                }
            }

            self.clear_display().await;

            match self.state {
                GameState::Menu => self.draw_title_text("Press to start..."),
                GameState::Playing => self.draw_game().await,
                GameState::LevelCompleted => {
                    write!(title_buff, "You win {} score", self.score).unwrap();
                    self.draw_title_text(&title_buff);
                }
                GameState::Dead => {
                    write!(title_buff, "You died {} score", self.score).unwrap();
                    self.draw_title_text(&title_buff);
                }
            }

            self.display.flush().await.unwrap();

            Timer::after(Duration::from_millis(10)).await;
        }
    }

    fn draw_title_text(&mut self, title: &str) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        Text::with_baseline(
            title,
            Point::new(
                // self.display.dimensions().0 as i32 / 2,
                5,
                self.display.dimensions().1 as i32 / 2,
            ),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .unwrap();
    }

    fn remove_balls(&mut self) {
        let balls_len = self.balls.len();
        self.balls
            .retain(|ball| ball.rect.top_left.y < self.display.dimensions().1 as i32);
        let remove_balls = balls_len - self.balls.len();
        if remove_balls > 0 && self.balls.is_empty() {
            self.player_lives = self.player_lives.saturating_sub(1);
            // To position the newly spawned ball in the center of Player
            // let player_center = self.player.rect.w * 0.5 + BALL_SIZE * 0.5;
            // self.balls.push(Ball::new(
            //     self.player.rect.point() + vec2(player_center, -50.),
            // ));
            let screen_dims = self.display.dimensions();
            let screen_width = screen_dims.0 as i32;
            let ball_pos = Point::new(screen_width / 2, screen_dims.1 as i32 / 2);

            self.balls
                .push(Ball::new(ball_pos, &mut self.rng, screen_width))
                .map_err(|_| "Failed to add Ball")
                .unwrap();
            if self.player_lives == 0 {
                self.state = GameState::Dead;
            }
        }
    }
    async fn move_player(&mut self) {
        let direction = PLAYER_DIRECTION.load(Ordering::Relaxed);

        match direction {
            PlayerDirection::Idle => {}
            PlayerDirection::Left => {
                // println!("Moving Left");
                let new_x = (self.player.rect.top_left.x - PLAYER_VELOCITY).max(0);

                self.player.rect =
                    Rectangle::new(Point::new(new_x, self.player.rect.top_left.y), PLAYER_SIZE);
            }
            PlayerDirection::Right => {
                // println!("Moving Right");
                let right_edge = self.display.dimensions().0 as i32 - PLAYER_SIZE.width as i32;
                let new_x = (self.player.rect.top_left.x + PLAYER_VELOCITY).min(right_edge);

                self.player.rect =
                    Rectangle::new(Point::new(new_x, self.player.rect.top_left.y), PLAYER_SIZE);
            }
        };
    }

    async fn move_balls(&mut self) {
        for ball in self.balls.iter_mut() {
            ball.update();
        }
    }

    fn collison_handle(&mut self) {
        // let mut spawn_later = vec![];
        for ball in self.balls.iter_mut() {
            resolve_collison(&mut ball.rect, &mut ball.vel, &self.player.rect);

            for block in self.blocks.iter_mut() {
                if resolve_collison(&mut ball.rect, &mut ball.vel, &block.rect) {
                    block.lives -= 1;
                    if block.lives <= 0 {
                        self.score += 10;
                        // if block.block_type == Blocktype::SpawnBallOnDeath {
                        //     spawn_later.push(Ball::new(ball.rect.point()));
                        // }
                    }
                }
            }
        }

        // for ball in spawn_later.into_iter() {
        //     self.balls.push(ball);
        // }
    }

    async fn draw_player(&mut self) {
        let style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::On)
            .build();

        self.player
            .rect
            .into_styled(style)
            .draw(&mut self.display)
            .unwrap();
    }

    async fn draw_blocks(&mut self) {
        for block in self.blocks.iter() {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();

            block
                .rect
                .into_styled(style)
                .draw(&mut self.display)
                .unwrap();
        }
    }

    async fn draw_balls(&mut self) {
        for ball in self.balls.iter() {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();

            ball.rect
                .into_styled(style)
                .draw(&mut self.display)
                .unwrap();
        }
    }
}

pub fn resolve_collison(a: &mut Rectangle, vel: &mut Point, b: &Rectangle) -> bool {
    let intersection = a.intersection(b);

    if intersection.size.width == 0 || intersection.size.height == 0 {
        return false;
    }

    let a_center = a.center();
    let b_center = b.center();
    let to = b_center - a_center;
    let to_signum = Point::new(to.x.signum(), to.y.signum());

    if intersection.size.width > intersection.size.height {
        a.top_left.y -= to_signum.y * intersection.size.height as i32;

        vel.y = if to_signum.y > 0 {
            -vel.y.abs()
        } else {
            vel.y.abs()
        };
    } else {
        a.top_left.x -= to_signum.x * intersection.size.width as i32;
        vel.x = if to_signum.x > 0 {
            vel.x.abs()
        } else {
            -vel.x.abs()
        };
    }

    true
}
