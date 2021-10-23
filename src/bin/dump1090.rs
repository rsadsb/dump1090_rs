// std
use std::io::prelude::*;
use std::io::Cursor;
use std::net::TcpListener;

// third-party
use byteorder::{BigEndian, ReadBytesExt};
use clap::App;

// crate
use dump1090_rs::{rtlsdr, MagnitudeBuffer, MODES_MAG_BUF_SAMPLES};

fn main() -> Result<(), &'static str> {
    let _matches = App::new("Rust dump1090")
        .version(clap::crate_version!())
        .author("John Stanford (johnwstanford@gmail.com)")
        .about("Translation of dump1090-mutability into Rust, intended to match bit-for-bit");

    let mut f_buffer: [u8; 2 * MODES_MAG_BUF_SAMPLES] = [0_u8; 2 * MODES_MAG_BUF_SAMPLES];
    let mut active: bool = true;

    let mut dev = rtlsdr::RtlSdrDevice::new(0)?;

    let available_gains = dev.get_tuner_gains()?;
    eprintln!("Available gains: {:?}", available_gains);

    let max_gain: i32 = *(available_gains.iter().max().unwrap());
    eprintln!("Max available gain: {:.1} [dB]", (max_gain as f32) * 0.1);

    dev.set_tuner_gain_mode(1)?;
    dev.set_tuner_gain(max_gain)?;
    if dev.set_freq_correction(0).is_err() {
        // For some reason, this function returns -2 when we set the frequency correction to 0
        // The same thing happens in dump1090, but the return value is never checked
        eprintln!("Warning: Nonzero return value from set_freq_correction");
    }
    dev.set_center_freq(1_090_000_000)?;
    dev.set_sample_rate(2_400_000)?;

    eprintln!("Set center freq to {:.4e} [Hz]", dev.get_center_freq()?);
    eprintln!(
        "Set freq correction to {} [ppm]",
        dev.get_freq_correction()?
    );
    eprintln!(
        "Set tuner gain to {:.1} [dB]",
        (dev.get_tuner_gain()? as f32) * 0.1
    );
    eprintln!("Set sample rate to {}", dev.get_sample_rate()?);

    dev.reset_buffer()?;

    let listener = TcpListener::bind("127.0.0.1:30002").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    let mut sockets = vec![];

    while active {
        if let Ok((s, _addr)) = listener.accept() {
            sockets.push(s);
        }

        let mut outbuf = MagnitudeBuffer::default();
        let read_result = dev.read(&mut f_buffer);
        match read_result {
            Err(_) | Ok(0) => active = false,
            Ok(n) => {
                // un-comment this for creating test data
                //std::fs::write("test_01.iq", &f_buffer[..n]);
                let mut rdr = Cursor::new(&f_buffer[..n]);

                while let Ok(iq) = rdr.read_u16::<BigEndian>() {
                    let this_mag: u16 = dump1090_rs::MAG_LUT[iq as usize];

                    outbuf.push(this_mag);
                }
            }
        }

        let resulting_data = dump1090_rs::demod_2400::demodulate2400(&outbuf).unwrap();
        if !resulting_data.is_empty() {
            println!("{:x?}", resulting_data);
            let resulting_data: Vec<String> = resulting_data
                .iter()
                .map(|a| {
                    let a = hex::encode(a);
                    format!("*{};\n", a)
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

    Ok(())
}
