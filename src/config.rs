use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std;
use toml;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub devices: HashMap<String, WatchedDevice>,
}

#[derive(Deserialize, Serialize)]
pub struct WatchedDevice {
    pub product: String,
    pub on_plugged: String,
    pub on_unplugged: String,
}

impl Config {
    pub fn from_file(path: std::path::PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let config: Config = toml::from_str(&buffer).unwrap();

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut devices = HashMap::new();
        devices.insert("ergodox".to_owned(), WatchedDevice {
            product: "3/feed/1307/111".to_owned(),
            on_plugged: "setxkbmap us".to_owned(),
            on_unplugged: "setxkbmap fr oss -option ctrl:nocaps".to_owned(),
        });
        Config { devices }
    }
}