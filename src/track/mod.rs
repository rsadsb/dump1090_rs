// This module includes functionality translated from track.c

use std::f64::consts::PI;
use std::sync::Mutex;

use crate::ModeSMessage;
use crate::{AddrType, AltitudeSource, DataSource, HeadingSource, SpeedSource};
use crate::{
    MODEAC_MSG_FLAG, MODEAC_MSG_MODEA_HIT, MODEAC_MSG_MODEA_ONLY, MODEAC_MSG_MODEC_HIT,
    MODEAC_MSG_MODEC_OLD,
};

lazy_static! {
    static ref AIRCRAFT_LIST: Mutex<Vec<aircraft::Aircraft>> = Mutex::new(vec![]);
}

mod aircraft;

#[derive(Debug, Default)]
pub struct DataValidityBox<T: Default> {
    source: DataSource,
    updated: u128,
    stale: u128,
    expires: u128,
    opt_contents: Option<T>,
}

impl<T: Default> DataValidityBox<T> {
    pub fn get_opt(&self) -> Option<&T> {
        self.opt_contents.as_ref()
    }
    pub fn direct_set(&mut self, v: T) {
        self.opt_contents = Some(v)
    }
    pub fn is_valid(&self) -> bool {
        self.source != DataSource::Invalid
    }
    pub fn direct_set_source(&mut self, src: DataSource) {
        self.source = src;
    }
    pub fn get_if_valid(&self) -> Option<&T> {
        if self.is_valid() {
            self.get_opt()
        } else {
            None
        }
    }

    pub fn same_source<U: Default>(&self, other: &DataValidityBox<U>) -> bool {
        self.source == other.source
    }
    pub fn time_between<U: Default>(&self, other: &DataValidityBox<U>) -> u128 {
        if self.updated > other.updated {
            self.updated - other.updated
        } else {
            other.updated - self.updated
        }
    }
    pub fn is_valid_with_constraints(
        &self,
        now_ms: u128,
        max_age_ms: u128,
        min_source: DataSource,
    ) -> bool {
        self.is_valid()
            && self.source >= min_source
            && !((self.updated < now_ms) && (now_ms - self.updated > max_age_ms))
    }

    pub fn data_age(&self, now_ms: u128) -> u128 {
        if self.source == DataSource::Invalid {
            std::u128::MAX
        } else if self.updated > now_ms {
            0
        } else {
            now_ms - self.updated
        }
    }

    pub fn copy_validity_from<U: Default>(&mut self, other: &DataValidityBox<U>) {
        self.source = other.source;
        self.updated = other.updated;
        self.stale = other.stale;
        self.expires = other.expires;
    }

    // track.c:115
    pub fn update<U, V: Fn(&mut T) -> U>(
        &mut self,
        mm: &ModeSMessage,
        now_ms: u128,
        f: V,
    ) -> Option<U> {
        let source: &DataSource = &mm.source;

        if *source < self.source && now_ms < self.stale {
            return None;
        }

        self.source = source.clone();
        self.updated = now_ms;
        self.stale = now_ms + 60000;
        self.expires = now_ms + 70000;

        if self.opt_contents.is_none() {
            self.opt_contents = Some(T::default())
        }

        self.opt_contents.as_mut().map(f)
    }

    pub fn opt_fn_update<U, V: Fn(&U) -> T>(
        &mut self,
        mm: &ModeSMessage,
        now_ms: u128,
        opt_input: &Option<U>,
        f: V,
    ) {
        if let Some(u) = opt_input {
            self.update(mm, now_ms, |v| *v = f(u));
        }
    }

    // track.c:128
    pub fn combine_validity<U: Default, V: Default>(
        &mut self,
        from1: &DataValidityBox<U>,
        from2: &DataValidityBox<V>,
    ) {
        if from1.source == DataSource::Invalid {
            self.source = from2.source;
            self.updated = from2.updated;
            self.stale = from2.stale;
            self.expires = from2.expires;
            return;
        }

        if from2.source == DataSource::Invalid {
            self.source = from1.source;
            self.updated = from1.updated;
            self.stale = from1.stale;
            self.expires = from1.expires;
            return;
        }

        self.source = std::cmp::min(from1.source, from2.source); // the worse of the two input sources
        self.updated = std::cmp::max(from1.updated, from2.updated); // the *later* of the two update times
        self.stale = std::cmp::min(from1.stale, from2.stale); // the earlier of the two stale times
        self.expires = std::cmp::min(from1.expires, from2.expires); // the earlier of the two expiry times
    }

