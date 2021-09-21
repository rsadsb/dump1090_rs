use crate::{
    AddrType, AirGround, AltitudeSource, AltitudeUnit, AngleType, CprType, DataSource,
    HeadingSource, ModeSMessage, OperationalStatus, SilType, SpeedSource, TargetStateStatus,
    TssAltitudeType, MODES_NON_ICAO_ADDRESS,
};

use super::{getbit, getbits, MAGIC_MLAT_TIMESTAMP};

pub const AIS_CHARSET: &str = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";

// mode_s.c::96
fn decode_id13_field(id13_field: u32) -> u32 {
    let mut hex_gillham: u32 = 0;

    if id13_field & 0x1000 != 0 {
        hex_gillham |= 0x0010;
    } // Bit 12 = C1
    if id13_field & 0x0800 != 0 {
        hex_gillham |= 0x1000;
    } // Bit 11 = A1
    if id13_field & 0x0400 != 0 {
        hex_gillham |= 0x0020;
    } // Bit 10 = C2
    if id13_field & 0x0200 != 0 {
        hex_gillham |= 0x2000;
    } // Bit  9 = A2
    if id13_field & 0x0100 != 0 {
        hex_gillham |= 0x0040;
    } // Bit  8 = C4
    if id13_field & 0x0080 != 0 {
        hex_gillham |= 0x4000;
    } // Bit  7 = A4
      //if id13_field & 0x0040 != 0 {hex_gillham |= 0x0800;} // Bit  6 = X  or M
    if id13_field & 0x0020 != 0 {
        hex_gillham |= 0x0100;
    } // Bit  5 = B1
    if id13_field & 0x0010 != 0 {
        hex_gillham |= 0x0001;
    } // Bit  4 = D1 or Q
    if id13_field & 0x0008 != 0 {
        hex_gillham |= 0x0200;
    } // Bit  3 = B2
    if id13_field & 0x0004 != 0 {
        hex_gillham |= 0x0002;
    } // Bit  2 = D2
    if id13_field & 0x0002 != 0 {
        hex_gillham |= 0x0400;
    } // Bit  1 = B4
    if id13_field & 0x0001 != 0 {
        hex_gillham |= 0x0004;
    } // Bit  0 = D4

    hex_gillham
}

// mode_s.c:122
fn decode_ac13_field(ac13_field: u32) -> Result<(i32, AltitudeUnit), &'static str> {
    let m_bit: u32 = ac13_field & 0x0040; // set = meters, clear = feet
    let q_bit: u32 = ac13_field & 0x0010; // set = 25 ft encoding, clear = Gillham Mode C encoding

    if m_bit == 0 {
        let altitude: i32 = if q_bit != 0 {
            // N is the 11 bit integer resulting from the removal of bit Q and M
            let n =
                ((ac13_field & 0x1F80) >> 2) | ((ac13_field & 0x0020) >> 1) | (ac13_field & 0x000F);

            // The final altitude is resulting number multiplied by 25, minus 1000.
            ((n * 25) - 1000) as i32
        } else {
            // N is an 11 bit Gillham coded altitude
            let n = crate::mode_ac::mode_a_to_mode_c(decode_id13_field(ac13_field))?;
            // if n < -12 {
            //     return Err("Invalid altitude");
            // }

            (100 * n) as i32
        };

        Ok((altitude, AltitudeUnit::Feet))
    } else {
        // *unit = UNIT_METERS;
        // TODO: Implement altitude when meter unit is selected
        // return INVALID_ALTITUDE;
        Err("Invalid altitude")
    }
}

// mode_s.c:156
fn decode_ac12_field(ac12_field: u32) -> Result<u32, &'static str> {
    let q_bit = ac12_field & 0x10; // Bit 48 = Q

    if q_bit != 0 {
        // N is the 11 bit integer resulting from the removal of bit Q at bit 4
        let n = ((ac12_field & 0x0FE0) >> 1) | (ac12_field & 0x000F);

        // The final altitude is the resulting number multiplied by 25, minus 1000.
        Ok((n * 25) - 1000)
    } else {
        // Make N a 13 bit Gillham coded altitude by inserting M=0 at bit 6
        let mut n = ((ac12_field & 0x0FC0) << 1) | (ac12_field & 0x003F);
        n = crate::mode_ac::mode_a_to_mode_c(decode_id13_field(n))?;
        // if n < -12 {
        //     return Err("Invalid altitude");
        // }

        Ok(100 * n)
    }
}

