pub mod config;
use anyhow::Result;
use anyhow::anyhow;
use config::{ConfigFile, PortConfig};
use serialport::{DataBits, Parity, StopBits, TTYPort};
use std::time::Duration;
use std::{fs::File, io::Read};

// Function that opens the serial port, performs a test read and then returns a result
pub async fn open_serial_port(config: PortConfig) -> Result<TTYPort> {
    match serialport::available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    serialport::SerialPortType::UsbPort(usb_port_info) => {
                        println!("{usb_port_info:?}");
                        if usb_port_info.serial_number.is_some_and(|sn| {
                            println!("{sn} -- {}", config.serial);
                            sn == config.serial
                        }) {
                            println!("Made it here");
                            match serialport::new(config.port, config.speed)
                                .data_bits(DataBits::Eight)
                                .parity(Parity::None)
                                .stop_bits(StopBits::One)
                                .timeout(Duration::from_millis(200))
                                .open_native()
                            {
                                Ok(port) => return Ok(port),
                                Err(_) => return Err(anyhow!("Failure to open!")),
                            }
                        }
                    }
                    serialport::SerialPortType::PciPort => {}
                    serialport::SerialPortType::BluetoothPort => {}
                    serialport::SerialPortType::Unknown => {}
                }
            }
        }
        Err(e) => {
            println!("An error was encountered when trying to list the available serial ports");
            println!("{e:?}")
        }
    };

    todo!()
}
// Function that consumes all the incoming serial port data and sends it to the appropriate
// channels
pub async fn consume_serial_port() {}

#[tokio::main]
async fn main() {
    // TODO: Open the config file and read the port information
    let mut file = File::open("./src/gps_config.toml").unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents).unwrap();
    let conf: ConfigFile = toml::from_str(&contents).unwrap();
    println!("{conf :?}");

    // TODO: Spawn a thread that reads the serial port and sends it through a channel

    // TODO: Spawn a thread waits for incoming messages and parses them
    let mut port = serialport::new(conf.port_config.port, conf.port_config.speed)
        .open_native()
        .expect("failed to open port");
    let mut serial_buf: Vec<u8> = vec![0; 5];
    port.read_exact(serial_buf.as_mut_slice())
        .expect("Found no data");
    println!("{}", String::from_utf8(serial_buf).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PortConfig, open_serial_port};

    #[tokio::test]
    async fn test_read_available() -> Result<()> {
        let mut config = PortConfig::default();
        config.serial = String::from("BYBBb115819");
        let mut port = open_serial_port(config).await?;
        let mut buf = [0u8; 64];
        let _ = port.read_exact(&mut buf);
        println!(
            "The buffer contains -- {}",
            String::from_utf8(buf.to_vec()).unwrap()
        );
        Ok(())
    }
}
