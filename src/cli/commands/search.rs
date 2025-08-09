use crate::fs::{walker::walk_directory, search::search_multiple_files, glob::glob};
use crate::terminal::output::{StyledText, Color, success_text, warning_text, dim_text, MultiStageProgress, GLOBAL_TASK_MONITOR, NotificationType};

pub fn run(query: &str, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let search_path = path.unwrap_or(".");
    
    // Start background task tracking
    let task_id = GLOBAL_TASK_MONITOR.start_task("search", &format!("Search for '{}'", query));
    
    // Create multi-stage progress
    let mut progress = MultiStageProgress::new(&format!("🔍 Forge Search: \"{}\"", query));
    progress.add_stage("Initialize", 0.1);
    progress.add_stage("Find Files", 0.3);
    progress.add_stage("Search Content", 0.5);
    progress.add_stage("Format Results", 0.1);
    
    println!("{}", progress.render(60));
    println!();
    
    // Stage 1: Initialize
    progress.start_stage(0)?;
    progress.update_stage_progress(0, 0.5, Some("Setting up search parameters"))?;
    
    println!("  {} {}", 
        StyledText::new("Query:").fg(Color::BrightYellow).bold(),
        StyledText::new(&format!("\"{}\"", query)).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Path:").fg(Color::BrightYellow).bold(),
        StyledText::new(search_path).fg(Color::BrightCyan));
    
    progress.update_stage_progress(0, 1.0, Some("Ready to search"))?;
    progress.complete_stage(0)?;
    
    // Stage 2: Find Files
    progress.start_stage(1)?;
    progress.update_stage_progress(1, 0.2, Some("Scanning directories"))?;
    
    // Update task progress
    GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
        task.progress = progress.clone();
    })?;
    
    // Check if search_path is a glob pattern
    let files = if search_path.contains('*') || search_path.contains('?') || search_path.contains('[') {
        progress.update_stage_progress(1, 0.5, Some("Expanding glob pattern"))?;
        let glob_files = glob(search_path)?;
        // Convert PathBuf to String for compatibility with existing code
        glob_files.iter().map(|p| p.to_string_lossy().to_string()).collect()
    } else {
        walk_directory(search_path)?
    };
    
    progress.update_stage_progress(1, 1.0, Some(&format!("Found {} files", files.len())))?;
    progress.complete_stage(1)?;
    
    if files.is_empty() {
        progress.fail_stage(1, "No files found")?;
        GLOBAL_TASK_MONITOR.fail_task(&task_id, &format!("No files found in: {}", search_path));
        println!("{}", warning_text(&format!("⚠️  No files found in: {}", search_path)));
        return Ok(());
    }

    // Stage 3: Search Content
    progress.start_stage(2)?;
    progress.update_stage_progress(2, 0.1, Some("Starting content search"))?;
    
    GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
        task.progress = progress.clone();
    })?;
    
    let results = search_multiple_files(&files, query)?;
    
    progress.update_stage_progress(2, 1.0, Some("Search complete"))?;
    progress.complete_stage(2)?;
    
    // Stage 4: Format Results  
    progress.start_stage(3)?;
    progress.update_stage_progress(3, 0.5, Some("Processing results"))?;
    
    if results.is_empty() {
        progress.fail_stage(3, "No matches found")?;
        GLOBAL_TASK_MONITOR.fail_task(&task_id, &format!("No matches found for: '{}'", query));
        println!("{}", warning_text(&format!("⚠️  No matches found for: \"{}\"", query)));
        println!("{}", dim_text("   Try a different query or search path"));
        return Ok(());
    }

    let total_matches = results.iter().map(|(_, matches)| matches.len()).sum::<usize>();
    progress.update_stage_progress(3, 1.0, Some(&format!("Found {} matches in {} files", total_matches, results.len())))?;
    progress.complete_stage(3)?;
    
    // Complete the background task
    GLOBAL_TASK_MONITOR.complete_task(&task_id, &format!("Found {} matches in {} files", total_matches, results.len()));
    
    // Add a success notification
    GLOBAL_TASK_MONITOR.add_notification(
        &format!("search-complete-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        &format!("🔍 Search completed: {} matches found for '{}'", total_matches, query),
        NotificationType::Success
    );
    
    println!("{}", progress.render(60));
    println!();
    
    println!("{} {} {} {} {}", 
        success_text("✅ Found"),
        StyledText::new(&total_matches.to_string()).fg(Color::BrightGreen).bold(),
        StyledText::new("matches in").fg(Color::White),
        StyledText::new(&results.len().to_string()).fg(Color::BrightCyan).bold(),
        StyledText::new("files:").fg(Color::White));
    println!();

    for (file_index, (file_path, matches)) in results.iter().enumerate() {
        println!("{} {} {} {}", 
            StyledText::new(&format!("📄 [{}]", file_index + 1)).fg(Color::BrightBlue).bold(),
            StyledText::new(file_path).fg(Color::BrightCyan),
            StyledText::new(&format!("({} matches)", matches.len())).fg(Color::BrightBlack),
            if matches.len() > 5 { dim_text("[showing first 5]") } else { StyledText::new("") });
        
        let display_matches = if matches.len() > 5 { &matches[..5] } else { matches };
        
        for (line_num, col, _len) in display_matches {
            // Read the line to show context
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let lines: Vec<&str> = content.lines().collect();
                if *line_num > 0 && *line_num <= lines.len() {
                    let line = lines[*line_num - 1];
                    let line_text = line.trim();
                    
                    // Show line number and content
                    println!("    {}{} {}", 
                        StyledText::new(&format!("{:>4}:", line_num)).fg(Color::BrightBlack),
                        StyledText::new(&format!("{:>3}", col)).fg(Color::BrightBlack),
                        highlight_match(line_text, query));
                }
            }
        }
        if matches.len() > 5 {
            println!("{}", dim_text(&format!("    ... and {} more matches", matches.len() - 5)));
        }
        println!();
    }
    
    // Show ETA information if available
    if let Some(_eta) = progress.get_eta_seconds() {
        println!("⏰ Process completed in {:.1}s", progress.start_time.elapsed().as_secs_f64());
    }

    Ok(())
}

