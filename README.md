# grocer

A simple Rust library for reading UPC barcodes from images.

## Usage

Add grocer to your `Cargo.toml` file.
```toml
[dependencies]
grocer = "0.1.2"
```

## Example

It's simple to read a barcode from an image file with minimal code.
```rust
use grocer;

fn main() {
	let scan_settings = grocer::ScannerSettings { high_speed: false, };
	let barcode: grocer::Barcode = grocer::scan_upc("images/image.png", scan_settings);
	println!("Code: {}", barcode.code);
}
```

### Why is it taking so long?

Scans can take a while mostly because the barcode can be anywhere in the image and needs 
to be found first. The whole image is scanned and every white section is checked to see if 
it is the leading quiet zone on the left side of a barcode. 

**To fix it**, for now you can compile with --release and it should go much faster.

I'm working on adding features soon that allow the user to cut down on reliability for 
speed if the application calls for it. For instance, if you know the barcode is always 
in the center of the image, changing the settings would allow for very fast scanning 
because it's easier to look for the barcode.


## Help

Help me out! If you have images of barcodes that won't scan with this library, let me know!
Please send me any images that don't scan (buzzqrd@protonmail.com). I want to make sure that 
this library is able to work for all barcodes within reason, and it will really help to 
test it on a wide variety of barcodes. I've been taking lots of pictures of barcodes and 
testing them with grocer. I've mostly had complete success, but I had some issues with a green 
barcode that I found, so im working on making the black-white boundary auto-adjust to 
capture a wider range of color-printed barcodes.

If you want to send a barcode to me, make sure it's a supported format (UPC-A for now) and 
email me at buzzqrd@protonmail.com.

Thank you!

## Future / In-Progress

- Proper errors
- Making scans that adjust the grayscale profile during scanning to scan barcodes made with different colors
- Wider range of features and customization to be able to disable those features
- Support for live video streams to scan barcodes from a computer-attatched camera
- Support for a wider range of linear barcodes (UPC-E, FNSKU, EAN, Pharmacode, etc.)
- Ability to read barcodes rotated vertically

## Changelog

0.1.0: Initial release
	- Basic barcode scanning
	- UPC-A support

0.1.1: Updated docs
	- Updates to some documentation

0.1.2: Important features update
	- More accurate scanning methods
	- Structs for settings selection
	- Return structs for barcode info
	- Rejects any barcodes of the wrong size (UPC-A = 12 chars)



