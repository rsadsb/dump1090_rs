// crate
// third-party
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_complex::Complex;

use crate::MagnitudeBuffer;

pub fn save_test_data(data: &[Complex<i16>]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let name = format!("test_{now}.iq");
    let mut file = std::fs::File::create(name).unwrap();

    for d in data {
        file.write_i16::<LittleEndian>(d.im).unwrap();
        file.write_i16::<LittleEndian>(d.re).unwrap();
    }
}

#[must_use]
pub fn read_test_data(filepath: &str) -> [Complex<i16>; 0x20000] {
    let mut file = std::fs::File::open(filepath).unwrap();
    let mut r_buf = [Complex::new(0, 0); 0x20000];

    let mut i = 0;
    loop {
        let im = file.read_i16::<LittleEndian>().unwrap();
        let re = file.read_i16::<LittleEndian>().unwrap();
        r_buf[i] = Complex::new(re, im);

        i += 1;
        if i == 0x20000 {
            break;
        }
    }

    r_buf
}

#[must_use]
#[inline]
pub fn to_mag(data: &[Complex<i16>]) -> MagnitudeBuffer {
    let mut outbuf = MagnitudeBuffer::default();

    for b in data {
        // Use exact sqrt calculation to preserve message decoding accuracy
        // Note: Approximations (alpha-max-beta-min, lookup tables) cause
        // incorrect message decoding due to sensitivity in preamble detection
        // and phase-locked loop. This matches what all modern dump1090
        // implementations do for SC16 format (see dump1090-mutability,
        // readsb, SDRplay/dump1090 convert.c)
        let i = b.im;
        let q = b.re;

        let fi = f32::from(i) / 32768.0;
        let fq = f32::from(q) / 32768.0;

        let mut mag_sqr = fi.mul_add(fi, fq * fq);
        // Clamp to 1.0 to handle any floating-point rounding edge cases
        // (matching readsb's approach)
        if mag_sqr > 1.0 {
            mag_sqr = 1.0;
        }
        let mag = f32::sqrt(mag_sqr);

        outbuf.push(mag.mul_add(f32::from(u16::MAX), 0.5) as u16);
    }
    outbuf
}
