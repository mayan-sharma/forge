#![allow(dead_code)]

use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::io::{self, Write};
use std::path::Path;
use crate::terminal::output::{success_text, error_text, warning_text, Spinner};

#[derive(Debug, Clone)]
pub struct ShellEnvironment {
    pub variables: HashMap<String, String>,
    pub working_directory: String,
    pub path: Vec<String>,
}

impl ShellEnvironment {
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        
        // Copy current environment
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }

        let working_directory = env::current_dir()
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
            .to_string_lossy()
            .to_string();

        let path = env::var("PATH")
            .unwrap_or_default()
            .split(if cfg!(windows) { ';' } else { ':' })
            .map(|s| s.to_string())
            .collect();

        ShellEnvironment {
            variables,
            working_directory,
            path,
        }
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn expand_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // Simple variable expansion for $VAR and ${VAR}
        for (key, value) in &self.variables {
            let var_patterns = [
                format!("${}", key),
                format!("${{{}}}", key),
            ];
            
            for pattern in &var_patterns {
                result = result.replace(pattern, value);
            }
        }
        
        result
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let expanded_path = self.expand_variables(path);
        let new_path = Path::new(&expanded_path);
        
        let absolute_path = if new_path.is_absolute() {
            new_path.to_path_buf()
        } else {
            Path::new(&self.working_directory).join(new_path)
        };

        if absolute_path.exists() && absolute_path.is_dir() {
            self.working_directory = absolute_path.to_string_lossy().to_string();
            env::set_current_dir(&absolute_path)?;
            Ok(())
        } else {
            Err(format!("Directory does not exist: {}", expanded_path).into())
        }
    }
}

pub struct Shell {
    environment: ShellEnvironment,
    history: Vec<String>,
    aliases: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        
        // Common aliases
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("la".to_string(), "ls -a".to_string());
        aliases.insert("..".to_string(), "cd ..".to_string());
        aliases.insert("...".to_string(), "cd ../..".to_string());
        
