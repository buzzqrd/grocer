# grocer

A simple Rust library for reading UPC barcodes from images.

## Usage

Add grocer to your `Cargo.toml` file.
```toml
[dependencies]
grocer = "0.1.0"
```

## Example

It's simple to read a barcode from an image file with minimal code.
```rust
fn main() {
	let upc_code = scan_upc("images/barcode_picture.png");
	println!("Scanned barcode: {}", upc_code);
}
```
