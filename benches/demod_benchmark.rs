// third-party
use assert_hex::assert_eq_hex;
use criterion::{criterion_group, criterion_main, Criterion};
use hexlit::hex;
use num_complex::Complex;

// crate
use libdump1090_rs::utils;

fn routine(data: [Complex<i16>; 0x20000], expected_data: &Vec<[u8; 14]>) {
    let outbuf = utils::to_mag(&data);
    let data = libdump1090_rs::demod_2400::demodulate2400(&outbuf).unwrap();
    assert_eq_hex!(expected_data, &*data);
}

fn criterion_benchmark(c: &mut Criterion) {
    let filename = "test_iq/test_1641427457780.iq";
    let data_01 = utils::read_test_data(filename);
    let expected_data_01 = Vec::from([
        hex!("8dad929358b9c6273f002169c02e"),
        hex!("8daa2bc4f82100020049b8db9449"),
        hex!("8daa2bc4f82100020049b8db9449"),
        hex!("8da0aaa058bf163fcf860013e840"),
    ]);

    let filename = "test_iq/test_1641428165033.iq";
    let data_02 = utils::read_test_data(filename);
    let expected_data_02 = Vec::from([
        hex!("8da79de99909932f780c9e2f2f8f"),
        hex!("8dac04d358a7820a86ac3709e689"),
        hex!("8dac04d3ea4288669b5c082751d4"),
        hex!("8da79de958bdf59c85104874adad"),
        hex!("5dad92936265f525be017735997b"),
    ]);

    let filename = "test_iq/test_1641428106243.iq";
    let data_03 = utils::read_test_data(filename);
    let expected_data_03 = Vec::from([
        hex!("8da8aac8990c30b51808aa24e573"),
        hex!("8dada6b9990cf61e4848af2a8656"),
        hex!("8da4ba025885462008fa0a4a6eb2"),
        hex!("8da4ba025885462008fa0a4a6eb2"),
        hex!("8da4ba0299115f301074a72db6ff"),
    ]);
    c.bench_function("01", |b| b.iter(|| routine(data_01, &expected_data_01)));
    c.bench_function("02", |b| b.iter(|| routine(data_02, &expected_data_02)));
    c.bench_function("03", |b| b.iter(|| routine(data_03, &expected_data_03)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