// mode_s.c:184
fn decode_movement_field(movement: u32) -> u32 {
    // Note : movement codes 0,125,126,127 are all invalid, but they are
    //        trapped for before this function is called.
    if movement > 123 {
        199
    }
    // > 175kt
    else if movement > 108 {
        ((movement - 108) * 5) + 100
    } else if movement > 93 {
        ((movement - 93) * 2) + 70
    } else if movement > 38 {
        (movement - 38) + 15
    } else if movement > 12 {
        ((movement - 11) >> 1) + 2
    } else if movement > 8 {
        ((movement - 6) >> 2) + 1
    } else {
        0
    }
}

pub fn decode(mm: &mut ModeSMessage) -> Result<(), &'static str> {
    if mm.msg.iter().all(|b| *b == 0x00) {
        return Err("All zeros");
    }

    mm.msgtype = getbits(&mm.msg, 1, 5) as u8;
    mm.msgbits = super::modes_message_len_by_type(mm.msgtype);
    mm.crc = crate::crc::modes_checksum(&mm.msg, mm.msgbits);

    match mm.msgtype {
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
            // These message types use Address/Parity, i.e. our CRC syndrome is the sender's ICAO address.
            // We can't tell if the CRC is correct or not as we don't know the correct address.
            // Accept the message if it appears to be from a previously-seen aircraft
            if !crate::icao_filter::icao_filter_test(mm.crc) {
                return Err("Failed ICAO filter test");
            }
            mm.source = DataSource::ModeS;
            mm.addr = mm.crc;
        }
        11 => {
            // 11: All-call reply
            // This message type uses Parity/Interrogator, i.e. our CRC syndrome is CL + IC from the uplink message
            // which we can't see. So we don't know if the CRC is correct or not.
            //
            // however! CL + IC only occupy the lower 7 bits of the CRC. So if we ignore those bits when testing
            // the CRC we can still try to detect/correct errors.
            mm.iid = mm.crc & 0x7f;
            if mm.crc & 0x00ff_ff80 != 0 {
                return Err("Failed CRC check for all-call reply");
            }
            mm.source = DataSource::ModeSChecked;
        }
        17 | 18 => {
            // 17: Extended squitter
            // 18: Extended squitter/non-transponder

            // These message types use Parity/Interrogator, but are specified to set II=0
            if mm.crc != 0 {
                return Err("Failed CRC check for extended squitter");
            }

            mm.source = DataSource::ADSB;
        }
        20 | 21 => {
            // 20: Comm-B, altitude reply
            // 21: Comm-B, identity reply

            // These message types either use Address/Parity (see DF0 etc)
            // or Data Parity where the requested BDS is also xored into the top byte.
            // So not only do we not know whether the CRC is right, we also don't know if
            // the ICAO is right! Ow.

            // Try an exact match
            if crate::icao_filter::icao_filter_test(mm.crc) {
                // OK.
                mm.source = DataSource::ModeS;
                mm.addr = mm.crc;
            } else {
                return Err("Failed CRC check for Comm-B altitude or identity reply");
            }
        }
        _ => {
            // All other message types, we don't know how to handle their CRCs, give up
            return Err("Message type whose CRC we don't know how to handle");
        }
    };

    // decode the bulk of the message

    // AA (Address announced)
    if let 11 | 17 | 18 = mm.msgtype {
        let bits = getbits(&mm.msg, 9, 32) as u32;
        mm.aa = bits;
        mm.addr = bits;
    }

    // AC (Altitude Code)
    if let 0 | 4 | 16 | 20 = mm.msgtype {
        mm.ac = getbits(&mm.msg, 20, 32) as u32;

        if mm.ac != 0 {
            if let Ok((altitude, altitude_unit)) = decode_ac13_field(mm.ac) {
                mm.altitude = Some((altitude, altitude_unit, AltitudeSource::Baro));
            }
        }
    }

    // AF (DF19 Application Field) not decoded

    // CA (Capability)
    if let 11 | 17 = mm.msgtype {
        mm.ca = getbits(&mm.msg, 6, 8) as u32;
        mm.airground = match mm.ca {
            0 | 6 | 7 => Some(AirGround::Uncertain),
            4 => Some(AirGround::Ground),
            5 => Some(AirGround::Airborne),
            _ => None,
        };
    }

    // CC (Cross-link capability)
    if let 0 = mm.msgtype {
        mm.cc = getbit(&mm.msg, 7) as u32;
    }

    // CF (Control field)
    if let 18 = mm.msgtype {
        mm.cf = getbits(&mm.msg, 5, 8) as u32;
    }

    // DR (Downlink Request)
    // FS (Flight Status)
    if let 4 | 5 | 20 | 21 = mm.msgtype {
        mm.dr = getbits(&mm.msg, 9, 13) as u32;
        mm.fs = getbits(&mm.msg, 6, 8) as u32;
        // mm->alert_valid = 1;
        // mm->spi_valid = 1;

        match mm.fs {
            0 => mm.airground = Some(AirGround::Uncertain),
            1 => mm.airground = Some(AirGround::Ground),
            2 => {
                mm.airground = Some(AirGround::Uncertain);
                // mm->alert = 1;
            }
            3 => {
                mm.airground = Some(AirGround::Ground);
                // mm->alert = 1;
            }
            4 => {
                mm.airground = Some(AirGround::Uncertain);
                // mm->alert = 1;
                // mm->spi = 1;
            }
            5 => {
                mm.airground = Some(AirGround::Uncertain);
                // mm->spi = 1;
            }
            _ => {
                // mm->spi_valid = 0;
                // mm->alert_valid = 0;
            }
        }
    }

    // ID (Identity)
    if let 5 | 21 = mm.msgtype {
        // Gillham encoded Squawk
        mm.id = getbits(&mm.msg, 20, 32) as u32;
        if mm.id != 0 {
            mm.squawk = Some(decode_id13_field(mm.id));
        }
    }

    // KE (Control, ELM)
    if mm.msgtype >= 24 && mm.msgtype <= 31 {
        mm.ke = getbit(&mm.msg, 4) as u32;
    }

    // MB (messsage, Comm-B)
    if let 20 | 21 = mm.msgtype {
        mm.mb.clone_from_slice(&mm.msg[4..11]);
        decode_comm_b(mm);
    }

    // MD (message, Comm-D)
    if mm.msgtype >= 24 && mm.msgtype <= 31 {
        mm.md.clone_from_slice(&mm.msg[1..11]);
    }

    // ME (message, extended squitter)
    if let 17 | 18 = mm.msgtype {
        mm.me.clone_from_slice(&mm.msg[4..11]);
        decode_extended_squitter(mm);
    }

    // MV (message, ACAS)
    if let 16 = mm.msgtype {
        mm.mv.clone_from_slice(&mm.msg[4..11]);
    }

    // ND (number of D-segment, Comm-D)
    if mm.msgtype >= 24 && mm.msgtype <= 31 {
        mm.nd = getbits(&mm.msg, 5, 8) as u32;
    }

    // RI (Reply information, ACAS)
    if let 0 | 16 = mm.msgtype {
        mm.ri = getbits(&mm.msg, 14, 17) as u32;
    }

    // SL (Sensitivity level, ACAS)
    if let 0 | 16 = mm.msgtype {
        mm.sl = getbits(&mm.msg, 9, 11) as u32;
    }

    // UM (Utility Message)
    if let 4 | 5 | 20 | 21 = mm.msgtype {
        mm.um = getbits(&mm.msg, 14, 19) as u32;
    }

    // VS (Vertical Status)
    if let 0 | 16 = mm.msgtype {
        mm.vs = getbit(&mm.msg, 6) as u32;
        mm.airground = if mm.vs != 0 {
            Some(AirGround::Ground)
        } else {
            Some(AirGround::Uncertain)
        };
    }

    if mm.msgtype == 17 || mm.msgtype == 18 || (mm.msgtype == 11 && mm.id == 0) {
        // No CRC errors seen, and either it was an DF17/18 extended squitter
        // or a DF11 acquisition squitter with II = 0. We probably have the right address.

        // We wait until here to do this as we may have needed to decode an ES to note
        // the type of address in DF18 messages.

        // NB this is the only place that adds addresses!
        crate::icao_filter::icao_filter_add(mm.addr);
    }

    // MLAT overrides all other sources
    if mm.remote && mm.timestamp_msg == MAGIC_MLAT_TIMESTAMP {
        mm.source = DataSource::MLAT;
    }

    Ok(())
}

