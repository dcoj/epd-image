use std::env;
mod dither;
mod epd;
mod png;
use std::path::Path;

pub const EPD_WIDTH: usize = 800;
pub const EPD_HEIGHT: usize = 480;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.png>", args[0]);
        std::process::exit(1);
    }

    let input_path = args[1].clone();

    let pixels = png::load_png(&input_path).unwrap();
    let (indexed_image, palette) = dither::dither_image(pixels).unwrap();

    // Also save a PNG preview file
    let preview_path = format!("{}_preview.png", generate_preview_path(&input_path));
    epd::save_epd("./output.epd", &indexed_image).expect("Could not save epd file");
    png::save_png("./output.png", &indexed_image, &palette).expect("Could not save png file");
    png::save_png(&preview_path, &indexed_image, &palette).expect("Could not save png file");
}

// Generate a preview PNG path from the EPD path
pub fn generate_preview_path(epd_path: &str) -> String {
    let path = Path::new(epd_path);
    // Just use the current directory to avoid permission issues
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    return stem.to_string();
}
