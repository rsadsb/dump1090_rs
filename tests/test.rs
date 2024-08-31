// third-part
use assert_hex::assert_eq_hex;
use hexlit::hex;
// crate
use libdump1090_rs::{demod_2400::demodulate2400, icao_filter::icao_flush, utils};

fn routine(filename: &str, expected_data: &Vec<Vec<u8>>) {
    // make sure icao starts in a deterministic position
    icao_flush();
    let buf = utils::read_test_data(filename);
    let outbuf = utils::to_mag(&buf);

    let data = demodulate2400(&outbuf).unwrap();
    for (a, b) in data.iter().zip(expected_data.iter()) {
        assert_eq_hex!(a.buffer(), *b);
    }
}

#[test]
fn test_01() {
    let filename = "test_iq/test_1641427457780.iq";
    let expected_data = Vec::from([
        hex!("8dad929358b9c6273f002169c02e").to_vec(),
        hex!("8daa2bc4f82100020049b8db9449").to_vec(),
        hex!("8daa2bc4f82100020049b8db9449").to_vec(),
        hex!("02e1971ce17c84").to_vec(),
        hex!("8da0aaa058bf163fcf860013e840").to_vec(),
    ]);
    routine(filename, &expected_data);
}

#[test]
fn test_02() {
    let filename = "test_iq/test_1641428165033.iq";
    let expected_data = Vec::from([
        hex!("8da79de99909932f780c9e2f2f8f").to_vec(),
        hex!("8dac04d358a7820a86ac3709e689").to_vec(),
        hex!("8dac04d3ea4288669b5c082751d4").to_vec(),
        hex!("8da79de958bdf59c85104874adad").to_vec(),
        hex!("5dad92936265f5").to_vec(),
        hex!("5dad92936265f525be017735997b").to_vec(),
    ]);
    routine(filename, &expected_data);
}

#[test]
fn test_03() {
    let filename = "test_iq/test_1641428106243.iq";
    let expected_data = Vec::from([
        hex!("8da8aac8990c30b51808aa24e573").to_vec(),
        hex!("02e19838bff1d9").to_vec(),
        hex!("8dada6b9990cf61e4848af2a8656").to_vec(),
        hex!("8da4ba025885462008fa0a4a6eb2").to_vec(),
        hex!("8da4ba025885462008fa0a4a6eb2").to_vec(),
        hex!("8da4ba0299115f301074a72db6ff").to_vec(),
    ]);

    routine(filename, &expected_data);
}
