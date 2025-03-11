use std::cmp::Ordering;

use esp_idf_svc::hal::delay::FreeRtos;
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

impl BillyBass<'_> {
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

            let _ = self.head_enable.set_duty(new_duty);
            FreeRtos::delay_ms(50);
        }
    }
}

impl BillyBass<'_> {
    // Set direction and apply the right pin configuration
    fn set_mouth_direction(&mut self, direction: MouthDirection) {
        // Only change pins if direction is actually changing
        if direction != self.current_mouth_direction {
            // First make sure motor is stopped
            if self.mouth_enable.get_duty() > 0 {
                self.mouth_enable.set_duty(0).unwrap();
                FreeRtos::delay_ms(20); // Brief delay after stopping
            }

            // Now set pins for the new direction
            match direction {
                MouthDirection::Opening => {
                    self.mouth_in1.set_low().unwrap();
                    self.mouth_in2.set_high().unwrap();
                }
                MouthDirection::Closing => {
                    self.mouth_in1.set_high().unwrap();
                    self.mouth_in2.set_low().unwrap();
                }
            }

            // Update the current direction
            self.current_mouth_direction = direction;
        }
    }

    // Simplified mouth control function
    pub fn mouth_set(&mut self, direction: MouthDirection, speed: u8) {
        // Set direction first (includes safety checks)
        self.set_mouth_direction(direction);

        // Now ramp up to the desired speed
        let target = speed as u32;
        let current = self.mouth_enable.get_duty();

        match self.mouth_enable.get_duty().cmp(&target) {
            Ordering::Less => {
                for duty in (current..=target).step_by(8) {
                    self.mouth_enable.set_duty(duty).unwrap();
                    FreeRtos::delay_ms(3);
                }
            }
            Ordering::Greater => {
                for duty in (target..current).rev().step_by(8) {
                    self.mouth_enable.set_duty(duty).unwrap();
                    FreeRtos::delay_ms(2);
                }
                self.mouth_enable.set_duty(target).unwrap(); // Ensure final target is set
            }
            Ordering::Equal => {
                // No need to do anything if already at target
            }
        }
    }

    // Convenience methods
    pub fn mouth_open(&mut self, speed: u8) {
        self.mouth_set(MouthDirection::Opening, speed);
    }

    pub fn mouth_close(&mut self, speed: u8) {
        self.mouth_set(MouthDirection::Closing, speed);
    }

    pub fn mouth_stop(&mut self) {
        // Ramp down to zero (using current direction)
        let current = self.mouth_enable.get_duty();
        for duty in (0..current).rev().step_by(8) {
            self.mouth_enable.set_duty(duty).unwrap();
            FreeRtos::delay_ms(2);
        }

        // Final set to zero and neutral control pins
        self.mouth_enable.set_duty(0).unwrap();
        FreeRtos::delay_ms(5); // Brief delay

        // Set pins to neutral state (both low)
        self.mouth_in1.set_low().unwrap();
        self.mouth_in2.set_low().unwrap();
    }

    pub fn speak_syllable(
        &mut self,
        open_speed: u8,
        close_speed: u8,
        open_time: u32,
        close_time: u32,
    ) {
        self.mouth_open(open_speed);
        FreeRtos::delay_ms(open_time);

        self.mouth_close(close_speed);
        FreeRtos::delay_ms(close_time);

        self.mouth_stop();
    }
}
