use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, Metadata};
use std::time::{Duration, SystemTime};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

/// Events that can occur to watched files
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Error(String),
}

/// A simple file watcher that polls the filesystem
pub struct FileWatcher {
    watched_paths: Vec<PathBuf>,
    file_states: HashMap<PathBuf, FileState>,
    poll_interval: Duration,
}

#[derive(Debug, Clone)]
struct FileState {
    exists: bool,
    modified: Option<SystemTime>,
    size: u64,
}

impl FileState {
    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            exists: true,
            modified: metadata.modified().ok(),
            size: metadata.len(),
        }
    }

    fn missing() -> Self {
        Self {
            exists: false,
            modified: None,
            size: 0,
        }
    }
}

impl FileWatcher {
    /// Create a new file watcher with default 1-second polling interval
    pub fn new() -> Self {
        Self {
            watched_paths: Vec::new(),
            file_states: HashMap::new(),
            poll_interval: Duration::from_secs(1),
        }
    }

    /// Set the polling interval (minimum 100ms)
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval.max(Duration::from_millis(100));
        self
    }

    /// Add a file or directory to watch
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();
        
        // Initialize the file state
        if let Ok(metadata) = fs::metadata(&path) {
            self.file_states.insert(path.clone(), FileState::from_metadata(&metadata));
        } else {
            self.file_states.insert(path.clone(), FileState::missing());
        }
        
        self.watched_paths.push(path);
        Ok(())
    }

    /// Start watching files and return a receiver for events
    pub fn start(mut self) -> Result<Receiver<FileEvent>, Box<dyn std::error::Error>> {
        let (sender, receiver) = mpsc::channel();
        
        // Initialize file states for all watched paths
        let paths_to_init = self.watched_paths.clone();
        for path in &paths_to_init {
            if let Err(e) = self.update_file_state(path, &sender) {
                sender.send(FileEvent::Error(format!("Failed to initialize {}: {}", path.display(), e))).ok();
            }
        }

        // Spawn background thread for polling
        thread::spawn(move || {
            loop {
                thread::sleep(self.poll_interval);
                
                for path in &self.watched_paths.clone() {
                    if let Err(e) = self.check_file_changes(path, &sender) {
                        sender.send(FileEvent::Error(format!("Error watching {}: {}", path.display(), e))).ok();
                    }
                }
            }
        });

        Ok(receiver)
    }

    /// Update the stored state for a file and detect changes
    fn check_file_changes(&mut self, path: &Path, sender: &Sender<FileEvent>) -> Result<(), Box<dyn std::error::Error>> {
        let current_state = if let Ok(metadata) = fs::metadata(path) {
            FileState::from_metadata(&metadata)
        } else {
            FileState::missing()
        };

        let previous_state = self.file_states.get(path).cloned().unwrap_or_else(FileState::missing);

        // Detect changes
        match (previous_state.exists, current_state.exists) {
            (false, true) => {
                // File was created
                sender.send(FileEvent::Created(path.to_path_buf())).ok();
            }
            (true, false) => {
                // File was deleted
                sender.send(FileEvent::Deleted(path.to_path_buf())).ok();
            }
            (true, true) => {
                // File exists - check for modifications
                if previous_state.modified != current_state.modified || 
                   previous_state.size != current_state.size {
                    sender.send(FileEvent::Modified(path.to_path_buf())).ok();
                }
            }
            (false, false) => {
                // File still doesn't exist - no event
            }
        }

        // Update stored state
        self.file_states.insert(path.to_path_buf(), current_state);
        Ok(())
    }

    /// Initialize file state without sending events
    fn update_file_state(&mut self, path: &Path, _sender: &Sender<FileEvent>) -> Result<(), Box<dyn std::error::Error>> {
        let state = if let Ok(metadata) = fs::metadata(path) {
            FileState::from_metadata(&metadata)
        } else {
            FileState::missing()
        };
        
        self.file_states.insert(path.to_path_buf(), state);
        Ok(())
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to watch a single file
pub fn watch_file<P: AsRef<Path>>(path: P) -> Result<Receiver<FileEvent>, Box<dyn std::error::Error>> {
    let mut watcher = FileWatcher::new();
    watcher.watch(path)?;
    watcher.start()
}

/// Convenience function to watch multiple files
pub fn watch_files<P: AsRef<Path>>(paths: &[P]) -> Result<Receiver<FileEvent>, Box<dyn std::error::Error>> {
    let mut watcher = FileWatcher::new();
    for path in paths {
        watcher.watch(path)?;
    }
    watcher.start()
}

/// Watch files matching a glob pattern
pub fn watch_glob(pattern: &str) -> Result<Receiver<FileEvent>, Box<dyn std::error::Error>> {
    use crate::fs::glob::glob;
    
    let paths = glob(pattern)?;
    let mut watcher = FileWatcher::new();
    
    for path in &paths {
        watcher.watch(path)?;
    }
    
    watcher.start()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert_eq!(watcher.watched_paths.len(), 0);
        assert_eq!(watcher.poll_interval, Duration::from_secs(1));
    }

    #[test]
    fn test_watch_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        
        let mut watcher = FileWatcher::new();
        watcher.watch(&file_path)?;
        
        assert_eq!(watcher.watched_paths.len(), 1);
        assert_eq!(watcher.watched_paths[0], file_path);
        
        Ok(())
    }

    #[test]
    fn test_poll_interval_minimum() {
        let watcher = FileWatcher::new().with_poll_interval(Duration::from_millis(50));
        assert_eq!(watcher.poll_interval, Duration::from_millis(100));
        
        let watcher2 = FileWatcher::new().with_poll_interval(Duration::from_millis(500));
        assert_eq!(watcher2.poll_interval, Duration::from_millis(500));
    }

    #[test] 
    fn test_file_state_creation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        
        // Create file
        File::create(&file_path)?;
        let metadata = fs::metadata(&file_path)?;
        let state = FileState::from_metadata(&metadata);
        
        assert!(state.exists);
        assert!(state.modified.is_some());
        assert_eq!(state.size, 0);
        
        Ok(())
    }

    // Note: Integration tests for actual file watching would require 
    // more complex setup and timing, which is better done in integration tests
}