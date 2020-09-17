
/* 
This crate is meant to be a direct C-to-Rust translation of the algorithms in the popular dump1090 program.
It was developed by referencing the version found at https://github.com/adsbxchange/dump1090-mutability
It matches bit-for-bit in almost every case, but there may be some edge cases where handling of rounding, non-deterministic
timing, and things like that might give results that are not quite identical.
*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate lazy_static;

pub mod cpr;
pub mod crc;
pub mod demod_2400;
pub mod icao_filter;
pub mod mode_ac;
pub mod mode_s;
pub mod rtlsdr;
pub mod track;

pub const MODES_MAG_BUF_SAMPLES:usize = 131072;

// dump1090.h:101
pub const MODEAC_MSG_SAMPLES:u32       = 50;     // include up to the SPI bit
pub const MODEAC_MSG_BYTES:u32         = 2;
pub const MODEAC_MSG_SQUELCH_LEVEL:u32 = 0x07FF; // Average signal strength limit
pub const MODEAC_MSG_FLAG:u32          = 1;
pub const MODEAC_MSG_MODES_HIT:u32     = 2;
pub const MODEAC_MSG_MODEA_HIT:u32     = 4;
pub const MODEAC_MSG_MODEC_HIT:u32     = 8;
pub const MODEAC_MSG_MODEA_ONLY:u32    = 16;
pub const MODEAC_MSG_MODEC_OLD:u32     = 32;

pub const TRAILING_SAMPLES:usize      = 326;
pub const MODES_LONG_MSG_BYTES:usize  = 14;
pub const MODES_SHORT_MSG_BYTES:usize = 7;

pub const MODES_NON_ICAO_ADDRESS:u32 = 16777216;

lazy_static! {
	pub static ref MAG_LUT:Vec<u16> = {
		let mut ans:Vec<u16> = vec![];

		for i in 0..256 {
			for q in 0..256 {
		        let fi = (i as f32 - 127.5) / 127.5;
		        let fq = (q as f32 - 127.5) / 127.5;
		        let magsq:f32 = match fi*fi + fq*fq {
		        	x if x > 1.0 => 1.0,
		        	x => x
		        };
		        let mag_f32 = magsq.sqrt();
		        let mag_f32_scaled = mag_f32 * 65535.0;
		        let mag_f32_rounded = mag_f32_scaled.round();
		    
		        let mag:u16 = mag_f32_rounded as u16;

		        ans.push(mag);
			}
		}

		ans
	};	
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataSource { Invalid, MLAT, ModeS, ModeSChecked, TISB, ADSB }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddrType {  ADSB_ICAO, ADSB_ICAO_NT, ADSR_ICAO, TISB_ICAO, ADSB_Other, ADSR_Other, TISB_Trackfile, TISB_Other, Unknown }

#[derive(Debug, Clone, Copy)]
pub enum AltitudeUnit { Feet, Meters }

#[derive(Debug, Clone)]	// dump1090.h:170
pub enum AltitudeSource { Baro, GNSS }

#[derive(Debug, Clone, Copy)]	// dump1090.h:175
pub enum AirGround { Invalid, Ground, Airborne, Uncertain }

#[derive(Debug, Clone)]	// dump1090.h:182
pub enum SpeedSource { GroundSpeed, IAS, TAS }

#[derive(Debug, Clone)]
pub enum HeadingSource { True, Magnetic }

#[derive(Debug, Clone)]
pub enum SilType { SilPerSample, SilPerHour }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CprType { Surface, Airborne, Coarse }

#[derive(Debug, Clone)]
pub enum TssAltitudeType { MCP, FMS }

#[derive(Debug, Clone)]
pub enum AngleType { Track, Heading }

impl Default for AddrType       { fn default() -> AddrType       { AddrType::Unknown     } }
impl Default for AirGround      { fn default() -> AirGround      { AirGround::Uncertain  } }
impl Default for AltitudeSource { fn default() -> AltitudeSource { AltitudeSource::Baro  } }
impl Default for CprType        { fn default() -> CprType        { CprType::Coarse       } }
impl Default for DataSource     { fn default() -> DataSource     { DataSource::Invalid   } }
impl Default for HeadingSource  { fn default() -> HeadingSource  { HeadingSource::True   } }
impl Default for SilType        { fn default() -> SilType        { SilType::SilPerSample } }
impl Default for AngleType      { fn default() -> AngleType      { AngleType::Track      } }

// dump1090.h:252
pub struct MagnitudeBuffer {
	pub data: Box<[u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES]>,
	pub length: usize,
	pub first_sample_timestamp_12mhz: usize,
	pub dropped: usize,
	pub total_power: f64
}

impl Default for MagnitudeBuffer {
	fn default() -> MagnitudeBuffer { MagnitudeBuffer { 
		data: Box::new([0u16; TRAILING_SAMPLES + MODES_MAG_BUF_SAMPLES]),
		length:0, first_sample_timestamp_12mhz:0, dropped:0, total_power:0.0
	}}
}

impl MagnitudeBuffer {
	
	pub fn push(&mut self, x:u16) {
		self.data[TRAILING_SAMPLES + self.length] = x;
		self.length += 1;
	}

}

#[derive(Default)]
pub struct Modes {
	pub mag_buffer_a:MagnitudeBuffer,
	pub mag_buffer_b:MagnitudeBuffer,
	pub use_buffer_a_next:bool
}

impl Modes {
	
	pub fn next_buffer(&mut self, fs:usize) -> &mut MagnitudeBuffer {

		if self.use_buffer_a_next { 
	
			self.mag_buffer_a.first_sample_timestamp_12mhz = self.mag_buffer_b.first_sample_timestamp_12mhz + ((12_000_000 * self.mag_buffer_b.length) / fs);
			if self.mag_buffer_b.length > 0 {
				let n = self.mag_buffer_b.length;
				self.mag_buffer_a.data[..TRAILING_SAMPLES].clone_from_slice(&self.mag_buffer_b.data[(n-TRAILING_SAMPLES)..n])
			};
			self.mag_buffer_a.length = 0;

			// Switch the active buffer for the next call
			self.use_buffer_a_next = false;

			&mut self.mag_buffer_a		

		} else { 

			self.mag_buffer_b.first_sample_timestamp_12mhz = self.mag_buffer_a.first_sample_timestamp_12mhz + ((12_000_000 * self.mag_buffer_a.length) / fs);
			if self.mag_buffer_a.length > 0 {
				let n = self.mag_buffer_a.length;
				self.mag_buffer_b.data[..TRAILING_SAMPLES].clone_from_slice(&self.mag_buffer_a.data[(n-TRAILING_SAMPLES)..n])
			};
			self.mag_buffer_b.length = 0;

			// Switch the active buffer for the next call
			self.use_buffer_a_next = true;

			&mut self.mag_buffer_b		

		}		

	}

}

#[derive(Debug, Default, Clone)]	// dump1090.h:380
pub struct ModeSMessage {
	pub msg:Vec<u8>,
	pub msgbits:usize,
	pub msgtype:u8,
	pub crc:u32,
	pub addr:u32,
	pub addrtype:Option<AddrType>,
	pub timestamp_msg:usize,
	pub remote:bool,
	pub signal_level:f64,
	pub score:i32,
	pub source:DataSource,

	// Raw data
	pub iid:u32, pub aa:u32, pub ac:u32, pub ca:u32, pub cc:u32,
    pub cf:u32,  pub dr:u32, pub fs:u32, pub id:u32, pub ke:u32,
    pub nd:u32,  pub ri:u32, pub sl:u32, pub um:u32, pub vs:u32,
    pub mb:[u8; 7], pub md:[u8; 10],
    pub me:[u8; 7], pub mv:[u8;  7],

    // Decoded data
    pub metype: u8,
    pub mesub: u8,

    pub altitude: Option<(i32, AltitudeUnit, AltitudeSource)>,
    pub gnss_delta: Option<i32>,
    pub heading: Option<(i32, HeadingSource)>,
    pub speed: Option<(u32, SpeedSource)>,
    pub vert_rate: Option<(i32, AltitudeSource)>,
    pub squawk: Option<u32>,
    pub callsign: Option<String>,
    pub category: Option<u8>,
    pub raw_cpr:Option<(u32, u32, bool, u32, CprType)>,
    pub decoded_cpr:Option<(f64, f64, bool)>,
    pub airground: Option<AirGround>,

    pub tss:Option<TargetStateStatus>,
    pub opstatus:Option<OperationalStatus>,
}

#[derive(Debug, Clone)]
pub struct TargetStateStatus {
	pub mode_valid: bool,    pub mode_autopilot: bool, pub mode_vnav: bool,
	pub mode_alt_hold: bool, pub mode_approach: bool,  pub acas_operational: bool,
	pub nac_p: u8,
	pub nic_baro: bool,
	pub sil: u8,
	pub sil_type: SilType,
	pub altitude:Option<(u32, TssAltitudeType)>,
	pub baro:Option<f32>,
	pub heading:Option<u32>
}

#[derive(Debug, Default, Clone)]
pub struct OperationalStatus {
    pub version: u8,

    pub om_acas_ra: bool, pub om_ident: bool, 
    pub om_atc: bool,     pub om_saf: bool,
    
    pub om_sda: u8,
    pub cc_acas: bool,
    pub cc_cdti: bool,
    pub cc_1090_in: bool,
    pub cc_arv: bool,
    pub cc_ts: bool,
    pub cc_tc: u8,
    pub cc_uat_in: bool,
    pub cc_poa: bool,
    pub cc_b2_low: bool,
    pub cc_nac_v: u8,
    pub cc_nic_supp_c: bool,
    pub cc_lw_valid: bool,

    pub nic_supp_a: bool,
    pub nac_p: u8,
    pub gva: u8,
    pub sil: u8,
    pub nic_baro: bool,

    pub sil_type:SilType,
    pub track_angle:AngleType,
    pub hrd:HeadingSource,

    pub cc_lw:u32,
    pub cc_antenna_offset:u32
}


