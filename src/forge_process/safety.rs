#![allow(dead_code)]

use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct CommandRisk {
    pub level: RiskLevel,
    pub reason: String,
    pub suggestions: Vec<String>,
}

pub struct SafetyChecker {
    dangerous_patterns: Vec<&'static str>,
    destructive_commands: HashSet<&'static str>,
    system_paths: Vec<&'static str>,
    allowed_commands: Option<HashSet<String>>,
}

impl SafetyChecker {
    pub fn new() -> Self {
        let mut destructive_commands = HashSet::new();
        destructive_commands.insert("rm");
        destructive_commands.insert("del");
        destructive_commands.insert("format");
        destructive_commands.insert("fdisk");
        destructive_commands.insert("mkfs");
        destructive_commands.insert("dd");
        destructive_commands.insert("shutdown");
        destructive_commands.insert("reboot");
        destructive_commands.insert("halt");
        destructive_commands.insert("poweroff");
        destructive_commands.insert("systemctl");

        SafetyChecker {
            dangerous_patterns: vec![
                "rm -rf",
                "rm -r",
                "sudo rm",
                "del /s",
                "format c:",
                "fdisk /mbr",
                "mkfs.",
                "dd if=/dev/zero",
                "dd if=/dev/random",
                "> /dev/",
                "chmod 000",
                "chmod -R 000",
                "chown root",
                "chown -R root",
                "init 0",
                "init 6",
                "killall -9",
                "pkill -9",
                ":(){ :|:& };:",  // Fork bomb
                "curl | sh",
                "wget | sh",
                "curl | bash",
                "wget | bash",
            ],
            destructive_commands,
            system_paths: vec![
                "/bin",
                "/sbin",
                "/usr/bin",
                "/usr/sbin",
                "/boot",
                "/dev",
                "/etc",
                "/proc",
                "/sys",
                "C:\\Windows",
                "C:\\Program Files",
                "C:\\System32",
            ],
            allowed_commands: None,
        }
    }

    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = Some(commands.into_iter().collect());
        self
    }

    pub fn assess_command(&self, command: &str) -> CommandRisk {
        let command_lower = command.to_lowercase();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.is_empty() {
            return CommandRisk {
                level: RiskLevel::Safe,
                reason: "Empty command".to_string(),
                suggestions: vec!["Specify a command to execute".to_string()],
            };
        }

        let base_command = parts[0];

        // Check if command is in allowlist (if configured)
        if let Some(allowed) = &self.allowed_commands {
            if !allowed.contains(base_command) {
                return CommandRisk {
                    level: RiskLevel::High,
                    reason: format!("Command '{}' is not in the allowed list", base_command),
                    suggestions: vec![
                        "Only pre-approved commands are allowed in this environment".to_string(),
                        format!("Allowed commands: {}", allowed.iter().cloned().collect::<Vec<_>>().join(", ")),
                    ],
                };
            }
        }

        // Check for critical patterns
        for pattern in &self.dangerous_patterns {
            if command_lower.contains(pattern) {
                return CommandRisk {
                    level: RiskLevel::Critical,
                    reason: format!("Contains dangerous pattern: {}", pattern),
                    suggestions: vec![
                        "This command could cause irreversible system damage".to_string(),
                        "Consider using safer alternatives or be extremely careful".to_string(),
                        "Always have backups before running destructive commands".to_string(),
                    ],
                };
            }
        }

        // Check base command for destructive potential
        if self.destructive_commands.contains(base_command) {
            let risk = self.assess_destructive_command(command, parts);
            if risk.level != RiskLevel::Safe {
                return risk;
            }
        }

        // Check for system path manipulation
        for sys_path in &self.system_paths {
            if command_lower.contains(&sys_path.to_lowercase()) {
                return CommandRisk {
                    level: RiskLevel::High,
                    reason: format!("Attempts to modify system path: {}", sys_path),
                    suggestions: vec![
                        "Modifying system paths can break your system".to_string(),
                        "Use extreme caution when working with system directories".to_string(),
                    ],
                };
            }
        }

        // Check for suspicious network activity
        if self.has_network_risk(command) {
            return CommandRisk {
                level: RiskLevel::Medium,
                reason: "Command downloads and executes content from the internet".to_string(),
                suggestions: vec![
                    "Review the source and content before executing".to_string(),
                    "Consider downloading and inspecting the script first".to_string(),
                ],
            };
        }

        // Check for privilege escalation
        if command_lower.starts_with("sudo ") || command_lower.contains("sudo ") {
            return CommandRisk {
                level: RiskLevel::Medium,
                reason: "Command requires elevated privileges".to_string(),
                suggestions: vec![
                    "Ensure you understand what the command does with elevated privileges".to_string(),
                    "Consider running without sudo first if possible".to_string(),
                ],
            };
        }

        // Check for recursive operations
        if command_lower.contains(" -r") || command_lower.contains(" --recursive") {
            return CommandRisk {
                level: RiskLevel::Low,
                reason: "Command performs recursive operations".to_string(),
                suggestions: vec![
                    "Be careful with recursive operations on large directory trees".to_string(),
                    "Consider testing on a small subset first".to_string(),
                ],
            };
        }

        CommandRisk {
            level: RiskLevel::Safe,
            reason: "Command appears safe".to_string(),
            suggestions: vec![],
        }
    }

    fn assess_destructive_command(&self, command: &str, parts: Vec<&str>) -> CommandRisk {
        let base_command = parts[0];
        
        match base_command {
            "rm" => {
                if parts.iter().any(|&arg| arg == "-rf" || arg == "-r") {
                    CommandRisk {
                        level: RiskLevel::Critical,
                        reason: "Recursive file deletion".to_string(),
                        suggestions: vec![
                            "This will delete files and directories recursively".to_string(),
                            "Make sure you have backups".to_string(),
                            "Double-check the target path".to_string(),
                        ],
                    }
                } else if parts.iter().any(|&arg| arg.starts_with('/') && arg.len() < 4) {
                    CommandRisk {
                        level: RiskLevel::Critical,
                        reason: "Attempting to delete system root directories".to_string(),
                        suggestions: vec![
                            "This could destroy your entire system".to_string(),
                            "Never delete root system directories".to_string(),
                        ],
                    }
                } else {
                    CommandRisk {
                        level: RiskLevel::Low,
                        reason: "File deletion command".to_string(),
                        suggestions: vec!["Ensure the target files are correct".to_string()],
                    }
                }
            }
            "dd" => {
                if command.contains("/dev/") {
                    CommandRisk {
                        level: RiskLevel::Critical,
                        reason: "Direct disk access with dd".to_string(),
                        suggestions: vec![
                            "This can overwrite disk data directly".to_string(),
                            "Wrong usage can destroy all data on the disk".to_string(),
                            "Verify the input/output devices carefully".to_string(),
                        ],
                    }
                } else {
                    CommandRisk {
                        level: RiskLevel::Medium,
                        reason: "Data copying with dd".to_string(),
                        suggestions: vec!["Verify source and destination paths".to_string()],
                    }
                }
            }
            "shutdown" | "reboot" | "halt" | "poweroff" => {
                CommandRisk {
                    level: RiskLevel::Medium,
                    reason: "System power control".to_string(),
                    suggestions: vec![
                        "This will shut down or restart the system".to_string(),
                        "Save your work before proceeding".to_string(),
                    ],
                }
            }
            "systemctl" => {
                if parts.iter().any(|&arg| arg == "stop" || arg == "disable" || arg == "mask") {
                    CommandRisk {
                        level: RiskLevel::High,
                        reason: "Stopping or disabling system services".to_string(),
                        suggestions: vec![
                            "This may affect system functionality".to_string(),
                            "Make sure you understand the service's purpose".to_string(),
                        ],
                    }
                } else {
                    CommandRisk {
                        level: RiskLevel::Low,
                        reason: "System service management".to_string(),
                        suggestions: vec!["Review the service and action carefully".to_string()],
                    }
                }
            }
            _ => CommandRisk {
                level: RiskLevel::Safe,
                reason: "Standard command usage".to_string(),
                suggestions: vec![],
            }
        }
    }

    fn has_network_risk(&self, command: &str) -> bool {
        (command.contains("curl") || command.contains("wget")) &&
        (command.contains("| sh") || command.contains("| bash") || 
         command.contains("|sh") || command.contains("|bash"))
    }

    pub fn is_command_allowed(&self, command: &str) -> bool {
        let risk = self.assess_command(command);
        matches!(risk.level, RiskLevel::Safe | RiskLevel::Low)
    }

    pub fn get_safe_alternatives(&self, command: &str) -> Vec<String> {
        let mut alternatives = Vec::new();
        let command_lower = command.to_lowercase();

        if command_lower.starts_with("rm -rf") {
            alternatives.push("Use 'rm -i' for interactive deletion".to_string());
            alternatives.push("Move files to trash instead of permanent deletion".to_string());
            alternatives.push("List files first with 'ls' to verify targets".to_string());
        }

        if command_lower.contains("curl") && command_lower.contains("| sh") {
            alternatives.push("Download the script first: curl <url> -o script.sh".to_string());
            alternatives.push("Review the script: cat script.sh".to_string());
            alternatives.push("Then execute if safe: bash script.sh".to_string());
        }

        if command_lower.starts_with("sudo") {
            alternatives.push("Try running without sudo first if possible".to_string());
            alternatives.push("Use specific sudo commands instead of sudo su".to_string());
        }

        alternatives
    }
}

