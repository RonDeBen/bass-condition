use esp_idf_svc::hal::delay::FreeRtos;

use crate::billy::BillyBass;

impl BillyBass {
    pub fn test_mouth_articulation(&mut self) {
        log::info!("========= Starting Mouth Articulation Tests =========");

        // Test 1: Find fastest reliable cycle time
        log::info!("===== Test 1: Determining fastest cycle time =====");
        for delay_ms in [100, 150, 200, 250, 300].iter() {
            log::info!(
                "Testing cycle with {}ms delay ({}ms per full open-close)",
                delay_ms,
                delay_ms * 2
            );
            for cycle in 1..=5 {
                log::info!("  Cycle {}/5: Opening mouth", cycle);
                self.mouth_open(180);
                FreeRtos::delay_ms(*delay_ms);

                log::info!("  Cycle {}/5: Closing mouth", cycle);
                self.mouth_stop();
                FreeRtos::delay_ms(*delay_ms);
            }
            log::info!("Completed test with {}ms delay", delay_ms);
            FreeRtos::delay_ms(500); // Pause between tests
        }

        // Test 2: Gradual opening and closing
        log::info!("===== Test 3: Testing smooth transitions =====");
        log::info!("Starting smooth opening sequence (0-255 in steps of 5)");
        // Smooth open
        for duty in (0..=255).step_by(5) {
            if duty % 50 == 0 {
                log::info!("  Opening: Duty cycle at {}", duty);
            }
            self.mouth_open(duty as u8);
            FreeRtos::delay_ms(10);
        }

        log::info!("Holding mouth fully open for 200ms");
        FreeRtos::delay_ms(200);

        log::info!("Starting smooth closing sequence (255-0 in steps of 5)");
        // Smooth close
        for duty in (0..=255).rev().step_by(5) {
            if duty % 50 == 0 {
                log::info!("  Closing: Duty cycle at {}", duty);
            }
            self.mouth_open(duty as u8);
            FreeRtos::delay_ms(10);
        }

        log::info!("========= Articulation tests complete =========");
    }

    pub fn test_active_closing(&mut self) {
        log::info!("===== Testing Active Mouth Closing =====");

        // Test different open-close speeds
        let speeds = [100, 150, 200, 255];
        let opening_times = [200, 300, 400];
        let closing_times = [50, 100, 150, 200];

        for &opening_time in &opening_times {
            for &closing_time in &closing_times {
                log::info!(
                    "Testing: Open for {}ms, Close for {}ms",
                    opening_time,
                    closing_time
                );

                for &speed in &speeds {
                    log::info!("  Using speed: {}", speed);
                    for cycle in 1..=3 {
                        log::info!("    Cycle {}/3", cycle);

                        // Open mouth
                        log::info!("      Opening mouth");
                        self.mouth_open(speed);
                        FreeRtos::delay_ms(opening_time);

                        // Actively close mouth
                        log::info!("      Actively closing mouth");
                        self.mouth_close(speed);
                        FreeRtos::delay_ms(closing_time);

                        // Stop motor briefly
                        self.mouth_stop();
                        FreeRtos::delay_ms(100);
                    }
                    FreeRtos::delay_ms(500); // Pause between speed tests
                }
                log::info!(
                    "Completed test for {}ms open, {}ms close",
                    opening_time,
                    closing_time
                );
                FreeRtos::delay_ms(1000); // Longer pause between timing tests
            }
        }

        // Test speech-like pattern with active closing
        log::info!("===== Testing Speech Pattern with Active Closing =====");

        // Each entry is (open_speed, close_speed, open_time, close_time)
        let speech_pattern = [
            (180, 200, 100, 50),  // Quick syllable
            (200, 255, 250, 100), // Emphasized syllable
            (150, 180, 80, 40),   // Short syllable
            (170, 200, 90, 60),   // Medium syllable
            (220, 255, 300, 150), // Final emphasized syllable
        ];

        for (i, &(open_speed, close_speed, open_time, close_time)) in
            speech_pattern.iter().enumerate()
        {
            log::info!("Speech syllable {}/{}:", i + 1, speech_pattern.len());
            log::info!(
                "  Opening mouth to speed {} for {}ms",
                open_speed,
                open_time
            );
            self.mouth_open(open_speed);
            FreeRtos::delay_ms(open_time);

            log::info!(
                "  Actively closing mouth at speed {} for {}ms",
                close_speed,
                close_time
            );
            self.mouth_close(close_speed);
            FreeRtos::delay_ms(close_time);

            // Stop and brief pause
            self.mouth_stop();
            FreeRtos::delay_ms(100);
        }

        log::info!("===== Active Closing Tests Complete =====");
    }
}
