use crate::error::{Error, Result};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, Router},
    Json,
};
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde::Deserialize;
use std::io::Cursor;
use std::{num::NonZeroU32, sync::Arc};
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

#[derive(Debug, Deserialize)]
struct TimeBucket {
    #[serde(rename = "timeBucket")]
    time_bucket: String,
}

#[derive(Debug, Deserialize)]
struct ThumbnailInfo {
    id: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PhotoClient {
    client: Client,
    config: ServerConfig,
}

impl PhotoClient {
    pub fn new(config: ServerConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    pub async fn get_optimised_image(&self) -> Result<Vec<u8>> {
        let height = 480;
        let width = 800;

        let jpg = self.get_recent_photo().await?;

        // let image: ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        //     ImageBuffer::from_vec(info.width.into(), info.height.into(), buf)
        //         .expect("Failed to create image buffer");

        let image = image::io::Reader::new(Cursor::new(jpg))
            .with_guessed_format()?
            .decode()?;

        println!("Got an image of {}x{}", image.width(), image.height(),);

        let res = smartcrop::find_best_crop(
            &image,
            NonZeroU32::new(width.into()).unwrap(),
            NonZeroU32::new(height.into()).unwrap(),
        )
        .expect("Failed to find crop");

        println!(
            "cropping to x{} y{} w{} h{} ({})",
            res.crop.x, res.crop.y, res.crop.width, res.crop.height, res.score.total
        );
        let c = res.crop;
        let cropped = image.crop_imm(c.x, c.y, c.width, c.height);
        let scaled = cropped.resize(width, height, image::imageops::FilterType::Lanczos3);
        let mut out = Vec::new();
        scaled.write_to(
            &mut Cursor::new(&mut out),
            image::ImageOutputFormat::Jpeg(100),
        )?;

        Ok(out.clone())
    }

    pub async fn get_recent_photo(&self) -> Result<Vec<u8>> {
        println!("Getting buckets");
        let time_bucket_id = &self.get_fav_bucket().await?;
        println!("Chose {}", time_bucket_id);
        let thumbnail_id = &self.get_photo_from_bucket(time_bucket_id).await?;
        println!("Got {}", thumbnail_id);
        // Now download the actual image using the thumbnail ID
        self.download_image(&thumbnail_id).await
    }

    pub async fn get_fav_bucket(&self) -> Result<String> {
        // First, get the timeline data
        let timeline_url = format!("{}/api/timeline/buckets", self.config.photo_api_base_url);

        let params = [("isFavorite", "true"), ("isTrashed", "false")];

        let response = self
            .client
            .get(&timeline_url)
            .header("x-api-key", &self.config.api_key)
            .query(&params)
            .send()
            .await?;

        println!("response {}", response.status());
        if !response.status().is_success() {
            return Err(Error::InvalidApiResponse);
        }

        let api_response: Vec<TimeBucket> = response.json().await?;

        // Get a random bucket
        let time_bucket_id = api_response
            .choose(&mut rand::rng())
            .ok_or(Error::NoThumbnailFound)?
            .time_bucket
            .clone();

        return Ok(time_bucket_id);
    }

    async fn get_photo_from_bucket(&self, time_bucket_id: &String) -> Result<String> {
        // First, get the timeline data
        let timeline_url = format!("{}/api/timeline/bucket", self.config.photo_api_base_url);

        let params = [
            ("isFavorite", "true"),
            ("isTrashed", "false"),
            ("timeBucket", &time_bucket_id),
            // ("visibility", "timeline"),
            // ("withPartners", "true"),
            ("withStacked", "true"),
        ];

        let response = self
            .client
            .get(&timeline_url)
            .header("x-api-key", &self.config.api_key)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::InvalidApiResponse);
        }

        let api_response: ThumbnailInfo = response.json().await?;

        // Get the first thumbnail ID
        let thumbnail_id = api_response
            .id
            .first()
            .ok_or(Error::NoThumbnailFound)?
            .clone();

        return Ok(thumbnail_id);
    }

    async fn download_image(&self, image_id: &str) -> Result<Vec<u8>> {
        let image_url = format!(
            "{}/api/assets/{}/thumbnail?size=preview",
            self.config.photo_api_base_url, image_id
        );

        let response = self
            .client
            .get(&image_url)
            .header("x-api-key", &self.config.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::InvalidApiResponse);
        }

        let image_data = response.bytes().await?;
        Ok(image_data.to_vec())
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
    println!("Starting server on {}", addr);
    println!("Serving static files from: {}", config.samples_dir);
    println!("Health check available at: http://{}/health", addr);
    println!("Recent photo endpoint: http://{}/recent", addr);
    println!("Static files: http://{}/samples/", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(Error::Io)?;
    axum::serve(listener, app).await.map_err(Error::Io)?;

    Ok(())
}

async fn get_recent(State(client): State<Arc<PhotoClient>>) -> impl IntoResponse {
    match client.get_optimised_image().await {
        Ok(image_data) => {
            let headers = [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "public, max-age=300"), // Cache for 5 minutes
            ];
            (headers, image_data).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching recent photo: {}", e);
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

    #[tokio::test]
    async fn test_photo_client_creation() {
        let config = ServerConfig::default();
        let client = PhotoClient::new(config);
        // Just verify we can create the client without panicking
        assert!(!client.config.photo_api_base_url.is_empty());
    }
}
