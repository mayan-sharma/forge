use std::collections::HashMap;

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.headers.insert("Content-Length".to_string(), body.len().to_string());
        self.body = Some(body);
        self
    }

    pub fn to_http_string(&self, host: &str) -> String {
        let mut request = format!("{} {} HTTP/1.1\r\n", self.method, self.path);
        request.push_str(&format!("Host: {}\r\n", host));
        
        for (key, value) in &self.headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        request.push_str("\r\n");
        
        if let Some(body) = &self.body {
            request.push_str(body);
        }
        
        request
    }
}