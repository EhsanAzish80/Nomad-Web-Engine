//! Network layer for the Nomad Web Engine.
//!
//! Handles HTTP/HTTPS requests, DNS resolution, and network protocols.

use reqwest::blocking::Client;
use std::time::Duration;
use thiserror::Error;

/// Maximum size of HTTP response (10 MB)
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum size of image files (5 MB)
pub const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024;

/// Default timeout for HTTP requests (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Network-related errors.
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Response too large (max: {MAX_RESPONSE_SIZE} bytes)")]
    ResponseTooLarge,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Too many redirects")]
    MaxRedirects,

    #[error("Image decode error: {0}")]
    ImageDecode(String),
}

/// Response from a network fetch.
pub struct FetchResponse {
    /// The final URL after redirects.
    pub url: String,
    /// The response body as a string.
    pub body: String,
    /// HTTP status code.
    pub status: u16,
}

/// Image data from a fetch operation.
pub struct ImageData {
    /// The final URL after redirects.
    pub url: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    pub rgba_data: Vec<u8>,
}

/// Network layer for fetching resources.
pub struct NetworkLayer {
    client: Client,
    timeout: Duration,
    max_size: usize,
}

impl NetworkLayer {
    /// Creates a new network layer with default settings.
    pub fn new() -> Result<Self, NetworkError> {
        Self::with_config(DEFAULT_TIMEOUT_SECS, MAX_RESPONSE_SIZE)
    }

    /// Creates a new network layer with custom configuration.
    pub fn with_config(timeout_secs: u64, max_size: usize) -> Result<Self, NetworkError> {
        let timeout = Duration::from_secs(timeout_secs);

        let client = Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("NomadWebEngine/0.1.0")
            .build()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;

        Ok(Self {
            client,
            timeout,
            max_size,
        })
    }

    /// Fetches content from a URL using HTTP/HTTPS GET.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    ///
    /// # Returns
    ///
    /// A `FetchResponse` containing the final URL, body, and status code.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The request times out
    /// - The response is too large
    /// - The request fails for any other reason
    pub fn fetch(&self, url: &str) -> Result<FetchResponse, NetworkError> {
        // Validate URL
        let parsed_url =
            reqwest::Url::parse(url).map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

        // Only allow HTTP and HTTPS
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(NetworkError::InvalidUrl(format!(
                "Unsupported scheme: {}",
                parsed_url.scheme()
            )));
        }

        // Send request
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    NetworkError::Timeout(self.timeout.as_secs())
                } else {
                    NetworkError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();

        // Check content length if available
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_size {
                return Err(NetworkError::ResponseTooLarge);
            }
        }

        // Read body with size limit
        let body = response
            .text()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;

        if body.len() > self.max_size {
            return Err(NetworkError::ResponseTooLarge);
        }

        Ok(FetchResponse {
            url: final_url,
            body,
            status,
        })
    }

    /// Sends a POST request to a URL with form data.
    ///
    /// Returns a `FetchResponse` on success, or a `NetworkError` if:
    /// - The URL is invalid
    /// - The request times out
    /// - The response is too large
    /// - The request fails for any other reason
    pub fn post(&self, url: &str, form_data: &str) -> Result<FetchResponse, NetworkError> {
        // Validate URL
        let parsed_url =
            reqwest::Url::parse(url).map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

        // Only allow HTTP and HTTPS
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(NetworkError::InvalidUrl(format!(
                "Unsupported scheme: {}",
                parsed_url.scheme()
            )));
        }

        // Send POST request with form data
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data.to_string())
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    NetworkError::Timeout(self.timeout.as_secs())
                } else {
                    NetworkError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();

        // Check content length if available
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_size {
                return Err(NetworkError::ResponseTooLarge);
            }
        }

        // Read body with size limit
        let body = response
            .text()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;

        if body.len() > self.max_size {
            return Err(NetworkError::ResponseTooLarge);
        }

        Ok(FetchResponse {
            url: final_url,
            body,
            status,
        })
    }

    /// Fetches an image from a URL and decodes it to RGBA format.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the image to fetch
    ///
    /// # Returns
    ///
    /// An `ImageData` containing the decoded image in RGBA format.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The request times out
    /// - The image is too large
    /// - The image cannot be decoded
    pub fn fetch_image(&self, url: &str) -> Result<ImageData, NetworkError> {
        // Validate URL
        let parsed_url =
            reqwest::Url::parse(url).map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

        // Only allow HTTP and HTTPS
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(NetworkError::InvalidUrl(format!(
                "Unsupported scheme: {}",
                parsed_url.scheme()
            )));
        }

        // Send request
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    NetworkError::Timeout(self.timeout.as_secs())
                } else {
                    NetworkError::RequestFailed(e.to_string())
                }
            })?;

        let final_url = response.url().to_string();

        // Check content length if available
        if let Some(content_length) = response.content_length() {
            if content_length as usize > MAX_IMAGE_SIZE {
                return Err(NetworkError::ResponseTooLarge);
            }
        }

        // Read bytes with size limit
        let bytes = response
            .bytes()
            .map_err(|e| NetworkError::RequestFailed(e.to_string()))?;

        if bytes.len() > MAX_IMAGE_SIZE {
            return Err(NetworkError::ResponseTooLarge);
        }

        // Decode image
        let img = image::load_from_memory(&bytes)
            .map_err(|e| NetworkError::ImageDecode(e.to_string()))?;

        // Convert to RGBA8
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        Ok(ImageData {
            url: final_url,
            width,
            height,
            rgba_data: rgba_img.into_raw(),
        })
    }
}

impl Default for NetworkLayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default NetworkLayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_layer_creation() {
        let network = NetworkLayer::new();
        assert!(network.is_ok());
    }

    #[test]
    fn test_invalid_scheme() {
        let network = NetworkLayer::new().unwrap();
        let result = network.fetch("ftp://example.com");
        assert!(matches!(result, Err(NetworkError::InvalidUrl(_))));
    }

    #[test]
    fn test_invalid_url() {
        let network = NetworkLayer::new().unwrap();
        let result = network.fetch("not a url");
        assert!(matches!(result, Err(NetworkError::InvalidUrl(_))));
    }

    #[test]
    fn test_custom_config() {
        let network = NetworkLayer::with_config(10, 1024);
        assert!(network.is_ok());
    }
}
