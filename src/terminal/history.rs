#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

pub struct CommandHistory {
    commands: VecDeque<String>,
    max_size: usize,
    current_index: Option<usize>,
    search_state: Option<HistorySearch>,
}

struct HistorySearch {
    pattern: String,
    matches: Vec<usize>,
    current_match: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        CommandHistory {
            commands: VecDeque::with_capacity(max_size),
            max_size,
            current_index: None,
            search_state: None,
        }
    }

    pub fn add_command(&mut self, command: String) {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() {
            return;
        }

        if let Some(last) = self.commands.back() {
            if last == &command {
                return;
            }
        }

        // Add the command
        if self.commands.len() >= self.max_size {
            self.commands.pop_front();
        }
        self.commands.push_back(command);

        // Reset navigation state
        self.current_index = None;
        self.search_state = None;
    }

    pub fn get_previous(&mut self) -> Option<&String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                // Start from the end
                self.current_index = Some(self.commands.len() - 1);
                self.commands.get(self.commands.len() - 1)
            }
            Some(index) => {
                if index > 0 {
                    self.current_index = Some(index - 1);
                    self.commands.get(index - 1)
                } else {
                    // Already at the beginning
                    self.commands.get(0)
                }
            }
        }
    }

    pub fn get_next(&mut self) -> Option<&String> {
        match self.current_index {
            None => None, // Not navigating
            Some(index) => {
                if index < self.commands.len() - 1 {
                    self.current_index = Some(index + 1);
                    self.commands.get(index + 1)
                } else {
                    // At the end, return to current (empty) command
                    self.current_index = None;
                    None
                }
            }
        }
    }

    pub fn reset_navigation(&mut self) {
        self.current_index = None;
        self.search_state = None;
    }

    pub fn start_search(&mut self, pattern: &str) {
        let pattern = pattern.to_lowercase();
        let mut matches = Vec::new();

        // Find all commands that contain the pattern
        for (i, command) in self.commands.iter().enumerate() {
            if command.to_lowercase().contains(&pattern) {
                matches.push(i);
            }
        }

        if !matches.is_empty() {
            // Start from the most recent match
            matches.reverse();
            self.search_state = Some(HistorySearch {
                pattern: pattern.clone(),
                matches,
                current_match: 0,
            });
        }
    }

    pub fn search_previous(&mut self) -> Option<&String> {
        if let Some(search) = &mut self.search_state {
            if search.current_match < search.matches.len() {
                let command_index = search.matches[search.current_match];
                search.current_match += 1;
                return self.commands.get(command_index);
            }
        }
        None
    }

    pub fn search_next(&mut self) -> Option<&String> {
        if let Some(search) = &mut self.search_state {
            if search.current_match > 0 {
                search.current_match -= 1;
                let command_index = search.matches[search.current_match];
                return self.commands.get(command_index);
            }
        }
        None
    }

    pub fn get_all_commands(&self) -> Vec<&String> {
        self.commands.iter().collect()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = None;
        self.search_state = None;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    // Save history to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = self.commands.iter()
            .map(|cmd| cmd.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)?;
        Ok(())
    }

    // Load history from file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        if !path.as_ref().exists() {
            return Ok(()); // File doesn't exist, that's fine
        }

        let content = fs::read_to_string(path)?;
        self.commands.clear();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if self.commands.len() >= self.max_size {
                    self.commands.pop_front();
                }
                self.commands.push_back(trimmed.to_string());
            }
        }

        self.current_index = None;
        self.search_state = None;
        Ok(())
    }

    // Get suggestions based on current input
    pub fn get_suggestions(&self, prefix: &str) -> Vec<&String> {
        if prefix.is_empty() {
            return Vec::new();
        }

        let prefix_lower = prefix.to_lowercase();
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Get recent matching commands (reverse order for most recent first)
        for command in self.commands.iter().rev() {
            if command.to_lowercase().starts_with(&prefix_lower) && seen.insert(command) {
                suggestions.push(command);
                if suggestions.len() >= 10 { // Limit suggestions
                    break;
                }
            }
        }

        suggestions
    }

    // Find commands that contain a substring (for fuzzy search)
    pub fn find_matching(&self, pattern: &str) -> Vec<&String> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let pattern_lower = pattern.to_lowercase();
        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for command in self.commands.iter().rev() {
            if command.to_lowercase().contains(&pattern_lower) && seen.insert(command) {
                matches.push(command);
                if matches.len() >= 20 { // Limit matches
                    break;
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_history_creation() {
        let history = CommandHistory::new(100);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new(3);
        
        history.add_command("command1".to_string());
        history.add_command("command2".to_string());
        history.add_command("command3".to_string());
        
        assert_eq!(history.len(), 3);
        
        // Adding a 4th command should remove the first
        history.add_command("command4".to_string());
        assert_eq!(history.len(), 3);
        
        let commands = history.get_all_commands();
        assert_eq!(commands, vec!["command2", "command3", "command4"]);
    }

    #[test]
    fn test_duplicate_prevention() {
        let mut history = CommandHistory::new(10);
        
        history.add_command("command1".to_string());
        history.add_command("command1".to_string()); // Duplicate
        history.add_command("command2".to_string());
        
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_navigation() {
        let mut history = CommandHistory::new(10);
        
        history.add_command("cmd1".to_string());
        history.add_command("cmd2".to_string());
        history.add_command("cmd3".to_string());
        
        // Navigate backwards
        assert_eq!(history.get_previous(), Some(&"cmd3".to_string()));
        assert_eq!(history.get_previous(), Some(&"cmd2".to_string()));
        assert_eq!(history.get_previous(), Some(&"cmd1".to_string()));
        assert_eq!(history.get_previous(), Some(&"cmd1".to_string())); // At beginning
        
        // Navigate forwards
        assert_eq!(history.get_next(), Some(&"cmd2".to_string()));
        assert_eq!(history.get_next(), Some(&"cmd3".to_string()));
        assert_eq!(history.get_next(), None); // Back to current
    }

    #[test]
    fn test_search() {
        let mut history = CommandHistory::new(10);
        
        history.add_command("git status".to_string());
        history.add_command("git add .".to_string());
        history.add_command("ls -la".to_string());
        history.add_command("git commit".to_string());
        
        history.start_search("git");
        assert_eq!(history.search_previous(), Some(&"git commit".to_string()));
        assert_eq!(history.search_previous(), Some(&"git add .".to_string()));
        assert_eq!(history.search_previous(), Some(&"git status".to_string()));
        assert_eq!(history.search_previous(), None); // No more matches
    }

    #[test]
    fn test_suggestions() {
        let mut history = CommandHistory::new(10);
        
        history.add_command("git status".to_string());
        history.add_command("git add .".to_string());
        history.add_command("grep pattern".to_string());
        history.add_command("git commit".to_string());
        
        let suggestions = history.get_suggestions("git");
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0], &"git commit".to_string()); // Most recent first
    }

    #[test]
    fn test_file_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("history.txt");
        
        let mut history1 = CommandHistory::new(10);
        history1.add_command("command1".to_string());
        history1.add_command("command2".to_string());
        history1.add_command("command3".to_string());
        
        // Save to file
        history1.save_to_file(&file_path).unwrap();
        assert!(file_path.exists());
        
        // Load into new history
        let mut history2 = CommandHistory::new(10);
        history2.load_from_file(&file_path).unwrap();
        
        assert_eq!(history2.len(), 3);
        let commands = history2.get_all_commands();
        assert_eq!(commands, vec!["command1", "command2", "command3"]);
    }
}