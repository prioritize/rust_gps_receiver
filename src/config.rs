use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub port_config: PortConfig,
}

#[derive(Deserialize, Debug)]
pub struct PortConfig {
    pub port: String,
    pub speed: u32,
    pub parity: String,
    pub serial: String,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            port: String::from("/dev/ttyUSB0"),
            speed: 4800,
            parity: String::from("8N1"),
            serial: String::from("XXXXXXXXXXX"),
        }
    }
}
