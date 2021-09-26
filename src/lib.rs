/*
This crate is meant to be a direct C-to-Rust translation of the algorithms in the popular dump1090 program.
It was developed by referencing the version found at https://github.com/adsbxchange/dump1090-mutability
It matches bit-for-bit in almost every case, but there may be some edge cases where handling of rounding, non-deterministic
timing, and things like that might give results that are not quite identical.
*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use lazy_static::lazy_static;

// public
pub mod demod_2400;
pub mod rtlsdr;

// private
mod crc;
mod icao_filter;
mod mode_s;

pub const MODES_MAG_BUF_SAMPLES: usize = 131_072;

const TRAILING_SAMPLES: usize = 326;
const MODES_LONG_MSG_BYTES: usize = 14;
const MODES_SHORT_MSG_BYTES: usize = 7;

lazy_static! {
    pub static ref MAG_LUT: Vec<u16> = {
        let mut ans: Vec<u16> = vec![];

        for i in 0..256 {
            for q in 0..256 {
                let fi = (i as f32 - 127.5) / 127.5;
                let fq = (q as f32 - 127.5) / 127.5;
                let magsq: f32 = match fi.mul_add(fi, fq * fq) {
                    x if x > 1.0 => 1.0,
                    x => x,
                };
                let mag_f32 = magsq.sqrt();
                let mag_f32_scaled = mag_f32 * 65535.0;
                let mag_f32_rounded = mag_f32_scaled.round();

                let mag: u16 = mag_f32_rounded as u16;

                ans.push(mag);
            }
        }

        ans
    };
}

// dump1090.h:252
pub struct MagnitudeBuffer {
    pub data: Box<[u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES]>,
    pub length: usize,
    pub first_sample_timestamp_12mhz: usize,
    pub dropped: usize,
    pub total_power: f64,
}

impl Default for MagnitudeBuffer {
    fn default() -> Self {
        Self {
            data: Box::new([0_u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES]),
            length: 0,
            first_sample_timestamp_12mhz: 0,
            dropped: 0,
            total_power: 0.0,
        }
    }
}

impl MagnitudeBuffer {
    pub fn push(&mut self, x: u16) {
        self.data[TRAILING_SAMPLES + self.length] = x;
        self.length += 1;
    }
}

#[derive(Default)]
pub struct Modes {
    pub mag_buffer_a: MagnitudeBuffer,
    pub mag_buffer_b: MagnitudeBuffer,
    pub use_buffer_a_next: bool,
}

impl Modes {
    pub fn next_buffer(&mut self, fs: usize) -> &mut MagnitudeBuffer {
        if self.use_buffer_a_next {
            self.mag_buffer_a.first_sample_timestamp_12mhz =
                self.mag_buffer_b.first_sample_timestamp_12mhz
                    + ((12_000_000 * self.mag_buffer_b.length) / fs);
            if self.mag_buffer_b.length > 0 {
                let n = self.mag_buffer_b.length;
                self.mag_buffer_a.data[..TRAILING_SAMPLES]
                    .clone_from_slice(&self.mag_buffer_b.data[(n - TRAILING_SAMPLES)..n])
            };
            self.mag_buffer_a.length = 0;

            // Switch the active buffer for the next call
            self.use_buffer_a_next = false;

            &mut self.mag_buffer_a
        } else {
            self.mag_buffer_b.first_sample_timestamp_12mhz =
                self.mag_buffer_a.first_sample_timestamp_12mhz
                    + ((12_000_000 * self.mag_buffer_a.length) / fs);
            if self.mag_buffer_a.length > 0 {
                let n = self.mag_buffer_a.length;
                self.mag_buffer_b.data[..TRAILING_SAMPLES]
                    .clone_from_slice(&self.mag_buffer_a.data[(n - TRAILING_SAMPLES)..n])
            };
            self.mag_buffer_b.length = 0;

            // Switch the active buffer for the next call
            self.use_buffer_a_next = true;

            &mut self.mag_buffer_b
        }
    }
}
