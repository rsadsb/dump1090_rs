#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::ModeSMessage;
use crate::{AddrType, AirGround, AltitudeSource, CprType, DataSource};

use super::DataValidityBox as DvBox;

#[derive(Debug)]
pub struct CircularBuffer_8_F64 {
    buffer: [f64; 8],
    idx: usize,
}

impl Default for CircularBuffer_8_F64 {
    fn default() -> Self {
        Self {
            buffer: [0.0; 8],
            idx: 0,
        }
    }
}

impl CircularBuffer_8_F64 {
    pub fn push(&mut self, x: f64) {
        self.buffer[self.idx] = x;
        self.idx = (self.idx + 1) % 8;
    }
}

#[derive(Debug, Default)]
pub struct Aircraft {
    pub addr: (u32, AddrType), // ICAO address and highest priority address type seen for this aircraft

    pub seen: u128,      // Time (millis) at which the last packet was received
    pub messages: usize, // Number of Mode S messages received

    pub signal_level: CircularBuffer_8_F64, // Last 8 Signal Amplitudes

    pub callsign: DvBox<String>, // Flight number

    pub altitude: DvBox<(i32, u32)>, // Altitude (Baro) and as a Mode C value
    pub altitude_gnss: DvBox<i32>,   // Altitude (GNSS)
    pub gnss_delta: DvBox<i32>,      // Difference between GNSS and Baro altitudes
    pub speed: DvBox<u32>,
    pub speed_ias: DvBox<u32>,
    pub speed_tas: DvBox<u32>,
    pub heading: DvBox<i32>,          // Heading (OK it's really the track)
    pub heading_magnetic: DvBox<i32>, // Heading
    pub vert_rate: DvBox<(i32, AltitudeSource)>, // Vertical rate
    pub squawk: DvBox<u32>,           // Squawk
    pub category: DvBox<u8>,          // Aircraft category A0 - D7 encoded as a single hex byte
    pub airground: DvBox<AirGround>,  // air/ground status

    pub cpr_odd: DvBox<(CprType, u32, u32, u32)>, // lat, lon, nuc
    pub cpr_even: DvBox<(CprType, u32, u32, u32)>, // lat, lon, nuc

    pub position: DvBox<(f64, f64, u32)>,

    pub mode_a_count: u64,  // Mode A Squawk hit Count
    pub mode_c_count: u64,  // Mode C Altitude hit Count
    pub mode_ac_flags: u32, // Flags for mode A/C recognition

    pub fatsv_emitted_altitude: i32,      // last FA emitted altitude
    pub fatsv_emitted_altitude_gnss: i32, //      -"-         GNSS altitude
    pub fatsv_emitted_heading: i32,       //      -"-         true track
    pub fatsv_emitted_heading_magnetic: i32, //      -"-         magnetic heading
    pub fatsv_emitted_speed: i32,         //      -"-         groundspeed
    pub fatsv_emitted_speed_ias: i32,     //      -"-         IAS
    pub fatsv_emitted_speed_tas: i32,     //      -"-         TAS
    pub fatsv_emitted_airground: AirGround, //      -"-         air/ground state
    pub fatsv_emitted_bds_10: [u8; 7],    //      -"-         BDS 1,0 message
    pub fatsv_emitted_bds_30: [u8; 7],    //      -"-         BDS 3,0 message
    pub fatsv_emitted_es_status: [u8; 7], //      -"-         ES operational status message
    pub fatsv_emitted_es_target: [u8; 7], //      -"-         ES target status message
    pub fatsv_emitted_es_acas_ra: [u8; 7], //      -"-         ES ACAS RA report message

    pub fatsv_last_emitted: u64, // time (millis) aircraft was last FA emitted

    pub first_message: ModeSMessage, // A copy of the first message we received for this aircraft.
}

impl Aircraft {
    pub fn update_position(
        &mut self,
        mm: &mut ModeSMessage,
        now_ms: u128,
    ) -> Result<(), &'static str> {
        let mut location_result = -1;
        let mut new_lat: f64 = 0.0;
        let mut new_lon: f64 = 0.0;
        let mut new_nuc: u32 = 0;

