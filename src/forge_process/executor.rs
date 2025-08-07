#![allow(dead_code)]

use std::process::{Command, Stdio};
use std::io::{self, Write, BufRead, BufReader};
use std::time::{Duration, Instant};
use std::thread;

use crate::terminal::output::{ProgressBar, Spinner, success_text, error_text, warning_text, info_text};
use super::safety::{SafetyChecker, RiskLevel};
use super::shell::{Shell, CommandResult};

#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    pub timeout: Option<Duration>,
    pub show_progress: bool,
    pub capture_output: bool,
    pub interactive: bool,
    pub safety_check: bool,
    pub working_directory: Option<String>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        ExecutionOptions {
            timeout: None,
            show_progress: true,
            capture_output: true,
            interactive: false,
            safety_check: true,
            working_directory: None,
        }
    }
}

pub struct ProcessExecutor {
    safety_checker: SafetyChecker,
    shell: Shell,
}

impl ProcessExecutor {
    pub fn new() -> Self {
        ProcessExecutor {
            safety_checker: SafetyChecker::new(),
            shell: Shell::new(),
        }
    }

    pub fn with_allowed_commands(commands: Vec<String>) -> Self {
        ProcessExecutor {
            safety_checker: SafetyChecker::new().with_allowed_commands(commands),
            shell: Shell::new(),
        }
    }

    pub fn execute(&mut self, command: &str, options: ExecutionOptions) -> Result<CommandResult, Box<dyn std::error::Error>> {
        if options.safety_check {
            self.perform_safety_check(command)?;
        }

        if options.show_progress {
            self.execute_with_progress(command, options)
        } else {
            self.execute_direct(command, options)
        }
    }

    pub fn execute_batch(&mut self, commands: Vec<&str>, options: ExecutionOptions) -> Result<Vec<CommandResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let total_commands = commands.len();

        if options.show_progress {
            let mut progress = ProgressBar::new(total_commands, 40)
                .with_title("Batch Execution");
            
            for (i, command) in commands.iter().enumerate() {
                progress.set_progress(i);
                println!("\r{}", progress.render());
                io::stdout().flush()?;

                let result = self.execute(command, options.clone())?;
                results.push(result);
                
                progress.increment();
            }
            
            progress.set_progress(total_commands);
            println!("\r{}", progress.render());
            println!(); // New line after progress bar
        } else {
            for command in commands {
                let result = self.execute(command, options.clone())?;
                results.push(result);
            }
        }

