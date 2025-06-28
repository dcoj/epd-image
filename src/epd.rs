use std::fs::File;
use std::io::{BufWriter, Write};

use crate::error::{Error, Result};
use crate::{EPD_HEIGHT, EPD_WIDTH};

pub fn save_epd(path: &str, indexed_image: &[u8]) -> Result<()> {
    // Write EPD file with our custom format
    let output_file = File::create(path)?;
    let mut writer = BufWriter::new(output_file);

    // Write a simple header inspired by
    // https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Picture_Processing
    writer.write_all(b"EPD7")?; // waveshare Magic number
    writer.write_all(&[1])?; // Version
    writer.write_all(&(EPD_WIDTH as u32).to_le_bytes())?;
    writer.write_all(&(EPD_HEIGHT as u32).to_le_bytes())?;

    // Process pixels two at a time and pack into a byte (4-bit)
    for y in 0..EPD_HEIGHT {
        for x in (0..EPD_WIDTH).step_by(2) {
            let index = y * EPD_WIDTH + x;
            let color1 = indexed_image[index];
            let color2 = if x + 1 < EPD_WIDTH {
                indexed_image[index + 1]
            } else {
                // can't happen as the width is hardcoded
                return Err(Error::WrongDimensions(EPD_WIDTH, EPD_HEIGHT));
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