        if let Some((_, _, cpr_odd, _, cpr_type)) = mm.raw_cpr {
            let max_elapsed: u128 = if cpr_type == CprType::Surface {
                // Surface: 25 seconds if >25kt or speed unknown, 50 seconds otherwise
                if let Some((speed, _)) = mm.speed {
                    if speed <= 25 {
                        50000
                    } else {
                        25000
                    }
                } else {
                    25000
                }
            } else {
                10000
            };

            // If we have enough recent data, try global CPR
            let cpr_same_type = if let (Some((even, _, _, _)), Some((odd, _, _, _))) =
                (self.cpr_even.get_opt(), self.cpr_odd.get_opt())
            {
                odd == even
            } else {
                false
            };

            if self.cpr_odd.is_valid()
                && self.cpr_even.is_valid()
                && self.cpr_odd.same_source(&self.cpr_even)
                && cpr_same_type
                && self.cpr_odd.time_between(&self.cpr_even) <= max_elapsed
            {
                location_result =
                    do_global_cpr(self, mm, now_ms, &mut new_lat, &mut new_lon, &mut new_nuc);

                if location_result == -2 {
                    // Global CPR failed because the position produced implausible results.
                    // This is bad data. Discard both odd and even messages and wait for a fresh pair.
                    // Also disable aircraft-relative positions until we have a new good position (but don't discard the
                    // recorded position itself)
                    self.cpr_even.direct_set_source(DataSource::Invalid);
                    self.cpr_odd.direct_set_source(DataSource::Invalid);
                    self.position.direct_set_source(DataSource::Invalid);
                    return Ok(());
                } else if location_result == -1 {
                    // No local reference for surface position available, or the two messages crossed a zone.
                    // Nonfatal, try again later.
                } else {
                    self.position
                        .combine_validity(&self.cpr_even, &self.cpr_odd);
                }
            }

            // Otherwise try relative CPR.
            let mut cpr_relative = false;
            if location_result == -1 {
                location_result =
                    do_local_cpr(self, mm, now_ms, &mut new_lat, &mut new_lon, &mut new_nuc);

                if location_result >= 0 {
                    cpr_relative = true; // true means local

                    if cpr_odd {
                        self.position.copy_validity_from(&self.cpr_odd);
                    } else {
                        self.position.copy_validity_from(&self.cpr_even);
                    }
                }
            }

            if location_result == 0 {
                // If we sucessfully decoded, back copy the results to mm so that we can print them in list output
                mm.decoded_cpr = Some((new_lat, new_lon, cpr_relative));

                // Update aircraft state
                self.position.direct_set((new_lat, new_lon, new_nuc));

                // update_range_histogram(new_lat, new_lon);
            }
        }

        Ok(())
    }

    // return true if it's OK for the aircraft to have travelled from its last known position
    // to a new position at (lat,lon,surface) at a time of now.
    // track.c:204
    pub fn speed_check(&self, lat: f64, lon: f64, now_ms: u128, surface: bool) -> bool {
        let elapsed: u128 = self.position.data_age(now_ms);

        let (a_lat, a_lon) = match self.position.get_if_valid() {
            Some((x, y, _)) => (*x, *y),
            None => return true, // no reference, assume OK
        };

        let mut speed: u32 = self
            .speed
            .get_if_valid()
            .copied()
            .or(self.speed_ias.get_if_valid().map(|s| (s * 4) / 3))
            .or(self.speed_tas.get_if_valid().map(|s| (s * 4) / 3))
            .unwrap_or(if surface { 100 } else { 600 }); // guess

        // Work out a reasonable speed to use:
        //  current speed + 1/3
        //  surface speed min 20kt, max 150kt
        //  airborne speed min 200kt, no max
        speed = speed * 4 / 3;
        if surface {
            speed = std::cmp::max(speed, 20);
            speed = std::cmp::min(speed, 150);
        } else {
            speed = std::cmp::max(speed, 200);
        }

        // 100m (surface) or 500m (airborne) base distance to allow for minor errors,
        // plus distance covered at the given speed for the elapsed time + 1 second.
        let range: f64 = ((elapsed as f64 + 1000.0) / 1000.0).mul_add(
            f64::from(speed) * 1852.0 / 3600.0,
            if surface { 0.1e3 } else { 0.5e3 },
        );

        // find actual distance
        let distance: f64 = super::greatcircle(a_lat, a_lon, lat, lon);

        distance <= range
    }
}

