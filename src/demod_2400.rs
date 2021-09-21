// This module includes functionality translated from demod_2400.c

use crate::mode_s;
use crate::{MagnitudeBuffer, MODES_LONG_MSG_BYTES};

fn slice_phase0(m: &[u16]) -> i32 {
    5 * i32::from(m[0]) - 3 * i32::from(m[1]) - 2 * i32::from(m[2])
}

fn slice_phase1(m: &[u16]) -> i32 {
    4 * i32::from(m[0]) - i32::from(m[1]) - 3 * i32::from(m[2])
}

fn slice_phase2(m: &[u16]) -> i32 {
    3 * i32::from(m[0]) + i32::from(m[1]) - 4 * i32::from(m[2])
}

fn slice_phase3(m: &[u16]) -> i32 {
    2 * i32::from(m[0]) + 3 * i32::from(m[1]) - 5 * i32::from(m[2])
}

fn slice_phase4(m: &[u16]) -> i32 {
    i32::from(m[0]) + 5 * i32::from(m[1]) - 5 * i32::from(m[2]) - i32::from(m[3])
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
            let mut bestphase: usize = 0;

            let mut msg: [u8; MODES_LONG_MSG_BYTES] = [0_u8; MODES_LONG_MSG_BYTES];
            for try_phase in 4..9 {
                let mut slice_loc: usize = j + 19 + (try_phase / 5);
                let mut phase: usize = try_phase % 5;

                for k in 0..(MODES_LONG_MSG_BYTES) {
                    let slice_this_byte: &[u16] = &data[slice_loc..];

                    let (next_slice_loc, next_phase, the_byte) = match phase {
                        0 => {
                            let mut the_byte = if slice_phase0(slice_this_byte) > 0 {
                                0x80
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[2..]) > 0 {
                                0x40
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[4..]) > 0 {
                                0x20
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[7..]) > 0 {
                                0x10
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[9..]) > 0 {
                                0x08
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[12..]) > 0 {
                                0x04
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[14..]) > 0 {
                                0x02
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[16..]) > 0 {
                                0x01
                            } else {
                                0x00
                            };

                            (slice_loc + 19, 1, the_byte)
                        }
                        1 => {
                            let mut the_byte = if slice_phase1(slice_this_byte) > 0 {
                                0x80
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[2..]) > 0 {
                                0x40
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[5..]) > 0 {
                                0x20
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[7..]) > 0 {
                                0x10
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[9..]) > 0 {
                                0x08
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[12..]) > 0 {
                                0x04
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[14..]) > 0 {
                                0x02
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[17..]) > 0 {
                                0x01
                            } else {
                                0x00
                            };

                            (slice_loc + 19, 2, the_byte)
                        }
                        2 => {
                            let mut the_byte = if slice_phase2(slice_this_byte) > 0 {
                                0x80
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[2..]) > 0 {
                                0x40
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[5..]) > 0 {
                                0x20
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[7..]) > 0 {
                                0x10
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[10..]) > 0 {
                                0x08
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[12..]) > 0 {
                                0x04
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[14..]) > 0 {
                                0x02
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[17..]) > 0 {
                                0x01
                            } else {
                                0x00
                            };

                            (slice_loc + 19, 3, the_byte)
                        }
                        3 => {
                            let mut the_byte = if slice_phase3(slice_this_byte) > 0 {
                                0x80
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[3..]) > 0 {
                                0x40
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[5..]) > 0 {
                                0x20
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[7..]) > 0 {
                                0x10
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[10..]) > 0 {
                                0x08
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[12..]) > 0 {
                                0x04
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[15..]) > 0 {
                                0x02
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[17..]) > 0 {
                                0x01
                            } else {
                                0x00
                            };

                            (slice_loc + 19, 4, the_byte)
                        }
                        4 => {
                            let mut the_byte = if slice_phase4(slice_this_byte) > 0 {
                                0x80
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[3..]) > 0 {
                                0x40
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[5..]) > 0 {
                                0x20
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase0(&slice_this_byte[8..]) > 0 {
                                0x10
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase2(&slice_this_byte[10..]) > 0 {
                                0x08
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase4(&slice_this_byte[12..]) > 0 {
                                0x04
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase1(&slice_this_byte[15..]) > 0 {
                                0x02
                            } else {
                                0x00
                            };
                            the_byte |= if slice_phase3(&slice_this_byte[17..]) > 0 {
                                0x01
                            } else {
                                0x00
                            };

                            (slice_loc + 20, 0, the_byte)
                        }
                        _ => panic!("Unexpected phase value"),
                    };

                    msg[k] = the_byte;
                    slice_loc = next_slice_loc;
                    phase = next_phase;
                }

                let score = mode_s::score_modes_message(&msg);

                if score > bestscore {
                    bestmsg.clone_from_slice(&msg);
                    bestscore = score;
                    bestphase = try_phase;
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
