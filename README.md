# Photo Desk

A Rust application for converting images to EPD (E-Paper Display) format and serving them via HTTP server.

## Features

- **CLI Mode**: Convert PNG images to EPD format with dithering and color quantization
- **Server Mode**: HTTP server for serving static files and fetching recent photos from a remote API
- **Image Processing**: Advanced dithering using Floyd-Steinberg algorithm with a 7-color palette
- **Static File Serving**: Serve assets from the `samples` directory
- **Photo API Integration**: Fetch recent photos from a remote photo service

## Installation

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
git clone <repository-url>
cd photo-desk
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

This will:
- Load the input PNG (must be 800x480 pixels)
- Apply Floyd-Steinberg dithering with a 7-color palette
- Generate three output files:
  - `output.epd` - EPD format file
  - `output.png` - Preview image
  - `<filename>_preview.png` - Named preview

#### Color Palette

The application uses a fixed 7-color palette optimized for e-paper displays:
- Black (0, 0, 0)
- White (255, 255, 255)
- Green (0, 255, 0)
- Blue (0, 0, 255)
- Red (255, 0, 0)
- Yellow (255, 255, 0)
- Orange (255, 128, 0)

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

- `PHOTO_API_KEY` - API key for the photo service (required for `/recent` endpoint)

### Server Examples

```bash
# Start the server
export PHOTO_API_KEY="your-api-key-here"
./target/release/framer server

# Test endpoints
curl http://localhost:3000/health
curl http://localhost:3000/recent > recent_photo.jpg
curl http://localhost:3000/samples/test.txt
```

## API Integration

The `/recent` endpoint integrates with a photo API to fetch recent favorite photos:

1. **API Request**: Makes a GET request to the timeline endpoint with specific filters
2. **Response Parsing**: Extracts the first thumbnail ID from the JSON response
3. **Image Download**: Downloads the actual image data using the thumbnail ID
4. **Direct Serving**: Returns the image data directly to the client (no redirect)

### API Request Details

- **Endpoint**: `https://photos.dcdc.dev/api/timeline/bucket`
- **Parameters**:
  - `isFavorite=true`
  - `isTrashed=false`
  - `timeBucket=2025-01-01`
  - `visibility=timeline`
  - `withPartners=true`
  - `withStacked=true`
- **Authentication**: Bearer token in Authorization header

## Development

### Project Structure

```
photo-desk/
├── src/
│   ├── main.rs          # CLI and server entry point
│   ├── error.rs         # Centralized error handling
│   ├── server.rs        # HTTP server implementation
│   ├── png.rs           # PNG loading and saving
│   ├── epd.rs           # EPD format handling
│   └── dither.rs        # Image dithering algorithms
├── samples/             # Static files served by HTTP server
├── Cargo.toml
└── README.md
```

### Dependencies

- **axum**: Web framework for HTTP server
- **tokio**: Async runtime
- **reqwest**: HTTP client for API requests
- **serde**: JSON serialization/deserialization
- **tower-http**: HTTP middleware and services
- **image**: Image processing
- **exoquant**: Color quantization and dithering
- **lodepng**: PNG handling

### Running in Development

```bash
# CLI mode
cargo run -- input.png

# Server mode
export PHOTO_API_KEY="your-key"
cargo run -- server
```

### Testing

```bash
cargo test
```

## Error Handling

The application uses a centralized error handling system with proper error propagation:

- **IO Errors**: File system operations
- **HTTP Errors**: Network requests and responses
- **Image Errors**: Image processing and format issues
- **API Errors**: Remote API communication failures

All errors are properly logged and returned as appropriate HTTP status codes in server mode.

## License

MIT OR Apache-2.0