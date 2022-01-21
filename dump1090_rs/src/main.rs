// std
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpListener};

// third-party
use clap::Parser;
use num_complex::Complex;

// crate
use libdump1090_rs::utils;

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
    /// soapysdr driver (sdr device)
    #[clap(long, default_value = "rtlsdr")]
    driver: String,
}

const RTLSDR_GAINS: &[(&str, f64)] = &[("TUNER", 49.6)];
const HACKRF_GAINS: &[(&str, f64)] = &[("LNA", 40.0), ("VGA", 52.0)];

fn main() -> Result<(), &'static str> {
    // parse opts
    let options = Options::parse();

    let gains = match options.driver.as_ref() {
        "rtlsdr" => RTLSDR_GAINS,
        "hackrf" => HACKRF_GAINS,
        _ => panic!("unsupported driver"),
    };

    // setup soapysdr
    let d = soapysdr::Device::new(&*format!("driver={}", options.driver)).unwrap();
    let channel = 0;

    d.set_frequency(soapysdr::Direction::Rx, channel, 1_090_000_000.0, ())
        .unwrap();
    println!("{:?}", d.frequency(soapysdr::Direction::Rx, channel));

    d.set_sample_rate(soapysdr::Direction::Rx, channel, 2_400_000.0)
        .unwrap();
    println!("{:?}", d.sample_rate(soapysdr::Direction::Rx, 0));

    println!("{:?}", d.list_gains(soapysdr::Direction::Rx, 0).unwrap());
    //d.set_gain_mode(soapysdr::Direction::Rx, channel, true).unwrap();


    for gain in gains {
        let (name, val) = gain;
        d.set_gain_element(soapysdr::Direction::Rx, channel, *name, *val)
            .unwrap();
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
