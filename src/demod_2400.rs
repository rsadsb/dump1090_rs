// This module includes functionality translated from demod_2400.c

use crate::{
    MODES_LONG_MSG_BYTES, MODES_SHORT_MSG_BYTES, MagnitudeBuffer, mode_s::score_modes_message,
};

/// DFs worth demodulating past the first byte (dump1090-fa's default set, DF24+ off).
const VALID_DF_SHORT: u32 = (1 << 0) | (1 << 4) | (1 << 5) | (1 << 11);
const VALID_DF_LONG: u32 = (1 << 16) | (1 << 17) | (1 << 18) | (1 << 20) | (1 << 21);

#[inline(always)]
fn slice_phase0(m: &[u16]) -> i32 {
    5 * i32::from(m[0]) - 3 * i32::from(m[1]) - 2 * i32::from(m[2])
}
#[inline(always)]
fn slice_phase1(m: &[u16]) -> i32 {
    4 * i32::from(m[0]) - i32::from(m[1]) - 3 * i32::from(m[2])
}
#[inline(always)]
fn slice_phase2(m: &[u16]) -> i32 {
    3 * i32::from(m[0]) + i32::from(m[1]) - 4 * i32::from(m[2])
}
#[inline(always)]
fn slice_phase3(m: &[u16]) -> i32 {
    2 * i32::from(m[0]) + 3 * i32::from(m[1]) - 5 * i32::from(m[2])
}
#[inline(always)]
fn slice_phase4(m: &[u16]) -> i32 {
    i32::from(m[0]) + 5 * i32::from(m[1]) - 5 * i32::from(m[2]) - i32::from(m[3])
}

