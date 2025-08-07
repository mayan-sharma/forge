use std::fs;
use std::path::Path;

pub struct DirectoryWalker {
    stack: Vec<String>,
    follow_symlinks: bool,
    max_depth: Option<usize>,
    current_depth: usize,
}

impl DirectoryWalker {
    pub fn new(root: &str) -> Self {
        Self {
            stack: vec![root.to_string()],
            follow_symlinks: false,
            max_depth: None,
            current_depth: 0,
        }
    }

    #[allow(dead_code)]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    #[allow(dead_code)]
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    #[allow(dead_code)]
    pub fn walk(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut all_files = Vec::new();

        while let Some(current_path) = self.stack.pop() {
            if let Some(max_depth) = self.max_depth {
                if self.current_depth >= max_depth {
                    continue;
                }
            }

            let path = Path::new(&current_path);
            
            if path.is_file() {
                all_files.push(current_path);
                continue;
            }

            if !path.is_dir() {
                continue;
            }

            match fs::read_dir(&current_path) {
                Ok(entries) => {
                    let mut dirs = Vec::new();
                    
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            let path_str = entry_path.to_string_lossy().to_string();

                            if entry_path.is_file() {
                                all_files.push(path_str);
                            } else if entry_path.is_dir() {
                                dirs.push(path_str);
                            } else if entry_path.is_symlink() && self.follow_symlinks {
                                if let Ok(target) = fs::read_link(&entry_path) {
                                    let target_str = target.to_string_lossy().to_string();
                                    if Path::new(&target_str).is_file() {
                                        all_files.push(path_str);
                                    } else if Path::new(&target_str).is_dir() {
                                        dirs.push(path_str);
                                    }
                                }
                            }
                        }
                    }

                    // Add directories to stack in reverse order for depth-first traversal
                    for dir in dirs.into_iter().rev() {
                        self.stack.push(dir);
                    }
                }
                Err(_) => {
                    // Skip directories we can't read
                    continue;
                }
            }
        }

        Ok(all_files)
    }
}

#[allow(dead_code)]
pub fn walk_directory(root: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut walker = DirectoryWalker::new(root);
    walker.walk()
}

#[allow(dead_code)]
pub fn walk_directory_with_depth(root: &str, max_depth: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut walker = DirectoryWalker::new(root).with_max_depth(max_depth);
    walker.walk()
}

#[allow(dead_code)]
pub fn find_files_by_extension(root: &str, extension: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let all_files = walk_directory(root)?;
    let extension = extension.to_lowercase();
    
    let filtered_files: Vec<String> = all_files
        .into_iter()
        .filter(|file| {
            if let Some(file_ext) = Path::new(file).extension() {
                file_ext.to_string_lossy().to_lowercase() == extension
            } else {
                false
            }
        })
        .collect();

    Ok(filtered_files)
}

#[allow(dead_code)]
pub fn find_files_by_name_pattern(root: &str, pattern: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let all_files = walk_directory(root)?;
    let pattern = pattern.to_lowercase();
    
    let filtered_files: Vec<String> = all_files
        .into_iter()
        .filter(|file| {
            if let Some(filename) = Path::new(file).file_name() {
                filename.to_string_lossy().to_lowercase().contains(&pattern)
            } else {
                false
            }
        })
        .collect();

    Ok(filtered_files)
}