use exoquant::*;
use image::{Rgba, RgbaImage};

use crate::{EPD_HEIGHT, EPD_WIDTH};

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

pub fn load_png(input_path: &str) -> Result<Vec<Color>, Error> {
    let input_image = lodepng::decode32_file(input_path).map_err(Error::LodepngError)?;

    // Check dimensions
    if input_image.width != EPD_WIDTH || input_image.height != EPD_HEIGHT {
        return Err(Error::WrongDimensions(
            input_image.width,
            input_image.height,
        ));
    }

    // Convert input_image to a Vec<Color> before quantization
    let pixels: Vec<Color> = input_image
        .buffer
        .iter()
        .map(|p| Color::new(p.r, p.g, p.b, p.a))
        .collect();

    return Ok(pixels);
}

// Save a PNG preview of the indexed image
pub fn save_png(path: &str, indexed_image: &[u8], palette: &[Color]) -> Result<(), Error> {
    // Create a new RGBA image
    let mut img = RgbaImage::new(EPD_WIDTH as u32, EPD_HEIGHT as u32);

    // Fill the image with the palette colors based on indexed_image
    for y in 0..EPD_HEIGHT {
        for x in 0..EPD_WIDTH {
            let idx = y * EPD_WIDTH + x;
            let color_idx = indexed_image[idx] as usize;

            // Make sure index is in bounds of our palette
            let color_idx = color_idx.min(palette.len() - 1);
            let color = &palette[color_idx];

            img.put_pixel(
                x as u32,
                y as u32,
                Rgba([color.r, color.g, color.b, color.a]),
            );
        }
    }

    // Save the image
    img.save(path).map_err(Error::ImageEncodeError)
}
