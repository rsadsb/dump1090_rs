/*
This crate is meant to be a direct C-to-Rust translation of the algorithms in the popular dump1090 program.
It was developed by referencing the version found at https://github.com/adsbxchange/dump1090-mutability
It matches bit-for-bit in almost every case, but there may be some edge cases where handling of rounding, non-deterministic
timing, and things like that might give results that are not quite identical.
*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// public
pub mod demod_2400;

// public(crate)
pub mod utils;

// private
mod crc;
pub mod icao_filter;
mod mode_s;

pub const MODES_MAG_BUF_SAMPLES: usize = 131_072;

const TRAILING_SAMPLES: usize = 326;
pub const MODES_LONG_MSG_BYTES: usize = 14;
pub const MODES_SHORT_MSG_BYTES: usize = 7;

// dump1090.h:252
#[derive(Copy, Clone, Debug)]
pub struct MagnitudeBuffer {
    pub data: [u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES],
    pub length: usize,
    pub first_sample_timestamp_12mhz: usize,
}

impl Default for MagnitudeBuffer {
    fn default() -> Self {
        Self {
            data: [0_u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES],
            length: 0,
            first_sample_timestamp_12mhz: 0,
        }
    }
}

impl MagnitudeBuffer {
    pub fn push(&mut self, x: u16) {
        // Write data starting at index 0, not TRAILING_SAMPLES offset
        // The demodulator reads from index 0, so data must start there
        self.data[self.length] = x;
        self.length += 1;
    }
}
