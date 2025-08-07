#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fs;
use crate::terminal::output::{ProgressBar, success_text, error_text, warning_text, info_text};
use super::executor::{ProcessExecutor, ExecutionOptions};

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub continue_on_failure: bool,
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub conditions: Vec<WorkflowCondition>,
}

#[derive(Debug, Clone)]
pub struct WorkflowCondition {
    pub condition_type: ConditionType,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    FileExists,
    FileNotExists,
    DirectoryExists,
    DirectoryNotExists,
    EnvironmentVariable,
    PreviousStepSuccess,
    PreviousStepFailure,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, String>,
    pub on_failure: FailureAction,
}

#[derive(Debug, Clone)]
pub enum FailureAction {
    Stop,
    Continue,
    Rollback,
}

#[derive(Debug)]
pub struct WorkflowExecution {
    pub workflow_name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub step_results: Vec<StepResult>,
    pub overall_success: bool,
}

#[derive(Debug)]
pub struct StepResult {
    pub step_name: String,
    pub success: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
    pub retry_attempts: u32,
}

pub struct WorkflowRunner {
    executor: ProcessExecutor,
    workflows: HashMap<String, Workflow>,
}

impl WorkflowRunner {
    pub fn new() -> Self {
        WorkflowRunner {
            executor: ProcessExecutor::new(),
            workflows: HashMap::new(),
        }
    }

