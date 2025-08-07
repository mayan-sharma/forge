use crate::config::client;
use crate::forge_process::executor::{ProcessExecutor, ExecutionOptions};
use crate::terminal::output::{success_text, error_text, info_text};

pub fn run(command: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if command.is_empty() {
        println!("No command provided.");
        return Ok(());
    }

    let full_command = command.join(" ");
    println!("{}", info_text("🔧 Forge Enhanced Executor"));
    println!("Command: {}", full_command);
    println!();

    // Use the new enhanced executor
    let mut executor = ProcessExecutor::new();
    
    let options = ExecutionOptions {
        timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes default
        show_progress: true,
        capture_output: true,
        interactive: false,
        safety_check: true,
        working_directory: None,
    };

    // Optional AI analysis if available
    if let Ok((client, model)) = client::create_ollama_client() {
        println!("{}", info_text("🤖 Getting AI analysis of command..."));
        
        let analysis_prompt = format!(
            "Analyze this command for safety and provide a brief explanation:\n\n\
            Command: {}\n\n\
            Respond with:\n\
            1. What this command does\n\
            2. Any safety concerns\n\
            3. Expected outcome\n\
            Keep it brief and clear.",
            full_command
        );

        if let Ok(analysis) = client.generate(&model, &analysis_prompt, false) {
            println!("{}", info_text("AI Analysis:"));
            println!("{}", analysis);
            println!();
        }
    }

    // Execute with enhanced executor
    match executor.execute(&full_command, options) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                println!("STDOUT:");
                println!("{}", result.stdout);
            }
            
            if !result.stderr.is_empty() {
                println!("STDERR:");
                println!("{}", error_text(&result.stderr));
            }
            
            println!("---");
            println!("Exit code: {}", result.exit_code);
            
            if result.success {
                println!("{}", success_text("✅ Command completed successfully"));
            } else {
                println!("{}", error_text("❌ Command failed"));
            }
        }
        Err(e) => {
            println!("{}", error_text(&format!("Failed to execute command: {}", e)));
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn is_dangerous_command(command: &str) -> bool {
    let dangerous_commands = [
        "rm -rf",
        "sudo rm",
        "del /s",
        "format",
        "fdisk",
        "mkfs",
        "dd if=",
        "shutdown",
        "reboot",
        "halt",
        "init 0",
        "init 6",
        "> /dev/",
        "chmod 000",
        "chown root",
    ];

    let command_lower = command.to_lowercase();
    dangerous_commands.iter().any(|&dangerous| command_lower.contains(dangerous))
}

