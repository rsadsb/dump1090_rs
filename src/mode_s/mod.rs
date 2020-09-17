
// This module includes functionality translated from mode_s.c

use itertools::Itertools;

use crate::{ModeSMessage, 
    AltitudeUnit, AltitudeSource, AddrType, SpeedSource, HeadingSource, AirGround, CprType, SilType, AngleType,
	MODES_LONG_MSG_BYTES, MODES_SHORT_MSG_BYTES, MODES_NON_ICAO_ADDRESS};
use crate::track;

pub const MAGIC_MLAT_TIMESTAMP:usize = 0xFF004D4C4154;

#[derive(Debug)]
// Navigation Uncertainty Category (NUC)
pub struct Position { lat_deg:f64, lon_deg:f64, nuc:u32 }

// Private module just to break up this file
mod decode;

// mode_s.c:77
pub fn modes_message_len_by_type(typ:u8) -> usize {
    if typ & 0x10 != 0 {
    	MODES_LONG_MSG_BYTES * 8
    } else { 
    	MODES_SHORT_MSG_BYTES * 8
    }
}

pub fn getbit(data:&[u8], bit_1idx:usize) -> usize {
	getbits(data, bit_1idx, bit_1idx)
}

// mode_s.c:215
pub fn getbits(data:&[u8], firstbit_1idx:usize, lastbit_1idx:usize) -> usize {
	let mut ans:usize = 0;

	// The original code uses indices that start at 1 and we need 0-indexed values
	let (firstbit, lastbit) = (firstbit_1idx-1, lastbit_1idx-1);

	for bit_idx in firstbit..=lastbit {
		ans *= 2;
		let byte_idx:usize = bit_idx/8;
		let mask = 2u8.pow(7u32 - (bit_idx as u32)%8);
		if (data[byte_idx] & mask) != 0u8 {
			ans += 1;
		}
	}

	ans
}

// mode_s.c:289
pub fn score_modes_message(msg:&[u8]) -> i32 {

	let validbits = msg.len() * 8;

	if validbits < 56 { return -2; }

	let msgtype = getbits(&msg, 1, 5);
	let msgbits = if (msgtype & 0x10) != 0 { 
		MODES_LONG_MSG_BYTES*8 
	} else {
		MODES_SHORT_MSG_BYTES*8
	};

	if validbits < msgbits { return -2; }
	if msg.iter().all(|b| *b == 0x00) { return -2; }

	let crc = super::crc::modes_checksum(msg, msgbits);

	match msgtype {
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

		    if super::icao_filter::icao_filter_test(crc) { 1000 } else { -1 }
		},
		11 => {
			// 11: All-call reply
	        let iid = crc & 0x7f;
	        let crc = crc & 0xffff80;
	        let addr = getbits(msg, 9, 32) as u32;
	
			match (crc, iid, super::icao_filter::icao_filter_test(addr)) {
				(0, 0, true ) => 1600,
				(0, 0, false) => 750,
				(0, _, true ) => 1000,
				(0, _, false) => -1,
				(_, _, _    ) => -2,
			}
		},
		17 | 18 => {
		    // 17: Extended squitter
		    // 18: Extended squitter/non-transponder
	        let addr = getbits(msg, 9, 32) as u32;

	        match (crc, super::icao_filter::icao_filter_test(addr)) {
	        	(0, true ) => 1800,
	        	(0, false) => 1400,
	        	(_, _    ) => -2
	        }
		},
		20 | 21 => {
		    // 20: Comm-B, altitude reply
		    // 21: Comm-B, identity reply
		    match super::icao_filter::icao_filter_test(crc) {
		    	true  => 1000,
		    	false => -2
		    }
		},
		_ => -2
	}
}

// mode_s.c:387
pub fn decode_mode_s_message(mm:&mut ModeSMessage) -> Result<(), &'static str> {
    decode::decode(mm)
}

// mode_s.c: 1289
// TODO: replace this with an implementation of the trait that formats a type as {}
fn addrtype_to_string(typ:&AddrType) -> &'static str {
    match typ {
        AddrType::ADSB_ICAO       => "Mode S / ADS-B", 
        AddrType::ADSB_ICAO_NT    => "ADS-B, non-transponder", 
        AddrType::ADSR_ICAO       => "ADS-R", 
        AddrType::TISB_ICAO       => "TIS-B", 
        AddrType::ADSB_Other      => "ADS-B, other addressing scheme", 
        AddrType::ADSR_Other      => "ADS-R, other addressing scheme", 
        AddrType::TISB_Trackfile  => "TIS-B, Mode A code and track file number", 
        AddrType::TISB_Other      => "TIS-B, other addressing scheme", 
        AddrType::Unknown         => "unknown addressing scheme"
    }
}



// mode_s.c:1325
fn hex_str(data:&[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).join("")
}

