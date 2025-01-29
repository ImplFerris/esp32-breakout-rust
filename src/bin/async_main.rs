#![no_std]
#![no_main]

use core::sync::atomic::Ordering;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp32_breakout_rust::{
    game::{self, Game},
    joystick,
};
use esp_backtrace as _;
use esp_hal::{
    gpio::{GpioPin, Input, Pull},
    prelude::*,
    rng::Rng,
};
use log::info;
use ssd1306::{
    mode::DisplayConfigAsync, prelude::DisplayRotation, size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306Async,
};

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    spawner
        .spawn(joystick::track_joystick(
            peripherals.GPIO13,
            peripherals.GPIO14,
            peripherals.ADC2,
        ))
        .unwrap();

    spawner.spawn(reset_btn(peripherals.GPIO32)).unwrap();

    let i2c = esp_hal::i2c::master::I2c::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config {
            frequency: 400.kHz(),
            timeout: Some(100),
        },
    )
    .with_scl(peripherals.GPIO18)
    .with_sda(peripherals.GPIO23)
    .into_async();

    let interface = I2CDisplayInterface::new(i2c);
    // initialize the display
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().await.unwrap();

    let rng = Rng::new(peripherals.RNG);

    let mut game = Game::new(display, rng);
    game.start().await;
}

// To Spawn balls when button pressed
#[embassy_executor::task]
pub async fn reset_btn(btn: GpioPin<32>) {
    let input_btn = Input::new(btn, Pull::Up);

    loop {
        if input_btn.is_low() {
            game::RESET_GAME.swap(true, Ordering::Relaxed);
            Timer::after(Duration::from_millis(100)).await;
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}