        Shell {
            environment: ShellEnvironment::new(),
            history: Vec::new(),
            aliases,
        }
    }

    pub fn execute_command(&mut self, command: &str) -> Result<CommandResult, Box<dyn std::error::Error>> {
        let command = command.trim();
        if command.is_empty() {
            return Ok(CommandResult::empty());
        }

        // Add to history
        self.history.push(command.to_string());

        // Expand variables and aliases
        let expanded_command = self.environment.expand_variables(command);
        let processed_command = self.resolve_aliases(&expanded_command);

        // Handle built-in commands
        if let Some(result) = self.handle_builtin(&processed_command)? {
            return Ok(result);
        }

        // Parse command and arguments
        let parts = self.parse_command_line(&processed_command);
        if parts.is_empty() {
            return Ok(CommandResult::empty());
        }

        let program = &parts[0];
        let args = &parts[1..];

        // Execute external command
        self.execute_external_command(program, args)
    }

    pub fn execute_with_progress<F>(&mut self, command: &str, mut progress_callback: F) -> Result<CommandResult, Box<dyn std::error::Error>>
    where
        F: FnMut(&str) -> (),
    {
        let command = command.trim();
        if command.is_empty() {
            return Ok(CommandResult::empty());
        }

        progress_callback("Parsing command...");
        
        let expanded_command = self.environment.expand_variables(command);
        let processed_command = self.resolve_aliases(&expanded_command);

        progress_callback("Executing command...");

        // For demonstration, show a spinner during execution
        let _spinner = Spinner::new().with_title("Running");
        
        // Execute the command (simplified for progress demo)
        let result = self.execute_command(&processed_command)?;
        
        progress_callback("Command completed");
        
        Ok(result)
    }

    pub fn execute_pipeline(&mut self, commands: &[&str]) -> Result<CommandResult, Box<dyn std::error::Error>> {
        if commands.is_empty() {
            return Ok(CommandResult::empty());
        }

        if commands.len() == 1 {
            return self.execute_command(commands[0]);
        }

        println!("Executing pipeline: {}", commands.join(" | "));

        // For a simple implementation, execute commands in sequence
        // In a full shell, this would involve actual piping
        let mut final_result = CommandResult::empty();

        for (i, command) in commands.iter().enumerate() {
            println!("Step {}: {}", i + 1, command);
            let result = self.execute_command(command)?;
            
            if !result.success {
                println!("{}", error_text(&format!("Pipeline failed at step {}", i + 1)));
                return Ok(result);
            }
            
            final_result = result;
        }

        println!("{}", success_text("Pipeline completed successfully"));
        Ok(final_result)
    }

    fn handle_builtin(&mut self, command: &str) -> Result<Option<CommandResult>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "cd" => {
                let path = if parts.len() > 1 {
                    parts[1]
                } else {
                    "~" // Home directory
                };
                
                match self.environment.change_directory(path) {
                    Ok(()) => Ok(Some(CommandResult::success(format!("Changed directory to {}", path)))),
                    Err(e) => Ok(Some(CommandResult::error(e.to_string()))),
                }
            }
            "pwd" => {
                Ok(Some(CommandResult::success(self.environment.working_directory.clone())))
            }
            "echo" => {
                let output = parts[1..].join(" ");
                Ok(Some(CommandResult::success(output)))
            }
            "set" => {
                if parts.len() == 3 {
                    self.environment.set_variable(parts[1], parts[2]);
                    Ok(Some(CommandResult::success(format!("Set {}={}", parts[1], parts[2]))))
                } else if parts.len() == 1 {
                    // Show all variables
                    let mut output = String::new();
                    for (key, value) in &self.environment.variables {
                        output.push_str(&format!("{}={}\n", key, value));
                    }
                    Ok(Some(CommandResult::success(output)))
                } else {
                    Ok(Some(CommandResult::error("Usage: set [VAR VALUE]".to_string())))
                }
            }
            "alias" => {
                if parts.len() == 3 {
                    self.aliases.insert(parts[1].to_string(), parts[2].to_string());
                    Ok(Some(CommandResult::success(format!("Alias set: {} -> {}", parts[1], parts[2]))))
                } else if parts.len() == 1 {
                    let mut output = String::new();
                    for (alias, command) in &self.aliases {
                        output.push_str(&format!("{}='{}'\n", alias, command));
                    }
                    Ok(Some(CommandResult::success(output)))
                } else {
                    Ok(Some(CommandResult::error("Usage: alias [NAME COMMAND]".to_string())))
                }
            }
            "history" => {
                let output = self.history.iter()
                    .enumerate()
                    .map(|(i, cmd)| format!("{}: {}", i + 1, cmd))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(Some(CommandResult::success(output)))
            }
            "exit" => {
                Ok(Some(CommandResult::exit()))
            }
            _ => Ok(None),
        }
    }

    fn resolve_aliases(&self, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return command.to_string();
        }

        if let Some(alias_value) = self.aliases.get(parts[0]) {
            if parts.len() > 1 {
                format!("{} {}", alias_value, parts[1..].join(" "))
            } else {
                alias_value.clone()
            }
        } else {
            command.to_string()
        }
    }

    fn parse_command_line(&self, command: &str) -> Vec<String> {
        // Simple command line parsing (doesn't handle all shell features)
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = command.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        parts
    }

    fn execute_external_command(&mut self, program: &str, args: &[String]) -> Result<CommandResult, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.current_dir(&self.environment.working_directory);
        
        // Set environment variables
        for (key, value) in &self.environment.variables {
            cmd.env(key, value);
        }

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                let success = output.status.success();

                Ok(CommandResult {
                    success,
                    exit_code,
                    stdout,
                    stderr,
                    is_exit: false,
                })
            }
            Err(e) => {
                Ok(CommandResult::error(format!("Failed to execute '{}': {}", program, e)))
            }
        }
    }

    pub fn run_interactive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", success_text("Forge Shell Interactive Mode"));
        println!("Type 'help' for available commands, 'exit' to quit");

        loop {
            print!("forge-shell:{}$ ", self.environment.working_directory);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if input == "help" {
                self.show_help();
                continue;
            }

            match self.execute_command(input) {
                Ok(result) => {
                    if result.is_exit {
                        println!("Goodbye!");
                        break;
                    }
                    
                    if !result.stdout.is_empty() {
                        print!("{}", result.stdout);
                    }
                    
                    if !result.stderr.is_empty() {
                        print!("{}", error_text(&result.stderr));
                    }

                    if !result.success {
                        println!("{}", warning_text(&format!("Command failed with exit code: {}", result.exit_code)));
                    }
                }
                Err(e) => {
                    println!("{}", error_text(&format!("Error: {}", e)));
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("Forge Shell Built-in Commands:");
        println!("  cd <path>        - Change directory");
        println!("  pwd              - Print working directory");
        println!("  echo <text>      - Print text");
        println!("  set [var value]  - Set/show environment variables");
        println!("  alias [name cmd] - Set/show command aliases");
        println!("  history          - Show command history");
        println!("  exit             - Exit shell");
        println!();
        println!("Current aliases:");
        for (alias, command) in &self.aliases {
            println!("  {} -> {}", alias, command);
        }
    }

    pub fn get_environment(&self) -> &ShellEnvironment {
        &self.environment
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub is_exit: bool,
}

impl CommandResult {
    pub fn success(output: String) -> Self {
        CommandResult {
            success: true,
            exit_code: 0,
            stdout: output,
            stderr: String::new(),
            is_exit: false,
        }
    }

    pub fn error(message: String) -> Self {
        CommandResult {
            success: false,
            exit_code: 1,
            stdout: String::new(),
            stderr: message,
            is_exit: false,
        }
    }

    pub fn exit() -> Self {
        CommandResult {
            success: true,
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            is_exit: true,
        }
    }

    pub fn empty() -> Self {
        CommandResult {
            success: true,
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            is_exit: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_variable_expansion() {
        let mut env = ShellEnvironment::new();
        env.set_variable("USER", "testuser");
        
        let expanded = env.expand_variables("Hello $USER, welcome to ${USER}'s home");
        assert!(expanded.contains("testuser"));
    }

    #[test]
    fn test_shell_builtin_echo() {
        let mut shell = Shell::new();
        let result = shell.execute_command("echo Hello World").unwrap();
        
        assert!(result.success);
        assert_eq!(result.stdout, "Hello World");
    }

    #[test]
    fn test_shell_alias() {
        let mut shell = Shell::new();
        
        // Set an alias
        let result = shell.execute_command("alias ll ls -la").unwrap();
        assert!(result.success);
        
        // Test alias resolution
        let resolved = shell.resolve_aliases("ll /tmp");
        assert_eq!(resolved, "ls -la /tmp");
    }

    #[test]
    fn test_command_parsing() {
        let shell = Shell::new();
        let parts = shell.parse_command_line("echo \"hello world\" test");
        
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "echo");
        assert_eq!(parts[1], "hello world");
        assert_eq!(parts[2], "test");
    }
}