// std
use std::io::Cursor;

// third-party
use byteorder::{BigEndian, ReadBytesExt};
use criterion::{criterion_group, criterion_main, Criterion};

// crate
use dump1090_rs::MagnitudeBuffer;

fn demod_iq(iq_buf: &[u8]) -> Vec<[u8; 14]> {
    let mut outbuf = MagnitudeBuffer::default();

    let mut rdr = Cursor::new(&iq_buf);

    while let Ok(iq) = rdr.read_u16::<BigEndian>() {
        let this_mag: u16 = dump1090_rs::MAG_LUT[iq as usize];

        outbuf.push(this_mag);
    }
    dump1090_rs::demod_2400::demodulate2400(&outbuf).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let f_buffer_00 = std::fs::read("tests/test_00.iq").unwrap();
    let f_buffer_01 = std::fs::read("tests/test_01.iq").unwrap();
    let f_buffer_02 = std::fs::read("tests/test_02.iq").unwrap();
    c.bench_function("00", |b| b.iter(|| demod_iq(&f_buffer_00)));
    c.bench_function("01", |b| b.iter(|| demod_iq(&f_buffer_01)));
    c.bench_function("02", |b| b.iter(|| demod_iq(&f_buffer_02)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
