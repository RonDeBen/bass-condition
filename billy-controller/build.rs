use std::env;

fn main() {
    embuild::espidf::sysenv::output();

    println!(
        "cargo:rustc-env=WIFI_SSID={}",
        env::var("WIFI_SSID").unwrap_or("default_ssid".into())
    );
    println!(
        "cargo:rustc-env=WIFI_PASS={}",
        env::var("WIFI_PASS").unwrap_or("default_pass".into())
    );
}
