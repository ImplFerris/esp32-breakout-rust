use core::sync::atomic::Ordering;

use embassy_time::{Duration, Timer};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    gpio::GpioPin,
    peripherals::ADC2,
    prelude::nb,
};

use crate::player::{PlayerDirection, PLAYER_DIRECTION};

const VRX_PIN: u8 = 13;
const VRY_PIN: u8 = 14;

#[embassy_executor::task]
pub async fn track_joystick(_vrx: GpioPin<VRX_PIN>, vry: GpioPin<VRY_PIN>, adc: ADC2) {
    let mut adc2_config = AdcConfig::new();
    // let mut vrx_pin = adc2_config.enable_pin(vrx, Attenuation::Attenuation11dB);
    let mut vry_pin = adc2_config.enable_pin(vry, Attenuation::Attenuation11dB);

    let mut adc2 = Adc::new(adc, adc2_config);

    loop {
        let Ok(adc_value): Result<u16, _> = nb::block!(adc2.read_oneshot(&mut vry_pin)) else {
            continue;
        };

        if adc_value > 3000 {
            PLAYER_DIRECTION.store(PlayerDirection::Left, Ordering::Relaxed);
        } else if adc_value < 1500 {
            PLAYER_DIRECTION.store(PlayerDirection::Right, Ordering::Relaxed);
        } else {
            PLAYER_DIRECTION.store(PlayerDirection::Idle, Ordering::Relaxed);
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}
