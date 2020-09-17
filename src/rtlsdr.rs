
use std::ffi::{CStr, CString};
use libc::c_char;

// TODO: remove all unwraps in this whole file

#[link(name = "rtlsdr")]
extern {
	fn rtlsdr_get_device_count() -> u32;
	fn rtlsdr_get_device_name(idx:u32) -> *const c_char;
	fn rtlsdr_get_device_usb_strings(idx:u32, 
						manufact:*mut c_char,
					    product:*mut c_char,
					    serial:*mut c_char) -> isize;

	fn rtlsdr_open(dev_ptr: &mut usize, idx:u32) -> isize;
	fn rtlsdr_close(dev_ptr: usize) -> isize;

	fn rtlsdr_set_center_freq(dev_ptr:usize, freq:u32) -> i32;
	fn rtlsdr_get_center_freq(dev_ptr:usize) -> u32;
	fn rtlsdr_set_freq_correction(dev_ptr:usize, ppm:i32) -> i32;
	fn rtlsdr_get_freq_correction(dev_ptr:usize) -> i32;
	fn rtlsdr_set_tuner_gain(dev_ptr:usize, gain:i32) -> i32;
	fn rtlsdr_get_tuner_gain(dev_ptr:usize) -> i32;
	fn rtlsdr_set_tuner_gain_mode(dev_ptr:usize, manual:i32) -> i32;
	fn rtlsdr_get_tuner_gains(dev_ptr:usize, gains:*mut i32) -> i32;
	fn rtlsdr_set_sample_rate(dev_ptr:usize, rate:u32) -> i32;
	fn rtlsdr_get_sample_rate(dev_ptr:usize) -> u32;

	fn rtlsdr_reset_buffer(dev_ptr:usize) -> i32;

	fn rtlsdr_read_sync(dev_ptr:usize, buf:*mut u8, len:isize, n_read:*mut isize) -> isize;

	// == Not yet implemented ==
	// rtlsdr_get_index_by_serial
	// rtlsdr_set_xtal_freq
	// rtlsdr_get_xtal_freq
	// rtlsdr_get_usb_strings
	// rtlsdr_write_eeprom
	// rtlsdr_get_center_freq
	// rtlsdr_get_freq_correction
	// rtlsdr_get_tuner_type
	// rtlsdr_set_tuner_bandwidth
	// rtlsdr_set_tuner_if_gain
	// rtlsdr_get_sample_rate
	// rtlsdr_set_testmode
	// rtlsdr_set_agc_mode
	// rtlsdr_set_direct_sampling
	// rtlsdr_get_direct_sampling
	// rtlsdr_set_offset_tuning
	// rtlsdr_get_offset_tuning
	// void(*rtlsdr_read_async_cb_t)
	// rtlsdr_wait_async
	// rtlsdr_read_async
	// rtlsdr_cancel_async
	// rtlsdr_set_bias_tee
	// rtlsdr_set_bias_tee_gpio
}

#[derive(Debug)]
pub struct RtlSdrDevice {
	// The RTL-SDR API gives me a usize that I can use to reference the device
	// it's managing for me.  I don't care that this usize happens to be the 
	// memory location of the device because I'm never going to access the device
	// without going through the API.
	dev:usize,
}

impl RtlSdrDevice {

	pub fn new(idx:u32) -> Result<Self, &'static str> {
		let mut dev:usize = 0;

		let result_code = unsafe {
			rtlsdr_open(&mut dev, idx)
		};

