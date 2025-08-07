#![allow(dead_code)]

use super::input::{InputReader, Key};
use super::history::CommandHistory;
use std::io::{self, Write};

pub struct LineEditor {
    input_reader: InputReader,
    history: CommandHistory,
    current_line: String,
    cursor_pos: usize,
    prompt: String,
    multiline_mode: bool,
    completion_callback: Option<Box<dyn Fn(&str) -> Vec<String>>>,
    suggestions: Vec<String>,
    suggestion_index: Option<usize>,
}

impl LineEditor {
    pub fn new() -> Self {
        LineEditor {
            input_reader: InputReader::new(),
            history: CommandHistory::new(1000),
            current_line: String::new(),
            cursor_pos: 0,
            prompt: "> ".to_string(),
            multiline_mode: false,
            completion_callback: None,
            suggestions: Vec::new(),
            suggestion_index: None,
        }
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    pub fn with_history_file(mut self, path: &str) -> Self {
        let _ = self.history.load_from_file(path);
        self
    }

    pub fn with_completion_callback<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(&str) -> Vec<String> + 'static,
    {
        self.completion_callback = Some(Box::new(callback));
        self
    }

    pub fn enable_multiline(&mut self) {
        self.multiline_mode = true;
    }

    pub fn disable_multiline(&mut self) {
        self.multiline_mode = false;
    }

    pub fn read_line(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self.current_line.clear();
        self.cursor_pos = 0;
        self.suggestions.clear();
        self.suggestion_index = None;
        self.history.reset_navigation();

        self.display_prompt()?;

        loop {
            let key = self.input_reader.read_key()?;
            
            match key {
                Key::Enter => {
                    if self.multiline_mode && self.should_continue_multiline() {
                        self.handle_multiline_continue()?;
                    } else {
                        self.finish_input()?;
                        break;
                    }
                }
                
                Key::Ctrl('c') => {
                    println!("^C");
                    self.current_line.clear();
                    return Err("Interrupted".into());
                }
                
                Key::Ctrl('d') => {
                    if self.current_line.is_empty() {
                        return Err("EOF".into());
                    } else {
                        self.delete_char_at_cursor()?;
                    }
                }
                
                Key::Backspace => {
                    self.handle_backspace()?;
                }
                
                Key::Delete => {
                    self.delete_char_at_cursor()?;
                }
                
                Key::ArrowLeft => {
                    self.move_cursor_left()?;
                }
                
                Key::ArrowRight => {
                    self.move_cursor_right()?;
                }
                
                Key::ArrowUp => {
                    self.handle_history_up()?;
                }
                
                Key::ArrowDown => {
                    self.handle_history_down()?;
                }
                
                Key::Home | Key::Ctrl('a') => {
                    self.move_cursor_to_start()?;
                }
                
                Key::End | Key::Ctrl('e') => {
                    self.move_cursor_to_end()?;
                }
                
                Key::Ctrl('k') => {
                    self.kill_to_end()?;
                }
                
                Key::Ctrl('u') => {
                    self.kill_to_start()?;
                }
                
                Key::Ctrl('w') => {
                    self.kill_word_backwards()?;
                }
                
                Key::Ctrl('l') => {
                    self.clear_screen_and_redraw()?;
                }
                
                Key::Tab => {
                    self.handle_tab_completion()?;
                }
                
                Key::Ctrl('r') => {
                    self.handle_reverse_search()?;
                }
                
                Key::Char(c) => {
                    self.insert_char(c)?;
                }
                
                Key::Space => {
                    self.insert_char(' ')?;
                }
                
                _ => {
                    // Ignore other keys
                }
            }
        }

        let result = self.current_line.clone();
        if !result.trim().is_empty() {
            self.history.add_command(result.clone());
        }

        Ok(result)
    }

    fn display_prompt(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("{}", self.prompt);
        io::stdout().flush()?;
        Ok(())
    }

    fn redraw_line(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Move to beginning of line and clear it
        print!("\r\x1B[K");
        
        // Display prompt and current line
        print!("{}{}", self.prompt, self.current_line);
        
        // Position cursor correctly
        let cursor_offset = self.cursor_pos;
        if cursor_offset < self.current_line.len() {
            let remaining = self.current_line.len() - cursor_offset;
            print!("\x1B[{}D", remaining);
        }
        
        io::stdout().flush()?;
        Ok(())
    }

    fn insert_char(&mut self, c: char) -> Result<(), Box<dyn std::error::Error>> {
        self.suggestions.clear();
        self.suggestion_index = None;
        
        self.current_line.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        
        self.redraw_line()?;
        Ok(())
    }

    fn handle_backspace(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.current_line.remove(self.cursor_pos);
            self.redraw_line()?;
        }
        Ok(())
    }

