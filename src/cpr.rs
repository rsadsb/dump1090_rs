
//=========================================================================
//
// Always positive MOD operation, used for CPR decoding.
//
// cpr.c:58
fn cpr_mod_int(a:i32, b:i32) -> i32 {
    let res = a % b;
    if res < 0 { res + b } else { res }
}

// cpr.c:64
fn cpr_mod_double(a:f64, b:f64) -> f64 {
    let res:f64 = a % b;
    if res < 0.0 { res + b } else { res }
}

//=========================================================================
//
// The NL function uses the precomputed table from 1090-WP-9-14
//
// cpr.c:75
fn cpr_nl_function(lat_signed:f64) -> i32 {

	let lat:f64 = lat_signed.abs();		// Symmetric

    if lat < 10.47047130 {return 59;}
    if lat < 14.82817437 {return 58;}
    if lat < 18.18626357 {return 57;}
    if lat < 21.02939493 {return 56;}
    if lat < 23.54504487 {return 55;}
    if lat < 25.82924707 {return 54;}
    if lat < 27.93898710 {return 53;}
    if lat < 29.91135686 {return 52;}
    if lat < 31.77209708 {return 51;}
    if lat < 33.53993436 {return 50;}
    if lat < 35.22899598 {return 49;}
    if lat < 36.85025108 {return 48;}
    if lat < 38.41241892 {return 47;}
    if lat < 39.92256684 {return 46;}
    if lat < 41.38651832 {return 45;}
    if lat < 42.80914012 {return 44;}
    if lat < 44.19454951 {return 43;}
    if lat < 45.54626723 {return 42;}
    if lat < 46.86733252 {return 41;}
    if lat < 48.16039128 {return 40;}
    if lat < 49.42776439 {return 39;}
    if lat < 50.67150166 {return 38;}
    if lat < 51.89342469 {return 37;}
    if lat < 53.09516153 {return 36;}
    if lat < 54.27817472 {return 35;}
    if lat < 55.44378444 {return 34;}
    if lat < 56.59318756 {return 33;}
    if lat < 57.72747354 {return 32;}
    if lat < 58.84763776 {return 31;}
    if lat < 59.95459277 {return 30;}
    if lat < 61.04917774 {return 29;}
    if lat < 62.13216659 {return 28;}
    if lat < 63.20427479 {return 27;}
    if lat < 64.26616523 {return 26;}
    if lat < 65.31845310 {return 25;}
    if lat < 66.36171008 {return 24;}
    if lat < 67.39646774 {return 23;}
    if lat < 68.42322022 {return 22;}
    if lat < 69.44242631 {return 21;}
    if lat < 70.45451075 {return 20;}
    if lat < 71.45986473 {return 19;}
    if lat < 72.45884545 {return 18;}
    if lat < 73.45177442 {return 17;}
    if lat < 74.43893416 {return 16;}
    if lat < 75.42056257 {return 15;}
    if lat < 76.39684391 {return 14;}
    if lat < 77.36789461 {return 13;}
    if lat < 78.33374083 {return 12;}
    if lat < 79.29428225 {return 11;}
    if lat < 80.24923213 {return 10;}
    if lat < 81.19801349 {return 9;}
    if lat < 82.13956981 {return 8;}
    if lat < 83.07199445 {return 7;}
    if lat < 83.99173563 {return 6;}
    if lat < 84.89166191 {return 5;}
    if lat < 85.75541621 {return 4;}
    if lat < 86.53536998 {return 3;}
    if lat < 87.00000000 {return 2;}
    else {return 1;}
}

// cpr.c:140
fn cpr_n_function(lat:f64, fflag:bool) -> i32 {
    let nl = cpr_nl_function(lat) - (if fflag { 1 } else { 0 });
    std::cmp::max(1, nl)
}

