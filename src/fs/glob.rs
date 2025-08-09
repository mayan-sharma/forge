use std::path::{Path, PathBuf};
use std::fs;

/// A simple glob pattern matcher that supports basic patterns
pub struct GlobMatcher {
    pattern: String,
}

impl GlobMatcher {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }

    /// Match a file path against the glob pattern
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.match_pattern(&self.pattern, &path_str)
    }

    /// Internal pattern matching implementation
    fn match_pattern(&self, pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        self.match_recursive(&pattern_chars, &text_chars, 0, 0)
    }

    fn match_recursive(&self, pattern: &[char], text: &[char], p_idx: usize, t_idx: usize) -> bool {
        // End of pattern
        if p_idx >= pattern.len() {
            return t_idx >= text.len();
        }

        // Handle ** (match zero or more directories)
        if p_idx + 1 < pattern.len() && pattern[p_idx] == '*' && pattern[p_idx + 1] == '*' {
            // Skip the ** and any following /
            let mut next_p = p_idx + 2;
            while next_p < pattern.len() && pattern[next_p] == '/' {
                next_p += 1;
            }

            // Try matching from current position to end of text
            for i in t_idx..=text.len() {
                // Skip to next directory boundary or end
                if i == text.len() || (i > t_idx && text[i - 1] == '/') {
                    if self.match_recursive(pattern, text, next_p, i) {
                        return true;
                    }
                }
            }
            return false;
        }

        // Handle * (match within a directory/filename)
        if pattern[p_idx] == '*' {
            // Match zero characters
            if self.match_recursive(pattern, text, p_idx + 1, t_idx) {
                return true;
            }
            
            // Match one or more characters (but not '/')
            for i in t_idx..text.len() {
                if text[i] == '/' {
                    break;
                }
                if self.match_recursive(pattern, text, p_idx + 1, i + 1) {
                    return true;
                }
            }
            return false;
        }

        // Handle ? (match single character, not '/')
        if pattern[p_idx] == '?' {
            if t_idx >= text.len() || text[t_idx] == '/' {
                return false;
            }
            return self.match_recursive(pattern, text, p_idx + 1, t_idx + 1);
        }

        // Handle character sets [...]
        if pattern[p_idx] == '[' {
            if let Some(end_bracket) = pattern[p_idx + 1..].iter().position(|&c| c == ']') {
                let set_end = p_idx + 1 + end_bracket;
                let char_set: Vec<char> = pattern[p_idx + 1..set_end].to_vec();
                
                if t_idx >= text.len() {
                    return false;
                }

                let target_char = text[t_idx];
                let mut matched = false;
                let mut negated = false;
                let mut i = 0;

                // Check for negation
                if !char_set.is_empty() && char_set[0] == '^' {
                    negated = true;
                    i = 1;
                }

                // Check character ranges and literals
                while i < char_set.len() {
                    if i + 2 < char_set.len() && char_set[i + 1] == '-' {
                        // Range like a-z
                        if target_char >= char_set[i] && target_char <= char_set[i + 2] {
                            matched = true;
                            break;
                        }
                        i += 3;
                    } else {
                        // Single character
                        if target_char == char_set[i] {
                            matched = true;
                            break;
                        }
                        i += 1;
                    }
                }

                if (matched && !negated) || (!matched && negated) {
                    return self.match_recursive(pattern, text, set_end + 1, t_idx + 1);
                } else {
                    return false;
                }
            }
        }

        // Handle literal characters
        if t_idx >= text.len() || pattern[p_idx] != text[t_idx] {
            return false;
        }

        self.match_recursive(pattern, text, p_idx + 1, t_idx + 1)
    }
}

/// Find all files matching a glob pattern
pub fn glob(pattern: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let matcher = GlobMatcher::new(pattern);
    let mut results = Vec::new();
    
    // Handle absolute vs relative patterns
    let (root, search_pattern) = if pattern.starts_with('/') {
        (PathBuf::from("/"), pattern)
    } else if pattern.starts_with("./") {
        (std::env::current_dir()?, pattern)
    } else {
        (std::env::current_dir()?, pattern)
    };

    // For ** patterns, we need recursive search
    if search_pattern.contains("**") {
        collect_recursive(&root, &matcher, &mut results)?;
    } else {
        // For non-recursive patterns, be more efficient
        collect_directory(&root, &matcher, &mut results)?;
    }

    // Sort results for consistent output
    results.sort();
    Ok(results)
}

/// Recursively collect files matching the pattern
fn collect_recursive(dir: &Path, matcher: &GlobMatcher, results: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if matcher.matches(&path) {
            results.push(path.clone());
        }

        if path.is_dir() {
            collect_recursive(&path, matcher, results)?;
        }
    }

    Ok(())
}

/// Collect files from a single directory (non-recursive)
fn collect_directory(dir: &Path, matcher: &GlobMatcher, results: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if matcher.matches(&path) {
            results.push(path);
        }
    }

    Ok(())
}

/// Utility function to expand glob patterns in file lists
pub fn expand_globs(patterns: &[String]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut all_files = Vec::new();
    
    for pattern in patterns {
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            // It's a glob pattern
            let mut matched_files = glob(pattern)?;
            all_files.append(&mut matched_files);
        } else {
            // It's a literal file path
            all_files.push(PathBuf::from(pattern));
        }
    }

    // Remove duplicates and sort
    all_files.sort();
    all_files.dedup();
    
    Ok(all_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_wildcard() {
        let matcher = GlobMatcher::new("*.rs");
        assert!(matcher.matches(Path::new("main.rs")));
        assert!(matcher.matches(Path::new("lib.rs")));
        assert!(!matcher.matches(Path::new("main.txt")));
    }

    #[test]
    fn test_double_wildcard() {
        let matcher = GlobMatcher::new("**/*.rs");
        assert!(matcher.matches(Path::new("src/main.rs")));
        assert!(matcher.matches(Path::new("src/lib/mod.rs")));
        assert!(matcher.matches(Path::new("tests/unit/test.rs")));
        assert!(!matcher.matches(Path::new("README.md")));
    }

    #[test]
    fn test_question_mark() {
        let matcher = GlobMatcher::new("test?.rs");
        assert!(matcher.matches(Path::new("test1.rs")));
        assert!(matcher.matches(Path::new("testa.rs")));
        assert!(!matcher.matches(Path::new("test12.rs")));
        assert!(!matcher.matches(Path::new("test.rs")));
    }

    #[test]
    fn test_character_sets() {
        let matcher = GlobMatcher::new("test[0-9].rs");
        assert!(matcher.matches(Path::new("test1.rs")));
        assert!(matcher.matches(Path::new("test9.rs")));
        assert!(!matcher.matches(Path::new("testa.rs")));
        
        let matcher2 = GlobMatcher::new("test[^0-9].rs");
        assert!(!matcher2.matches(Path::new("test1.rs")));
        assert!(matcher2.matches(Path::new("testa.rs")));
    }

    #[test]
    fn test_complex_patterns() {
        let matcher = GlobMatcher::new("src/**/*.{rs,toml}");
        // This would need extension to support {a,b} syntax
        // For now, testing the ** part
        let simple_matcher = GlobMatcher::new("src/**/*.rs");
        assert!(simple_matcher.matches(Path::new("src/main.rs")));
        assert!(simple_matcher.matches(Path::new("src/cli/commands/mod.rs")));
    }
}