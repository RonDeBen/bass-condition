pub mod billy;

use esp_idf_svc::hal::delay::FreeRtos;

use crate::billy::BillyBass;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut billy = BillyBass::new();

    log::info!("Billy Bass controller initialized.");

    // Test loop
    loop {
        // Hold head position
        log::info!("Setting head position...");
        billy.head_hold_soft();

        // Let head stabilize before testing mouth
        FreeRtos::delay_ms(500);

        // Test mouth open/close cycles
        // log::info!("Testing mouth - basic open/close cycles");
        // billy.mouth_cycle(6, 255, 500);
        billy.test_active_closing();
        // Pause between tests
        FreeRtos::delay_ms(1000);

        // // Pause between test iterations
        log::info!("Test complete, pausing...");
        billy.head_slowed_stop();
        FreeRtos::delay_ms(1000);
    }
}
