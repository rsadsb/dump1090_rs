//=========================================================================
//
// Always positive MOD operation, used for CPR decoding.
//
// cpr.c:58
fn cpr_mod_int(a: i32, b: i32) -> i32 {
    let res = a % b;
    if res < 0 {
        res + b
    } else {
        res
    }
}

// cpr.c:64
fn cpr_mod_double(a: f64, b: f64) -> f64 {
    let res: f64 = a % b;
    if res < 0.0 {
        res + b
    } else {
        res
    }
}

//=========================================================================
//
// The NL function uses the precomputed table from 1090-WP-9-14
//
// cpr.c:75
fn cpr_nl_function(lat_signed: f64) -> i32 {
    let lat: f64 = lat_signed.abs(); // Symmetric

    if lat < 10.470_471_30 {
        return 59;
    }
    if lat < 14.828_174_37 {
        return 58;
    }
    if lat < 18.186_263_57 {
        return 57;
    }
    if lat < 21.029_394_93 {
        return 56;
    }
    if lat < 23.545_044_87 {
        return 55;
    }
    if lat < 25.829_247_07 {
        return 54;
    }
    if lat < 27.938_987_10 {
        return 53;
    }
    if lat < 29.911_356_86 {
        return 52;
    }
    if lat < 31.772_097_08 {
        return 51;
    }
    if lat < 33.539_934_36 {
        return 50;
    }
    if lat < 35.228_995_98 {
        return 49;
    }
    if lat < 36.850_251_08 {
        return 48;
    }
    if lat < 38.412_418_92 {
        return 47;
    }
    if lat < 39.922_566_84 {
        return 46;
    }
    if lat < 41.386_518_32 {
        return 45;
    }
    if lat < 42.809_140_12 {
        return 44;
    }
    if lat < 44.194_549_51 {
        return 43;
    }
    if lat < 45.546_267_23 {
        return 42;
    }
    if lat < 46.867_332_52 {
        return 41;
    }
    if lat < 48.160_391_28 {
        return 40;
    }
    if lat < 49.427_764_39 {
        return 39;
    }
    if lat < 50.671_501_66 {
        return 38;
    }
    if lat < 51.893_424_69 {
        return 37;
    }
    if lat < 53.095_161_53 {
        return 36;
    }
    if lat < 54.278_174_72 {
        return 35;
    }
    if lat < 55.443_784_44 {
        return 34;
    }
    if lat < 56.593_187_56 {
        return 33;
    }
    if lat < 57.727_473_54 {
        return 32;
    }
    if lat < 58.847_637_76 {
        return 31;
    }
    if lat < 59.954_592_77 {
        return 30;
    }
    if lat < 61.049_177_74 {
        return 29;
    }
    if lat < 62.132_166_59 {
        return 28;
    }
    if lat < 63.204_274_79 {
        return 27;
    }
    if lat < 64.266_165_23 {
        return 26;
    }
    if lat < 65.318_453_10 {
        return 25;
    }
    if lat < 66.361_710_08 {
        return 24;
    }
    if lat < 67.396_467_74 {
        return 23;
    }
    if lat < 68.423_220_22 {
        return 22;
    }
    if lat < 69.442_426_31 {
        return 21;
    }
    if lat < 70.454_510_75 {
        return 20;
    }
    if lat < 71.459_864_73 {
        return 19;
    }
    if lat < 72.458_845_45 {
        return 18;
    }
    if lat < 73.451_774_42 {
        return 17;
    }
    if lat < 74.438_934_16 {
        return 16;
    }
    if lat < 75.420_562_57 {
        return 15;
    }
    if lat < 76.396_843_91 {
        return 14;
    }
    if lat < 77.367_894_61 {
        return 13;
    }
    if lat < 78.333_740_83 {
        return 12;
    }
    if lat < 79.294_282_25 {
        return 11;
    }
    if lat < 80.249_232_13 {
        return 10;
    }
    if lat < 81.198_013_49 {
        return 9;
    }
    if lat < 82.139_569_81 {
        return 8;
    }
    if lat < 83.071_994_45 {
        return 7;
    }
    if lat < 83.991_735_63 {
        return 6;
    }
    if lat < 84.891_661_91 {
        return 5;
    }
    if lat < 85.755_416_21 {
        return 4;
    }
    if lat < 86.535_369_98 {
        return 3;
    }
    if lat < 87.000_000_00 {
        2
    } else {
        1
    }
}

// cpr.c:140
fn cpr_n_function(lat: f64, fflag: bool) -> i32 {
    let nl = cpr_nl_function(lat) - (if fflag { 1 } else { 0 });
    std::cmp::max(1, nl)
}

fn cpr_dlon_function(lat: f64, fflag: bool, surface: bool) -> f64 {
    (if surface { 90.0 } else { 360.0 }) / f64::from(cpr_n_function(lat, fflag))
}

