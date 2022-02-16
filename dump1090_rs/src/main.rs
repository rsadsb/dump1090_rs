// std
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpListener};

// third-party
use clap::Parser;
use num_complex::Complex;
use serde::{Deserialize, Serialize};

// crate
use libdump1090_rs::utils;

#[derive(Debug, Deserialize, Serialize)]
struct SdrConfig {
    pub sdrs: Vec<Sdr>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Sdr {
    pub driver: String,
    pub gain: Vec<Gain>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Gain {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Parser)]
#[clap(
    version,
    name = "dump1090_rs",
    author = "wcampbell0x2a",
    about = "ADS-B Demodulator and Server"
)]
struct Options {
    /// ip address
    #[clap(long, default_value = "127.0.0.1")]
    host: Ipv4Addr,

    /// port
    #[clap(long, default_value = "30002")]
    port: u16,

    /// soapysdr driver name (sdr device) from default `config.toml` or `--custom-config`
    #[clap(long, default_value = "rtlsdr")]
    driver: String,

    /// filepath for config.toml file overriding or adding sdr gain values
    ///
    /// An example:
    /// ```
    /// [[sdr]]
    /// driver = "custom"
    /// [[sdr.gain]]
    /// name = "GAIN"
    /// value = 0.0
    /// ```
    #[clap(long)]
    custom_config: Option<String>,
}

fn main() -> Result<(), &'static str> {
    // read in default compiled config
    let mut config: SdrConfig = toml::from_str(include_str!("../config.toml")).unwrap();

    // parse opts
    let options = Options::parse();

    // parse config from custom filepath
    if let Some(config_filepath) = options.custom_config {
        let custom_config: SdrConfig =
            toml::from_str(&std::fs::read_to_string(&config_filepath).unwrap()).unwrap();
        println!("[-] read in custom config: {config_filepath}");
        // push new configs to the front, so that the `find` method finds these first
        for sdr in custom_config.sdrs {
            config.sdrs.insert(0, sdr);
        }
    }

    // setup soapysdr driver
    println!("[-] using driver: {}", options.driver);
    let d = soapysdr::Device::new(&*format!("driver={}", options.driver)).unwrap();
    let channel = 0;

    d.set_frequency(soapysdr::Direction::Rx, channel, 1_090_000_000.0, ())
        .unwrap();
    println!(
        "[-] frequency: {:?}",
        d.frequency(soapysdr::Direction::Rx, channel)
    );

    d.set_sample_rate(soapysdr::Direction::Rx, channel, 2_400_000.0)
        .unwrap();
    println!(
        "[-] sample rate: {:?}",
        d.sample_rate(soapysdr::Direction::Rx, 0)
    );

    println!(
        "[-] available gains: {:?}",
        d.list_gains(soapysdr::Direction::Rx, 0).unwrap()
    );

    // check if --driver exists in config
    if let Some(sdr) = config.sdrs.iter().find(|a| a.driver == options.driver) {
        for gain in &sdr.gain {
            println!("[-] Setting gain: {} = {}", gain.name, gain.value);
            d.set_gain_element(soapysdr::Direction::Rx, channel, &*gain.name, gain.value)
                .unwrap();
        }
    } else {
        println!("[-] --driver gain values not found in config, not setting gain values");
    }

    let mut stream = d.rx_stream::<Complex<i16>>(&[channel]).unwrap();

    let mut buf = vec![Complex::new(0, 0); stream.mtu().unwrap()];
    stream.activate(None).unwrap();

    // bind to listener port
    let listener = TcpListener::bind((options.host, options.port)).unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    let mut sockets = vec![];

    loop {
        if let Ok((s, _addr)) = listener.accept() {
            sockets.push(s);
        }

        if let Ok(len) = stream.read(&[&mut buf], 5_000_000) {
            //utils::save_test_data(&buf[..len]);
            let buf = &buf[..len];
            let outbuf = utils::to_mag(buf);
            let resulting_data = libdump1090_rs::demod_2400::demodulate2400(&outbuf).unwrap();
            if !resulting_data.is_empty() {
                let resulting_data: Vec<String> = resulting_data
                    .iter()
                    .map(|a| {
                        let a = hex::encode(a);
                        let a = format!("*{};\n", a);
                        println!("{}", &a[..a.len() - 1]);
                        a
                    })
                    .collect();

                let mut remove_indexs = vec![];
                for (i, mut socket) in &mut sockets.iter().enumerate() {
                    for msg in &resulting_data {
                        // write, or add to remove list if ConnectionReset
                        if let Err(e) = socket.write_all(msg.as_bytes()) {
                            if e.kind() == std::io::ErrorKind::ConnectionReset {
                                remove_indexs.push(i);
                                break;
                            }
                        }
                    }
                }

                // remove
                for i in remove_indexs {
                    sockets.remove(i);
                }
            }
        }
    }
}
