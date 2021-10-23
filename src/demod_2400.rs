// This module includes functionality translated from demod_2400.c

use crate::mode_s;
use crate::{MagnitudeBuffer, MODES_LONG_MSG_BYTES};

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
    fn increment_index(self, index: usize) -> usize {
        index
            + match self {
                Self::Zero | Self::Two | Self::One => 2,
                Self::Four | Self::Three => 3,
            }
    }

    /// Calculate the PPM bit
    fn calculate_bit(self, m: &[u16]) -> i32 {
        let m0 = i32::from(m[0]);
        let m1 = i32::from(m[1]);
        let m2 = i32::from(m[2]);
        match self {
            Self::Zero => 5 * m0 - 3 * m1 - 2 * m2,
            Self::One => 4 * m0 - m1 - 3 * m2,
            Self::Two => 3 * m0 + m1 - 4 * m2,
            Self::Three => 2 * m0 + 3 * m1 - 5 * m2,
            Self::Four => m0 + 5 * m1 - 5 * m2 - i32::from(m[3]),
        }
    }
}

pub fn demodulate2400(mag: &MagnitudeBuffer) -> Result<Vec<[u8; 14]>, &'static str> {
    let mut results = vec![];

    let data = &mag.data;

    let mut skip_count: usize = 0;
    'jloop: for j in 0..mag.length {
        if skip_count > 0 {
            skip_count -= 1;
            continue 'jloop;
        }

        let preamble: &[u16] = &data[j..];

        if let Some((high, base_signal, base_noise)) = check_preamble(preamble) {
            // Check for enough signal
            if base_signal * 2 < 3 * base_noise {
                // about 3.5dB SNR
                continue 'jloop;
            }

            // Check that the "quiet" bits 6,7,15,16,17 are actually quiet
            if i32::from(data[j + 5]) >= high
                || i32::from(data[j + 6]) >= high
                || i32::from(data[j + 7]) >= high
                || i32::from(data[j + 8]) >= high
                || i32::from(data[j + 14]) >= high
                || i32::from(data[j + 15]) >= high
                || i32::from(data[j + 16]) >= high
                || i32::from(data[j + 17]) >= high
                || i32::from(data[j + 18]) >= high
            {
                continue 'jloop;
            }

            // Try all phases
            let mut bestmsg: [u8; MODES_LONG_MSG_BYTES] = [0_u8; MODES_LONG_MSG_BYTES];
            let mut bestscore: i32 = -2;

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
                        if phase.calculate_bit(&slice_this_byte[index..]) > 0 {
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

                let score = mode_s::score_modes_message(&msg);

                if score > bestscore {
                    bestmsg.clone_from_slice(&msg);
                    bestscore = score;
                }
            }

            // Do we have a candidate?
            if bestscore < 0 {
                continue 'jloop;
            }
            results.push(bestmsg);
        }
    }

    Ok(results)
}

fn check_preamble(preamble: &[u16]) -> Option<(i32, u32, u32)> {
    // quick check: we must have a rising edge 0->1 and a falling edge 12->13
    if !(preamble[0] < preamble[1] && preamble[12] > preamble[13]) {
        return None;
    }

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
