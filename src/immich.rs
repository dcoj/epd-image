use crate::error::{Error, Result};
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde::Deserialize;

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
    config: crate::ServerConfig,
}

impl PhotoClient {
    pub fn new(config: crate::ServerConfig) -> Self {
        let client = Client::new();
        Self { client, config }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = crate::ServerConfig::default();
        assert_eq!(config.photo_api_base_url, "https://photos.dcdc.dev");
        assert_eq!(config.port, 3000);
        assert_eq!(config.samples_dir, "samples");
    }

    #[tokio::test]
    async fn test_photo_client_creation() {
        let config = crate::ServerConfig::default();
        let client = PhotoClient::new(config);
        // Just verify we can create the client without panicking
        assert!(!client.config.photo_api_base_url.is_empty());
    }
}
