pub mod mouth;

use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_svc::hal::units::Hertz;

pub struct BillyBass<'d> {
    // Head motor control
    head_enable: LedcDriver<'d>,
    head_in3: PinDriver<'d, Gpio25, Output>,
    head_in4: PinDriver<'d, Gpio14, Output>,

    // Mouth motor control
    mouth_enable: LedcDriver<'d>,
    mouth_in1: PinDriver<'d, Gpio27, Output>,
    mouth_in2: PinDriver<'d, Gpio26, Output>,
    current_mouth_direction: MouthDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouthDirection {
    Opening,
    Closing,
}

impl Default for BillyBass<'_> {
    fn default() -> Self {
        BillyBass::new()
    }
}

impl BillyBass<'_> {
    pub fn new() -> Self {
        let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

        let timer_config = esp_idf_svc::hal::ledc::config::TimerConfig::new()
            .frequency(Hertz(25_000)) // 25kHz (Above human hearing range)
            .resolution(Resolution::Bits8);

        let head_timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config).unwrap();

        let mouth_timer = LedcTimerDriver::new(peripherals.ledc.timer1, &timer_config).unwrap();

        let head_enable = LedcDriver::new(
            peripherals.ledc.channel0,
            head_timer,
            peripherals.pins.gpio16,
        )
        .unwrap();

        let mouth_enable = LedcDriver::new(
            peripherals.ledc.channel1,
            mouth_timer,
            peripherals.pins.gpio5,
        )
        .unwrap();

        let head_in3 = PinDriver::output(peripherals.pins.gpio25).unwrap();
        let head_in4 = PinDriver::output(peripherals.pins.gpio14).unwrap();
        let mouth_in1 = PinDriver::output(peripherals.pins.gpio27).unwrap();
        let mouth_in2 = PinDriver::output(peripherals.pins.gpio26).unwrap();

        BillyBass {
            head_enable,
            head_in3,
            head_in4,
            mouth_enable,
            mouth_in1,
            mouth_in2,
            current_mouth_direction: MouthDirection::Opening,
        }
    }
}
