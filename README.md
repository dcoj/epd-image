# EPD Image Creator

A Rust application for creating and serving images in Waveshare's "3bit" EPD (E-Paper Display) format. For more information on the image data see [here](https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(E)_Manual#Programming_Principles).

The custom EPD format fits 2 pixels into a single byte, leading to an image size of 192KB (+ header) for an uncompressed 800x480 image. Given this low bit depth and fixed aspect ratio the code crops and resizes the image to 800x480 before dithering the image to provide the best possible output for the given display.

This codebase targets the [Waveshare 7.3inch ACeP 7-Color E-Paper E-Ink Display Module](https://www.waveshare.com/7.3inch-e-paper-hat-f.htm) but could probably be adapted to other 'custom colour' EPaper displays depending on their protocol.

This POC is also integrated with [Immich](https://immich.app/), a local 'Google Photos', and can dynamically serve optimized photos from a users library at random.


## Features

- **CLI Mode**: Convert any local PNG image to EPD format with various dithering and color quantization options (good for experimenting and pre-rendering if you want to store images files directly on an eink display).

- **Server Mode**: HTTP server for serving static files and a dynamic endpoint that fetches a recent photo from immich and processes it in realtime. This can be used with my [photo-frame](https://github.com/dcoj/photo-frame) repo and an ESP32S3 to randomly display images on a display.

## Installation

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
git clone https://github.com/dcoj/epd-image
cd epd-image
cargo build --release
```

## Usage

### CLI Mode - Image Conversion

Convert a PNG image to EPD format:

```bash
# Direct conversion (backward compatible)
./target/release/framer input.png

# Explicit convert command
./target/release/framer convert input.png
```

#### Color Palette

The application uses a fixed 7-color palette optimized for the Waveshare 7-Colour E-Paper display:
- Black (0, 0, 0)
- White (255, 255, 255)
- Green (0, 255, 0)
- Blue (0, 0, 255)
- Red (255, 0, 0)
- Yellow (255, 255, 0)
- Orange (255, 128, 0)

Probably the colour values could be improved to better represent the actual colour outputs but the [official ACT from Waveshare](https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(E)_Manual#Operating_Steps) also doesn't provide this.

### Server Mode

Start the HTTP server:

```bash
./target/release/framer server
```

The server runs on port 3000 by default and provides:

#### Endpoints

- **GET /health** - Health check endpoint
- **GET /recent** - Fetch and serve a recent photo from the configured photo API
- **GET /samples/** - Serve static files from the `samples` directory

#### Environment Variables

- `PHOTO_API_KEY` - API key for Immich (required for `/recent` endpoint)
- `PHOTO_API_URL` - An Immich server address (required for `/recent` endpoint)


### Dependencies

- **axum**: Web framework for HTTP server
- **[exoquant](https://exoticorn.github.io/exoquant-rs/exoquant/)**: Provides various color quantization and dithering processors
- **[smartcrop2](https://docs.rs/smartcrop2/latest/smartcrop/)**: Intelligent cropping based on image content

## License

MIT OR Apache-2.0