// mode_s.c:1332
fn es_type_has_subtype(metype:u8) -> bool {
    if metype <= 18 { return false; }

    if metype >= 20 && metype <= 22 {
        return false;
    }

    true
}

fn es_type_name(metype:u8, mesub:u8) -> &'static str { 
    // Nested OR patterns are experimental, so handle these cases at the top
    if let 1 | 2 | 3 | 4 = metype { return "Aircraft identification and category"; }
    if let 5 | 6 | 7 | 8 = metype { return "Surface position"; }
    if let 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 = metype { return "Airborne position (barometric altitude)"; }
    if let 20 | 21 | 22 = metype { return "Airborne position (GNSS altitude)"; }

    match (metype, mesub) {
        ( 0, _) => "No position information (airborne or surface)",
        (19, 1) => "Airborne velocity over ground, subsonic",
        (19, 2) => "Airborne velocity over ground, supersonic",
        (19, 3) => "Airspeed and heading, subsonic",
        (19, 4) => "Airspeed and heading, supersonic",
        (19, _) => "Unknown",
        (23, 0) => "Test message",
        (23, 7) => "National use / 1090-WP-15-20 Mode A squawk",
        (23, _) => "Unknown",
        (24, _) => "Reserved for surface system status",
        (27, _) => "Reserved for trajectory change",
        (28, 1) => "Emergency/priority status",
        (28, 2) => "ACAS RA broadcast",
        (28, _) => "Unknown",
        (29, 0) => "Target state and status (V1)",
        (29, 1) => "Target state and status (V2)",
        (29, _) => "Unknown",
        (30, _) => "Aircraft Operational Coordination",
        (31, 0) => "Aircraft operational status (airborne)",
        (31, 1) => "Aircraft operational status (surface)",
        (31, _) => "Unknown",
        ( _, _) => "Unknown"
    }
}

fn df_to_string(df:u8) -> &'static str { match df {
    0  => "Short Air-Air Surveillance",
    4  => "Survelliance, Altitude Reply",
    5  => "Survelliance, Identity Reply",
    11 => "All Call Reply",
    16 => "Long Air-Air ACAS",
    17 => "Extended Squitter",
    18 => "Extended Squitter (Non-Transponder)",
    19 => "Extended Squitter (Military)",
    20 => "Comm-B, Altitude Reply",
    21 => "Comm-B, Identity Reply",
    22 => "Military Use",
    24 => "Comm-D Extended Length Message",
    25 => "Comm-D Extended Length Message",
    26 => "Comm-D Extended Length Message",
    27 => "Comm-D Extended Length Message",
    28 => "Comm-D Extended Length Message",
    29 => "Comm-D Extended Length Message",
    30 => "Comm-D Extended Length Message",
    31 => "Comm-D Extended Length Message",
    32 => "Mode A/C Reply",
    1 | 2 | 3 | 6 | 7 | 8 | 9 | 10 | 12 | 13 | 14 | 15 | 23 => "reserved",
    _  => "out of range"
}}

