// This module includes functionality translated from mode_s.c

use crate::{
    demod_2400::MsgLen,
    icao_filter::{icao_filter_add, ICAO_FILTER_ADSB_NT},
    MODES_LONG_MSG_BYTES, MODES_SHORT_MSG_BYTES,
};

use super::{crc::{modes_checksum, fix_single_bit_error}, icao_filter::icao_filter_test};

// mode_s.c:215
#[must_use]
#[inline(always)]
pub fn getbits(data: &[u8], firstbit_1idx: usize, lastbit_1idx: usize) -> usize {
    let mut ans: usize = 0;

    // The original code uses indices that start at 1 and we need 0-indexed values
    let (firstbit, lastbit) = (firstbit_1idx - 1, lastbit_1idx - 1);

    for bit_idx in firstbit..=lastbit {
        ans <<= 1;
        let byte_idx: usize = bit_idx / 8;
        let mask = 1u8 << (7 - (bit_idx % 8));
        if (data[byte_idx] & mask) != 0_u8 {
            ans |= 1;
        }
    }

    ans
}

// mode_s.c:289
#[must_use]
pub fn score_modes_message(msg: &[u8]) -> Option<(MsgLen, i32)> {
    let validbits = msg.len() * 8;

    if validbits < MODES_SHORT_MSG_BYTES * 8 {
        return None;
    }

    let df = getbits(msg, 1, 5);
    let (msgbits, msglen) = if (df & 0x10) != 0 {
        (MODES_LONG_MSG_BYTES * 8, MsgLen::Long)
    } else {
        (MODES_SHORT_MSG_BYTES * 8, MsgLen::Short)
    };

    if validbits < msgbits {
        return None;
    }
    if msg.iter().all(|b| *b == 0x00) {
        return None;
    }

    let res = match df {
        0 | 4 | 5 => {
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

            let crc = modes_checksum(msg, msgbits);
            if icao_filter_test(crc) {
                1000
            } else {
                -1
            }
        }
        11 => {
            // 11: All-call reply
            let crc = modes_checksum(msg, msgbits);
            let iid = crc & 0x7f;
            let crc = crc & 0x00ff_ff80;
            let addr = getbits(msg, 9, 32) as u32;

            match (crc, iid, icao_filter_test(addr)) {
                (0, 0, true) => 1600,
                (0, 0, false) => {
                    icao_filter_add(addr);
                    750
                }
                (0, _, true) => 1000,
                (0, _, false) => -1,
                (_, _, _) => -2,
            }
        }
        17 | 18 => {
            // 17: Extended squitter
            // 18: Extended squitter/non-transponder
            let addr = getbits(msg, 9, 32) as u32;

            let crc = modes_checksum(msg, msgbits);
            match (crc, icao_filter_test(addr)) {
                (0, true) => 1800,
                (0, false) => {
                    if df == 17 {
                        icao_filter_add(addr);
                    } else {
                        icao_filter_add(addr | ICAO_FILTER_ADSB_NT);
                    }
                    1400
                }
                (_, _) => -2,
            }
        }
        16 | 20 | 21 => {
            // 16: long air-air surveillance
            // 20: Comm-B, altitude reply
            // 21: Comm-B, identity reply
            let crc = modes_checksum(msg, MODES_LONG_MSG_BYTES * 8);
            match icao_filter_test(crc) {
                true => 1000,
                false => -2,
            }
        }
        24..=31 => {
            // 24: Comm-D (ELM)
            // 25: Comm-D (ELM)
            // 26: Comm-D (ELM)
            // 27: Comm-D (ELM)
            // 28: Comm-D (ELM)
            // 29: Comm-D (ELM)
            // 30: Comm-D (ELM)
            // 31: Comm-D (ELM)
            let crc = modes_checksum(msg, MODES_LONG_MSG_BYTES * 8);
            match icao_filter_test(crc) {
                true => 1000,
                false => -2,
            }
        }
        _ => -2,
    };

    Some((msglen, res))
}
