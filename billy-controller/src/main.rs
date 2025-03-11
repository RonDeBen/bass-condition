pub mod billy;
pub mod websocket_server;

use std::sync::{Arc, Mutex};

use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::delay::FreeRtos, nvs::EspDefaultNvsPartition,
};

use anyhow::{anyhow, Result};

use crate::billy::BillyBass;
use crate::websocket_server::ApiServer;

// WiFi credentials - replace with your actual values or use env vars
const SERVER_PORT: u16 = 8080;

fn main() -> Result<()> {
    // Initialize system
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting Billy Bass Controller");

    // Take peripherals once for the entire application
    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    // Create the BillyBass controller, passing just the parts of peripherals it needs
    let billy = BillyBass::new(peripherals.pins, peripherals.ledc);
    let fish = Arc::new(Mutex::new(billy));

    // Test the fish briefly to ensure hardware is working
    {
        let mut fish = fish.lock().unwrap();
        log::info!("Running quick hardware test...");
        fish.head_hold_soft();
        fish.mouth_open(150);
        FreeRtos::delay_ms(300);
        fish.mouth_close(150);
        FreeRtos::delay_ms(300);
        fish.mouth_stop();
        fish.head_slowed_stop();
    }

    // Set up WiFi
    log::info!("Connecting to WiFi...");
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi_res = esp_idf_svc::wifi::EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs));
    if let Err(e) = wifi_res {
        log::error!("{:?}", e);
    }

    let mut wifi = wifi_res?;
    log::info!("Got past this wifi");

    // Convert WiFi credentials to heapless strings
    let wifi_ssid: &str = &std::env::var("WIFI_SSID").expect("please set ssid");
    let wifi_pass: &str = &std::env::var("WIFI_PASSWORD").expect("please set wifi password");

    // Convert to heapless strings
    let mut ssid_hl: heapless::String<32> = heapless::String::new();
    let mut pass_hl: heapless::String<64> = heapless::String::new();

    // Push SSID and password into heapless strings
    ssid_hl
        .push_str(wifi_ssid)
        .map_err(|_| anyhow!("SSID too long"))?;
    pass_hl
        .push_str(wifi_pass)
        .map_err(|_| anyhow!("Password too long"))?;
    log::info!("hstrings work");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid_hl,
        password: pass_hl,
        ..Default::default()
    }))?;
    log::info!("configuration set");

    wifi.start()?;
    log::info!("wifi started");
    wifi.connect()?;
    log::info!("wifi connected");

    // Wait for connection
    let mut connected = false;
    for i in 0..20 {
        if wifi.is_connected()? {
            connected = true;
            break;
        }
        log::info!("Waiting for connection... {}/20", i + 1);
        FreeRtos::delay_ms(500);
    }

    if !connected {
        return Err(anyhow!("Failed to connect to WiFi"));
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;
    log::info!("Connected to WiFi! IP: {:?}", ip_info.ip);

    // Start HTTP server
    log::info!("Starting API server on port {}...", SERVER_PORT);
    let _server = ApiServer::new(fish, SERVER_PORT)?;

    log::info!(
        "Server started! Connect to http://{:?}:{} to view status",
        ip_info.ip,
        SERVER_PORT
    );
    log::info!(
        "Send commands to http://{:?}:{}/command via POST",
        ip_info.ip,
        SERVER_PORT
    );

    // Keep the program running
    loop {
        FreeRtos::delay_ms(1000);
    }
}
