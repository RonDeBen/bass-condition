use crate::BillyBass;

use anyhow::Result;
use embedded_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::Write;
use shared_models::BillyBassCommand;
use std::sync::{Arc, Mutex};

pub struct ApiServer<'e> {
    _server: EspHttpServer<'e>,
}

impl ApiServer<'_> {
    pub fn new(fish: Arc<Mutex<BillyBass>>, port: u16) -> Result<Self> {
        let mut server = EspHttpServer::new(&Configuration {
            http_port: port,
            ..Default::default()
        })?;

        // Regular HTTP handler for root
        server.fn_handler(
            "/",
            Method::Get,
            |request| -> Result<(), esp_idf_svc::hal::io::EspIOError> {
                let html = r#"<!DOCTYPE html>
    <html>
        <head><title>Billy Bass Controller</title></head>
        <body>
            <h1>Billy Bass API Server</h1>
            <p>Send commands to /command endpoint via POST</p>
        </body>
    </html>"#;

                let mut response = request.into_ok_response()?;
                response.write_all(html.as_bytes())?;
                Ok(())
            },
        )?;

        // Command endpoint
        let fish_clone = fish.clone();
        server.fn_handler(
            "/command",
            Method::Post,
            move |mut request| -> Result<(), esp_idf_svc::hal::io::EspIOError> {
                let fish = fish_clone.clone();

                // Read request body
                let mut buf = [0u8; 1024]; // Adjust size as needed
                let read_size = request.read(&mut buf)?;
                let data = &buf[0..read_size];

                log::info!("Received command data: {} bytes", data.len());

                // Parse command
                match serde_json::from_slice::<BillyBassCommand>(data) {
                    Ok(command) => {
                        log::info!(
                            "Processing command with {} mouth movements",
                            command.mouth_movements.len()
                        );

                        // Execute the command - this will block until complete
                        process_command(fish.clone(), &command);

                        let mut response = request.into_ok_response()?;
                        response.write_all(b"Command processed successfully")?;
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("Failed to parse command: {}", e);
                        let mut response = request.into_status_response(400)?;
                        response.write_all(format!("Error: {}", e).as_bytes())?;
                        Ok(())
                    }
                }
            },
        )?;

        log::info!("API server started on port {}", port);

        Ok(ApiServer { _server: server })
    }
}

fn process_command(fish: Arc<Mutex<BillyBass>>, command: &BillyBassCommand) {
    log::info!(
        "Executing command with {} mouth movements",
        command.mouth_movements.len()
    );

    // Acquire mutex once for the entire command processing
    if let Ok(mut fish) = fish.lock() {
        // Process each movement in sequence
        for movement in &command.mouth_movements {
            log::info!(
                "Movement at {}ms: {} at speed {} for {}ms",
                movement.start_time_ms,
                if movement.is_opening {
                    "Opening"
                } else {
                    "Closing"
                },
                movement.speed,
                movement.duration_ms
            );

            if movement.is_opening {
                fish.mouth_open(movement.speed);
            } else {
                fish.mouth_close(movement.speed);
            }

            esp_idf_svc::hal::delay::FreeRtos::delay_ms(movement.duration_ms);
            fish.mouth_stop();
        }

        log::info!("Command execution completed");
    } else {
        log::error!("Failed to acquire lock on fish - another operation in progress");
    }
}
