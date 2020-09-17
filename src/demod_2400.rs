
// This module includes functionality translated from demod_2400.c

use crate::{MagnitudeBuffer, ModeSMessage, MODES_LONG_MSG_BYTES};
use crate::mode_s;

fn slice_phase0(m:&[u16]) -> i32 {
    5 * (m[0] as i32) - 3 * (m[1] as i32) - 2 * (m[2] as i32)
}

fn slice_phase1(m:&[u16]) -> i32 {
    4 * (m[0] as i32) - (m[1] as i32) - 3 * (m[2] as i32)
}

fn slice_phase2(m:&[u16]) -> i32 {
    3 * (m[0] as i32) + (m[1] as i32) - 4 * (m[2] as i32)
}

fn slice_phase3(m:&[u16]) -> i32 {
    2 * (m[0] as i32) + 3 * (m[1] as i32) - 5 * (m[2] as i32)
}

fn slice_phase4(m:&[u16]) -> i32 {
    (m[0] as i32) + 5 * (m[1] as i32) - 5 * (m[2] as i32) - (m[3] as i32)
}

pub fn demodulate2400(mag:&MagnitudeBuffer, fs:usize) -> Result<(), &'static str> {

    let data = &mag.data;

	if fs != 2400000 { return Err("2.4e6 [samples/sec] is the only sample rate supported"); }

    let mut skip_count:usize = 0;
	'jloop: for j in 0..mag.length {

        if skip_count > 0 {
            skip_count -= 1;
            continue 'jloop;
        }

        let preamble:&[u16] = &data[j..];

		if let Some((high, base_signal, base_noise)) = check_preamble(preamble) {

	        // Check for enough signal
	        if base_signal * 2 < 3 * base_noise {  // about 3.5dB SNR
	            continue 'jloop;
	        }		

	        // Check that the "quiet" bits 6,7,15,16,17 are actually quiet
	        if data[j+5] as i32 >= high ||
	           data[j+6] as i32 >= high ||
	           data[j+7] as i32 >= high ||
	           data[j+8] as i32 >= high ||
	           data[j+14] as i32 >= high ||
	           data[j+15] as i32 >= high ||
	           data[j+16] as i32 >= high ||
	           data[j+17] as i32 >= high ||
	           data[j+18] as i32 >= high {
	            continue 'jloop;
	        }

	        // Try all phases
            let mut bestmsg:[u8; MODES_LONG_MSG_BYTES] = [0u8; MODES_LONG_MSG_BYTES];
            let mut bestscore:i32 = -2;
            let mut bestphase:usize = 0;

            let mut msg:[u8; MODES_LONG_MSG_BYTES] = [0u8; MODES_LONG_MSG_BYTES];
			for try_phase in 4..9 {

				let mut slice_loc:usize = j+19+(try_phase/5);
				let mut phase:usize = try_phase % 5;

				for k in 0..(MODES_LONG_MSG_BYTES) {

					let slice_this_byte:&[u16] = &data[slice_loc..];

					let (next_slice_loc, next_phase, the_byte) = match phase {
						0 => {
                            let mut the_byte = if slice_phase0( slice_this_byte)       > 0 { 0x80 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[ 2..]) > 0 { 0x40 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[ 4..]) > 0 { 0x20 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[ 7..]) > 0 { 0x10 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[ 9..]) > 0 { 0x08 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[12..]) > 0 { 0x04 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[14..]) > 0 { 0x02 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[16..]) > 0 { 0x01 } else { 0x00 };

                            (slice_loc + 19, 1, the_byte)
                        },
                        1 => {
                            let mut the_byte = if slice_phase1( slice_this_byte)       > 0 { 0x80 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[ 2..]) > 0 { 0x40 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[ 5..]) > 0 { 0x20 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[ 7..]) > 0 { 0x10 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[ 9..]) > 0 { 0x08 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[12..]) > 0 { 0x04 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[14..]) > 0 { 0x02 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[17..]) > 0 { 0x01 } else { 0x00 };

                            (slice_loc + 19, 2, the_byte)
                        },
                        2 => {
                            let mut the_byte = if slice_phase2( slice_this_byte)       > 0 { 0x80 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[ 2..]) > 0 { 0x40 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[ 5..]) > 0 { 0x20 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[ 7..]) > 0 { 0x10 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[10..]) > 0 { 0x08 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[12..]) > 0 { 0x04 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[14..]) > 0 { 0x02 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[17..]) > 0 { 0x01 } else { 0x00 };

                            (slice_loc + 19, 3, the_byte)
                        },
                        3 => {
                            let mut the_byte = if slice_phase3( slice_this_byte)       > 0 { 0x80 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[ 3..]) > 0 { 0x40 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[ 5..]) > 0 { 0x20 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[ 7..]) > 0 { 0x10 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[10..]) > 0 { 0x08 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[12..]) > 0 { 0x04 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[15..]) > 0 { 0x02 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[17..]) > 0 { 0x01 } else { 0x00 };

                            (slice_loc + 19, 4, the_byte)
                        },
                        4 => {
                            let mut the_byte = if slice_phase4( slice_this_byte)       > 0 { 0x80 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[ 3..]) > 0 { 0x40 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[ 5..]) > 0 { 0x20 } else { 0x00 };
                            the_byte |=        if slice_phase0(&slice_this_byte[ 8..]) > 0 { 0x10 } else { 0x00 };
                            the_byte |=        if slice_phase2(&slice_this_byte[10..]) > 0 { 0x08 } else { 0x00 };
                            the_byte |=        if slice_phase4(&slice_this_byte[12..]) > 0 { 0x04 } else { 0x00 };
                            the_byte |=        if slice_phase1(&slice_this_byte[15..]) > 0 { 0x02 } else { 0x00 };
                            the_byte |=        if slice_phase3(&slice_this_byte[17..]) > 0 { 0x01 } else { 0x00 };

                            (slice_loc + 20, 0, the_byte)
                        },
                        _ => panic!("Unexpected phase value")
						
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

            let msglen = mode_s::modes_message_len_by_type(bestmsg[0] >> 3);

            let mut mm = ModeSMessage::default();
            mm.msg = bestmsg.to_vec();
            mm.timestamp_msg = mag.first_sample_timestamp_12mhz + (j*5) + bestphase;
            mm.score = bestscore;

            // println!("\nt={:.3}, Power={:8.1}, j={:6}, bestscore={:4}", 
            //     mm.timestamp_msg as f32 / 12.0e6, mag.total_power, j, bestscore);

            // Decode the received message
            if let Err(_) = mode_s::decode_mode_s_message(&mut mm) {
                continue 'jloop;
            }

            // Measure signal power
            {
                let mut scaled_signal_power:usize = 0;
                let signal_len:usize = (msglen*12)/5;

                for k in 0..signal_len {
                    let mag:usize = data[j+19+k] as usize;
                    scaled_signal_power += mag * mag;
                }

                let signal_power = scaled_signal_power as f64 / 65535.0 / 65535.0;
                mm.signal_level = signal_power / (signal_len as f64);

                // Modes.stats_current.signal_power_sum += signal_power;
                // Modes.stats_current.signal_power_count += signal_len;
                // sum_scaled_signal_power += scaled_signal_power;

                // if (mm.signalLevel > Modes.stats_current.peak_signal_power)
                //     Modes.stats_current.peak_signal_power = mm.signalLevel;
                // if (mm.signalLevel > 0.50119)
                //     Modes.stats_current.strong_signal_count++; // signal power above -3dBFS

            }

            // Skip over the message:
            // (we actually skip to 8 bits before the end of the message,
            //  because we can often decode two messages that *almost* collide,
            //  where the preamble of the second message clobbered the last
            //  few bits of the first message, but the message bits didn't
            //  overlap)
            skip_count = (msglen*12)/5;

            mode_s::use_mode_s_message(&mut mm);
            eprintln!("");
		}

	}

	Ok(())
}

fn check_preamble(preamble:&[u16]) -> Option<(i32, u32, u32)> {

    // quick check: we must have a rising edge 0->1 and a falling edge 12->13
    if !(preamble[0] < preamble[1] && preamble[12] > preamble[13]) {
       return None;
    }

    if preamble[1] > preamble[2] &&                                       // 1
       preamble[2] < preamble[3] && preamble[3] > preamble[4] &&          // 3
       preamble[8] < preamble[9] && preamble[9] > preamble[10] &&         // 9
       preamble[10] < preamble[11] {                                      // 11-12
        // peaks at 1,3,9,11-12: phase 3
        let high = (preamble[1] as i32 + preamble[3] as i32 + preamble[9] as i32 + preamble[11] as i32 + preamble[12] as i32) / 4;
        let base_signal = preamble[1] as u32 + preamble[3] as u32 + preamble[9] as u32;
        let base_noise  = preamble[5] as u32 + preamble[6] as u32 + preamble[7] as u32;
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                // 1
              preamble[2] < preamble[3] && preamble[3] > preamble[4] &&   // 3
              preamble[8] < preamble[9] && preamble[9] > preamble[10] &&  // 9
              preamble[11] < preamble[12] {                               // 12
        // peaks at 1,3,9,12: phase 4
        let high = (preamble[1] as i32 + preamble[3] as i32 + preamble[9] as i32 + preamble[12] as i32) / 4;
        let base_signal = preamble[1] as u32 + preamble[3] as u32 + preamble[9] as u32 + preamble[12] as u32;
        let base_noise  = preamble[5] as u32 + preamble[6] as u32 + preamble[7] as u32 + preamble[8] as u32;
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                // 1
              preamble[2] < preamble[3] && preamble[4] > preamble[5] &&   // 3-4
              preamble[8] < preamble[9] && preamble[10] > preamble[11] && // 9-10
              preamble[11] < preamble[12] {                               // 12
        // peaks at 1,3-4,9-10,12: phase 5
        let high = (preamble[1] as i32 + preamble[3] as i32 + preamble[4] as i32 + preamble[9] as i32 + preamble[10] as i32 + preamble[12] as i32) / 4;
        let base_signal = preamble[1] as u32 + preamble[12] as u32;
        let base_noise  = preamble[6] as u32 + preamble[7] as u32;
        Some((high, base_signal, base_noise))
    } else if preamble[1] > preamble[2] &&                                 // 1
              preamble[3] < preamble[4] && preamble[4] > preamble[5] &&    // 4
              preamble[9] < preamble[10] && preamble[10] > preamble[11] && // 10
              preamble[11] < preamble[12] {                                // 12
        // peaks at 1,4,10,12: phase 6
        let high = (preamble[1] as i32 + preamble[4] as i32 + preamble[10] as i32 + preamble[12] as i32) / 4;
        let base_signal = preamble[1] as u32 + preamble[4] as u32 + preamble[10] as u32 + preamble[12] as u32;
        let base_noise  = preamble[5] as u32 + preamble[6] as u32 + preamble[7] as u32  + preamble[8] as u32;
        Some((high, base_signal, base_noise))
    } else if preamble[2] > preamble[3] &&                                 // 1-2
              preamble[3] < preamble[4] && preamble[4] > preamble[5] &&    // 4
              preamble[9] < preamble[10] && preamble[10] > preamble[11] && // 10
              preamble[11] < preamble[12] {                                // 12
        // peaks at 1-2,4,10,12: phase 7
        let high = (preamble[1] as i32 + preamble[2] as i32 + preamble[4] as i32 + preamble[10] as i32 + preamble[12] as i32) / 4;
        let base_signal = preamble[4] as u32 + preamble[10] as u32 + preamble[12] as u32;
        let base_noise  = preamble[6] as u32 + preamble[7] as u32  + preamble[8] as u32;
        Some((high, base_signal, base_noise))
    } else { 
    	None 
    }

}