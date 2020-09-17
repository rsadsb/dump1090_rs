
/* aviation::adsb::dump1090::mode_ac

This module includes functionality translated from mode_ac.c
*/

pub fn mode_a_to_mode_c(mode_a:u32) -> Result<u32, &'static str> {
	let mut five_hundreds:u32 = 0;
  	let mut one_hundreds:u32  = 0;

  	// check zero bits are zero, D1 set is illegal; C1,,C4 cannot be Zero
  	if (mode_a & 0xFFFF8889) != 0 || (mode_a & 0x000000F0) == 0 {
      	return Err("Invalid altitude");
  	}

  	if mode_a & 0x0010 != 0 {one_hundreds ^= 0x007;} // C1
  	if mode_a & 0x0020 != 0 {one_hundreds ^= 0x003;} // C2
  	if mode_a & 0x0040 != 0 {one_hundreds ^= 0x001;} // C4

  	// Remove 7s from OneHundreds (Make 7->5, snd 5->7). 
  	if (one_hundreds & 5) == 5 { one_hundreds ^= 2; }

  	// Check for invalid codes, only 1 to 5 are valid 
  	if one_hundreds > 5 {
      	return Err("Invalid altitude");
  	}

	// if mode_a & 0x0001 {five_hundreds ^= 0x1FF;} // D1 never used for altitude
  	if mode_a & 0x0002 != 0 {five_hundreds ^= 0x0FF;} // D2
  	if mode_a & 0x0004 != 0 {five_hundreds ^= 0x07F;} // D4

  	if mode_a & 0x1000 != 0 {five_hundreds ^= 0x03F;} // A1
  	if mode_a & 0x2000 != 0 {five_hundreds ^= 0x01F;} // A2
  	if mode_a & 0x4000 != 0 {five_hundreds ^= 0x00F;} // A4

  	if mode_a & 0x0100 != 0 {five_hundreds ^= 0x007;} // B1 
  	if mode_a & 0x0200 != 0 {five_hundreds ^= 0x003;} // B2
  	if mode_a & 0x0400 != 0 {five_hundreds ^= 0x001;} // B4
    
  // Correct order of one_hundreds. 
  if five_hundreds & 1 != 0 {one_hundreds = 6 - one_hundreds;} 

  Ok((five_hundreds * 5) + one_hundreds - 13)
}
