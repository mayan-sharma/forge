use std::io::Write;
use std::net::TcpStream;
use crate::http::{json, request::HttpRequest, response::HttpResponse};

pub struct OllamaClient {
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            base_url: base_url.to_string(),
        })
    }

    pub fn generate(&self, model: &str, prompt: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        if stream {
            let mut result = String::new();
            self.generate_stream(model, prompt, |chunk| {
                result.push_str(chunk);
                Ok(())
            })?;
            Ok(result)
        } else {
            self.generate_non_stream(model, prompt)
        }
    }

    pub fn generate_stream<F>(&self, model: &str, prompt: &str, callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        // Parse URL
        let url = format!("{}/api/generate", self.base_url);
        let url = url.strip_prefix("http://").ok_or("Invalid URL format")?;
        
        let (host, port) = self.parse_url(url)?;
        let path = self.extract_path(url, "/api/generate");

        // Create JSON payload for streaming
        let json_body = json::serialize_ollama_request(model, prompt, true);

        // Create HTTP request
        let request = HttpRequest::new("POST", path)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_header("Connection", "close")
            .with_body(json_body);

        // Connect and send request
        let mut stream = TcpStream::connect((host, port))?;
        let request_str = request.to_http_string(host);
        stream.write_all(request_str.as_bytes())?;
        stream.flush()?;

        // Parse response headers
        let response = HttpResponse::parse_streaming(stream, callback)?;

        if response.status_code != 200 {
            return Err(format!("HTTP error {}: {}", response.status_code, response.status_text).into());
        }

        Ok(())
    }

    fn generate_non_stream(&self, model: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse URL
        let url = format!("{}/api/generate", self.base_url);
        let url = url.strip_prefix("http://").ok_or("Invalid URL format")?;
        
        let (host, port) = self.parse_url(url)?;
        let path = self.extract_path(url, "/api/generate");

        // Create JSON payload for non-streaming
        let json_body = json::serialize_ollama_request(model, prompt, false);

        // Create HTTP request
        let request = HttpRequest::new("POST", path)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_header("Connection", "close")
            .with_body(json_body);

        // Connect and send request
        let mut stream = TcpStream::connect((host, port))?;
        let request_str = request.to_http_string(host);
        stream.write_all(request_str.as_bytes())?;
        stream.flush()?;

        // Parse response
        let response = HttpResponse::parse(stream)?;

        if response.status_code != 200 {
            return Err(format!("HTTP error {}: {}", response.status_code, response.status_text).into());
        }

        // Parse JSON response
        let json_response = json::parse_json(&response.body)?;
        
        if let Some(obj) = json_response.as_object() {
            if let Some(response_text) = obj.get("response") {
                if let Some(text) = response_text.as_string() {
                    return Ok(text.clone());
                }
            }
        }

        Err("Failed to extract response from JSON".into())
    }

    pub fn list_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Parse URL for /api/tags endpoint
        let url = format!("{}/api/tags", self.base_url);
        let url = url.strip_prefix("http://").ok_or("Invalid URL format")?;
        
        let (host, port) = self.parse_url(url)?;
        let path = self.extract_path(url, "/api/tags");

        // Create HTTP request
        let request = HttpRequest::new("GET", path)
            .with_header("Accept", "application/json")
            .with_header("Connection", "close");

        // Connect and send request
        let mut stream = TcpStream::connect((host, port))?;
        let request_str = request.to_http_string(host);
        stream.write_all(request_str.as_bytes())?;
        stream.flush()?;

        // Parse response
        let response = HttpResponse::parse(stream)?;

        if response.status_code != 200 {
            return Err(format!("HTTP error {}: {}", response.status_code, response.status_text).into());
        }

        // Parse JSON response and extract model names
        let json_response = json::parse_json(&response.body)?;
        let mut models = Vec::new();

        if let Some(obj) = json_response.as_object() {
            if let Some(models_array) = obj.get("models") {
                if let json::JsonValue::Array(arr) = models_array {
                    for model in arr {
                        if let Some(model_obj) = model.as_object() {
                            if let Some(name) = model_obj.get("name") {
                                if let Some(name_str) = name.as_string() {
                                    models.push(name_str.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(models)
    }

    // Helper methods to reduce code duplication
    fn parse_url<'a>(&self, url: &'a str) -> Result<(&'a str, u16), Box<dyn std::error::Error>> {
        let (host, port) = if let Some(colon_pos) = url.find(':') {
            let host = &url[..colon_pos];
            let port_part = &url[colon_pos + 1..];
            let port = if let Some(slash_pos) = port_part.find('/') {
                port_part[..slash_pos].parse().unwrap_or(80)
            } else {
                port_part.parse().unwrap_or(80)
            };
            (host, port)
        } else {
            let host = if let Some(slash_pos) = url.find('/') {
                &url[..slash_pos]
            } else {
                url
            };
            (host, 80)
        };
        Ok((host, port))
    }

    fn extract_path<'a>(&self, url: &'a str, default_path: &'a str) -> &'a str {
        if let Some(slash_pos) = url.find('/') {
            &url[slash_pos..]
        } else {
            default_path
        }
    }
}