// mode_s.c:1433
pub fn display_mode_s_message(mm:&ModeSMessage) {
    // Show the raw message.
    print!("*");
    // eprint!("*");

    for j in 0..(mm.msgbits/8) { 
        print!("{:02x}", mm.msg[j]); 
        // eprint!("{:02x}", mm.msg[j]); 
    }
    
    print!(";\n");
    // eprint!(";\n");

    if mm.msgtype < 32 { println!("CRC: {:06x}", mm.crc); }

    if mm.signal_level > 0.0 { println!("RSSI: {:.1} dBFS", 10.0 * mm.signal_level.log10()); }

    if mm.score != 0 { println!("Score: {}", mm.score); }

    if mm.timestamp_msg != 0 {
        if mm.timestamp_msg == MAGIC_MLAT_TIMESTAMP {
            println!("This is a synthetic MLAT message.");
        } else {
            println!("Time: {:.2}us", mm.timestamp_msg as f64 / 12.0);
        }
    }

    match mm.msgtype {
        0  => println!("DF:0 addr:{:06X} VS:{} CC:{} SL:{} RI:{} AC:{}", mm.addr, mm.vs, mm.cc, mm.sl, mm.ri, mm.ac),
        4  => println!("DF:4 addr:{:06X} FS:{} DR:{} UM:{} AC:{}", mm.addr, mm.fs, mm.dr, mm.um, mm.ac),
        5  => println!("DF:5 addr:{:06X} FS:{} DR:{} UM:{} ID:{}", mm.addr, mm.fs, mm.dr, mm.um, mm.id),
        11 => println!("DF:11 AA:{:06X} IID:{} CA:{}", mm.aa, mm.iid, mm.ca),
        16 => println!("DF:16 addr:{:06x} VS:{} SL:{} RI:{} AC:{} MV:{}", mm.addr, mm.vs, mm.sl, mm.ri, mm.ac, hex_str(&mm.mv)),
        17 => println!("DF:17 AA:{:06X} CA:{} ME:{}", mm.aa, mm.ca, hex_str(&mm.me)),
        18 => println!("DF:18 AA:{:06X} CF:{} ME:{}", mm.aa, mm.cf, hex_str(&mm.me)),
        20 => println!("DF:20 addr:{:06X} FS:{} DR:{} UM:{} AC:{} MB:{}", mm.addr, mm.fs, mm.dr, mm.um, mm.ac, hex_str(&mm.mb)),
        21 => println!("DF:21 addr:{:06x} FS:{} DR:{} UM:{} ID:{} MB:{}", mm.addr, mm.fs, mm.dr, mm.um, mm.id, hex_str(&mm.mb)),
        24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 => println!("DF:24 addr:{:06x} KE:{} ND:{} MD:{}", mm.addr, mm.ke, mm.nd, hex_str(&mm.md)),
        _  => ()
    }

    print!(" {}", df_to_string(mm.msgtype));

    if let 17 | 18 = mm.msgtype {
        if es_type_has_subtype(mm.metype) {
            print!(" {} ({}/{})", es_type_name(mm.metype, mm.mesub), mm.metype, mm.mesub);
        } else {
            print!(" {} ({})", es_type_name(mm.metype, mm.mesub), mm.metype);
        }
    }
    print!("\n");

    {    
        // Note: I'm not sure if defaulting to ADSB_ICAO is really what we want, but it seems to be what the C code
        // does.  It represents this field with an enum where 0 is ADSB_ICAO
        let addrtype = mm.addrtype.clone().unwrap_or(AddrType::ADSB_ICAO);
        if mm.addr & MODES_NON_ICAO_ADDRESS != 0 {
            println!("  Other Address: {:06X} ({})", mm.addr & 0xFFFFFF, addrtype_to_string(&addrtype));
        } else {
            println!("  ICAO Address:  {:06X} ({})", mm.addr, addrtype_to_string(&addrtype));
        }
    }

    match mm.airground {
        None => (),
        Some(AirGround::Ground)    => println!("  Air/Ground:    ground"),
        Some(AirGround::Airborne)  => println!("  Air/Ground:    airborne"),
        Some(AirGround::Invalid)   => println!("  Air/Ground:    invalid"),
        Some(AirGround::Uncertain) => println!("  Air/Ground:    airborne?"),
    }

    if let Some((h, unit, src)) = &mm.altitude {
        let unit_str:&'static str = match unit {
            AltitudeUnit::Feet   => "ft",
            AltitudeUnit::Meters => "m"
        };
        let src_str:&'static str = match src {
            AltitudeSource::Baro => "barometric",
            AltitudeSource::GNSS => "GNSS"
        };
        println!("  Altitude:      {} {} {}", h, unit_str, src_str);
    }

    if let Some(d)        = mm.gnss_delta { println!("  GNSS delta:    {} ft", d);  }
    if let Some((hdg, _)) = mm.heading    { println!("  Heading:       {}", hdg);   }

    match mm.speed {
        Some((v, SpeedSource::GroundSpeed)) => println!("  Speed:         {} kt groundspeed", v),
        Some((v, SpeedSource::IAS))         => println!("  Speed:         {} kt IAS", v),
        Some((v, SpeedSource::TAS))         => println!("  Speed:         {} kt TAS", v),
        None                                => (),
    }

    match mm.vert_rate {
        Some((rate, AltitudeSource::Baro)) => println!("  Vertical rate: {} ft/min barometric", rate),
        Some((rate, AltitudeSource::GNSS)) => println!("  Vertical rate: {} ft/min GNSS", rate),
        None => ()
    }

    if let Some(squawk)   =  mm.squawk   { println!("  Squawk:        {:04x}", squawk);   }
    if let Some(callsign) = &mm.callsign { println!("  Ident:         {}",     callsign); }
    if let Some(category) =  mm.category { println!("  Category:      {:02X}", category); }

    if let Some((cpr_lat, cpr_lon, cpr_odd, nucp, typ)) = &mm.raw_cpr {
        let typ_str = match typ {
            CprType::Surface  => "Surface",
            CprType::Airborne => "Airborne",
            CprType::Coarse   => "TIS-B Coarse"
        };
        print!("  CPR type:      {}\n  CPR odd flag:  {}\n  CPR NUCp/NIC:  {}\n",
               typ_str, if *cpr_odd { "odd" } else { "even" }, nucp);

        if let Some((lat, lon, is_relative)) = mm.decoded_cpr {
            print!("  CPR latitude:  {:.5} ({})\n  CPR longitude: {:.5} ({})\n  CPR decoding:  {}\n",
                   lat, cpr_lat, lon, cpr_lon, if is_relative { "local" } else { "global" });
        } else {
            print!("  CPR latitude:  ({})\n  CPR longitude: ({})\n  CPR decoding:  none\n", cpr_lat, cpr_lon);
        }
    }

    if let Some(opstatus) = &mm.opstatus {
        println!("  Aircraft Operational Status:");
        println!("    Version:            {}", opstatus.version);

        print!("    Capability classes: ");
        if opstatus.cc_acas                { print!("ACAS ");    }
        if opstatus.cc_cdti                { print!("CDTI ");    }
        if opstatus.cc_1090_in             { print!("1090IN ");  }
        if opstatus.cc_arv                 { print!("ARV ");     }
        if opstatus.cc_ts                  { print!("TS ");      }
        if opstatus.cc_tc != 0             { print!("TC={} ",         opstatus.cc_tc);             }
        if opstatus.cc_uat_in              { print!("UATIN ");   }
        if opstatus.cc_poa                 { print!("POA ");     }
        if opstatus.cc_b2_low              { print!("B2-LOW ");  }
        if opstatus.cc_nac_v != 0          { print!("NACv={} ",       opstatus.cc_nac_v);          }
        if opstatus.cc_nic_supp_c          { print!("NIC-C=1 "); }
        if opstatus.cc_lw_valid            { print!("L/W={} ",        opstatus.cc_lw);             }
        if opstatus.cc_antenna_offset != 0 { print!("GPS-OFFSET={} ", opstatus.cc_antenna_offset); }
        print!("\n");

        print!("    Operational modes:  ");
        if opstatus.om_acas_ra  { print!("ACASRA "); }
        if opstatus.om_ident    { print!("IDENT ");  }
        if opstatus.om_atc      { print!("ATC ");    }
        if opstatus.om_saf      { print!("SAF ");    }
        if opstatus.om_sda != 0 { print!("SDA={} ", opstatus.om_sda); }
        print!("\n");

        if opstatus.nic_supp_a { println!("    NIC-A:              {}", if opstatus.nic_supp_a {1} else {0} ); }
        if opstatus.nac_p      != 0 { println!("    NACp:               {}", opstatus.nac_p);      }
        if opstatus.gva        != 0 { println!("    GVA:                {}", opstatus.gva);        }
        if opstatus.sil        != 0 {
            let sil_type_str = match &opstatus.sil_type {
                SilType::SilPerHour => "per hour",
                SilType::SilPerSample => "per sample"
            };
            println!("    SIL:                {} ({})", opstatus.sil, sil_type_str);
        }
        if opstatus.nic_baro { println!("    NICbaro:            {}", if opstatus.nic_baro {1} else {0} );   }

        if mm.mesub == 1 {
            println!("    Heading type:      {}",  match opstatus.track_angle { AngleType::Heading => "heading", _ => "track angle" });
        }
        println!("    Heading reference:  {}",  match opstatus.hrd { HeadingSource::True => "true north", _ => "magnetic north"});
    }

    if let Some(tss) = &mm.tss {
        println!("  Target State and Status:");
        if let Some((alt, typ)) = &tss.altitude {
            println!("    Target altitude:   {:?}, {} ft", typ, alt);
        }
        if let Some(baro) = &tss.baro {
            println!("    Altimeter setting: {:.1} millibars", baro);
        }
        if let Some(hdg) = &tss.heading {
            println!("    Target heading:    {}", hdg);
        }
        if tss.mode_valid {
            print!("    Active modes:      ");
            if tss.mode_autopilot { print!("autopilot ");     } 
            if tss.mode_vnav      { print!("VNAV ");          }
            if tss.mode_alt_hold  { print!("altitude-hold "); }
            if tss.mode_approach  { print!("approach ");      }
            print!("\n");
        }

        let sil_type_str = match &tss.sil_type {
            SilType::SilPerHour => "per hour",
            SilType::SilPerSample => "per sample"
        };

        println!("    ACAS:              {}", if tss.acas_operational { "operational" } else { "NOT operational" });
        println!("    NACp:              {}", tss.nac_p);
        println!("    NICbaro:           {}", if tss.nic_baro {1} else {0});
        println!("    SIL:               {} ({})", tss.sil, sil_type_str); // from opstatus?
    }

    print!("\n");
}

// mode_s.c:1707
pub fn use_mode_s_message(mm:&mut ModeSMessage) {   

    // Track aircraft state
    track::update_from_message(mm);

    // // In non-interactive non-quiet mode, display messages on standard output
    // if (!Modes.interactive && !Modes.quiet && (!Modes.show_only || mm->addr == Modes.show_only)) {
    display_mode_s_message(mm);
    // }

}