// File system safety checks
pub struct FileSystemSafety;

impl FileSystemSafety {
    pub fn is_safe_path(path: &str) -> bool {
        let path = Path::new(path);
        
        // Check for dangerous paths
        if let Some(path_str) = path.to_str() {
            let dangerous_paths = [
                "/", "/bin", "/sbin", "/boot", "/dev", "/etc", "/proc", "/sys",
                "/usr/bin", "/usr/sbin", "/var/log", "/var/lib",
                "C:\\", "C:\\Windows", "C:\\Program Files", "C:\\System32",
            ];

            for dangerous in &dangerous_paths {
                if path_str == *dangerous || path_str.starts_with(&format!("{}/", dangerous)) {
                    return false;
                }
            }
        }

        // Check for relative path traversal
        if path.to_string_lossy().contains("..") {
            return false;
        }

        true
    }

    pub fn suggest_safe_path(dangerous_path: &str) -> Option<String> {
        if dangerous_path.starts_with('/') && !dangerous_path.starts_with("/home") {
            Some(format!("/tmp{}", dangerous_path))
        } else if dangerous_path.starts_with("C:\\") && !dangerous_path.starts_with("C:\\Users") {
            Some(format!("C:\\temp{}", &dangerous_path[3..]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        let checker = SafetyChecker::new();
        let risk = checker.assess_command("ls -la");
        assert_eq!(risk.level, RiskLevel::Safe);
    }

    #[test]
    fn test_dangerous_rm() {
        let checker = SafetyChecker::new();
        let risk = checker.assess_command("rm -rf /");
        assert_eq!(risk.level, RiskLevel::Critical);
    }

    #[test]
    fn test_sudo_command() {
        let checker = SafetyChecker::new();
        let risk = checker.assess_command("sudo apt update");
        assert_eq!(risk.level, RiskLevel::Medium);
    }

    #[test]
    fn test_network_risk() {
        let checker = SafetyChecker::new();
        let risk = checker.assess_command("curl https://example.com/script.sh | sh");
        assert_eq!(risk.level, RiskLevel::Medium);
    }

    #[test]
    fn test_allowlist() {
        let allowed_commands = vec!["git".to_string(), "npm".to_string(), "cargo".to_string()];
        let checker = SafetyChecker::new().with_allowed_commands(allowed_commands);
        
        let risk = checker.assess_command("git status");
        assert_eq!(risk.level, RiskLevel::Safe);
        
        let risk = checker.assess_command("rm file.txt");
        assert_eq!(risk.level, RiskLevel::High);
    }

    #[test]
    fn test_file_path_safety() {
        assert!(FileSystemSafety::is_safe_path("/home/user/documents"));
        assert!(!FileSystemSafety::is_safe_path("/etc/passwd"));
        assert!(!FileSystemSafety::is_safe_path("../../../etc/passwd"));
    }
}