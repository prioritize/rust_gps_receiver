pub mod config;
use anyhow::Result;
use anyhow::anyhow;
use config::{ConfigFile, PortConfig};
use serialport::TTYPort;
use std::fs::File;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio_serial::{DataBits, Parity, StopBits};

// Function that opens the serial port, performs a test read and then returns a result
pub async fn open_serial_port(config: PortConfig) -> Result<TTYPort> {
    match tokio_serial::available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    tokio_serial::SerialPortType::UsbPort(usb_port_info) => {
                        println!("{usb_port_info:?}");
                        if usb_port_info.serial_number.is_some_and(|sn| {
                            println!("{sn} -- {}", config.serial);
                            sn == config.serial
                        }) {
                            println!("Made it here");
                            match tokio_serial::new(config.port, config.speed)
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
                    tokio_serial::SerialPortType::PciPort => {}
                    tokio_serial::SerialPortType::BluetoothPort => {}
                    tokio_serial::SerialPortType::Unknown => {}
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
pub async fn consume_serial_port(port: &mut TTYPort) {
    let mut internal_buf = [0u8; 256];
    loop {
        match port.read(&mut internal_buf) {
            Ok(a) => todo!(),
            Err(_) => todo!(),
        }
    }
}

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
    let mut port = tokio_serial::new(conf.port_config.port, conf.port_config.speed)
        .open_native()
        .expect("failed to open port");
    let mut serial_buf: Vec<u8> = vec![0; 5];
    port.read(serial_buf.as_mut_slice()).expect("Found no data");
    println!("{}", String::from_utf8(serial_buf).unwrap());
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;
    use crate::{PortConfig, open_serial_port};

    #[tokio::test]
    async fn test_read_available() -> Result<()> {
        let config = PortConfig {
            serial: String::from("BYBBb115819"),
            ..Default::default()
        };
        let mut port = open_serial_port(config).await?;
        let mut incoming_buffer: Vec<u8> = Vec::new();
        let mut buf = [0u8; 64];
        for _ in 0..100 {
            let _ = port.read(buf.as_mut_slice());
            let cap_string = match String::from_utf8(buf.to_vec()) {
                Ok(v) => Some(v),
                Err(_) => {
                    println!("{buf:?}");
                    None
                }
            };
            sleep(Duration::from_millis(100));
            println!("The buffer contains -- {cap_string:?}",);
        }
        Ok(())
    }
}
