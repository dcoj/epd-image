use std::env;
mod dither;
mod epd;
mod error;
mod png;
use std::path::Path;

use error::Result;

pub const EPD_WIDTH: usize = 800;
pub const EPD_HEIGHT: usize = 480;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.png>", args[0]);
        std::process::exit(1);
    }

    let input_path = args[1].clone();

    let pixels = png::load_png(&input_path)?;
    let (indexed_image, palette) = dither::dither_image(pixels)?;

    // Also save a PNG preview file
    let preview_path = format!("{}_preview.png", generate_preview_path(&input_path));
    epd::save_epd("./output.epd", &indexed_image)?;
    png::save_png("./output.png", &indexed_image, &palette)?;
    png::save_png(&preview_path, &indexed_image, &palette)?;

    Ok(())
}

// Generate a preview PNG path from the EPD path
pub fn generate_preview_path(epd_path: &str) -> String {
    let path = Path::new(epd_path);
    // Just use the current directory to avoid permission issues
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    return stem.to_string();
}