fn highlight_match(line: &str, query: &str) -> String {
    if let Some(pos) = line.to_lowercase().find(&query.to_lowercase()) {
        let before = &line[..pos];
        let matched = &line[pos..pos + query.len()];
        let after = &line[pos + query.len()..];
        
        format!("{}{}{}", 
            before,
            format!("{}", StyledText::new(matched).fg(Color::BrightYellow).bold()),
            after)
    } else {
        line.to_string()
    }
}

/// Enhanced search function that supports both glob patterns and regular directory search
pub fn run_with_glob(query: &str, pattern: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let search_pattern = pattern.unwrap_or("**/*");
    
    // Start background task tracking
    let task_id = GLOBAL_TASK_MONITOR.start_task("search", &format!("Search for '{}' in pattern '{}'", query, search_pattern));
    
    // Create multi-stage progress
    let mut progress = MultiStageProgress::new(&format!("🔍 Forge Glob Search: \"{}\"", query));
    progress.add_stage("Initialize", 0.1);
    progress.add_stage("Expand Pattern", 0.3);
    progress.add_stage("Search Content", 0.5);
    progress.add_stage("Format Results", 0.1);
    
    println!("{}", progress.render(60));
    println!();
    
    // Stage 1: Initialize
    progress.start_stage(0)?;
    progress.update_stage_progress(0, 0.5, Some("Setting up search parameters"))?;
    
    println!("  {} {}", 
        StyledText::new("Query:").fg(Color::BrightYellow).bold(),
        StyledText::new(&format!("\"{}\"", query)).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Pattern:").fg(Color::BrightYellow).bold(),
        StyledText::new(search_pattern).fg(Color::BrightMagenta));
    
    progress.update_stage_progress(0, 1.0, Some("Ready to search"))?;
    progress.complete_stage(0)?;
    
    // Stage 2: Expand Pattern
    progress.start_stage(1)?;
    progress.update_stage_progress(1, 0.2, Some("Expanding glob pattern"))?;
    
    // Update task progress
    GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
        task.progress = progress.clone();
    })?;
    
    let file_paths = glob(search_pattern)?;
    
    // Filter to only files (not directories) and convert to String
    let files: Vec<String> = file_paths.iter()
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    
    progress.update_stage_progress(1, 1.0, Some(&format!("Found {} files matching pattern", files.len())))?;
    progress.complete_stage(1)?;
    
    if files.is_empty() {
        progress.fail_stage(1, "No files found matching pattern")?;
        GLOBAL_TASK_MONITOR.fail_task(&task_id, &format!("No files found matching pattern: {}", search_pattern));
        println!("{}", warning_text(&format!("⚠️  No files found matching pattern: {}", search_pattern)));
        return Ok(());
    }

    // Stage 3: Search Content
    progress.start_stage(2)?;
    progress.update_stage_progress(2, 0.1, Some("Starting content search"))?;
    
    GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
        task.progress = progress.clone();
    })?;
    
    let results = search_multiple_files(&files, query)?;
    
    progress.update_stage_progress(2, 1.0, Some("Search complete"))?;
    progress.complete_stage(2)?;
    
    // Stage 4: Format Results  
    progress.start_stage(3)?;
    progress.update_stage_progress(3, 0.5, Some("Processing results"))?;
    
    if results.is_empty() {
        progress.fail_stage(3, "No matches found")?;
        GLOBAL_TASK_MONITOR.fail_task(&task_id, &format!("No matches found for: '{}'", query));
        println!("{}", warning_text(&format!("⚠️  No matches found for: \"{}\"", query)));
        println!("{}", dim_text("   Try a different query or search pattern"));
        return Ok(());
    }

    let total_matches = results.iter().map(|(_, matches)| matches.len()).sum::<usize>();
    progress.update_stage_progress(3, 1.0, Some(&format!("Found {} matches in {} files", total_matches, results.len())))?;
    progress.complete_stage(3)?;
    
    // Complete the background task
    GLOBAL_TASK_MONITOR.complete_task(&task_id, &format!("Found {} matches in {} files", total_matches, results.len()));
    
    // Add a success notification
    GLOBAL_TASK_MONITOR.add_notification(
        &format!("search-complete-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        &format!("🔍 Glob search completed: {} matches found for '{}'", total_matches, query),
        NotificationType::Success
    );
    
    println!("{}", progress.render(60));
    println!();
    
    println!("{} {} {} {} {}", 
        success_text("✅ Found"),
        StyledText::new(&total_matches.to_string()).fg(Color::BrightGreen).bold(),
        StyledText::new("matches in").fg(Color::White),
        StyledText::new(&results.len().to_string()).fg(Color::BrightCyan).bold(),
        StyledText::new("files:").fg(Color::White));
    println!();

    for (file_index, (file_path, matches)) in results.iter().enumerate() {
        println!("{} {} {} {}", 
            StyledText::new(&format!("📄 [{}]", file_index + 1)).fg(Color::BrightBlue).bold(),
            StyledText::new(file_path).fg(Color::BrightCyan),
            StyledText::new(&format!("({} matches)", matches.len())).fg(Color::BrightBlack),
            if matches.len() > 5 { dim_text("[showing first 5]") } else { StyledText::new("") });
        
        let display_matches = if matches.len() > 5 { &matches[..5] } else { matches };
        
        for (line_num, col, _len) in display_matches {
            // Read the line to show context
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let lines: Vec<&str> = content.lines().collect();
                if *line_num > 0 && *line_num <= lines.len() {
                    let line = lines[*line_num - 1];
                    let line_text = line.trim();
                    
                    // Show line number and content
                    println!("    {}{} {}", 
                        StyledText::new(&format!("{:>4}:", line_num)).fg(Color::BrightBlack),
                        StyledText::new(&format!("{:>3}", col)).fg(Color::BrightBlack),
                        highlight_match(line_text, query));
                }
            }
        }
        if matches.len() > 5 {
            println!("{}", dim_text(&format!("    ... and {} more matches", matches.len() - 5)));
        }
        println!();
    }
    
    // Show ETA information if available
    if let Some(_eta) = progress.get_eta_seconds() {
        println!("⏰ Process completed in {:.1}s", progress.start_time.elapsed().as_secs_f64());
    }

    Ok(())
}