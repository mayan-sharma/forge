use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use crate::http::json;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn parse<R: Read>(reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf_reader = BufReader::new(reader);
        
        // Parse status line
        let mut status_line = String::new();
        buf_reader.read_line(&mut status_line)?;
        let status_parts: Vec<&str> = status_line.trim().split_whitespace().collect();
        
        if status_parts.len() < 3 {
            return Err("Invalid HTTP response status line".into());
        }
        
        let status_code: u16 = status_parts[1].parse()
            .map_err(|_| "Invalid status code")?;
        let status_text = status_parts[2..].join(" ");
        
        // Parse headers
        let mut headers = HashMap::new();
        let mut content_length = 0usize;
        let mut is_chunked = false;
        
        loop {
            let mut header_line = String::new();
            buf_reader.read_line(&mut header_line)?;
            let header_line = header_line.trim();
            
            if header_line.is_empty() {
                break;
            }
            
            if let Some(colon_pos) = header_line.find(':') {
                let key = header_line[..colon_pos].trim().to_lowercase();
                let value = header_line[colon_pos + 1..].trim().to_string();
                
                if key == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                } else if key == "transfer-encoding" && value.contains("chunked") {
                    is_chunked = true;
                }
                
                headers.insert(key, value);
            }
        }
        
        // Parse body
        let body = if is_chunked {
            Self::read_chunked_body(&mut buf_reader)?
        } else if content_length > 0 {
            let mut body_bytes = vec![0; content_length];
            buf_reader.read_exact(&mut body_bytes)?;
            String::from_utf8_lossy(&body_bytes).to_string()
        } else {
            String::new()
        };
        
        Ok(HttpResponse {
            status_code,
            status_text,
            headers,
            body,
        })
    }

    // Streaming version that calls a callback for each JSON line
    pub fn parse_streaming<R, F>(reader: R, callback: F) -> Result<Self, Box<dyn std::error::Error>>
    where
        R: Read,
        F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        let mut buf_reader = BufReader::new(reader);
        
        // Parse status line
        let mut status_line = String::new();
        buf_reader.read_line(&mut status_line)?;
        let status_parts: Vec<&str> = status_line.trim().split_whitespace().collect();
        
        if status_parts.len() < 3 {
            return Err("Invalid HTTP response status line".into());
        }
        
        let status_code: u16 = status_parts[1].parse()
            .map_err(|_| "Invalid status code")?;
        let status_text = status_parts[2..].join(" ");
        
        // Parse headers
        let mut headers = HashMap::new();
        let mut content_length = 0usize;
        let mut is_chunked = false;
        
        loop {
            let mut header_line = String::new();
            buf_reader.read_line(&mut header_line)?;
            let header_line = header_line.trim();
            
            if header_line.is_empty() {
                break;
            }
            
            if let Some(colon_pos) = header_line.find(':') {
                let key = header_line[..colon_pos].trim().to_lowercase();
                let value = header_line[colon_pos + 1..].trim().to_string();
                
                if key == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                } else if key == "transfer-encoding" && value.contains("chunked") {
                    is_chunked = true;
                }
                
                headers.insert(key, value);
            }
        }
        
        // For streaming responses, read line by line and call callback
        if is_chunked {
            Self::read_chunked_body_streaming(&mut buf_reader, callback)?;
        } else {
            Self::read_body_streaming(&mut buf_reader, content_length, callback)?;
        }
        
        Ok(HttpResponse {
            status_code,
            status_text,
            headers,
            body: String::new(), // Body is consumed by the callback
        })
    }
    
    fn read_chunked_body<R: BufRead>(reader: &mut R) -> Result<String, Box<dyn std::error::Error>> {
        let mut body = String::new();
        
        loop {
            let mut chunk_size_line = String::new();
            reader.read_line(&mut chunk_size_line)?;
            let chunk_size_str = chunk_size_line.trim().split(';').next().unwrap_or("0");
            let chunk_size = usize::from_str_radix(chunk_size_str, 16)
                .map_err(|_| "Invalid chunk size")?;
            
            if chunk_size == 0 {
                // Read final CRLF
                let mut final_line = String::new();
                reader.read_line(&mut final_line)?;
                break;
            }
            
            let mut chunk_data = vec![0; chunk_size];
            reader.read_exact(&mut chunk_data)?;
            body.push_str(&String::from_utf8_lossy(&chunk_data));
            
            // Read trailing CRLF
            let mut trailing_line = String::new();
            reader.read_line(&mut trailing_line)?;
        }
        
        Ok(body)
    }

    fn read_chunked_body_streaming<R, F>(reader: &mut R, mut callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        R: BufRead,
        F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        loop {
            let mut chunk_size_line = String::new();
            reader.read_line(&mut chunk_size_line)?;
            let chunk_size_str = chunk_size_line.trim().split(';').next().unwrap_or("0");
            let chunk_size = usize::from_str_radix(chunk_size_str, 16)
                .map_err(|_| "Invalid chunk size")?;
            
            if chunk_size == 0 {
                // Read final CRLF
                let mut final_line = String::new();
                reader.read_line(&mut final_line)?;
                break;
            }
            
            let mut chunk_data = vec![0; chunk_size];
            reader.read_exact(&mut chunk_data)?;
            let chunk_str = String::from_utf8_lossy(&chunk_data);
            
            // Process each line in the chunk for JSON streaming
            for line in chunk_str.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    Self::process_json_line(line, &mut callback)?;
                }
            }
            
            // Read trailing CRLF
            let mut trailing_line = String::new();
            reader.read_line(&mut trailing_line)?;
        }
        
        Ok(())
    }

    fn read_body_streaming<R, F>(reader: &mut R, content_length: usize, mut callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        R: BufRead,
        F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        if content_length > 0 {
            let mut body_bytes = vec![0; content_length];
            reader.read_exact(&mut body_bytes)?;
            let body_str = String::from_utf8_lossy(&body_bytes);
            
            // Process each line for JSON streaming
            for line in body_str.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    Self::process_json_line(line, &mut callback)?;
                }
            }
        } else {
            // Read line by line until EOF
            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    Self::process_json_line(trimmed, &mut callback)?;
                }
                line.clear();
            }
        }
        
        Ok(())
    }

    fn process_json_line<F>(line: &str, callback: &mut F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        // Parse JSON line and extract response content
        if let Ok(json_obj) = json::parse_json(line) {
            if let Some(obj) = json_obj.as_object() {
                if let Some(response_text) = obj.get("response") {
                    if let Some(text) = response_text.as_string() {
                        callback(text)?;
                    }
                }
            }
        }
        Ok(())
    }
}