use crate::fs::operations::read_file;

pub struct TextSearcher {
    case_sensitive: bool,
    whole_word: bool,
}

impl TextSearcher {
    pub fn new() -> Self {
        Self {
            case_sensitive: true,
            whole_word: false,
        }
    }

    #[allow(dead_code)]
    pub fn case_insensitive(mut self) -> Self {
        self.case_sensitive = false;
        self
    }

    #[allow(dead_code)]
    pub fn whole_word(mut self) -> Self {
        self.whole_word = true;
        self
    }

    #[allow(dead_code)]
    pub fn search_in_text(&self, text: &str, pattern: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        
        let search_text = if self.case_sensitive { text.to_string() } else { text.to_lowercase() };
        let search_pattern = if self.case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };

        let mut start = 0;
        while let Some(pos) = search_text[start..].find(&search_pattern) {
            let actual_pos = start + pos;
            
            if self.whole_word && !self.is_word_boundary(text, actual_pos, search_pattern.len()) {
                start = actual_pos + 1;
                continue;
            }

            matches.push((actual_pos, search_pattern.len()));
            start = actual_pos + 1;
        }

        matches
    }

    #[allow(dead_code)]
    pub fn search_in_file(&self, file_path: &str, pattern: &str) -> Result<Vec<(usize, usize, usize)>, Box<dyn std::error::Error>> {
        let content = read_file(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            let line_matches = self.search_in_text(line, pattern);
            for (col, len) in line_matches {
                matches.push((line_num + 1, col + 1, len)); // 1-based indexing
            }
        }

        Ok(matches)
    }

    fn is_word_boundary(&self, text: &str, pos: usize, pattern_len: usize) -> bool {
        let chars: Vec<char> = text.chars().collect();
        
        // Check start boundary
        let start_ok = if pos == 0 {
            true
        } else {
            let prev_char = chars[pos - 1];
            !prev_char.is_alphanumeric() && prev_char != '_'
        };

        // Check end boundary
        let end_pos = pos + pattern_len;
        let end_ok = if end_pos >= chars.len() {
            true
        } else {
            let next_char = chars[end_pos];
            !next_char.is_alphanumeric() && next_char != '_'
        };

        start_ok && end_ok
    }
}

impl Default for TextSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn search_text(text: &str, pattern: &str) -> Vec<(usize, usize)> {
    TextSearcher::new().search_in_text(text, pattern)
}

#[allow(dead_code)]
pub fn search_text_case_insensitive(text: &str, pattern: &str) -> Vec<(usize, usize)> {
    TextSearcher::new().case_insensitive().search_in_text(text, pattern)
}

#[allow(dead_code)]
pub fn search_file(file_path: &str, pattern: &str) -> Result<Vec<(usize, usize, usize)>, Box<dyn std::error::Error>> {
    TextSearcher::new().search_in_file(file_path, pattern)
}

#[allow(dead_code)]
pub fn search_multiple_files(file_paths: &[String], pattern: &str) -> Result<Vec<(String, Vec<(usize, usize, usize)>)>, Box<dyn std::error::Error>> {
    let searcher = TextSearcher::new();
    let mut results = Vec::new();

    for file_path in file_paths {
        match searcher.search_in_file(file_path, pattern) {
            Ok(matches) => {
                if !matches.is_empty() {
                    results.push((file_path.clone(), matches));
                }
            }
            Err(_) => {
                // Skip files that can't be read
                continue;
            }
        }
    }

    Ok(results)
}