    fn delete_char_at_cursor(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos < self.current_line.len() {
            self.current_line.remove(self.cursor_pos);
            self.redraw_line()?;
        }
        Ok(())
    }

    fn move_cursor_left(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            print!("\x1B[D"); // Move cursor left
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn move_cursor_right(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos < self.current_line.len() {
            self.cursor_pos += 1;
            print!("\x1B[C"); // Move cursor right
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn move_cursor_to_start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos > 0 {
            print!("\x1B[{}D", self.cursor_pos);
            self.cursor_pos = 0;
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn move_cursor_to_end(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos < self.current_line.len() {
            let distance = self.current_line.len() - self.cursor_pos;
            print!("\x1B[{}C", distance);
            self.cursor_pos = self.current_line.len();
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn handle_history_up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(command) = self.history.get_previous() {
            self.current_line = command.clone();
            self.cursor_pos = self.current_line.len();
            self.redraw_line()?;
        }
        Ok(())
    }

    fn handle_history_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.history.get_next() {
            Some(command) => {
                self.current_line = command.clone();
                self.cursor_pos = self.current_line.len();
                self.redraw_line()?;
            }
            None => {
                // Back to current empty line
                self.current_line.clear();
                self.cursor_pos = 0;
                self.redraw_line()?;
            }
        }
        Ok(())
    }

    fn kill_to_end(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_line.truncate(self.cursor_pos);
        self.redraw_line()?;
        Ok(())
    }

    fn kill_to_start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_line.drain(0..self.cursor_pos);
        self.cursor_pos = 0;
        self.redraw_line()?;
        Ok(())
    }

    fn kill_word_backwards(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cursor_pos == 0 {
            return Ok(());
        }

        let mut new_pos = self.cursor_pos;
        let chars: Vec<char> = self.current_line.chars().collect();
        
        // Skip whitespace backwards
        while new_pos > 0 && chars[new_pos - 1].is_whitespace() {
            new_pos -= 1;
        }
        
        // Skip word characters backwards
        while new_pos > 0 && !chars[new_pos - 1].is_whitespace() {
            new_pos -= 1;
        }
        
        // Remove the range
        self.current_line.drain(new_pos..self.cursor_pos);
        self.cursor_pos = new_pos;
        self.redraw_line()?;
        Ok(())
    }

    fn clear_screen_and_redraw(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("\x1B[2J\x1B[H"); // Clear screen and move to top
        self.redraw_line()?;
        Ok(())
    }

    fn handle_tab_completion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(callback) = &self.completion_callback {
            let prefix = self.get_current_word();
            if !prefix.is_empty() {
                let completions = callback(&prefix);
                
                if completions.len() == 1 {
                    // Single completion - insert it
                    self.complete_current_word(&completions[0])?;
                } else if completions.len() > 1 {
                    // Multiple completions - show them
                    self.show_completions(&completions)?;
                }
            }
        }
        Ok(())
    }

    fn handle_reverse_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        print!("\r\x1B[K");
        print!("(reverse-i-search)`': ");
        io::stdout().flush()?;
        
        let mut search_term = String::new();
        let mut search_active = true;
        
        while search_active {
            let key = self.input_reader.read_key()?;
            
            match key {
                Key::Char(c) => {
                    search_term.push(c);
                    self.update_search_display(&search_term)?;
                }
                Key::Backspace => {
                    if !search_term.is_empty() {
                        search_term.pop();
                        self.update_search_display(&search_term)?;
                    }
                }
                Key::Ctrl('r') => {
                    // Continue search
                    if let Some(result) = self.find_in_history(&search_term) {
                        self.current_line = result.clone();
                        self.cursor_pos = self.current_line.len();
                    }
                    self.update_search_display(&search_term)?;
                }
                Key::Enter | Key::Ctrl('c') | Key::Escape => {
                    search_active = false;
                }
                _ => {}
            }
        }
        
        print!("\r\x1B[K");
        self.redraw_line()?;
        Ok(())
    }

    fn update_search_display(&self, search_term: &str) -> Result<(), Box<dyn std::error::Error>> {
        print!("\r\x1B[K");
        if let Some(result) = self.find_in_history(search_term) {
            print!("(reverse-i-search)`{}': {}", search_term, result);
        } else {
            print!("(failed reverse-i-search)`{}': ", search_term);
        }
        io::stdout().flush()?;
        Ok(())
    }

    fn find_in_history(&self, search_term: &str) -> Option<&String> {
        if search_term.is_empty() {
            return None;
        }
        
        let matches = self.history.find_matching(search_term);
        matches.first().copied()
    }

    fn get_current_word(&self) -> String {
        let chars: Vec<char> = self.current_line.chars().collect();
        let mut start = self.cursor_pos;
        
        // Find the start of the current word
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }
        
        chars[start..self.cursor_pos].iter().collect()
    }