    // track.c:145
    // Note that the rhs doesn't necessarily have to box the same type for this to work
    pub fn compare_validity<U: Default>(&self, rhs: &DataValidityBox<U>, now_ms: u128) -> i8 {
        if now_ms < self.stale && self.source > rhs.source {
            1
        } else if now_ms < rhs.stale && self.source < rhs.source {
            -1
        } else if self.updated > rhs.updated {
            1
        } else if self.updated < rhs.updated {
            -1
        } else {
            0
        }
    }
}

// Distance between points on a spherical earth.
// This has up to 0.5% error because the earth isn't actually spherical
// (but we don't use it in situations where that matters)
// track.c:165
fn greatcircle(lat0_deg: f64, lon0_deg: f64, lat1_deg: f64, lon1_deg: f64) -> f64 {
    let rad_per_deg = PI / 180.0;
    let lat0 = lat0_deg * rad_per_deg;
    let lon0 = lon0_deg * rad_per_deg;
    let lat1 = lat1_deg * rad_per_deg;
    let lon1 = lon1_deg * rad_per_deg;

    let dlat = (lat1 - lat0).abs();
    let dlon = (lon1 - lon0).abs();

    // use haversine for small distances for better numerical stability
    if dlat < 0.001 && dlon < 0.001 {
        let a = (dlat / 2.0).sin() * (dlat / 2.0).sin()
            + lat0.cos() * lat1.cos() * (dlon / 2.0).sin() * (dlon / 2.0).sin();
        return 6371.0e3 * 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    }

    // spherical law of cosines
    6371.0e3 * (lat0.sin() * lat1.sin() + lat0.cos() * lat1.cos() * dlon.cos()).acos()
}

