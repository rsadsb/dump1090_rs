// This module includes functionality translated from mode_s.c

use crate::{MODES_LONG_MSG_BYTES, MODES_SHORT_MSG_BYTES};

// mode_s.c:215
#[must_use]
pub fn getbits(data: &[u8], firstbit_1idx: usize, lastbit_1idx: usize) -> usize {
    let mut ans: usize = 0;

    // The original code uses indices that start at 1 and we need 0-indexed values
    let (firstbit, lastbit) = (firstbit_1idx - 1, lastbit_1idx - 1);

    for bit_idx in firstbit..=lastbit {
        ans *= 2;
        let byte_idx: usize = bit_idx / 8;
        let mask = 2_u8.pow(7_u32 - (bit_idx as u32) % 8);
        if (data[byte_idx] & mask) != 0_u8 {
            ans += 1;
        }
    }

    ans
}

// mode_s.c:289
#[must_use]
pub fn score_modes_message(msg: &[u8]) -> i32 {
    let validbits = msg.len() * 8;

    if validbits < 56 {
        return -2;
    }

    let msgtype = getbits(msg, 1, 5);
    let msgbits = if (msgtype & 0x10) != 0 {
        MODES_LONG_MSG_BYTES * 8
    } else {
        MODES_SHORT_MSG_BYTES * 8
    };

    if validbits < msgbits {
        return -2;
    }
    if msg.iter().all(|b| *b == 0x00) {
        return -2;
    }

    let crc = super::crc::modes_checksum(msg, msgbits);

    match msgtype {
        0 | 4 | 5 | 16 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 => {
            // 0:  short air-air surveillance
            // 4:  surveillance, altitude reply
            // 5:  surveillance, altitude reply
            // 16: long air-air surveillance
            // 24: Comm-D (ELM)
            // 25: Comm-D (ELM)
            // 26: Comm-D (ELM)
            // 27: Comm-D (ELM)
            // 28: Comm-D (ELM)
            // 29: Comm-D (ELM)
            // 30: Comm-D (ELM)
            // 31: Comm-D (ELM)

            if super::icao_filter::icao_filter_test(crc) {
                1000
            } else {
                -1
            }
        }
        11 => {
            // 11: All-call reply
            let iid = crc & 0x7f;
            let crc = crc & 0x00ff_ff80;
            let addr = getbits(msg, 9, 32) as u32;

            match (crc, iid, super::icao_filter::icao_filter_test(addr)) {
                (0, 0, true) => 1600,
                (0, 0, false) => 750,
                (0, _, true) => 1000,
                (0, _, false) => -1,
                (_, _, _) => -2,
            }
        }
        17 | 18 => {
            // 17: Extended squitter
            // 18: Extended squitter/non-transponder
            let addr = getbits(msg, 9, 32) as u32;

            match (crc, super::icao_filter::icao_filter_test(addr)) {
                (0, true) => 1800,
                (0, false) => 1400,
                (_, _) => -2,
            }
        }
        20 | 21 => {
            // 20: Comm-B, altitude reply
            // 21: Comm-B, identity reply
            match super::icao_filter::icao_filter_test(crc) {
                true => 1000,
                false => -2,
            }
        }
        _ => -2,
    }
}
