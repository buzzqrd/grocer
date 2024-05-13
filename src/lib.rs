
//! grocer
//! A simple library for reading barcodes from an image
//! 


use std::env;
use std::error::Error;
use image::GenericImageView;


#[derive(Debug)]
struct WhiteZone {
	end: u32,
	size: u32,
}

#[derive(Clone, Debug)]
struct Bar {
	color: u32,
	modules: u32,
	width: u32,
}

fn pixel_bw(pixel: image::Rgba<u8>) -> u32 {
	let thresh = 110; // Default: 110
	let grayscale = {
		let mut sum: u32 = 0;
		for i in 0..3 {
			sum += pixel[i] as u32;
		}
		sum / 3
	};
	if grayscale < thresh {
		0
	} else {
		1
	}
}


fn get_white_zones_list(file: &str, y: u32) -> Vec<WhiteZone> {
	let mut zones: Vec<WhiteZone> = Vec::new();

	let img = image::open(file).unwrap();
	let width = img.dimensions().0;
	
	let mut zone_size: u32 = 0;
	let mut last_bw: u32 = 1;
	for x in 0..width {
		let pix = img.get_pixel(x, y);
		let bw = pixel_bw(pix);
		
		// End of zone
		if bw == 0 && last_bw == 1{
			let new_zone = WhiteZone {
				end: x,
				size: zone_size,
			};
			zones.push(new_zone);
			zone_size = 0;
		}

		if bw == 1 {
			zone_size += 1;
		}
		
		last_bw = bw;
	}
	// Add the last zone if it was white
	if last_bw == 1 {
		let new_zone = WhiteZone {
			end: width,
			size: zone_size,
		};
		zones.push(new_zone);
	}

	zones.sort_by_key(|el| el.size);
	zones.reverse();
	zones
}


fn collect_bars(file: &str, y: u32, quiet_zone: &WhiteZone) -> Result<Vec<Bar>, Box<dyn Error>> {
	let mut bars: Vec<Bar> = Vec::new();
	let img = image::open(file).unwrap();
	let width = img.dimensions().0;

	let mut last_bw: u32 = 0;
	let mut zone_size: u32 = 0;

	// Get each bar's width and color
	for x in quiet_zone.end..width {
		let pix = img.get_pixel(x, y);
		let bw = pixel_bw(pix);
		
		if last_bw != bw {
			let bar = Bar {
				color: last_bw,
				width: zone_size,
				modules: 0,
			};
			bars.push(bar);
			zone_size = 0;
		}
		zone_size += 1;
		last_bw = bw;
	}
	
	// Add the last zone if it was white
	if last_bw == 1 {
		let bar = Bar {
			color: last_bw,
			width: zone_size,
			modules: 0,
		};
		bars.push(bar);
	}
	
	
	if bars.len() < 2 || bars[1].color != 1 {
		// Error. We should have a start zone 
		// with a white bar as the second bar
		return Err("Invalid start region.".into());	
	}

	// First white bar size
	let white_bar_size = bars[1].width;

	// Find the ending quiet zone index
	let quiet_zone_bar_size: u32 = 8;
	let quiet_zone_min_width = white_bar_size * quiet_zone_bar_size;
	
	let mut quiet_zone_end = 0;
	for i in 0..bars.len() {
		if bars[i].width >= quiet_zone_min_width {
			quiet_zone_end = i;
			break;
		}
	}
	
	if quiet_zone_end == 0 {
		return Err("Error. No acceptable start or end zone.".into());
	}


	Ok((&bars[..quiet_zone_end-1]).to_vec())
}


fn barlist_to_upc(barlist: Vec<Bar>) -> String {
	let mut upc_code = String::from("");
	let upc_lookup = [
	(3, 2, 1, 1),
	(2, 2, 2, 1),
	(2, 1, 2, 2),
	(1, 4, 1, 1),
	(1, 1, 3, 2),
	(1, 2, 3, 1),
	(1, 1, 1, 4),
	(1, 3, 1, 2),
	(1, 2, 1, 3),
	(3, 1, 1, 2),
	];		

	// Needs to skip Start, End, and Middle
	let new_barlist = barlist.clone();
	
	let barlist_start = &new_barlist[..27];
	let barlist_end   = &new_barlist[32..];

	let full_barlist = [barlist_start, barlist_end].concat().to_vec();

	// Skip the start & end patterns
	for i in (3..full_barlist.len()-3).step_by(4){
		let digit = (full_barlist[i+0].modules,
			full_barlist[i+1].modules,
			full_barlist[i+2].modules,
			full_barlist[i+3].modules);
		for k in 0..upc_lookup.len() {
			if upc_lookup[k] == digit {
				let _ = &upc_code.push_str(&k.to_string());
				break;
			}
		}
	}
	upc_code
}

fn is_code_upc(code: &str) -> bool{
	let mut pos: u32 = 3;
	let mut sum: u32 = 0;
	for c in code.chars() {
		let val: u32 = match c.to_string().trim().parse() {
			Ok(num) => num,
			Err(_) => return false,
		};
		sum += pos * val;
		pos = (pos + 2) % 4; // alternate 1 & 3 
	}
	if sum % 10 == 0 {
		return true ;
	}
	return false ;
}


fn scan_upc_line(file: &str, height_percent: f64) -> String {
	let upc_modules = 95;
	
	let mut code: String = String::from("");
	let img = image::open(file).unwrap();
	let y: f64 = img.dimensions().1.into();
	let mid: u32 = (y * height_percent) as u32;

	// Get the list of white zones
	let zones: Vec<WhiteZone> = get_white_zones_list(file, mid);

	for start_zone in zones {

		// Try to get a code for each zone
		let mut barlist = match collect_bars(file, mid, &start_zone) {
			Ok(list) => list,
			Err(_) => continue,
		};

		// Make sure there are enough bars
		if barlist.len() < 58 {
			continue;
		}

		// Get width of barcode in pixels
		let mut barcode_width: u32 = 0;
		for bar in &barlist {
			barcode_width += bar.width;
		}

		let module_size: f64 = (barcode_width as f64) / (upc_modules as f64);
		//println!("Module size: {}", module_size);
		
		// Get the module size of each bar
		for i in 0..barlist.len() {
			let mut module_count: u32 = 0;
			let mut sample_x: f64 = (module_size as f64) / 2.0;
			while sample_x < (barlist[i].width as f64) {
				module_count += 1;
				sample_x += module_size;
			}
			
			barlist[i].modules = module_count;
		}

		code = barlist_to_upc(barlist);
		//println!("CC: {}", code);	
	
		let check = is_code_upc(&code);	
		if check {
			//println!("The UPC code is valid");
		} else {
			//println!("The UPC code is invalid");
			code = String::from("");
		}
	
		break;
	}
	code
}




/// Scans an image file horizontally for any UPC-A codes and returns the code as a string.
///
/// # Parameters
/// 
/// file: A string of the image filename to read from.
///
/// # Return
///
/// Returns a String of the 12-digit UPC code if the scan was successful. Returns an empty string if no barcode was found.
///
/// # Examples
/// 
/// ```
///	let upc_code = grocer::scan_upc("images/image.png");
/// ```
pub fn scan_upc(file: &str) -> String {
	let mut code: String = String::from("");

	for percent in (0..100).step_by(5) {
		code = scan_upc_line(file, (percent as f64) / 100.0);
		if code != "" {
			break;
		}
	}
	code
}


