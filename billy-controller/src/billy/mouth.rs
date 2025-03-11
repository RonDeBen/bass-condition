use super::{BillyBass, MouthDirection};

use esp_idf_svc::hal::delay::FreeRtos;
use std::cmp::Ordering;

impl BillyBass<'_> {
    /// Set direction and apply the right pin configuration
    fn set_mouth_direction(&mut self, direction: MouthDirection) {
        // Only change pins if direction is actually changing
        if direction != self.current_mouth_direction {
            // First make sure motor is stopped
            if self.mouth_enable.get_duty() > 0 {
                self.mouth_enable.set_duty(0).unwrap();
                FreeRtos::delay_ms(20);
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

            self.current_mouth_direction = direction;
        }
    }

    pub fn mouth_set(&mut self, direction: MouthDirection, speed: u8) {
        self.set_mouth_direction(direction);

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
        FreeRtos::delay_ms(5);

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
