use crate::crop::crop_image;
use crate::error::{Error, Result};
use crate::immich::PhotoClient;
use crate::{EPD_HEIGHT, EPD_WIDTH};
use std::io::Cursor;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, Router},
    Json,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub api_key: String,
    pub photo_api_base_url: String,
    pub samples_dir: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("PHOTO_API_KEY")
                .expect("Please provide a PHOTO_API_KEY before starting."),
            photo_api_base_url: std::env::var("PHOTO_API_URL")
                .expect("Please provide a PHOTO_API_URL before starting."),
            samples_dir: "samples".to_string(),
            port: 3005,
        }
    }
}

pub async fn start_server(config: ServerConfig) -> Result<()> {
    let photo_client = Arc::new(PhotoClient::new(config.clone()));

    // Create samples directory if it doesn't exist
    tokio::fs::create_dir_all(&config.samples_dir).await?;

    let app = Router::new()
        .route("/recent", get(get_recent))
        .nest_service("/samples", ServeDir::new(&config.samples_dir))
        .route("/health", get(health_check))
        .with_state(photo_client);

    let addr = format!("0.0.0.0:{}", config.port);
    println!("Starting server on {addr}");
    println!("Serving static files from: {}", config.samples_dir);
    println!("Health check available at: http://{addr}/health");
    println!("Recent photo endpoint: http://{addr}/recent");
    println!("Static files: http://{addr}/samples/");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(Error::Io)?;
    axum::serve(listener, app).await.map_err(Error::Io)?;

    Ok(())
}

pub async fn get_recent_image(client: Arc<PhotoClient>) -> Result<Vec<u8>> {
    let jpg = client.get_recent_photo().await.unwrap();
    let image = image::io::Reader::new(Cursor::new(jpg))
        .with_guessed_format()
        .unwrap()
        .decode()?;

    println!("Got an image of {}x{}", image.width(), image.height(),);

    let crop = crop_image(image, EPD_WIDTH as u32, EPD_HEIGHT as u32).await?;
    Ok(crop)
}

async fn get_recent(State(client): State<Arc<PhotoClient>>) -> impl IntoResponse {
    match get_recent_image(client).await {
        Ok(image_data) => {
            let headers = [
                (header::CONTENT_TYPE, "image/epd"),
                (header::CACHE_CONTROL, "public, max-age=5"), // Cache for 5 minutes
            ];
            (headers, image_data).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching recent photo: {e}");
            let error_response = Json(serde_json::json!({
                "error": "Failed to fetch recent photo"
            }));
            (StatusCode::INTERNAL_SERVER_ERROR, error_response).into_response()
        }
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "photo-desk-server"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.photo_api_base_url, "https://photos.dcdc.dev");
        assert_eq!(config.port, 3000);
        assert_eq!(config.samples_dir, "samples");
    }
}
