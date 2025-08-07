use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

#[derive(Debug)]
pub struct JsonError(pub String);

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JSON Error: {}", self.0)
    }
}

impl std::error::Error for JsonError {}

pub struct JsonParser {
    input: Vec<char>,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Err(JsonError("Unexpected end of input".to_string()));
        }

        match self.input[self.pos] {
            '"' => self.parse_string(),
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            't' | 'f' => self.parse_boolean(),
            'n' => self.parse_null(),
            c if c.is_numeric() || c == '-' => self.parse_number(),
            _ => Err(JsonError(format!("Unexpected character: {}", self.input[self.pos]))),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, JsonError> {
        if self.input[self.pos] != '"' {
            return Err(JsonError("Expected '\"'".to_string()));
        }
        self.pos += 1;

        let mut string = String::new();
        while self.pos < self.input.len() && self.input[self.pos] != '"' {
            if self.input[self.pos] == '\\' && self.pos + 1 < self.input.len() {
                self.pos += 1;
                match self.input[self.pos] {
                    '"' => string.push('"'),
                    '\\' => string.push('\\'),
                    '/' => string.push('/'),
                    'b' => string.push('\u{0008}'),
                    'f' => string.push('\u{000C}'),
                    'n' => string.push('\n'),
                    'r' => string.push('\r'),
                    't' => string.push('\t'),
                    _ => return Err(JsonError("Invalid escape sequence".to_string())),
                }
            } else {
                string.push(self.input[self.pos]);
            }
            self.pos += 1;
        }

        if self.pos >= self.input.len() {
            return Err(JsonError("Unterminated string".to_string()));
        }

        self.pos += 1; // Skip closing quote
        Ok(JsonValue::String(string))
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        if self.input[self.pos] != '{' {
            return Err(JsonError("Expected '{'".to_string()));
        }
        self.pos += 1;

        let mut object = HashMap::new();
        self.skip_whitespace();

        if self.pos < self.input.len() && self.input[self.pos] == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            
            if self.pos >= self.input.len() {
                return Err(JsonError("Unterminated object".to_string()));
            }

            // Parse key
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err(JsonError("Expected string key".to_string())),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != ':' {
                return Err(JsonError("Expected ':'".to_string()));
            }
            self.pos += 1;

            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err(JsonError("Unterminated object".to_string()));
            }

            match self.input[self.pos] {
                ',' => {
                    self.pos += 1;
                    continue;
                }
                '}' => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(JsonError("Expected ',' or '}'".to_string())),
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        if self.input[self.pos] != '[' {
            return Err(JsonError("Expected '['".to_string()));
        }
        self.pos += 1;

        let mut array = Vec::new();
        self.skip_whitespace();

        if self.pos < self.input.len() && self.input[self.pos] == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err(JsonError("Unterminated array".to_string()));
            }

            match self.input[self.pos] {
                ',' => {
                    self.pos += 1;
                    continue;
                }
                ']' => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(JsonError("Expected ',' or ']'".to_string())),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, JsonError> {
        if self.pos + 3 < self.input.len() && 
           self.input[self.pos..self.pos+4].iter().collect::<String>() == "true" {
            self.pos += 4;
            Ok(JsonValue::Boolean(true))
        } else if self.pos + 4 < self.input.len() && 
                  self.input[self.pos..self.pos+5].iter().collect::<String>() == "false" {
            self.pos += 5;
            Ok(JsonValue::Boolean(false))
        } else {
            Err(JsonError("Invalid boolean".to_string()))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
        if self.pos + 3 < self.input.len() && 
           self.input[self.pos..self.pos+4].iter().collect::<String>() == "null" {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(JsonError("Invalid null".to_string()))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.pos;
        
        if self.input[self.pos] == '-' {
            self.pos += 1;
        }

        if self.pos >= self.input.len() || !self.input[self.pos].is_numeric() {
            return Err(JsonError("Invalid number".to_string()));
        }

        while self.pos < self.input.len() && 
              (self.input[self.pos].is_numeric() || self.input[self.pos] == '.') {
            self.pos += 1;
        }

        let number_str: String = self.input[start..self.pos].iter().collect();
        match number_str.parse::<f64>() {
            Ok(n) => Ok(JsonValue::Number(n)),
            Err(_) => Err(JsonError("Invalid number format".to_string())),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
}

impl JsonValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

pub fn serialize_ollama_request(model: &str, prompt: &str, stream: bool) -> String {
    format!(
        r#"{{"model":"{}","prompt":"{}","stream":{}}}"#,
        model.replace('"', r#"\""#),
        prompt.replace('"', r#"\""#).replace('\n', r#"\n"#),
        stream
    )
}

pub fn parse_json(input: &str) -> Result<JsonValue, JsonError> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}