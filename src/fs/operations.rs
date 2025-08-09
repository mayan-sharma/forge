use std::fs;
use std::path::Path;

pub fn read_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_file(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn delete_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn create_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn list_directory(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                entries.push(name_str.to_string());
            }
        }
    }
    
    entries.sort();
    Ok(entries)
}

#[allow(dead_code)]
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[allow(dead_code)]
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

#[allow(dead_code)]
pub fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}

#[allow(dead_code)]
pub fn get_file_size(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

#[allow(dead_code)]
pub fn copy_file(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::copy(src, dest)?;
    Ok(())
}

#[allow(dead_code)]
pub fn move_file(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::rename(src, dest)?;
    Ok(())
}

#[allow(dead_code)]
pub fn append_to_file(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_file_lines(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().map(|line| line.to_string()).collect())
}

#[allow(dead_code)]
pub fn get_current_dir() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    Ok(current_dir.to_string_lossy().to_string())
}

#[allow(dead_code)]
pub fn change_dir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_current_dir(path)?;
    Ok(())
}

#[cfg(unix)]
#[allow(dead_code)]
pub fn set_permissions(path: &str, mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    
    let permissions = Permissions::from_mode(mode);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn set_permissions(_path: &str, _mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    // Windows doesn't use Unix-style permissions
    Ok(())
}

#[allow(dead_code)]
pub fn is_readable(path: &str) -> bool {
    if let Ok(file) = std::fs::File::open(path) {
        drop(file);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_writable(path: &str) -> bool {
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
        drop(file);
        true
    } else {
        // Try to create if it doesn't exist
        if let Ok(file) = std::fs::OpenOptions::new().write(true).create(true).open(path) {
            drop(file);
            let _ = std::fs::remove_file(path); // Clean up test file
            true
        } else {
            false
        }
    }
}

// === Bulk Operations ===

use std::path::PathBuf;

/// Result of a bulk operation on a single file
#[derive(Debug, Clone)]
pub struct BulkOpResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

impl BulkOpResult {
    pub fn success(path: PathBuf) -> Self {
        Self {
            path,
            success: true,
            error: None,
        }
    }

    pub fn error(path: PathBuf, error: String) -> Self {
        Self {
            path,
            success: false,
            error: Some(error),
        }
    }
}

/// Summary of bulk operation results
#[derive(Debug)]
pub struct BulkOpSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<BulkOpResult>,
}

impl BulkOpSummary {
    pub fn new(results: Vec<BulkOpResult>) -> Self {
        let total = results.len();
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = total - succeeded;
        
        Self {
            total,
            succeeded,
            failed,
            results,
        }
    }

    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

/// Copy multiple files to a destination directory
#[allow(dead_code)]
pub fn bulk_copy<P: AsRef<Path>>(files: &[P], dest_dir: &str) -> BulkOpSummary {
    let mut results = Vec::new();
    
    // Ensure destination directory exists
    if let Err(e) = create_directory(dest_dir) {
        // If we can't create the dest directory, all operations fail
        for file in files {
            results.push(BulkOpResult::error(
                file.as_ref().to_path_buf(),
                format!("Failed to create destination directory: {}", e)
            ));
        }
        return BulkOpSummary::new(results);
    }

    for file in files {
        let source_path = file.as_ref();
        let file_name = match source_path.file_name() {
            Some(name) => name,
            None => {
                results.push(BulkOpResult::error(
                    source_path.to_path_buf(),
                    "Invalid file name".to_string()
                ));
                continue;
            }
        };

        let dest_path = Path::new(dest_dir).join(file_name);
        
        match fs::copy(source_path, &dest_path) {
            Ok(_) => results.push(BulkOpResult::success(source_path.to_path_buf())),
            Err(e) => results.push(BulkOpResult::error(
                source_path.to_path_buf(),
                format!("Copy failed: {}", e)
            )),
        }
    }

    BulkOpSummary::new(results)
}

/// Move multiple files to a destination directory
#[allow(dead_code)]
pub fn bulk_move<P: AsRef<Path>>(files: &[P], dest_dir: &str) -> BulkOpSummary {
    let mut results = Vec::new();
    
    // Ensure destination directory exists
    if let Err(e) = create_directory(dest_dir) {
        for file in files {
            results.push(BulkOpResult::error(
                file.as_ref().to_path_buf(),
                format!("Failed to create destination directory: {}", e)
            ));
        }
        return BulkOpSummary::new(results);
    }

    for file in files {
        let source_path = file.as_ref();
        let file_name = match source_path.file_name() {
            Some(name) => name,
            None => {
                results.push(BulkOpResult::error(
                    source_path.to_path_buf(),
                    "Invalid file name".to_string()
                ));
                continue;
            }
        };

        let dest_path = Path::new(dest_dir).join(file_name);
        
        match fs::rename(source_path, &dest_path) {
            Ok(_) => results.push(BulkOpResult::success(source_path.to_path_buf())),
            Err(e) => results.push(BulkOpResult::error(
                source_path.to_path_buf(),
                format!("Move failed: {}", e)
            )),
        }
    }

    BulkOpSummary::new(results)
}

/// Delete multiple files
#[allow(dead_code)]
pub fn bulk_delete<P: AsRef<Path>>(files: &[P]) -> BulkOpSummary {
    let mut results = Vec::new();

    for file in files {
        let path = file.as_ref();
        
        let result = if path.is_file() {
            fs::remove_file(path)
        } else if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            Ok(()) // File doesn't exist, consider it a success
        };

        match result {
            Ok(_) => results.push(BulkOpResult::success(path.to_path_buf())),
            Err(e) => results.push(BulkOpResult::error(
                path.to_path_buf(),
                format!("Delete failed: {}", e)
            )),
        }
    }

    BulkOpSummary::new(results)
}

/// Apply a text transformation to multiple files
#[allow(dead_code)]
pub fn bulk_transform<P: AsRef<Path>, F>(files: &[P], mut transform_fn: F) -> BulkOpSummary
where
    F: FnMut(&str) -> Result<String, Box<dyn std::error::Error>>,
{
    let mut results = Vec::new();

    for file in files {
        let path = file.as_ref();
        
        match read_file(&path.to_string_lossy()) {
            Ok(content) => {
                match transform_fn(&content) {
                    Ok(transformed) => {
                        match write_file(&path.to_string_lossy(), &transformed) {
                            Ok(_) => results.push(BulkOpResult::success(path.to_path_buf())),
                            Err(e) => results.push(BulkOpResult::error(
                                path.to_path_buf(),
                                format!("Write failed: {}", e)
                            )),
                        }
                    }
                    Err(e) => results.push(BulkOpResult::error(
                        path.to_path_buf(),
                        format!("Transform failed: {}", e)
                    )),
                }
            }
            Err(e) => results.push(BulkOpResult::error(
                path.to_path_buf(),
                format!("Read failed: {}", e)
            )),
        }
    }

    BulkOpSummary::new(results)
}

/// Search and replace text in multiple files
#[allow(dead_code)]
pub fn bulk_search_replace<P: AsRef<Path>>(
    files: &[P], 
    search: &str, 
    replace: &str
) -> BulkOpSummary {
    bulk_transform(files, |content| {
        Ok(content.replace(search, replace))
    })
}

/// Add a prefix to multiple file names
#[allow(dead_code)]
pub fn bulk_rename_prefix<P: AsRef<Path>>(files: &[P], prefix: &str) -> BulkOpSummary {
    let mut results = Vec::new();

    for file in files {
        let path = file.as_ref();
        
        if let Some(parent) = path.parent() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    let new_name = format!("{}{}", prefix, name_str);
                    let new_path = parent.join(new_name);
                    
                    match fs::rename(path, &new_path) {
                        Ok(_) => results.push(BulkOpResult::success(path.to_path_buf())),
                        Err(e) => results.push(BulkOpResult::error(
                            path.to_path_buf(),
                            format!("Rename failed: {}", e)
                        )),
                    }
                } else {
                    results.push(BulkOpResult::error(
                        path.to_path_buf(),
                        "Invalid file name encoding".to_string()
                    ));
                }
            } else {
                results.push(BulkOpResult::error(
                    path.to_path_buf(),
                    "No file name found".to_string()
                ));
            }
        } else {
            results.push(BulkOpResult::error(
                path.to_path_buf(),
                "No parent directory found".to_string()
            ));
        }
    }

    BulkOpSummary::new(results)
}