fn cpr_dlon_function(lat:f64, fflag:bool, surface:bool) -> f64 {
    (if surface { 90.0 } else { 360.0 }) / (cpr_n_function(lat, fflag) as f64)
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
pub fn decode_cpr_airborne(even_cprlat:u32, even_cprlon:u32, odd_cprlat:u32, odd_cprlon:u32,
                           fflag:bool, out_lat:&mut f64, out_lon:&mut f64) -> i32
{
    let air_dlat0:f64 = 360.0 / 60.0;
    let air_dlat1:f64 = 360.0 / 59.0;
    let lat0:f64 = even_cprlat as f64;
    let lat1:f64 = odd_cprlat  as f64;
    let lon0:f64 = even_cprlon as f64;
    let lon1:f64 = odd_cprlon  as f64;

    // Compute the Latitude Index "j"
    let j = (((59.0*lat0 - 60.0*lat1) / 131072.0) + 0.5).floor() as i32;
    
    let mut rlat0 = air_dlat0 * (cpr_mod_int(j,60) as f64 + (lat0 / 131072.0));
    let mut rlat1 = air_dlat1 * (cpr_mod_int(j,59) as f64 + (lat1 / 131072.0));

    if rlat0 >= 270.0 { rlat0 -= 360.0; }
    if rlat1 >= 270.0 { rlat1 -= 360.0; }

    // Check to see that the latitude is in range: -90 .. +90
    if rlat0 < -90.0 || rlat0 > 90.0 || rlat1 < -90.0 || rlat1 > 90.0 {
        return -2; // bad data
    }

    // Check that both are in the same latitude zone, or abort.
    if cpr_nl_function(rlat0) != cpr_nl_function(rlat1) {
        return -1; // positions crossed a latitude zone, try again later
    }

    // Compute ni and the Longitude Index "m"
    let (mut rlon, rlat) = if fflag { // Use odd packet.
        let ni = cpr_n_function(rlat1, true);
        let m  = ((((lon0 * (cpr_nl_function(rlat1) as f64 - 1.0)) - (lon1 * cpr_nl_function(rlat1) as f64)) / 131072.0) + 0.5).floor() as i32;
        
        (cpr_dlon_function(rlat1, true, false) * ((cpr_mod_int(m, ni) as f64) + (lon1/131072.0)), rlat1)
    } else {     // Use even packet.
        let ni = cpr_n_function(rlat0, false);
        let m  = ((((lon0 * (cpr_nl_function(rlat0)-1) as f64) - (lon1 * cpr_nl_function(rlat0) as f64)) / 131072.0) + 0.5).floor() as i32;
        
        (cpr_dlon_function(rlat0, false, false) * ((cpr_mod_int(m, ni) as f64) + (lon0/131072.0)), rlat0)
    };

    // Renormalize to -180 .. +180
    rlon -= ( (rlon + 180.0) / 360.0 ).floor() * 360.0;

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
pub fn decode_cpr_relative(reflat:f64, reflon:f64, cprlat:u32, cprlon:u32,
                           fflag:bool, surface:bool) -> Result<(f64, f64), &'static str>
{
    let air_dlat = (if surface { 90.0 } else { 360.0 }) / (if fflag { 59.0 } else { 60.0 });

    let fractional_lat:f64 = cprlat as f64 / 131072.0;
    let fractional_lon:f64 = cprlon as f64 / 131072.0;

    // Compute the Latitude Index "j"
    let j:i32 = ((reflat/air_dlat).floor() + (0.5 + (cpr_mod_double(reflat, air_dlat)/air_dlat) - fractional_lat).floor()) as i32;

    let mut rlat:f64 = air_dlat * (j as f64 + fractional_lat);
    if rlat >= 270.0 { rlat -= 360.0; }

    if rlat < -90.0 || rlat > 90.0            { return Err("Invalid latitude in decode_cpr_relative (out of range)");            }
    if (rlat - reflat).abs() > air_dlat / 2.0 { return Err("Invalid latitude in decode_cpr_relative (more than 1/2 cell away)"); }

    // Compute the Longitude Index "m"
    let air_dlon = cpr_dlon_function(rlat, fflag, surface);
    let m:i32 = ((reflon/air_dlon).floor() + (0.5 + (cpr_mod_double(reflon, air_dlon)/air_dlon) - fractional_lon).floor()) as i32;
    
    let mut rlon:f64 = air_dlon * (m as f64 + fractional_lon);
    if rlon > 180.0 { rlon -= 360.0; }

    if (rlon - reflon).abs() > air_dlon / 2.0 { 
        return Err("Invalid longitude in decode_cpr_relative (more than 1/2 cell away)"); 
    }

    Ok((rlat, rlon))

}