// track.c:520
pub fn update_from_message(mm: &mut ModeSMessage) {
    // Convert 12 MHz clock ticks to milliseconds; the original code uses the system timestamp, but that's not deterministic
    // The `--throttle` command line argument should theoretically make it close, but even that seems to vary enough to make a difference
    let now_ms: u128 = (mm.timestamp_msg as u128) / 12_000;

    // Lookup our aircraft or create a new one
    if let Ok(mut aircraft_list) = AIRCRAFT_LIST.lock() {
        let need_new_entry: bool = aircraft_list.iter().find(|a| a.addr.0 == mm.addr).is_none();
        if need_new_entry {
            // track.c: 59
            let mut aircraft = aircraft::Aircraft::default();

            aircraft.addr = (mm.addr, mm.addrtype.clone().unwrap_or(AddrType::default()));

            // start off with the "last emitted" ACAS RA being blank (just the BDS 3,0
            // or ES type code)
            aircraft.fatsv_emitted_bds_30[0] = 0x30;
            aircraft.fatsv_emitted_es_acas_ra[0] = 0xE2;

            // mm->msgtype 32 is used to represent Mode A/C. These values can never change, so
            // set them once here during initialisation, and don't bother to set them every
            // time this ModeA/C is received again in the future
            if mm.msgtype == 32 {
                aircraft.mode_ac_flags = MODEAC_MSG_FLAG;
                if let None = mm.altitude {
                    aircraft.mode_ac_flags |= MODEAC_MSG_MODEA_ONLY;
                }
            }

            // Copy the first message so we can emit it later when a second message arrives.
            aircraft.first_message = mm.clone();

            aircraft_list.push(aircraft);
        }

        // Update the aircraft track.  It may have been there before or we may have just now
        // created it, but either way, it should be in the list and we need a mutable pointer to it
        let mut a: &mut aircraft::Aircraft = aircraft_list
            .iter_mut()
            .find(|a| a.addr.0 == mm.addr)
            .unwrap();

        if mm.signal_level > 0.0 {
            a.signal_level.push(mm.signal_level);
        }

        a.seen = now_ms;
        a.messages += 1;

        // update addrtype, we only ever go towards "more direct" types
        a.addr = match (a.addr.clone(), mm.addrtype.clone()) {
            ((addr, old_typ), None) => (addr, old_typ),
            ((addr, old_typ), Some(new_typ)) => {
                if new_typ < old_typ {
                    (addr, new_typ)
                } else {
                    (addr, old_typ)
                }
            }
        };

        if let Some((_mm_altitude, _, AltitudeSource::Baro)) = mm.altitude {
            let reset_mode_c_count: Option<bool> = a.altitude.update(&mm, now_ms, |alt_tuple| {
                let (alt, alt_mode_c) = alt_tuple;
                let mode_c = (*alt as u32 + 49) / 100;

                let ans: bool = mode_c != *alt_mode_c;

                // TODO: The compiler says these are never used, but I'm leaving them commented out for now and I'll try to confirm later.
                // alt = mm_altitude;
                // alt_mode_c = mode_c;

                ans
            });

            if let Some(true) = reset_mode_c_count {
                a.mode_c_count = 0; //....zero the hit count
                a.mode_ac_flags &= !MODEAC_MSG_MODEC_HIT;
            }
        }

        if let Some(mm_squawk) = mm.squawk {
            let reset_mode_a_count: Option<bool> = a.squawk.update(&mm, now_ms, |squawk| {
                let ans: bool = *squawk != mm_squawk;

                *squawk = mm_squawk;

                ans
            });

            if let Some(true) = reset_mode_a_count {
                a.mode_a_count = 0; //....zero the hit count
                a.mode_ac_flags &= !MODEAC_MSG_MODEA_HIT;
            }
        }

        if let Some((mm_alt, _, AltitudeSource::GNSS)) = mm.altitude {
            a.altitude_gnss.update(&mm, now_ms, |alt| *alt = mm_alt);
        }

        a.gnss_delta
            .opt_fn_update(&mm, now_ms, &mm.gnss_delta, |x| *x);

        match mm.heading {
            Some((mm_hdg, HeadingSource::True)) => {
                a.heading.update(&mm, now_ms, |hdg| *hdg = mm_hdg);
            }
            Some((mm_hdg, HeadingSource::Magnetic)) => {
                a.heading_magnetic.update(&mm, now_ms, |hdg| *hdg = mm_hdg);
            }
            _ => (),
        }

        match mm.speed {
            Some((mm_spd, SpeedSource::GroundSpeed)) => {
                a.speed.update(&mm, now_ms, |spd| *spd = mm_spd);
            }
            Some((mm_spd, SpeedSource::IAS)) => {
                a.speed_ias.update(&mm, now_ms, |spd| *spd = mm_spd);
            }
            Some((mm_spd, SpeedSource::TAS)) => {
                a.speed_tas.update(&mm, now_ms, |spd| *spd = mm_spd);
            }
            _ => (),
        }

        // Primitives and simple enums implement Copy and can just be dereferenced
        // More complicated datatypes like tuples and Strings need to be explicitly cloned
        // Cloning and/or copying is what we want here because the Aircraft struct is
        // going to live longer than the ModeSMessage
        a.vert_rate
            .opt_fn_update(&mm, now_ms, &mm.vert_rate, |x| x.clone());
        a.category.opt_fn_update(&mm, now_ms, &mm.category, |x| *x);
        a.airground
            .opt_fn_update(&mm, now_ms, &mm.airground, |x| *x);
        a.callsign
            .opt_fn_update(&mm, now_ms, &mm.callsign, |x| x.clone());

        if let Some((mm_cpr_lat, mm_cpr_lon, mm_cpr_odd, mm_cpr_nucp, mm_cpr_type)) = mm.raw_cpr {
            if mm_cpr_odd {
                a.cpr_odd.update(&mm, now_ms, |x| {
                    *x = (mm_cpr_type, mm_cpr_lat, mm_cpr_lon, mm_cpr_nucp)
                });
            } else {
                a.cpr_even.update(&mm, now_ms, |x| {
                    *x = (mm_cpr_type, mm_cpr_lat, mm_cpr_lon, mm_cpr_nucp)
                });
            }
        }

        // Now handle derived data

        // derive GNSS if we have baro + delta
        if a.altitude.compare_validity(&a.altitude_gnss, now_ms) > 0
            && a.gnss_delta.compare_validity(&a.altitude_gnss, now_ms) > 0
        {
            // Baro and delta are both more recent than GNSS, derive GNSS from baro + delta
            if let (Some((baro_alt, _)), Some(gnss_delta)) =
                (a.altitude.get_opt(), a.gnss_delta.get_opt())
            {
                a.altitude_gnss.direct_set(baro_alt + gnss_delta);
            }

            a.altitude_gnss.combine_validity(&a.altitude, &a.gnss_delta);
        }

        // If we've got a new cprlat or cprlon
        if mm.raw_cpr.is_some() {
            a.update_position(mm, now_ms).unwrap();
        }

        if mm.msgtype == 32 {
            let flags = a.mode_ac_flags;
            if (flags & (MODEAC_MSG_MODEC_HIT | MODEAC_MSG_MODEC_OLD)) == MODEAC_MSG_MODEC_OLD {
                //
                // This Mode-C doesn't currently hit any known Mode-S, but it used to because MODEAC_MSG_MODEC_OLD is
                // set  So the aircraft it used to match has either changed altitude, or gone out of our receiver range
                //
                // We've now received this Mode-A/C again, so it must be a new aircraft. It could be another aircraft
                // at the same Mode-C altitude, or it could be a new airctraft with a new Mods-A squawk.
                //
                // To avoid masking this aircraft from the interactive display, clear the MODEAC_MSG_MODES_OLD flag
                // and set messages to 1;
                //
                a.mode_ac_flags = flags & !MODEAC_MSG_MODEC_OLD;
                a.messages = 1;
            }
        }
    }
}