    fn complete_current_word(&mut self, completion: &str) -> Result<(), Box<dyn std::error::Error>> {
        let current_word = self.get_current_word();
        let word_start = self.cursor_pos - current_word.len();
        
        // Replace current word with completion
        self.current_line.drain(word_start..self.cursor_pos);
        self.current_line.insert_str(word_start, completion);
        self.cursor_pos = word_start + completion.len();
        
        self.redraw_line()?;
        Ok(())
    }

    fn show_completions(&self, completions: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        println!(); // New line
        
        // Show completions in columns
        let term_width = 80; // Assume 80 columns for now
        let max_width = completions.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
        let columns = (term_width / max_width).max(1);
        
        for (i, completion) in completions.iter().enumerate() {
            print!("{:<width$}", completion, width = max_width);
            if (i + 1) % columns == 0 {
                println!();
            }
        }
        
        if completions.len() % columns != 0 {
            println!();
        }
        
        self.redraw_line()?;
        Ok(())
    }

    fn should_continue_multiline(&self) -> bool {
        // Simple heuristic: continue if line ends with backslash or has unclosed brackets
        let trimmed = self.current_line.trim_end();
        if trimmed.ends_with('\\') {
            return true;
        }
        
        // Count brackets/braces/parens
        let mut paren_count = 0;
        let mut brace_count = 0;
        let mut bracket_count = 0;
        
        for c in self.current_line.chars() {
            match c {
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }
        }
        
        paren_count > 0 || brace_count > 0 || bracket_count > 0
    }

    fn handle_multiline_continue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_line.push('\n');
        println!(); // Move to next line
        print!("... "); // Continuation prompt
        io::stdout().flush()?;
        Ok(())
    }

    fn finish_input(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(); // Move to next line
        Ok(())
    }

    pub fn save_history(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.history.save_to_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = LineEditor::new();
        assert_eq!(editor.prompt, "> ");
        assert_eq!(editor.current_line, "");
        assert_eq!(editor.cursor_pos, 0);
        assert!(!editor.multiline_mode);
    }

    #[test]
    fn test_editor_with_prompt() {
        let editor = LineEditor::new().with_prompt("$ ");
        assert_eq!(editor.prompt, "$ ");
    }

    #[test]
    fn test_multiline_mode() {
        let mut editor = LineEditor::new();
        assert!(!editor.multiline_mode);
        
        editor.enable_multiline();
        assert!(editor.multiline_mode);
        
        editor.disable_multiline();
        assert!(!editor.multiline_mode);
    }
}