        Ok(results)
    }

    pub fn execute_pipeline(&mut self, commands: &[&str], options: ExecutionOptions) -> Result<CommandResult, Box<dyn std::error::Error>> {
        if options.safety_check {
            for command in commands {
                self.perform_safety_check(command)?;
            }
        }

        if options.show_progress {
            println!("{}", info_text("Executing pipeline..."));
        }

        self.shell.execute_pipeline(commands)
    }

    fn perform_safety_check(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let risk = self.safety_checker.assess_command(command);
        
        match risk.level {
            RiskLevel::Safe => Ok(()),
            RiskLevel::Low => {
                println!("{}", warning_text(&format!("⚠️  Low risk: {}", risk.reason)));
                for suggestion in &risk.suggestions {
                    println!("  💡 {}", suggestion);
                }
                self.prompt_user_confirmation("Proceed with execution? (y/N): ")
            }
            RiskLevel::Medium => {
                println!("{}", warning_text(&format!("⚠️  Medium risk: {}", risk.reason)));
                for suggestion in &risk.suggestions {
                    println!("  💡 {}", suggestion);
                }
                
                let alternatives = self.safety_checker.get_safe_alternatives(command);
                if !alternatives.is_empty() {
                    println!("Safe alternatives:");
                    for alt in alternatives {
                        println!("  ✅ {}", alt);
                    }
                }
                
                self.prompt_user_confirmation("Are you sure you want to proceed? (y/N): ")
            }
            RiskLevel::High => {
                println!("{}", error_text(&format!("🚨 High risk: {}", risk.reason)));
                for suggestion in &risk.suggestions {
                    println!("  💡 {}", suggestion);
                }
                
                let alternatives = self.safety_checker.get_safe_alternatives(command);
                if !alternatives.is_empty() {
                    println!("Safe alternatives:");
                    for alt in alternatives {
                        println!("  ✅ {}", alt);
                    }
                }
                
                self.prompt_user_confirmation("This is dangerous! Are you absolutely sure? (type 'YES' to confirm): ")
                    .and_then(|_| {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if input.trim() == "YES" {
                            Ok(())
                        } else {
                            Err("Command execution cancelled for safety".into())
                        }
                    })
            }
            RiskLevel::Critical => {
                println!("{}", error_text(&format!("🛑 CRITICAL DANGER: {}", risk.reason)));
                println!("{}", error_text("This command could cause irreversible damage!"));
                for suggestion in &risk.suggestions {
                    println!("  💡 {}", suggestion);
                }
                
                let alternatives = self.safety_checker.get_safe_alternatives(command);
                if !alternatives.is_empty() {
                    println!("Safe alternatives:");
                    for alt in alternatives {
                        println!("  ✅ {}", alt);
                    }
                }
                
                Err("Critical command blocked for safety. Use --force-dangerous to override.".into())
            }
        }
    }

    fn prompt_user_confirmation(&self, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            Ok(())
        } else {
            Err("Command execution cancelled by user".into())
        }
    }

    fn execute_with_progress(&mut self, command: &str, _options: ExecutionOptions) -> Result<CommandResult, Box<dyn std::error::Error>> {
        let spinner = Spinner::new().with_title(&format!("Executing: {}", command));
        
        // Show spinner in a separate thread (simplified version)
        let start_time = Instant::now();
        
        println!("{} {}", spinner.render(), command);
        
        let result = self.shell.execute_command(command)?;
        
        let duration = start_time.elapsed();
        
        if result.success {
            println!("{} Command completed in {:.2}s", 
                success_text("✅"), duration.as_secs_f64());
        } else {
            println!("{} Command failed in {:.2}s", 
                error_text("❌"), duration.as_secs_f64());
        }

        Ok(result)
    }

    fn execute_direct(&mut self, command: &str, _options: ExecutionOptions) -> Result<CommandResult, Box<dyn std::error::Error>> {
        self.shell.execute_command(command)
    }

    pub fn run_interactive_shell(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", info_text("Starting Forge Interactive Shell"));
        println!("Enhanced with safety checks and progress indicators");
        println!("Type 'help' for commands, 'exit' to quit\n");

        self.shell.run_interactive()
    }

    pub fn get_command_suggestions(&self, partial_command: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Basic command suggestions
        let common_commands = [
            "ls", "cd", "pwd", "echo", "cat", "grep", "find", "ps", "top", "df", "du",
            "git", "cargo", "npm", "yarn", "python", "node", "java", "gcc", "make",
            "curl", "wget", "ssh", "scp", "rsync", "tar", "gzip", "unzip",
        ];

        for cmd in &common_commands {
            if cmd.starts_with(partial_command) {
                suggestions.push(cmd.to_string());
            }
        }

        // Add shell history matches
        for historical_cmd in self.shell.get_history() {
            if historical_cmd.starts_with(partial_command) && !suggestions.contains(historical_cmd) {
                suggestions.push(historical_cmd.clone());
            }
        }

        suggestions.sort();
        suggestions.truncate(10); // Limit suggestions
        suggestions
    }

    pub fn analyze_performance(&self, command: &str) -> Result<PerformanceAnalysis, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage()?;
        
        // Execute command for analysis (would be actual execution in practice)
        thread::sleep(Duration::from_millis(100)); // Simulate execution
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage()?;
        
        Ok(PerformanceAnalysis {
            command: command.to_string(),
            duration,
            memory_used: end_memory.saturating_sub(start_memory),
            cpu_usage: 0.0, // Would implement actual CPU monitoring
        })
    }

    fn get_memory_usage(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Simplified memory usage - in practice would use system APIs
        Ok(1024 * 1024) // 1MB placeholder
    }
}

#[derive(Debug)]
pub struct PerformanceAnalysis {
    pub command: String,
    pub duration: Duration,
    pub memory_used: u64,
    pub cpu_usage: f64,
}

impl PerformanceAnalysis {
    pub fn report(&self) {
        println!("Performance Analysis for: {}", self.command);
        println!("  Duration: {:.3}s", self.duration.as_secs_f64());
        println!("  Memory: {} KB", self.memory_used / 1024);
        println!("  CPU: {:.1}%", self.cpu_usage);
    }
}

// Specialized executor for long-running processes
pub struct LongRunningExecutor {
    executor: ProcessExecutor,
}

impl LongRunningExecutor {
    pub fn new() -> Self {
        LongRunningExecutor {
            executor: ProcessExecutor::new(),
        }
    }

    pub fn execute_with_live_output(&mut self, command: &str) -> Result<CommandResult, Box<dyn std::error::Error>> {
        println!("{}", info_text(&format!("Starting: {}", command)));
        
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".into());
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        
        // Handle stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("OUT: {}", line);
                    }
                }
            });
        }

        // Handle stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("{}", error_text(&format!("ERR: {}", line)));
                    }
                }
            });
        }

        let status = child.wait()?;
        
        let result = CommandResult {
            success: status.success(),
            exit_code: status.code().unwrap_or(-1),
            stdout: String::new(), // Already printed live
            stderr: String::new(), // Already printed live
            is_exit: false,
        };

        if result.success {
            println!("{}", success_text("✅ Process completed successfully"));
        } else {
            println!("{}", error_text(&format!("❌ Process failed with exit code: {}", result.exit_code)));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let _executor = ProcessExecutor::new();
        let _executor_with_allowlist = ProcessExecutor::with_allowed_commands(
            vec!["git".to_string(), "cargo".to_string()]
        );
    }

    #[test]
    fn test_execution_options() {
        let options = ExecutionOptions {
            timeout: Some(Duration::from_secs(30)),
            show_progress: true,
            ..Default::default()
        };
        
        assert_eq!(options.timeout, Some(Duration::from_secs(30)));
        assert!(options.show_progress);
        assert!(options.safety_check);
    }

    #[test]
    fn test_command_suggestions() {
        let executor = ProcessExecutor::new();
        let suggestions = executor.get_command_suggestions("gi");
        
        assert!(suggestions.contains(&"git".to_string()));
    }
}