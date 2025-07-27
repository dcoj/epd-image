use std::env;
mod crop;
mod dither;
mod epd;
mod error;
mod immich;
mod png;
mod server;
use std::path::Path;

use dotenv::dotenv;
use error::Result;
use server::{start_server, ServerConfig};

pub const EPD_WIDTH: usize = 800;
pub const EPD_HEIGHT: usize = 480;

#[tokio::main]
async fn main() {
    dotenv().ok();
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server().await,
        "convert" => run_cli(&args).await,
        _ => {
            // Default to server
            run_server().await
        }
    }
}

async fn run_server() -> Result<()> {
    println!("Starting photo-desk server...");

    let config = ServerConfig::default();

    if config.api_key.is_empty() {
        eprintln!("Warning: PHOTO_API_KEY environment variable not set");
        eprintln!("The /recent endpoint will not work without a valid API key");
    }

    start_server(config).await
}

async fn run_cli(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        print_usage(&args[0]);
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

fn print_usage(program_name: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} <input.png>        Convert PNG to EPD format",
        program_name
    );
    eprintln!(
        "  {} convert <input.png> Convert PNG to EPD format",
        program_name
    );
    eprintln!("  {} server             Start HTTP server", program_name);
    eprintln!();
    eprintln!("Server Environment Variables:");
    eprintln!("  PHOTO_API_KEY         API key for photo service (required for /recent endpoint)");
}

// Generate a preview PNG path from the EPD path
pub fn generate_preview_path(epd_path: &str) -> String {
    let path = Path::new(epd_path);
    // Just use the current directory to avoid permission issues
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    stem.to_string()
}
