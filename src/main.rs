extern crate libudev;
extern crate notify_rust;
#[macro_use] extern crate serde_derive;
extern crate toml;
extern crate xdg;

use libudev::{Context, Device, Enumerator, Event, EventType, Monitor};
use notify_rust::Notification;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;
use std::fs::File;

mod config;

const SLEEP_DURATION: u64 = 1000;

struct DeviceManager {
    config: config::Config,
}

impl DeviceManager {
    fn handle_device(&self, device: &Device) {
        for (name, watched) in &self.config.devices {
            if is_product(&watched.product, device) {
                notify(&format!("{} plugged", name));
                execute(&watched.on_plugged);
            }
        }
    }

    fn handle_change(&self, event: &Event) {
        for (name, watched) in &self.config.devices {
            if is_product(&watched.product, event.device()) {
                match event.event_type() {
                    EventType::Add => {
                        notify(&format!("{} plugged", name));
                        execute(&watched.on_plugged);
                    },
                    EventType::Remove => {
                        notify(&format!("{} unplugged", name));
                        execute(&watched.on_unplugged);
                    },
                    _ => { break }
                }
            }
        }
    }
}

fn main() {
    let config = get_config().unwrap();
    let manager = DeviceManager { config };

    let context = Context::new().unwrap();
    let mut enumerator = Enumerator::new(&context).unwrap();

    enumerator.match_subsystem("input").unwrap();

    for device in enumerator.scan_devices().unwrap() {
        manager.handle_device(&device);
    }

    let mut monitor = Monitor::new(&context).unwrap();
    assert!(monitor.match_subsystem("input").is_ok());
    let mut socket = monitor.listen().unwrap();

    loop {
        match socket.receive_event() {
            Some(event) => manager.handle_change(&event),
            None => sleep(Duration::from_millis(SLEEP_DURATION)),
        }
    }
}

fn is_product(product: &str, device: &Device) -> bool {
    for property in device.properties() {
        if property.name() == "PRODUCT" && property.value() == product {
            return true;
        }
    }

    false
}

fn execute(command: &str) {
    let cmd: Vec<_> = command.split(" ").collect();
    Command::new(cmd[0])
        .args(&cmd[1..])
        .spawn()
        .expect("Failed to run command");
}

fn notify(message: &str) {
    Notification::new()
        .summary("Auto xkbmap")
        .body(message)
        .icon("keyboard")
        .show()
        .unwrap();
}

/// Load configuration from file.
/// Create configuration file from defaults if needed.
fn get_config() -> Result<config::Config, std::io::Error> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?;
    let config_path = xdg_dirs.place_config_file("config.toml")?;

    if !config_path.exists() {
        let toml = toml::to_string(&config::Config::default()).unwrap();
        let mut file = File::create(&config_path)?;
        file.write_all(toml.as_bytes())?;
    }

    config::Config::from_file(config_path)
}