// TODO: consider putting inside the Aircraft impl block
fn do_global_cpr(
    a: &mut Aircraft,
    mm: &ModeSMessage,
    now_ms: u128,
    lat: &mut f64,
    lon: &mut f64,
    nuc: &mut u32,
) -> i32 {
    // TODO: as always, try to get rid of unwraps at some point
    let (_, _, fflag, _, mm_cpr_type) = mm.raw_cpr.unwrap();
    let surface: bool = mm_cpr_type == CprType::Surface;

    let (_, odd_lat, odd_lon, odd_nuc) = a.cpr_odd.get_opt().unwrap();
    let (_, even_lat, even_lon, even_nuc) = a.cpr_even.get_opt().unwrap();

    *nuc = std::cmp::min(*even_nuc, *odd_nuc); // worst of the two positions

    let result = if surface {
        // surface global CPR
        // find reference location
        /*double reflat, reflon;

        if (trackDataValidEx(&a->position_valid, now, 50000, SOURCE_INVALID)) { // Ok to try aircraft relative first
            reflat = a->lat;
            reflon = a->lon;
            if (a->pos_nuc < *nuc)
                *nuc = a->pos_nuc;
        } else if (Modes.bUserFlags & MODES_USER_LATLON_VALID) {
            reflat = Modes.fUserLat;
            reflon = Modes.fUserLon;
        } else {
            // No local reference, give up
            return (-1);
        }

        result = decodeCPRsurface(reflat, reflon,
                                  a->cpr_even_lat, a->cpr_even_lon,
                                  a->cpr_odd_lat, a->cpr_odd_lon,
                                  fflag,
                                  lat, lon);*/
        panic!("Need to implement decodeCPRsurface");
    } else {
        // airborne global CPR
        crate::cpr::decode_cpr_airborne(*even_lat, *even_lon, *odd_lat, *odd_lon, fflag, lat, lon)
    };

    if result < 0 {
        return result;
    }

    // for mlat results, skip the speed check
    if let DataSource::MLAT = mm.source {
        return result;
    }

    // check speed limit
    if a.position.is_valid()
        && a.position.get_opt().unwrap().2 >= *nuc
        && !a.speed_check(*lat, *lon, now_ms, surface)
    {
        return -2;
    }

    result
}

fn do_local_cpr(
    a: &mut Aircraft,
    mm: &ModeSMessage,
    now_ms: u128,
    lat: &mut f64,
    lon: &mut f64,
    nuc: &mut u32,
) -> i32 {
    // relative CPR
    // find reference location
    let (cpr_lat, cpr_lon, fflag, cpr_nucp, mm_cpr_type) = mm.raw_cpr.unwrap();
    let surface: bool = mm_cpr_type == CprType::Surface;

    *nuc = cpr_nucp;

    let (reflat, reflon, range_limit) =
        if a.position
            .is_valid_with_constraints(now_ms, 50000, DataSource::Invalid)
        {
            let (a_lat, a_lon, pos_nuc) = a.position.get_opt().unwrap();
            *nuc = std::cmp::min(*nuc, *pos_nuc);

            (*a_lat, *a_lon, 50.0e3)
        } else {
            // No local reference, give up
            return -1;
        };

    match crate::cpr::decode_cpr_relative(reflat, reflon, cpr_lat, cpr_lon, fflag, surface) {
        Ok((rlat, rlon)) => {
            *lat = rlat;
            *lon = rlon;
        }
        Err(_) => return -1,
    }

    // check range limit
    if range_limit > 0.0 && super::greatcircle(reflat, reflon, *lat, *lon) > range_limit {
        return -1;
    }

    // check speed limit
    if a.position.is_valid()
        && a.position.get_opt().unwrap().2 >= *nuc
        && !a.speed_check(*lat, *lon, now_ms, surface)
    {
        return -1;
    }

    0
}