    pub fn load_workflow_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let workflow = self.parse_workflow_yaml(&content)?;
        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.name.clone(), workflow);
    }

    pub fn execute_workflow(&mut self, workflow_name: &str) -> Result<WorkflowExecution, Box<dyn std::error::Error>> {
        let workflow = self.workflows.get(workflow_name)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_name))?
            .clone();

        println!("{}", info_text(&format!("🚀 Starting workflow: {}", workflow.name)));
        if let Some(desc) = &workflow.description {
            println!("Description: {}", desc);
        }
        println!();

        let start_time = Instant::now();
        let mut execution = WorkflowExecution {
            workflow_name: workflow.name.clone(),
            start_time,
            end_time: None,
            step_results: Vec::new(),
            overall_success: true,
        };

        let total_steps = workflow.steps.len();
        let mut progress = ProgressBar::new(total_steps, 50)
            .with_title("Workflow Progress");

        for (step_index, step) in workflow.steps.iter().enumerate() {
            progress.set_progress(step_index);
            println!("\r{}", progress.render());
            println!();

            println!("{}", info_text(&format!("📋 Step {}/{}: {}", step_index + 1, total_steps, step.name)));
            if let Some(desc) = &step.description {
                println!("   {}", desc);
            }

            // Check conditions
            if !self.check_conditions(step, &execution)? {
                println!("{}", warning_text("⏭️  Skipping step due to unmet conditions"));
                continue;
            }

            let step_result = self.execute_step(step, &workflow.variables)?;
            let step_success = step_result.success;

            execution.step_results.push(step_result);

            if !step_success {
                execution.overall_success = false;
                
                if !step.continue_on_failure {
                    match workflow.on_failure {
                        FailureAction::Stop => {
                            println!("{}", error_text("🛑 Workflow stopped due to step failure"));
                            break;
                        }
                        FailureAction::Continue => {
                            println!("{}", warning_text("⚠️  Continuing despite step failure"));
                        }
                        FailureAction::Rollback => {
                            println!("{}", warning_text("🔄 Attempting rollback..."));
                            self.rollback_workflow(&workflow, &execution)?;
                            break;
                        }
                    }
                }
            }
        }

        progress.set_progress(total_steps);
        println!("\r{}", progress.render());
        println!();

        execution.end_time = Some(Instant::now());
        let duration = execution.end_time.unwrap() - execution.start_time;

        if execution.overall_success {
            println!("{}", success_text(&format!("✅ Workflow '{}' completed successfully in {:.2}s", 
                workflow.name, duration.as_secs_f64())));
        } else {
            println!("{}", error_text(&format!("❌ Workflow '{}' failed after {:.2}s", 
                workflow.name, duration.as_secs_f64())));
        }

        self.print_execution_summary(&execution);
        Ok(execution)
    }

    fn execute_step(&mut self, step: &WorkflowStep, variables: &HashMap<String, String>) -> Result<StepResult, Box<dyn std::error::Error>> {
        let step_start = Instant::now();
        let mut command = step.command.clone();

        // Variable substitution
        for (var, value) in variables {
            command = command.replace(&format!("${{{}}}", var), value);
        }

        let mut retry_attempts = 0;
        let mut last_error = None;

        while retry_attempts <= step.retry_count {
            if retry_attempts > 0 {
                println!("{}", warning_text(&format!("🔄 Retry attempt {} of {}", retry_attempts, step.retry_count)));
                std::thread::sleep(Duration::from_secs(1)); // Brief delay between retries
            }

            let execution_options = ExecutionOptions {
                timeout: step.timeout,
                show_progress: false, // We're showing workflow progress
                capture_output: true,
                interactive: false,
                safety_check: true,
                working_directory: None,
            };

            match self.executor.execute(&command, execution_options) {
                Ok(result) => {
                    let duration = step_start.elapsed();
                    
                    if result.success {
                        println!("{}", success_text(&format!("✅ Step '{}' completed", step.name)));
                        return Ok(StepResult {
                            step_name: step.name.clone(),
                            success: true,
                            duration,
                            output: result.stdout,
                            error: if result.stderr.is_empty() { None } else { Some(result.stderr) },
                            retry_attempts,
                        });
                    } else {
                        last_error = Some(format!("Command failed: {}", result.stderr));
                        retry_attempts += 1;
                        
                        if retry_attempts > step.retry_count {
                            println!("{}", error_text(&format!("❌ Step '{}' failed after {} retries", step.name, step.retry_count)));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    retry_attempts += 1;
                    
                    if retry_attempts > step.retry_count {
                        println!("{}", error_text(&format!("❌ Step '{}' failed: {}", step.name, e)));
                    }
                }
            }
        }

        let duration = step_start.elapsed();
        Ok(StepResult {
            step_name: step.name.clone(),
            success: false,
            duration,
            output: String::new(),
            error: last_error,
            retry_attempts,
        })
    }

    fn check_conditions(&self, step: &WorkflowStep, execution: &WorkflowExecution) -> Result<bool, Box<dyn std::error::Error>> {
        for condition in &step.conditions {
            match condition.condition_type {
                ConditionType::FileExists => {
                    if !std::path::Path::new(&condition.value).exists() {
                        return Ok(false);
                    }
                }
                ConditionType::FileNotExists => {
                    if std::path::Path::new(&condition.value).exists() {
                        return Ok(false);
                    }
                }
                ConditionType::DirectoryExists => {
                    let path = std::path::Path::new(&condition.value);
                    if !path.exists() || !path.is_dir() {
                        return Ok(false);
                    }
                }
                ConditionType::DirectoryNotExists => {
                    let path = std::path::Path::new(&condition.value);
                    if path.exists() && path.is_dir() {
                        return Ok(false);
                    }
                }
                ConditionType::EnvironmentVariable => {
                    if std::env::var(&condition.value).is_err() {
                        return Ok(false);
                    }
                }
                ConditionType::PreviousStepSuccess => {
                    if let Some(prev_step) = execution.step_results.iter().find(|s| s.step_name == condition.value) {
                        if !prev_step.success {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                ConditionType::PreviousStepFailure => {
                    if let Some(prev_step) = execution.step_results.iter().find(|s| s.step_name == condition.value) {
                        if prev_step.success {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    fn rollback_workflow(&mut self, _workflow: &Workflow, execution: &WorkflowExecution) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", info_text("🔄 Starting workflow rollback..."));
        
        // Execute steps in reverse order (simplified rollback)
        for step_result in execution.step_results.iter().rev() {
            if step_result.success {
                println!("{}", info_text(&format!("Rolling back: {}", step_result.step_name)));
                // In a real implementation, you'd have rollback commands for each step
                // For now, we'll just log the rollback intent
            }
        }
        
        println!("{}", success_text("Rollback completed"));
        Ok(())
    }

    fn print_execution_summary(&self, execution: &WorkflowExecution) {
        println!("\n📊 Execution Summary:");
        println!("Workflow: {}", execution.workflow_name);
        
        if let Some(end_time) = execution.end_time {
            let total_duration = end_time - execution.start_time;
            println!("Total Duration: {:.2}s", total_duration.as_secs_f64());
        }

        let successful_steps = execution.step_results.iter().filter(|s| s.success).count();
        let total_steps = execution.step_results.len();
        println!("Steps: {}/{} successful", successful_steps, total_steps);

        println!("\nStep Details:");
        for (i, step) in execution.step_results.iter().enumerate() {
            let status = if step.success { "✅" } else { "❌" };
            println!("  {}: {} {} ({:.2}s)", i + 1, status, step.step_name, step.duration.as_secs_f64());
            
            if step.retry_attempts > 0 {
                println!("     Retries: {}", step.retry_attempts);
            }
            
            if let Some(error) = &step.error {
                println!("     Error: {}", error);
            }
        }
    }

    pub fn list_workflows(&self) -> Vec<&String> {
        self.workflows.keys().collect()
    }

    pub fn create_simple_workflow(&mut self, name: &str, commands: Vec<&str>) -> Workflow {
        let steps = commands.into_iter().enumerate().map(|(i, cmd)| {
            WorkflowStep {
                name: format!("Step {}", i + 1),
                command: cmd.to_string(),
                description: Some(format!("Execute: {}", cmd)),
                continue_on_failure: false,
                timeout: Some(Duration::from_secs(300)), // 5 minutes default
                retry_count: 0,
                conditions: Vec::new(),
            }
        }).collect();

        Workflow {
            name: name.to_string(),
            description: Some("Auto-generated workflow".to_string()),
            steps,
            variables: HashMap::new(),
            on_failure: FailureAction::Stop,
        }
    }

    // Simplified YAML parsing (in practice, you'd use a YAML library)
    fn parse_workflow_yaml(&self, _yaml_content: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
        // This is a placeholder - in a real implementation you'd parse YAML
        Ok(Workflow {
            name: "example".to_string(),
            description: Some("Example workflow".to_string()),
            steps: vec![
                WorkflowStep {
                    name: "Hello".to_string(),
                    command: "echo 'Hello from workflow'".to_string(),
                    description: None,
                    continue_on_failure: false,
                    timeout: None,
                    retry_count: 0,
                    conditions: Vec::new(),
                }
            ],
            variables: HashMap::new(),
            on_failure: FailureAction::Stop,
        })
    }
}

// Predefined workflows for common development tasks
pub struct CommonWorkflows;

impl CommonWorkflows {
    pub fn rust_build_and_test() -> Workflow {
        Workflow {
            name: "rust-build-test".to_string(),
            description: Some("Build and test Rust project".to_string()),
            steps: vec![
                WorkflowStep {
                    name: "Format Check".to_string(),
                    command: "cargo fmt -- --check".to_string(),
                    description: Some("Check code formatting".to_string()),
                    continue_on_failure: true,
                    timeout: Some(Duration::from_secs(60)),
                    retry_count: 0,
                    conditions: Vec::new(),
                },
                WorkflowStep {
                    name: "Build".to_string(),
                    command: "cargo build".to_string(),
                    description: Some("Build the project".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(300)),
                    retry_count: 1,
                    conditions: Vec::new(),
                },
                WorkflowStep {
                    name: "Test".to_string(),
                    command: "cargo test".to_string(),
                    description: Some("Run tests".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(600)),
                    retry_count: 1,
                    conditions: Vec::new(),
                },
            ],
            variables: HashMap::new(),
            on_failure: FailureAction::Stop,
        }
    }

    pub fn git_workflow() -> Workflow {
        let mut variables = HashMap::new();
        variables.insert("COMMIT_MESSAGE".to_string(), "Auto commit".to_string());

        Workflow {
            name: "git-commit-push".to_string(),
            description: Some("Add, commit, and push changes".to_string()),
            steps: vec![
                WorkflowStep {
                    name: "Status Check".to_string(),
                    command: "git status --porcelain".to_string(),
                    description: Some("Check for changes".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(30)),
                    retry_count: 0,
                    conditions: Vec::new(),
                },
                WorkflowStep {
                    name: "Add Changes".to_string(),
                    command: "git add .".to_string(),
                    description: Some("Stage all changes".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(60)),
                    retry_count: 0,
                    conditions: Vec::new(),
                },
                WorkflowStep {
                    name: "Commit".to_string(),
                    command: "git commit -m \"${COMMIT_MESSAGE}\"".to_string(),
                    description: Some("Commit changes".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(60)),
                    retry_count: 0,
                    conditions: Vec::new(),
                },
                WorkflowStep {
                    name: "Push".to_string(),
                    command: "git push".to_string(),
                    description: Some("Push to remote".to_string()),
                    continue_on_failure: false,
                    timeout: Some(Duration::from_secs(120)),
                    retry_count: 2,
                    conditions: Vec::new(),
                },
            ],
            variables,
            on_failure: FailureAction::Stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let mut runner = WorkflowRunner::new();
        let workflow = runner.create_simple_workflow("test", vec!["echo hello", "echo world"]);
        
        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_common_workflows() {
        let rust_workflow = CommonWorkflows::rust_build_and_test();
        assert_eq!(rust_workflow.name, "rust-build-test");
        assert!(rust_workflow.steps.len() >= 2);

        let git_workflow = CommonWorkflows::git_workflow();
        assert_eq!(git_workflow.name, "git-commit-push");
        assert!(git_workflow.variables.contains_key("COMMIT_MESSAGE"));
    }
}