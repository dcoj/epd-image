use std::fs::File;
use std::io::{BufWriter, Write};

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

pub fn save_epd(path: &str, indexed_image: &[u8]) -> Result<(), Error> {
    // Write EPD file with our custom format
    let output_file = File::create(path)?;
    let mut writer = BufWriter::new(output_file);

    // Write a simple header
    writer.write_all(b"EPD7")?; // Magic number
    writer.write_all(&[1])?; // Version
    writer.write_all(&(EPD_WIDTH as u32).to_le_bytes())?;
    writer.write_all(&(EPD_HEIGHT as u32).to_le_bytes())?;

    // Process pixels two at a time and pack into bytes
    for y in 0..EPD_HEIGHT {
        for x in (0..EPD_WIDTH).step_by(2) {
            let index = y * EPD_WIDTH + x;
            let color1 = indexed_image[index];

            let color2 = if x + 1 < EPD_WIDTH {
                indexed_image[index + 1]
            } else {
                1 // White padding for odd width
            };

            // Pack two 4-bit colors into one byte
            let byte = (color1 << 4) | color2;
            writer.write_all(&[byte])?;
        }
    }

    writer.flush()?;
    println!("EPD file saved successfully!");

    return Ok(());
}
