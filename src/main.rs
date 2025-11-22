pub mod config;
use anyhow::Result;
use anyhow::anyhow;
use bytes::BytesMut;
use config::{ConfigFile, PortConfig};
use std::os::unix::thread;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Instant;
use tokio::time::Sleep;
use tokio::time::sleep;
use tokio::time::sleep_until;
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;
use tokio_serial::{DataBits, Parity, StopBits};
use tokio_stream::StreamExt;
use tokio_util::codec::LinesCodec;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::sync::CancellationToken;

pub enum GPS {
    RMC,
    GGA,
    GSA,
    GSV,
}

// TODO: This function will spawn a thread and be sent messages, and send them to the appropriate parsers channel
pub async fn message_listener() {}

// TODO: This function will spawn a thread that listens for lines coming across the serial port,
// parse the "Header" and then sends them to the appropriate channel
pub async fn line_parser(msg_rx: &mut Receiver<String>, out_tx: Sender<String>) {
    println!("Into Line Parser");
    if let Some(msg) = msg_rx.recv().await {
        match &msg[3..7] {
            "GGA" => out_tx.send(String::from("GGA")).await.unwrap(),
            "RMC" => out_tx.send(String::from("RMC")).await.unwrap(),
            "GSA" => out_tx.send(String::from("GSA")).await.unwrap(),
            "GSV" => out_tx.send(String::from("GSV")).await.unwrap(),
            _ => out_tx.send(msg).await.unwrap(),
        }
    }
}

pub async fn setup(cfg: &PortConfig) -> Result<()> {
    // Check the serial ports
    let stream = match open_serial_port(cfg).await {
        Ok(s) => s,
        Err(_) => return Err(anyhow!("Unable to open the serial port")),
    };
    let (line_tx, mut line_rx) = sync::mpsc::channel(10);
    //let (msg_tx, mut msg_rx) = sync::mpsc::channel(10);
    let (out_tx, mut out_rx) = mpsc::channel(10);
    tokio::task::spawn(async move {
        consume_serial_port(stream, Some(Duration::from_secs(5)), line_tx.clone()).await
    });
    tokio::task::spawn(async move {
        loop {
            line_parser(&mut line_rx, out_tx.clone()).await;
        }
    });
    tokio::task::spawn(async move {
        loop {
            if let Some(x) = out_rx.recv().await {
                println!("{x}");
            }
        }
    });
    println!("Got through all Tokio Spawns");

    tokio::time::sleep(Duration::from_secs(5));
    // Open the serial port
    // Check for good data
    // Spawn the line_parser
    // Spawn the message_listener
    // Spawn a thread for each message type

    Ok(())
}

pub struct SerialBuffer {
    pub rx: tokio::sync::mpsc::Receiver<String>,
    pub tx: tokio::sync::mpsc::Sender<String>,
    buf: Vec<String>,
    current: Vec<u8>,
    token: CancellationToken,
}
impl SerialBuffer {
    async fn watch(&mut self) {
        // Thoughts Process:
        // Incoming message is added to the back of the buffer.
        // When the buffer hits a certain size we trigger a search for a message $->\n
        // We construct the message, remove it from the buffer, move all the data forward
        // and then pass the message out
        loop {
            if let Some(v) = self.rx.recv().await {
                println!("received");
                println!("length: {}", self.buf.len());
                println!("{v}");
                self.buf.push(v);
            }
            // let msg: &[u8] = &self.buf[first..last];
            // self.tx
            //     .send(String::from_utf8_lossy(&self.buf[first..last]))
            //     .await;
        }
    }
}
// Function that opens the serial port, performs a test read and then returns a result
pub async fn open_serial_port(config: &PortConfig) -> Result<SerialStream> {
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
                            match tokio_serial::new(&config.port, config.speed)
                                .data_bits(DataBits::Eight)
                                .parity(Parity::None)
                                .stop_bits(StopBits::One)
                                .timeout(Duration::from_millis(200))
                                .open_native_async()
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
pub async fn consume_serial_port(port: SerialStream, dur: Option<Duration>, tx: Sender<String>) {
    let mut _internal_buf = [0u8; 256];
    let mut reader = FramedRead::new(port, LinesCodec::new());
    println!("serialport is consumed");
    match dur {
        Some(d) => {
            let now = Instant::now();
            while let Some(result) = reader.next().await {
                match result {
                    Ok(bytes) => {
                        let _ = tx.send(bytes).await;
                    }
                    Err(_) => todo!(),
                }
                if now.elapsed() > d {
                    break;
                }
            }
        }
        None => {
            while let Some(result) = reader.next().await {
                match result {
                    Ok(bytes) => {
                        let _ = tx.send(bytes).await;
                    }
                    Err(_) => todo!(),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // TODO: Open the config file and read the port information
    let mut file = File::open("./src/gps_config.toml").await.unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents).await.unwrap();
    let conf: ConfigFile = toml::from_str(&contents).unwrap();
    println!("{conf :?}");

    // TODO: Spawn a thread that reads the serial port and sends it through a channel

    // TODO: Spawn a thread waits for incoming messages and parses them
    let mut port = tokio_serial::new(conf.port_config.port, conf.port_config.speed)
        .open_native_async()
        .expect("failed to open port");
    let mut serial_buf: Vec<u8> = vec![0; 5];
    let l = port
        .read(serial_buf.as_mut_slice())
        .await
        .expect("Found no data");
    println!("{}", String::from_utf8(serial_buf[0..l].to_vec()).unwrap());
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;
    use crate::{PortConfig, open_serial_port};

    #[tokio::test]
    #[ignore]
    async fn test_read_available() -> Result<()> {
        let config = PortConfig {
            serial: String::from("BYBBb115819"),
            ..Default::default()
        };
        let mut port = open_serial_port(&config).await?;
        let mut incoming_buffer: Vec<u8> = Vec::new();
        let mut buf = [0u8; 64];
        let (ser_tx, mut ser_rx) = mpsc::channel(10);
        let (msg_tx, msg_rx) = mpsc::channel(20);
        let mut sb = SerialBuffer {
            rx: ser_rx,
            tx: msg_tx,
            buf: vec![],
            current: vec![],
            token: CancellationToken::new(),
        };
        // TODO: Refactor this code to not have the buffer owned by the Serial Buffer
        // TODO: This currently forces the watch function and the
        tokio::spawn(async move {
            loop {
                sb.watch().await;
            }
        });
        consume_serial_port(port, Some(Duration::from_secs(2)), ser_tx).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_setup() {
        let mut file = File::open("./src/gps_config.toml").await.unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents).await.unwrap();
        let cfg: ConfigFile = toml::from_str(&contents).unwrap();
        let _ = setup(&cfg.port_config).await;
    }
}