// mode_s.c:1082
fn decode_extended_squitter(mm: &mut ModeSMessage) {
    // unsigned char *me = mm->ME;
    // unsigned metype = mm->metype = ;
    mm.metype = getbits(&mm.me, 1, 5) as u8;
    let mut check_imf: bool = false;

    // Check CF on DF18 to work out the format of the ES and whether we need to look for an IMF bit
    if let 18 = mm.msgtype {
        match mm.cf {
            0 => mm.addrtype = Some(AddrType::ADSB_ICAO_NT), // ADS-B Message from a non-transponder device, AA field holds 24-bit ICAO aircraft address
            1 => {
                // Reserved for ADS-B Message in which the AA field holds anonymous address or ground vehicle address or fixed obstruction address
                mm.addrtype = Some(AddrType::ADSB_Other);
                mm.addr |= MODES_NON_ICAO_ADDRESS;
            }
            2 => {
                // Fine TIS-B Message
                // IMF=0: AA field contains the 24-bit ICAO aircraft address
                // IMF=1: AA field contains the 12-bit Mode A code followed by a 12-bit track file number
                mm.source = DataSource::TISB;
                mm.addrtype = Some(AddrType::TISB_ICAO);
                check_imf = true;
            }
            3 => {
                //   Coarse TIS-B airborne position and velocity.
                // IMF=0: AA field contains the 24-bit ICAO aircraft address
                // IMF=1: AA field contains the 12-bit Mode A code followed by a 12-bit track file number

                // For now we only look at the IMF bit.
                mm.source = DataSource::TISB;
                mm.addrtype = Some(AddrType::TISB_ICAO);
                panic!("Need to implement IMF decoding for coarse TIS-B airborne position and velocity");
                // if (getbit(me, 1))
                //     setIMF(mm);
            }
            5 => {
                // Fine TIS-B Message, AA field contains a non-ICAO 24-bit address
                mm.addrtype = Some(AddrType::TISB_Other);
                mm.source = DataSource::TISB;
                mm.addr |= MODES_NON_ICAO_ADDRESS;
            }
            6 => {
                // Rebroadcast of ADS-B Message from an alternate data link
                // IMF=0: AA field holds 24-bit ICAO aircraft address
                // IMF=1: AA field holds anonymous address or ground vehicle address or fixed obstruction address
                mm.addrtype = Some(AddrType::ADSR_ICAO);
                check_imf = true;
            }
            _ => {
                // All others, we don't know the format.
                mm.addrtype = Some(AddrType::Unknown);
                mm.addr |= MODES_NON_ICAO_ADDRESS; // assume non-ICAO
            }
        }
    }

    match mm.metype {
        1 | 2 | 3 | 4 => {
            // Aircraft Identification and Category
            // from decodeESIdentAndCategory at mode_s.c:690
            mm.mesub = getbits(&mm.me, 6, 8) as u8;

            let mut callsign = String::new();
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 9, 14) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 15, 20) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 21, 26) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 27, 32) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 33, 38) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 39, 44) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 45, 50) as usize)
                    .unwrap(),
            );
            callsign.push(
                AIS_CHARSET
                    .chars()
                    .nth(getbits(&mm.me, 51, 56) as usize)
                    .unwrap(),
            );

            // A common failure mode seems to be to intermittently send
            // all zeros. Catch that here.
            if callsign != "@@@@@@@@" {
                mm.callsign = Some(callsign);
            }

            mm.category = Some(((0x0E - mm.metype) << 4) | mm.mesub);
        }
        19 => {
            // mode_s.c:738
            // Airborne Velocity Message

            mm.mesub = getbits(&mm.me, 6, 8) as u8;

            if check_imf && getbit(&mm.me, 9) != 0 {
                set_imf(&mut mm.addr, &mut mm.addrtype);
            }

            if mm.mesub < 1 || mm.mesub > 4 {
                return;
            }

            mm.vert_rate = match getbits(&mm.me, 38, 46) {
                0 => None,
                x => {
                    let scale = if getbit(&mm.me, 37) != 0 { -64 } else { 64 };
                    let source = if getbit(&mm.me, 36) != 0 {
                        AltitudeSource::GNSS
                    } else {
                        AltitudeSource::Baro
                    };
                    Some(((x - 1) as i32 * scale, source))
                }
            };

            match mm.mesub {
                1 | 2 => {
                    let ew_raw = getbits(&mm.me, 15, 24) as i32;
                    let ns_raw = getbits(&mm.me, 26, 35) as i32;

                    if ew_raw != 0 && ns_raw != 0 {
                        let scale: i32 = if mm.mesub == 2 { 4 } else { 1 };
                        let ew_vel: f64 = f64::from({
                            let sign: i32 = if getbit(&mm.me, 14) != 0 { -1 } else { 1 };
                            (ew_raw - 1) * sign * scale
                        });
                        let ns_vel: f64 = f64::from({
                            let sign: i32 = if getbit(&mm.me, 25) != 0 { -1 } else { 1 };
                            (ns_raw - 1) * sign * scale
                        });

                        // Compute velocity and angle from the two speed components
                        let speed = (ns_vel.mul_add(ns_vel, ew_vel * ew_vel) + 0.5).sqrt();
                        mm.speed = Some((speed as u32, SpeedSource::GroundSpeed));

                        mm.heading = match speed {
                            // Floats are currently discouraged in patterns and will become a hard error in future versions.  However, guards are
                            // different.  See issue #41620 <https://github.com/rust-lang/rust/issues/41620>
                            x if x == 0.0 => None,
                            _ => {
                                let mut heading: i32 = ew_vel
                                    .atan2(ns_vel)
                                    .mul_add(180.0 / std::f64::consts::PI, 0.5)
                                    as i32;
                                // We don't want negative values but a 0-360 scale
                                if heading < 0 {
                                    heading += 360;
                                }
                                Some((heading, HeadingSource::True))
                            }
                        }
                    }
                }
                3 | 4 => {
                    panic!("Implement 3 | 4 case");
                    /*unsigned airspeed = getbits(me, 26, 35);
                    if (airspeed) {
                        mm->speed = (airspeed - 1) * (mm->mesub == 4 ? 4 : 1);
                        mm->speed_source = getbit(me, 25) ? SPEED_TAS : SPEED_IAS;
                        mm->speed_valid = 1;
                    }

                    if (getbit(me, 14)) {
                        mm->heading = getbits(me, 15, 24);
                        mm->heading_source = HEADING_MAGNETIC;
                        mm->heading_valid = 1;
                    }*/
                }
                _ => (),
            }

            mm.gnss_delta = match getbits(&mm.me, 50, 56) {
                0 => None,
                x => {
                    let scale: i32 = if getbit(&mm.me, 49) != 0 { -25 } else { 25 };
                    Some((x as i32 - 1) * scale)
                }
            };
        }
        5 | 6 | 7 | 8 => {
            // Surface position and movement
            if check_imf && getbit(&mm.me, 21) != 0 {
                set_imf(&mut mm.addr, &mut mm.addrtype);
            }

            mm.airground = Some(AirGround::Ground);
            mm.raw_cpr = {
                let lat: u32 = getbits(&mm.me, 23, 39) as u32;
                let lon: u32 = getbits(&mm.me, 40, 56) as u32;
                let odd: bool = getbit(&mm.me, 22) != 0;
                let nucp: u32 = u32::from(14 - mm.metype);
                let typ = CprType::Surface;
                Some((lat, lon, odd, nucp, typ))
            };

            let movement = getbits(&mm.me, 6, 12) as u32;
            if movement > 0 && movement < 125 {
                mm.speed = Some((decode_movement_field(movement), SpeedSource::GroundSpeed))
            }

            if getbit(&mm.me, 13) != 0 {
                let hdg = ((getbits(&mm.me, 14, 20) * 360) / 128) as i32;
                mm.heading = Some((hdg, HeadingSource::True));
            }
        }
        0 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 20 | 21 | 22 => {
            // Airborne position and altitude
            // mode_s.c:843
            if check_imf && getbit(&mm.me, 8) != 0 {
                set_imf(&mut mm.addr, &mut mm.addrtype);
            }

            let ac12_field = getbits(&mm.me, 9, 20);

            if mm.metype == 0 {
                // mm->cpr_nucp = 0;
                mm.raw_cpr = None; // NOTE: not sure this is exactly the same thing
            } else {
                // Catch some common failure modes and don't mark them as valid
                // (so they won't be used for positioning)

                let cpr_lat = getbits(&mm.me, 23, 39) as u32;
                let cpr_lon = getbits(&mm.me, 40, 56) as u32;

                mm.raw_cpr = if ac12_field == 0
                    && cpr_lon == 0
                    && (cpr_lat & 0x0fff) == 0
                    && mm.metype == 15
                {
                    // Seen from at least:
                    //   400F3F (Eurocopter ECC155 B1) - Bristow Helicopters
                    //   4008F3 (BAE ATP) - Atlantic Airlines
                    //   400648 (BAE ATP) - Atlantic Airlines
                    // altitude == 0, longitude == 0, type == 15 and zeros in latitude LSB.
                    // Can alternate with valid reports having type == 14
                    // Modes.stats_current.cpr_filtered++;
                    None
                } else {
                    // Otherwise, assume it's valid.
                    let typ = CprType::Airborne;
                    let odd = getbit(&mm.me, 22) != 0;

                    let nucp = u32::from(if mm.metype == 18 || mm.metype == 22 {
                        0
                    } else if mm.metype < 18 {
                        18 - mm.metype
                    } else {
                        29 - mm.metype
                    });

                    Some((cpr_lat, cpr_lon, odd, nucp, typ))
                };
            }

            if ac12_field != 0 {
                // Only attempt to decode if a valid (non zero) altitude is present
                if let Ok(altitude) = decode_ac12_field(ac12_field as u32) {
                    let unit = AltitudeUnit::Feet; // TODO: unhardcode this
                    let src = match mm.metype {
                        20 | 21 | 22 => AltitudeSource::GNSS,
                        _ => AltitudeSource::Baro,
                    };
                    mm.altitude = Some((altitude as i32, unit, src));
                }
            }
        }
        23 => panic!("Need to implement decodeESTestMessage"),
        24 => (), // Reserved for Surface System Status
        28 => {
            // Extended Squitter Aircraft Status
            mm.mesub = getbits(&mm.me, 6, 8) as u8;

            if mm.mesub == 1 {
                // Emergency status squawk field
                let id13_field = getbits(&mm.me, 12, 24) as u32;
                if id13_field != 0 {
                    mm.squawk = Some(decode_id13_field(id13_field));
                }

                if check_imf && getbit(&mm.me, 56) != 0 {
                    set_imf(&mut mm.addr, &mut mm.addrtype);
                }
            }
        }
        29 => {
            mm.mesub = getbits(&mm.me, 6, 7) as u8; // an unusual message: only 2 bits of subtype

            if check_imf && getbit(&mm.me, 51) != 0 {
                set_imf(&mut mm.addr, &mut mm.addrtype);
            }

            if mm.mesub == 0 { // Target state and status, V1
                 // TODO: need RTCA/DO-260A
            } else if mm.mesub == 1 {
                // Target state and status, V2
                // TODO: check the decoding of sil_type
                let sil_type = if getbit(&mm.me, 8) == 0 {
                    SilType::SilPerSample
                } else {
                    SilType::SilPerHour
                };
                let alt_type = if getbit(&mm.me, 9) != 0 {
                    TssAltitudeType::FMS
                } else {
                    TssAltitudeType::MCP
                };

                let alt_bits = getbits(&mm.me, 10, 20);
                let altitude = if alt_bits == 0 {
                    None
                } else {
                    Some(((alt_bits as u32 - 1) * 32, alt_type))
                };

                let baro_bits = getbits(&mm.me, 21, 29);
                let baro = if baro_bits == 0 {
                    None
                } else {
                    Some((baro_bits as f32 - 1.0).mul_add(0.8, 800.0))
                };

                let heading = if getbit(&mm.me, 30) != 0 {
                    // two's complement -180..+180, which is conveniently
                    // also the same as unsigned 0..360
                    Some((getbits(&mm.me, 31, 39) as u32 * 180) / 256)
                } else {
                    None
                };

                let nac_p: u8 = getbits(&mm.me, 40, 43) as u8;
                let nic_baro: bool = getbit(&mm.me, 44) != 0;
                let sil: u8 = getbits(&mm.me, 45, 46) as u8;

                let mode_valid: bool = getbit(&mm.me, 47) != 0;
                let mode_autopilot: bool = getbit(&mm.me, 48) != 0;
                let mode_vnav: bool = getbit(&mm.me, 49) != 0;
                let mode_alt_hold: bool = getbit(&mm.me, 50) != 0;
                let mode_approach: bool = getbit(&mm.me, 52) != 0;
                let acas_operational: bool = getbit(&mm.me, 53) != 0;

                mm.tss = Some(TargetStateStatus {
                    mode_valid,
                    mode_autopilot,
                    mode_vnav,
                    mode_alt_hold,
                    mode_approach,
                    acas_operational,
                    nac_p,
                    nic_baro,
                    sil,
                    sil_type,
                    altitude,
                    baro,
                    heading,
                });
            }
        }
        30 => (), // Aircraft Operational Coordination
        31 => {
            mm.mesub = getbits(&mm.me, 6, 8) as u8;

            // Aircraft Operational Status
            if check_imf && getbit(&mm.me, 56) != 0 {
                set_imf(&mut mm.addr, &mut mm.addrtype);
            }

            if let 0 | 1 = mm.mesub {
                let mut opstatus = OperationalStatus::default();

                opstatus.version = getbits(&mm.me, 41, 43) as u8;

                match opstatus.version {
                    0 => (),
                    1 => {
                        if getbits(&mm.me, 25, 26) == 0 {
                            opstatus.om_acas_ra = getbit(&mm.me, 27) != 0;
                            opstatus.om_ident = getbit(&mm.me, 28) != 0;
                            opstatus.om_atc = getbit(&mm.me, 29) != 0;
                        }

                        if mm.mesub == 0
                            && getbits(&mm.me, 9, 10) == 0
                            && getbits(&mm.me, 13, 14) == 0
                        {
                            // airborne
                            opstatus.cc_acas = getbit(&mm.me, 11) == 0;
                            opstatus.cc_cdti = getbit(&mm.me, 12) != 0;
                            opstatus.cc_arv = getbit(&mm.me, 15) != 0;
                            opstatus.cc_ts = getbit(&mm.me, 16) != 0;
                            opstatus.cc_tc = getbits(&mm.me, 17, 18) as u8;
                        } else if mm.mesub == 1
                            && getbits(&mm.me, 9, 10) == 0
                            && getbits(&mm.me, 13, 14) == 0
                        {
                            // surface
                            opstatus.cc_poa = getbit(&mm.me, 11) != 0;
                            opstatus.cc_cdti = getbit(&mm.me, 12) != 0;
                            opstatus.cc_b2_low = getbit(&mm.me, 15) != 0;
                            opstatus.cc_lw_valid = true;
                            opstatus.cc_lw = getbits(&mm.me, 21, 24) as u32;
                        }

                        opstatus.nic_supp_a = getbit(&mm.me, 44) != 0;
                        opstatus.nac_p = getbits(&mm.me, 45, 48) as u8;
                        opstatus.sil = getbits(&mm.me, 51, 52) as u8;

                        if mm.mesub == 0 {
                            opstatus.nic_baro = getbit(&mm.me, 53) != 0;
                        } else {
                            opstatus.track_angle = if getbit(&mm.me, 53) != 0 {
                                AngleType::Track
                            } else {
                                AngleType::Heading
                            };
                        }
                        opstatus.hrd = if getbit(&mm.me, 54) != 0 {
                            HeadingSource::Magnetic
                        } else {
                            HeadingSource::True
                        };
                    }
                    _ => {
                        if getbits(&mm.me, 25, 26) == 0 {
                            opstatus.om_acas_ra = getbit(&mm.me, 27) != 0;
                            opstatus.om_ident = getbit(&mm.me, 28) != 0;
                            opstatus.om_atc = getbit(&mm.me, 29) != 0;
                            opstatus.om_saf = getbit(&mm.me, 30) != 0;
                            opstatus.om_sda = getbits(&mm.me, 31, 32) as u8;
                        }

                        if mm.mesub == 0
                            && getbits(&mm.me, 9, 10) == 0
                            && getbits(&mm.me, 13, 14) == 0
                        {
                            // airborne
                            opstatus.cc_acas = getbit(&mm.me, 11) != 0;
                            opstatus.cc_1090_in = getbit(&mm.me, 12) != 0;
                            opstatus.cc_arv = getbit(&mm.me, 15) != 0;
                            opstatus.cc_ts = getbit(&mm.me, 16) != 0;
                            opstatus.cc_tc = getbits(&mm.me, 17, 18) as u8;
                            opstatus.cc_uat_in = getbit(&mm.me, 19) != 0;
                        } else if mm.mesub == 1
                            && getbits(&mm.me, 9, 10) == 0
                            && getbits(&mm.me, 13, 14) == 0
                        {
                            // surface
                            opstatus.cc_poa = getbit(&mm.me, 11) != 0;
                            opstatus.cc_1090_in = getbit(&mm.me, 12) != 0;
                            opstatus.cc_b2_low = getbit(&mm.me, 15) != 0;
                            opstatus.cc_uat_in = getbit(&mm.me, 16) != 0;
                            opstatus.cc_nac_v = getbits(&mm.me, 17, 19) as u8;
                            opstatus.cc_nic_supp_c = getbit(&mm.me, 20) != 0;
                            opstatus.cc_lw_valid = true;
                            opstatus.cc_lw = getbits(&mm.me, 21, 24) as u32;
                            opstatus.cc_antenna_offset = getbits(&mm.me, 33, 40) as u32;
                        }

                        opstatus.nic_supp_a = getbit(&mm.me, 44) != 0;
                        opstatus.nac_p = getbits(&mm.me, 45, 48) as u8;
                        opstatus.sil = getbits(&mm.me, 51, 52) as u8;

                        if mm.mesub == 0 {
                            opstatus.gva = getbits(&mm.me, 49, 50) as u8;
                            opstatus.nic_baro = getbit(&mm.me, 53) != 0;
                        } else {
                            opstatus.track_angle = if getbit(&mm.me, 53) != 0 {
                                AngleType::Track
                            } else {
                                AngleType::Heading
                            };
                        }

                        opstatus.hrd = if getbit(&mm.me, 54) != 0 {
                            HeadingSource::Magnetic
                        } else {
                            HeadingSource::True
                        };
                        opstatus.sil_type = if getbit(&mm.me, 55) != 0 {
                            SilType::SilPerSample
                        } else {
                            SilType::SilPerHour
                        };
                    }
                }

                mm.opstatus = Some(opstatus);
            }
        }
        _ => (),
    }
}

