// crate
use crate::MagnitudeBuffer;

// third-party
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_complex::Complex;

pub fn save_test_data(data: &[Complex<i16>]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let name = format!("test_{}.iq", now);
    let mut file = std::fs::File::create(name).unwrap();

    for d in data {
        file.write_i16::<LittleEndian>(d.im).unwrap();
        file.write_i16::<LittleEndian>(d.re).unwrap();
    }
}

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

pub fn to_mag(data: &[Complex<i16>]) -> MagnitudeBuffer {
    let mut outbuf = MagnitudeBuffer::default();
    for b in data {
        // TODO: lookup table
        let i = b.im;
        let q = b.re;

        let fi = i as f32 / (1 << 15) as f32;
        let fq = q as f32 / (1 << 15) as f32;

        let mag_sqr = fi * fi + fq * fq;
        let mag = f32::sqrt(mag_sqr);
        outbuf.push((mag * u16::MAX as f32 + 0.5) as u16);
    }
    outbuf
}
