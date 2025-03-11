pub mod billy_bass_tests;
pub mod mouth;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver, Resolution, LEDC};
use esp_idf_svc::hal::units::Hertz;

pub struct BillyBass {
    // Head motor control
    head_enable: LedcDriver<'static>,
    head_in3: PinDriver<'static, Gpio25, Output>,
    head_in4: PinDriver<'static, Gpio14, Output>,
    // Mouth motor control
    mouth_enable: LedcDriver<'static>,
    mouth_in1: PinDriver<'static, Gpio27, Output>,
    mouth_in2: PinDriver<'static, Gpio26, Output>,
    current_mouth_direction: MouthDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouthDirection {
    Opening,
    Closing,
}

impl BillyBass {
    pub fn new(pins: Pins, ledc: LEDC) -> Self {
        let timer_config = esp_idf_svc::hal::ledc::config::TimerConfig::new()
            .frequency(Hertz(25_000)) // 25kHz (Above human hearing range)
            .resolution(Resolution::Bits8);

        let head_timer = LedcTimerDriver::new(ledc.timer0, &timer_config).unwrap();
        let mouth_timer = LedcTimerDriver::new(ledc.timer1, &timer_config).unwrap();

        let head_enable = LedcDriver::new(ledc.channel0, head_timer, pins.gpio16).unwrap();

        let mouth_enable = LedcDriver::new(ledc.channel1, mouth_timer, pins.gpio5).unwrap();

        let head_in3 = PinDriver::output(pins.gpio25).unwrap();
        let head_in4 = PinDriver::output(pins.gpio14).unwrap();
        let mouth_in1 = PinDriver::output(pins.gpio27).unwrap();
        let mouth_in2 = PinDriver::output(pins.gpio26).unwrap();

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

// Fix the typo in head_slowed_stop
impl BillyBass {
    pub fn head_hold_soft(&mut self) {
        self.head_in3.set_high().unwrap();
        self.head_in4.set_low().unwrap();

        // Ramp up duty cycle
        for duty in (0..256).step_by(16) {
            self.head_enable.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5); // 5ms delay, total ramp time ~80ms
        }
    }

    pub fn head_slowed_stop(&mut self) {
        while self.head_enable.get_duty() > 0 {
            let current_duty = self.head_enable.get_duty();
            // Decrease by at least 1 to avoid infinite loop due to rounding errors
            let new_duty = (current_duty as f32 * 0.95).max(0.0) as u32;
            if new_duty == current_duty {
                break; // Prevent infinite loop
            }
            let _ = self.head_enable.set_duty(new_duty); // Fix the typo here
            FreeRtos::delay_ms(50);
        }
    }
}
