use crate::fs::operations::{read_file, write_file, file_exists};
use crate::config::client;

pub fn run(file_path: &str, instruction: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Forge File Editor");
    println!("File: {}", file_path);
    println!();

    // Check if file exists
    if !file_exists(file_path) {
        println!("File does not exist: {}", file_path);
        print!("Create it? (y/N): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return Ok(());
        }
        
        write_file(file_path, "")?;
        println!("Created empty file: {}", file_path);
    }

    // Read current file content
    let content = read_file(file_path)?;
    println!("Current content ({} characters):", content.len());
    println!("---");
    println!("{}", content);
    println!("---");
    println!();

    let (client, model) = client::create_ollama_client()?;

    let instruction = match instruction {
        Some(instr) => instr.to_string(),
        None => {
            println!("What would you like me to do with this file?");
            print!("Instruction: ");
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if instruction.is_empty() {
        println!("No instruction provided.");
        return Ok(());
    }

    println!("Processing instruction: {}", instruction);
    println!("Generating changes...");

    let prompt = format!(
        "You are a code editor assistant. The user wants to modify a file.\n\n\
        Current file content:\n\
        ```\n{}\n```\n\n\
        User instruction: {}\n\n\
        Please provide the complete updated file content. \
        Only respond with the new file content, no explanations or markdown formatting.",
        content, instruction
    );

    match client.generate(&model, &prompt, false) {
        Ok(response) => {
            // Clean up response (remove potential markdown formatting)
            let new_content = response
                .trim()
                .strip_prefix("```")
                .unwrap_or(&response)
                .strip_suffix("```")
                .unwrap_or(&response)
                .trim();

            println!("Proposed changes:");
            println!("---");
            println!("{}", new_content);
            println!("---");
            println!();

            print!("Apply these changes? (y/N): ");
            std::io::stdout().flush()?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                // Create backup
                let backup_path = format!("{}.backup", file_path);
                write_file(&backup_path, &content)?;
                println!("Created backup: {}", backup_path);

                // Apply changes
                write_file(file_path, new_content)?;
                println!("Changes applied to: {}", file_path);
            } else {
                println!("Changes discarded.");
            }
        }
        Err(e) => {
            println!("Error generating changes: {}", e);
        }
    }

    Ok(())
}

use std::io::Write;