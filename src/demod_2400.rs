// This module includes functionality translated from demod_2400.c

use crate::{
    mode_s::score_modes_message,
    crc::fix_single_bit_error,
    MagnitudeBuffer, MODES_LONG_MSG_BYTES, MODES_SHORT_MSG_BYTES,
};

#[derive(Clone, Copy, Debug)]
enum Phase {
    /// 0|2|4|1|3|0|2|4 -> One
    Zero,
    /// 1|3|0|2|4|1|3|0 -> Two
    One,
    /// 2|4|1|3|0|2|4|1 -> Three
    Two,
    /// 3|0|2|4|1|3|0|2 -> Four
    Three,
    /// 4|1|3|0|2|4|1|3 -> Zero
    Four,
}

impl From<usize> for Phase {
    #[inline(always)]
    fn from(num: usize) -> Self {
        match num % 5 {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => unimplemented!(),
        }
    }
}

impl Phase {
    /// Increment from 0..4 for incrementing the starting phase
    #[inline(always)]
    fn next_start(self) -> Self {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Zero,
        }
    }

    /// Increment by expected next phase transition for bit denoting
    #[inline(always)]
    fn next(self) -> Self {
        match self {
            Self::Zero => Self::Two,
            Self::Two => Self::Four,
            Self::Four => Self::One,
            Self::One => Self::Three,
            Self::Three => Self::Zero,
        }
    }

    /// Amount of mag indexs used, for adding to the next start index
    #[inline(always)]
    fn increment_index(self, index: usize) -> usize {
        index
            + match self {
                Self::Zero | Self::Two | Self::One => 2,
                Self::Four | Self::Three => 3,
            }
    }

    /// Calculate the PPM bit
    ///
    /// Coefficients updated 2020 by wiedehopf (readsb) - hand-tuned on real samples
    /// for better weak-signal performance. See readsb demod_2400.c lines 74-93.
    /// Note: phase2 is slightly DC unbalanced but produces better results.
    #[inline(always)]
    fn calculate_bit(self, m: &[u16]) -> i32 {
        let m0 = i32::from(m[0]);
        let m1 = i32::from(m[1]);
        let m2 = i32::from(m[2]);
        match self {
            Self::Zero => 18 * m0 - 15 * m1 - 3 * m2,
            Self::One => 14 * m0 - 5 * m1 - 9 * m2,
            Self::Two => 16 * m0 + 5 * m1 - 20 * m2,
            Self::Three => 7 * m0 + 11 * m1 - 18 * m2,
            Self::Four => 4 * m0 + 15 * m1 - 20 * m2 + i32::from(m[3]),
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
    // Pre-allocate capacity for typical message count (reduces reallocations)
    let mut results = Vec::with_capacity(64);

    let data = &mag.data;

    let mut skip_count: usize = 0;
    'jloop: for j in 0..mag.length {
        if skip_count > 0 {
            skip_count -= 1;
            continue 'jloop;
        }

        if let Some((high, base_signal, base_noise)) = check_preamble(&data[j..j + 14]) {
            // Check for enough signal
            if base_signal * 2 < 3 * base_noise {
                // about 3.5dB SNR
                continue 'jloop;
            }

            // Check that the "quiet" bits 6,7,15,16,17 are actually quiet
            // Safety: j < mag.length <= MODES_MAG_BUF_SAMPLES (131072),
            // and data.len() = TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES (131398),
            // so j+18 < 131090 which is always < 131398. Bounds are guaranteed.
            unsafe {
                if i32::from(*data.get_unchecked(j + 5)) >= high
                    || i32::from(*data.get_unchecked(j + 6)) >= high
                    || i32::from(*data.get_unchecked(j + 7)) >= high
                    || i32::from(*data.get_unchecked(j + 8)) >= high
                    || i32::from(*data.get_unchecked(j + 14)) >= high
                    || i32::from(*data.get_unchecked(j + 15)) >= high
                    || i32::from(*data.get_unchecked(j + 16)) >= high
                    || i32::from(*data.get_unchecked(j + 17)) >= high
                    || i32::from(*data.get_unchecked(j + 18)) >= high
                {
                    continue 'jloop;
                }
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
                let mut phase = Phase::from(try_phase);

                for msg in msg.iter_mut().take(MODES_LONG_MSG_BYTES) {
                    let slice_this_byte: &[u16] = &data[slice_loc..];

                    let starting_phase = phase;
                    let mut the_byte = 0x00;
                    let mut index = 0;
                    // for each phase-bit
                    for i in 0..8 {
                        // find if phase distance denotes a high bit
                        if phase.calculate_bit(&slice_this_byte[index..index + 4]) > 0 {
                            the_byte |= 1 << (7 - i);
                        }
                        // increment to next phase, increase index
                        index = phase.increment_index(index);
                        phase = phase.next();
                    }
                    // save bytes and move the next starting phase
                    *msg = the_byte;
                    slice_loc += index;
                    phase = starting_phase.next_start();
                }

                if let Some((msglen, score)) = score_modes_message(&msg) {
                    if score > bestmsg.score {
                        bestmsg.msglen = msglen;
                        bestmsg.msg.clone_from_slice(&msg);
                        bestmsg.score = score;

                        let mut scaled_signal_power = 0_u64;
                        let signal_len = msg.len() * 12 / 5;
                        for k in 0..signal_len {
                            let mag = data[j + 19 + k] as u64;
                            scaled_signal_power += mag * mag;
                        }
                        let signal_power = scaled_signal_power as f64 / 65535.0 / 65535.0;
                        bestmsg.signal_level = signal_power / signal_len as f64;
                    }
                }
            }

            // Do we have a candidate?
            if bestmsg.score < 0 {
                continue 'jloop;
            }

            // Try to recover messages with single-bit errors if CRC failed but we have data
            if bestmsg.score < 1000 {
                let msgbits = match bestmsg.msglen {
                    MsgLen::Short => MODES_SHORT_MSG_BYTES * 8,
                    MsgLen::Long => MODES_LONG_MSG_BYTES * 8,
                };

                let msg_slice = match bestmsg.msglen {
                    MsgLen::Short => &mut bestmsg.msg[..MODES_SHORT_MSG_BYTES],
                    MsgLen::Long => &mut bestmsg.msg[..MODES_LONG_MSG_BYTES],
                };

                if fix_single_bit_error(msg_slice, msgbits).is_some() {
                    // Successfully corrected! Re-score the message
                    if let Some((_, new_score)) = score_modes_message(msg_slice) {
                        if new_score > bestmsg.score {
                            bestmsg.score = new_score;

                            // Recalculate signal_level if it wasn't set yet (all phases failed initially)
                            if bestmsg.signal_level == 0.0 {
                                let mut scaled_signal_power = 0_u64;
                                let signal_len = msg_slice.len() * 12 / 5;
                                for k in 0..signal_len {
                                    let mag = data[j + 19 + k] as u64;
                                    scaled_signal_power += mag * mag;
                                }
                                let signal_power = scaled_signal_power as f64 / 65535.0 / 65535.0;
                                bestmsg.signal_level = signal_power / signal_len as f64;
                            }
                        }
                    }
                }
            }

            // Final check - only push if we have a valid score
            if bestmsg.score >= 0 {
                results.push(bestmsg);
            }
        }
    }

    Ok(results)
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
