use exoquant::*;

use crate::error::Result;
use crate::EPD_WIDTH;

pub fn dither_image(pixels: Vec<Color>) -> Result<(Vec<u8>, [Color; 7])> {
    println!("Converting image");

    // Define the fixed palette
    let palette: [Color; 7] = [
        Color::new(0, 0, 0, 255),       // Black
        Color::new(255, 255, 255, 255), // White
        Color::new(0, 255, 0, 255),     // Green
        Color::new(0, 0, 255, 255),     // Blue
        Color::new(255, 0, 0, 255),     // Red
        Color::new(255, 255, 0, 255),   // Yellow
        Color::new(255, 128, 0, 255),   // Orange
    ];

    println!("Remapping image to palette");
    let colorspace = SimpleColorSpace::default();
    let ditherer = ditherer::FloydSteinberg::new();

    // Create a remapper with our fixed palette
    let remapper = Remapper::new(&palette, &colorspace, &ditherer);

    // Remap the image pixels to our palette indices
    let indexed_image: Vec<u8> = remapper.remap(&pixels, EPD_WIDTH);

    Ok((indexed_image, palette))
}
