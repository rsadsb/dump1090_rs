use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use libdump1090_rs::{demod_2400::demodulate2400, icao_filter::icao_flush, utils};
use num_complex::Complex;

fn routine(data: [Complex<i16>; 0x20000]) {
    // make sure icao starts in a deterministic position
    icao_flush();
    let outbuf = utils::to_mag(&data);
    let _ = black_box(demodulate2400(&outbuf).unwrap());
}

fn criterion_benchmark(c: &mut Criterion) {
    let filename = "test_iq/test_1641427457780.iq";
    let data_01 = utils::read_test_data(filename);

    let filename = "test_iq/test_1641428165033.iq";
    let data_02 = utils::read_test_data(filename);

    let filename = "test_iq/test_1641428106243.iq";
    let data_03 = utils::read_test_data(filename);
    c.bench_function("01", |b| b.iter(|| routine(data_01)));
    c.bench_function("02", |b| b.iter(|| routine(data_02)));
    c.bench_function("03", |b| b.iter(|| routine(data_03)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
