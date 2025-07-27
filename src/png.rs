use exoquant::*;
use image::{Rgba, RgbaImage};

use crate::error::{Error, Result};
use crate::{EPD_HEIGHT, EPD_WIDTH};

pub fn load_png(input_path: &str) -> Result<Vec<Color>> {
    let input_image = lodepng::decode32_file(input_path)?;

    // Before we do anything, check the image dimensions are correct
    if input_image.width != EPD_WIDTH || input_image.height != EPD_HEIGHT {
        return Err(Error::WrongDimensions(
            input_image.width,
            input_image.height,
        ));
    }

    // Convert input_image to a Vec<Color> for quantization
    let pixels: Vec<Color> = input_image
        .buffer
        .iter()
        .map(|p| Color::new(p.r, p.g, p.b, p.a))
        .collect();

    Ok(pixels)
}

// Save a PNG of the indexed image
pub fn save_png(path: &str, indexed_image: &[u8], palette: &[Color]) -> Result<()> {
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
    img.save(path)?;
    Ok(())
}