//
//=========================================================================
//
// This algorithm comes from:
// http://www.lll.lu/~edward/edward/adsb/DecodingADSBposition.html.
//
// A few remarks:
// 1) 131072 is 2^17 since CPR latitude and longitude are encoded in 17 bits.
//
// cpr.c:160
pub fn decode_cpr_airborne(
    even_cprlat: u32,
    even_cprlon: u32,
    odd_cprlat: u32,
    odd_cprlon: u32,
    fflag: bool,
    out_lat: &mut f64,
    out_lon: &mut f64,
) -> i32 {
    let air_dlat0: f64 = 360.0 / 60.0;
    let air_dlat1: f64 = 360.0 / 59.0;
    let lat0: f64 = f64::from(even_cprlat);
    let lat1: f64 = f64::from(odd_cprlat);
    let lon0: f64 = f64::from(even_cprlon);
    let lon1: f64 = f64::from(odd_cprlon);

    // Compute the Latitude Index "j"
    let j = (((59.0 * lat0 - 60.0 * lat1) / 131_072.0) + 0.5).floor() as i32;

    let mut rlat0 = air_dlat0 * (f64::from(cpr_mod_int(j, 60)) + (lat0 / 131_072.0));
    let mut rlat1 = air_dlat1 * (f64::from(cpr_mod_int(j, 59)) + (lat1 / 131_072.0));

    if rlat0 >= 270.0 {
        rlat0 -= 360.0;
    }
    if rlat1 >= 270.0 {
        rlat1 -= 360.0;
    }

    // Check to see that the latitude is in range: -90 .. +90
    if !(-90.0..=90.0).contains(&rlat0) || rlat1 < -90.0 || rlat1 > 90.0 {
        return -2; // bad data
    }

    // Check that both are in the same latitude zone, or abort.
    if cpr_nl_function(rlat0) != cpr_nl_function(rlat1) {
        return -1; // positions crossed a latitude zone, try again later
    }

    // Compute ni and the Longitude Index "m"
    let (mut rlon, rlat) = if fflag {
        // Use odd packet.
        let ni = cpr_n_function(rlat1, true);
        let m = ((((lon0 * (f64::from(cpr_nl_function(rlat1)) - 1.0))
            - (lon1 * f64::from(cpr_nl_function(rlat1))))
            / 131_072.0)
            + 0.5)
            .floor() as i32;

        (
            cpr_dlon_function(rlat1, true, false)
                * (f64::from(cpr_mod_int(m, ni)) + (lon1 / 131_072.0)),
            rlat1,
        )
    } else {
        // Use even packet.
        let ni = cpr_n_function(rlat0, false);
        let m = ((((lon0 * f64::from(cpr_nl_function(rlat0) - 1))
            - (lon1 * f64::from(cpr_nl_function(rlat0))))
            / 131_072.0)
            + 0.5)
            .floor() as i32;

        (
            cpr_dlon_function(rlat0, false, false)
                * (f64::from(cpr_mod_int(m, ni)) + (lon0 / 131_072.0)),
            rlat0,
        )
    };

    // Renormalize to -180 .. +180
    rlon -= ((rlon + 180.0) / 360.0).floor() * 360.0;

    *out_lat = rlat;
    *out_lon = rlon;

    0
}

//=========================================================================
//
// This algorithm comes from:
// 1090-WP29-07-Draft_CPR101 (which also defines decodeCPR() )
//
// Despite what the earlier comment here said, we should *not* be using trunc().
// See Figure 5-5 / 5-6 and note that floor is applied to (0.5 + fRP - fEP), not
// directly to (fRP - fEP). Eq 38 is correct.
//
// cpr.c:323
pub fn decode_cpr_relative(
    reflat: f64,
    reflon: f64,
    cprlat: u32,
    cprlon: u32,
    fflag: bool,
    surface: bool,
) -> Result<(f64, f64), &'static str> {
    let air_dlat = (if surface { 90.0 } else { 360.0 }) / (if fflag { 59.0 } else { 60.0 });

    let fractional_lat: f64 = f64::from(cprlat) / 131_072.0;
    let fractional_lon: f64 = f64::from(cprlon) / 131_072.0;

    // Compute the Latitude Index "j"
    let j: i32 = ((reflat / air_dlat).floor()
        + (0.5 + (cpr_mod_double(reflat, air_dlat) / air_dlat) - fractional_lat).floor())
        as i32;

    let mut rlat: f64 = air_dlat * (f64::from(j) + fractional_lat);
    if rlat >= 270.0 {
        rlat -= 360.0;
    }

    if !(-90.0..=90.0).contains(&rlat) {
        return Err("Invalid latitude in decode_cpr_relative (out of range)");
    }
    if (rlat - reflat).abs() > air_dlat / 2.0 {
        return Err("Invalid latitude in decode_cpr_relative (more than 1/2 cell away)");
    }

    // Compute the Longitude Index "m"
    let air_dlon = cpr_dlon_function(rlat, fflag, surface);
    let m: i32 = ((reflon / air_dlon).floor()
        + (0.5 + (cpr_mod_double(reflon, air_dlon) / air_dlon) - fractional_lon).floor())
        as i32;

    let mut rlon: f64 = air_dlon * (f64::from(m) + fractional_lon);
    if rlon > 180.0 {
        rlon -= 360.0;
    }

    if (rlon - reflon).abs() > air_dlon / 2.0 {
        return Err("Invalid longitude in decode_cpr_relative (more than 1/2 cell away)");
    }

    Ok((rlat, rlon))
}
