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