/// One byte (8 PPM bits, 19 or 20 samples) starting at sample phase `phase` (0..5),
/// unrolled per phase with constant offsets as in dump1090-fa's demod_2400.c.
#[inline(always)]
fn decode_byte(m: &[u16; 20], phase: u8) -> u8 {
    let b = |v: i32, bit: u8| if v > 0 { bit } else { 0 };
    match phase {
        0 => {
            b(slice_phase0(&m[0..]), 0x80)
                | b(slice_phase2(&m[2..]), 0x40)
                | b(slice_phase4(&m[4..]), 0x20)
                | b(slice_phase1(&m[7..]), 0x10)
                | b(slice_phase3(&m[9..]), 0x08)
                | b(slice_phase0(&m[12..]), 0x04)
                | b(slice_phase2(&m[14..]), 0x02)
                | b(slice_phase4(&m[16..]), 0x01)
        }
        1 => {
            b(slice_phase1(&m[0..]), 0x80)
                | b(slice_phase3(&m[2..]), 0x40)
                | b(slice_phase0(&m[5..]), 0x20)
                | b(slice_phase2(&m[7..]), 0x10)
                | b(slice_phase4(&m[9..]), 0x08)
                | b(slice_phase1(&m[12..]), 0x04)
                | b(slice_phase3(&m[14..]), 0x02)
                | b(slice_phase0(&m[17..]), 0x01)
        }
        2 => {
            b(slice_phase2(&m[0..]), 0x80)
                | b(slice_phase4(&m[2..]), 0x40)
                | b(slice_phase1(&m[5..]), 0x20)
                | b(slice_phase3(&m[7..]), 0x10)
                | b(slice_phase0(&m[10..]), 0x08)
                | b(slice_phase2(&m[12..]), 0x04)
                | b(slice_phase4(&m[14..]), 0x02)
                | b(slice_phase1(&m[17..]), 0x01)
        }
        3 => {
            b(slice_phase3(&m[0..]), 0x80)
                | b(slice_phase0(&m[3..]), 0x40)
                | b(slice_phase2(&m[5..]), 0x20)
                | b(slice_phase4(&m[7..]), 0x10)
                | b(slice_phase1(&m[10..]), 0x08)
                | b(slice_phase3(&m[12..]), 0x04)
                | b(slice_phase0(&m[15..]), 0x02)
                | b(slice_phase2(&m[17..]), 0x01)
        }
        _ => {
            b(slice_phase4(&m[0..]), 0x80)
                | b(slice_phase1(&m[3..]), 0x40)
                | b(slice_phase3(&m[5..]), 0x20)
                | b(slice_phase0(&m[8..]), 0x10)
                | b(slice_phase2(&m[10..]), 0x08)
                | b(slice_phase4(&m[12..]), 0x04)
                | b(slice_phase1(&m[15..]), 0x02)
                | b(slice_phase3(&m[17..]), 0x01)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MsgLen {
    Short,
    Long,
}

#[derive(Debug)]
pub struct ModeSMessage {
    /// Type of message
    msglen: MsgLen,
    /// Binary message
    msg: [u8; MODES_LONG_MSG_BYTES],
    ///  RSSI, in the range [0..1], as a fraction of full-scale power
    signal_level: f64,
    /// Scoring from scoreModesMessage, if used
    score: i32,
}

impl ModeSMessage {
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        match self.msglen {
            MsgLen::Short => &self.msg[..MODES_SHORT_MSG_BYTES],
            MsgLen::Long => &self.msg[..MODES_LONG_MSG_BYTES],
        }
    }
}

#[inline(always)]
pub fn demodulate2400(mag: &MagnitudeBuffer) -> Result<Vec<ModeSMessage>, &'static str> {
    let mut results = vec![];

    let data = &mag.data[..];

    // Keep this per-sample loop tiny: with the candidate code inlined here, LLVM kept ~30
    // strength-reduced `j + k` indices alive and spilled them to the stack, updating every
    // one of them for every sample.
    let mut j = 0;
    while j < mag.length {
        // quick check: we must have a rising edge 0->1 and a falling edge 12->13
        let p = &data[j..j + 14];
        if p[0] < p[1]
            && p[12] > p[13]
            && let Some(msg) = demodulate_candidate(data, j)
        {
            // Skip over the message, to 8 bits before its end (as dump1090-fa does: the
            // preamble of a following message can clobber the last bits of this one).
            let msgbits = msg.buffer().len() * 8;
            j += (msgbits + 8) * 12 / 5 - 8 * 12 / 5;
            results.push(msg);
        }
        j += 1;
    }

    Ok(results)
}

/// Full preamble, signal and quiet-bit checks for a possible message starting at sample `j`,
/// then all five phases. Rare (a few % of samples), so kept out of line.
#[inline(never)]
fn demodulate_candidate(data: &[u16], j: usize) -> Option<ModeSMessage> {
    let (high, base_signal, base_noise) = check_preamble(&data[j..j + 14])?;
    // Check for enough signal
    if base_signal * 2 < 3 * base_noise {
        // about 3.5dB SNR
        return None;
    }

    // Check that the "quiet" bits 6,7,15,16,17 are actually quiet
    let q = &data[j..j + 19];
    if [5, 6, 7, 8, 14, 15, 16, 17, 18].iter().any(|&k| i32::from(q[k]) >= high) {
        return None;
    }

    // Try all phases
    let mut bestmsg = ModeSMessage {
        msg: [0_u8; MODES_LONG_MSG_BYTES],
        signal_level: 0.,
        score: -2,
        msglen: MsgLen::Short,
    };

    let mut msg: [u8; MODES_LONG_MSG_BYTES] = [0_u8; MODES_LONG_MSG_BYTES];

    for try_phase in 4..9 {
        let mut slice_loc: usize = j + 19 + (try_phase / 5);
        let mut phase = (try_phase % 5) as u8;

        // Decode byte 0 first and only continue for DFs we can use, as dump1090-fa
        // does: for noise this ends most trials after one byte instead of fourteen.
        let mut bytelen = 1;
        let mut i = 0;
        while i < bytelen {
            let window: &[u16; 20] = data[slice_loc..slice_loc + 20].try_into().unwrap();
            let the_byte = decode_byte(window, phase);
            msg[i] = the_byte;
            slice_loc += if phase == 4 { 20 } else { 19 };
            phase = if phase == 4 { 0 } else { phase + 1 };
            if i == 0 {
                let df = the_byte >> 3;
                if VALID_DF_LONG & (1 << df) != 0 {
                    bytelen = MODES_LONG_MSG_BYTES;
                } else if VALID_DF_SHORT & (1 << df) != 0 {
                    bytelen = MODES_SHORT_MSG_BYTES;
                }
            }
            i += 1;
        }
        if bytelen == 1 {
            continue;
        }
        // Keep the unused tail of a short message from an earlier phase out of scoring.
        msg[bytelen..].fill(0);

        if let Some((msglen, score)) = score_modes_message(&msg)
            && score > bestmsg.score
        {
            bestmsg.msglen = msglen;
            bestmsg.msg.clone_from_slice(&msg);
            bestmsg.score = score;

            let signal_len = msg.len() * 12 / 5;
            let scaled_signal_power: u64 = data[j + 19..j + 19 + signal_len]
                .iter()
                .map(|&m| u64::from(m) * u64::from(m))
                .sum();
            let signal_power = scaled_signal_power as f64 / 65535.0 / 65535.0;
            bestmsg.signal_level = signal_power / signal_len as f64;
        }
    }

    // Do we have a candidate?
    (bestmsg.score >= 0).then_some(bestmsg)
}

#[inline(always)]
fn check_preamble(preamble: &[u16]) -> Option<(i32, u32, u32)> {
    // This gets rid of the 3 core::panicking::panic_bounds_check calls,
    // but doesn't look to improve performance
    assert!(preamble.len() == 14);

    // quick check: we must have a rising edge 0->1 and a falling edge 12->13
    if !(preamble[0] < preamble[1] && preamble[12] > preamble[13]) {
        return None;
    }

    // check the rising and falling edges of signal
    if preamble[1] > preamble[2] &&                                       // 1
       preamble[2] < preamble[3] && preamble[3] > preamble[4] &&          // 3
       preamble[8] < preamble[9] && preamble[9] > preamble[10] &&         // 9
       preamble[10] < preamble[11]
    {
        // 11-12
        // peaks at 1,3,9,11-12: phase 3
        let high = (i32::from(preamble[1])
            + i32::from(preamble[3])
            + i32::from(preamble[9])
            + i32::from(preamble[11])
            + i32::from(preamble[12]))
            / 4;
        let base_signal = u32::from(preamble[1]) + u32::from(preamble[3]) + u32::from(preamble[9]);
        let base_noise = u32::from(preamble[5]) + u32::from(preamble[6]) + u32::from(preamble[7]);
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                // 1
              preamble[2] < preamble[3] && preamble[3] > preamble[4] &&   // 3
              preamble[8] < preamble[9] && preamble[9] > preamble[10] &&  // 9
              preamble[11] < preamble[12]
    {
        // 12
        // peaks at 1,3,9,12: phase 4
        let high = (i32::from(preamble[1])
            + i32::from(preamble[3])
            + i32::from(preamble[9])
            + i32::from(preamble[12]))
            / 4;
        let base_signal = u32::from(preamble[1])
            + u32::from(preamble[3])
            + u32::from(preamble[9])
            + u32::from(preamble[12]);
        let base_noise = u32::from(preamble[5])
            + u32::from(preamble[6])
            + u32::from(preamble[7])
            + u32::from(preamble[8]);
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                // 1
              preamble[2] < preamble[3] && preamble[4] > preamble[5] &&   // 3-4
              preamble[8] < preamble[9] && preamble[10] > preamble[11] && // 9-10
              preamble[11] < preamble[12]
    {
        // 12
        // peaks at 1,3-4,9-10,12: phase 5
        let high = (i32::from(preamble[1])
            + i32::from(preamble[3])
            + i32::from(preamble[4])
            + i32::from(preamble[9])
            + i32::from(preamble[10])
            + i32::from(preamble[12]))
            / 4;
        let base_signal = u32::from(preamble[1]) + u32::from(preamble[12]);
        let base_noise = u32::from(preamble[6]) + u32::from(preamble[7]);
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                 // 1
              preamble[3] < preamble[4] && preamble[4] > preamble[5] &&    // 4
              preamble[9] < preamble[10] && preamble[10] > preamble[11] && // 10
              preamble[11] < preamble[12]
    {
        // 12
        // peaks at 1,4,10,12: phase 6
        let high = (i32::from(preamble[1])
            + i32::from(preamble[4])
            + i32::from(preamble[10])
            + i32::from(preamble[12]))
            / 4;
        let base_signal = u32::from(preamble[1])
            + u32::from(preamble[4])
            + u32::from(preamble[10])
            + u32::from(preamble[12]);
        let base_noise = u32::from(preamble[5])
            + u32::from(preamble[6])
            + u32::from(preamble[7])
            + u32::from(preamble[8]);
        Some((high, base_signal, base_noise))
    } else if preamble[2] > preamble[3] &&                                 // 1-2
              preamble[3] < preamble[4] && preamble[4] > preamble[5] &&    // 4
              preamble[9] < preamble[10] && preamble[10] > preamble[11] && // 10
              preamble[11] < preamble[12]
    {
        // 12
        // peaks at 1-2,4,10,12: phase 7
        let high = (i32::from(preamble[1])
            + i32::from(preamble[2])
            + i32::from(preamble[4])
            + i32::from(preamble[10])
            + i32::from(preamble[12]))
            / 4;
        let base_signal =
            u32::from(preamble[4]) + u32::from(preamble[10]) + u32::from(preamble[12]);
        let base_noise = u32::from(preamble[6]) + u32::from(preamble[7]) + u32::from(preamble[8]);
        Some((high, base_signal, base_noise))
    } else {
        None
    }
}