/// Add a suffix to multiple file names (before the extension)
#[allow(dead_code)]
pub fn bulk_rename_suffix<P: AsRef<Path>>(files: &[P], suffix: &str) -> BulkOpSummary {
    let mut results = Vec::new();

    for file in files {
        let path = file.as_ref();
        
        if let Some(parent) = path.parent() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    let new_name = if let Some(dot_pos) = name_str.rfind('.') {
                        // Has extension - insert suffix before it
                        let (base, ext) = name_str.split_at(dot_pos);
                        format!("{}{}{}", base, suffix, ext)
                    } else {
                        // No extension - just append suffix
                        format!("{}{}", name_str, suffix)
                    };
                    
                    let new_path = parent.join(new_name);
                    
                    match fs::rename(path, &new_path) {
                        Ok(_) => results.push(BulkOpResult::success(path.to_path_buf())),
                        Err(e) => results.push(BulkOpResult::error(
                            path.to_path_buf(),
                            format!("Rename failed: {}", e)
                        )),
                    }
                } else {
                    results.push(BulkOpResult::error(
                        path.to_path_buf(),
                        "Invalid file name encoding".to_string()
                    ));
                }
            } else {
                results.push(BulkOpResult::error(
                    path.to_path_buf(),
                    "No file name found".to_string()
                ));
            }
        } else {
            results.push(BulkOpResult::error(
                path.to_path_buf(),
                "No parent directory found".to_string()
            ));
        }
    }

    BulkOpSummary::new(results)
}

/// Count lines of code in multiple files
#[allow(dead_code)]
pub fn bulk_count_lines<P: AsRef<Path>>(files: &[P]) -> Result<Vec<(PathBuf, usize)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    for file in files {
        let path = file.as_ref();
        match read_file(&path.to_string_lossy()) {
            Ok(content) => {
                let line_count = content.lines().count();
                results.push((path.to_path_buf(), line_count));
            }
            Err(_) => {
                // Skip files that can't be read
                results.push((path.to_path_buf(), 0));
            }
        }
    }

    Ok(results)
}