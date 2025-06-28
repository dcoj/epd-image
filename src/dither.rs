use exoquant::*;

use crate::EPD_WIDTH;

#[derive(Debug)]
pub enum Error {
    Image(image::ImageError),
    WrongDimensions(usize, usize),
    LodepngError(lodepng::Error),
    Io(std::io::Error),
    ImageEncodeError(image::error::ImageError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<lodepng::Error> for Error {
    fn from(e: lodepng::Error) -> Self {
        Error::LodepngError(e)
    }
}

impl From<image::error::ImageError> for Error {
    fn from(e: image::error::ImageError) -> Self {
        Error::ImageEncodeError(e)
    }
}

pub fn dither_image(pixels: Vec<Color>) -> Result<(Vec<u8>, [Color; 7]), Error> {
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