// mode_s.c:1188
fn decode_comm_b(mm: &mut ModeSMessage) {
    // This is a bit hairy as we don't know what the requested register was
    if getbits(&mm.msg, 33, 40) == 0x20 {
        // BDS 2,0 Aircraft Identification

        // from decodeBDS20 at mode_s.c:662
        let mut callsign = String::new();
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 41, 46) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 47, 52) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 53, 58) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 59, 64) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 65, 70) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 71, 76) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 77, 82) as usize)
                .unwrap(),
        );
        callsign.push(
            AIS_CHARSET
                .chars()
                .nth(getbits(&mm.msg, 83, 88) as usize)
                .unwrap(),
        );

        // Catch possible bad decodings since BDS2,0 is not
        // 100% reliable: accept only alphanumeric data
        mm.callsign = if callsign
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        {
            Some(callsign)
        } else {
            None
        }
    }
}

// mode_s.c:714
fn set_imf(addr: &mut u32, opt_addrtype: &mut Option<AddrType>) {
    *addr |= MODES_NON_ICAO_ADDRESS;

    let opt_new_addrtype = match opt_addrtype {
        Some(AddrType::ADSB_ICAO | AddrType::ADSB_ICAO_NT) => Some(AddrType::ADSB_Other), // Shouldn't happen, but let's try to handle it
        Some(AddrType::TISB_ICAO) => Some(AddrType::TISB_Trackfile),
        Some(AddrType::ADSR_ICAO) => Some(AddrType::ADSR_Other),
        _ => None,
    };

    if let Some(x) = opt_new_addrtype {
        opt_addrtype.replace(x);
    }
}