		match result_code {
			0 => Ok(RtlSdrDevice{ dev }),
			_ => Err("Expected result code of zero")
		}
					
	}

	pub fn set_center_freq(&mut self, freq:u32) -> Result<(), &'static str> {
		let result_code = unsafe {
			rtlsdr_set_center_freq(self.dev, freq)
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Expected result code of zero")
		}
	}

	pub fn get_center_freq(&self) -> Result<u32, &'static str> {
		let result = unsafe {
			rtlsdr_get_center_freq(self.dev)
		};

		match result {
			0 => Err("Zero returned from get_center_freq, which indicates failure"),
			f => Ok(f)
		}
	}

	pub fn set_freq_correction(&mut self, ppm:i32) -> Result<(), &'static str> {

		let result_code = unsafe {
			rtlsdr_set_freq_correction(self.dev, ppm)
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Unable to set frequency correction")
		}
		
	}

	pub fn get_freq_correction(&self) -> Result<i32, &'static str> {

		Ok(unsafe {
			rtlsdr_get_freq_correction(self.dev)
		})

	}

	pub fn set_tuner_gain_mode(&mut self, manual:i32) -> Result<(), &'static str> {

		let result_code = unsafe {
			rtlsdr_set_tuner_gain_mode(self.dev, manual)
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Unable to set tuner gain mode")
		}
	}

	pub fn set_tuner_gain(&mut self, gain:i32) -> Result<(), &'static str> {

		let result_code = unsafe {
			rtlsdr_set_tuner_gain(self.dev, gain)
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Unable to set tuner gain")
		}

	}

	pub fn get_tuner_gain(&self) -> Result<i32, &'static str> {
		Ok(unsafe {
			rtlsdr_get_tuner_gain(self.dev)
		})
	}

	pub fn get_tuner_gains(&self) -> Result<Vec<i32>, &'static str> {
		
		let mut gains:Vec<i32> = vec![0; 32];
		let num_gains = unsafe {
			rtlsdr_get_tuner_gains(self.dev, &mut gains[0])
		};

		if num_gains > 32 { return Err("Didn't expect more than 32 gains"); }
		gains.truncate(num_gains as usize);

		Ok(gains)
	}

	pub fn set_sample_rate(&mut self, rate:u32) -> Result<(), &'static str> {

		let result_code:i32 = unsafe {
			rtlsdr_set_sample_rate(self.dev, rate)
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Unable to set sample rate")
		}

	}

	pub fn get_sample_rate(&self) -> Result<u32, &'static str> {

		let result:u32 = unsafe { rtlsdr_get_sample_rate(self.dev) };

		match result {
			0 => Err("Invalid sample rate received"),
			n => Ok(n)
		}
	}

	pub fn reset_buffer(&mut self) -> Result<(), &'static str> {
		let result_code = unsafe {
			rtlsdr_reset_buffer(self.dev)			
		};

		match result_code {
			0 => Ok(()),
			_ => Err("Failed to reset buffer"),
		}
	}

	pub fn read_sync(&mut self) -> Result<Vec<u8>, &'static str> {
		let mut n_read:isize = 0;
		let mut rcv_buff:Vec<u8> = vec![0u8; 512];

		let result_code = unsafe {	
			rtlsdr_read_sync(self.dev, &mut rcv_buff[0], 512, &mut n_read)
		};

		rcv_buff.truncate(n_read as usize);

		match result_code {
			0 => Ok(rcv_buff),
			_ => Err("Failure to read bytes")
		}
	}

}

impl std::ops::Drop for RtlSdrDevice {
	fn drop(&mut self) {
		unsafe {
			println!("Closing RtlSdrDevice at {}", self.dev);
			rtlsdr_close(self.dev);
		}
	}
}

#[derive(Debug)]
pub struct UsbStrings { manufacturer:String, product:String, serial:String }

pub fn get_device_count() -> u32 {
	unsafe { rtlsdr_get_device_count() }
}

pub fn get_device_name(idx:u32) -> &'static str {
	unsafe { 
		let ptr = rtlsdr_get_device_name(idx);
		CStr::from_ptr(ptr).to_str().unwrap() 
	}
}

pub fn get_device_usb_strings(idx:u32) -> Result<UsbStrings, &'static str> {
	let mfg:*mut c_char = CString::new(vec![20u8; 256]).unwrap().into_raw();
	let prd:*mut c_char = CString::new(vec![20u8; 256]).unwrap().into_raw();
	let ser:*mut c_char = CString::new(vec![20u8; 256]).unwrap().into_raw();

	unsafe {  
		let return_code = rtlsdr_get_device_usb_strings(idx, mfg, prd, ser);
	
		if return_code == 0 {
			let manufacturer:String = CString::from_raw(mfg).into_string().unwrap();
			let product:String      = CString::from_raw(prd).into_string().unwrap();
			let serial:String       = CString::from_raw(ser).into_string().unwrap();
			Ok(UsbStrings{ manufacturer, product, serial })
		} else { Err("API call failed") }

	}

}