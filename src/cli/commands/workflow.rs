use crate::forge_process::workflow::{WorkflowRunner, CommonWorkflows};
use crate::terminal::output::{
    success_text, error_text,
    EnhancedProgressBar, StatusIndicator, StatusType, Table, BorderStyle,
    TerminalControl, BoxDrawing, Spinner
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        show_help();
        return Ok(());
    }

    let mut runner = WorkflowRunner::new();
    
    // Add common workflows
    runner.add_workflow(CommonWorkflows::rust_build_and_test());
    runner.add_workflow(CommonWorkflows::git_workflow());

    match args[0].as_str() {
        "list" => {
            show_workflow_list(&runner);
        }
        "run" => {
            if args.len() < 2 {
                let status = StatusIndicator::new(StatusType::Error, "Workflow name required");
                println!("{}", status.render());
                println!("Usage: forge workflow run <workflow-name>");
                return Ok(());
            }
            
            let workflow_name = &args[1];
            run_workflow_with_progress(&mut runner, workflow_name)?;
        }
        "create" => {
            if args.len() < 3 {
                println!("{}", error_text("Error: Workflow name and commands required"));
                println!("Usage: forge workflow create <name> <command1> [command2] ...");
                return Ok(());
            }
            
            let workflow_name = &args[1];
            let commands: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            
            let workflow = runner.create_simple_workflow(workflow_name, commands);
            runner.add_workflow(workflow);
            
            println!("{}", success_text(&format!("✅ Created workflow: {}", workflow_name)));
        }
        "demo" => {
            run_demo_workflow(&mut runner)?;
        }
        _ => {
            println!("{}", error_text(&format!("Unknown workflow command: {}", args[0])));
            show_help();
        }
    }

    Ok(())
}

fn show_help() {
    // Create a nice bordered help display
    let header_box = BoxDrawing::double_border(80, 3, Some("Forge Workflow System"));
    for line in header_box {
        println!("{}", line);
    }
    println!();
    
    println!("USAGE:");
    println!("    forge workflow <COMMAND> [OPTIONS]");
    println!();
    
    // Create a table for commands
    let mut table = Table::new(vec!["Command", "Description", "Example"]).border_style(BorderStyle::Single);
    table.add_row(vec!["list", "List available workflows", "forge workflow list"]);
    table.add_row(vec!["run <name>", "Execute a workflow", "forge workflow run rust-build-test"]);
    table.add_row(vec!["create <name> <cmd>...", "Create new workflow", "forge workflow create my-build 'cargo build'"]);
    table.add_row(vec!["demo", "Run demonstration", "forge workflow demo"]);
    
    println!("{}", table.render());
}

fn show_workflow_list(runner: &WorkflowRunner) {
    let status = StatusIndicator::new(StatusType::Info, "Available Workflows");
    println!("{}", status.render());
    println!();
    
    let workflows = runner.list_workflows();
    if workflows.is_empty() {
        let empty_status = StatusIndicator::new(StatusType::Warning, "No custom workflows available");
        println!("{}", empty_status.render());
    } else {
        let mut table = Table::new(vec!["#", "Workflow Name"]).border_style(BorderStyle::None);
        for (i, workflow) in workflows.iter().enumerate() {
            table.add_row(vec![&format!("{}", i + 1), workflow]);
        }
        println!("{}", table.render());
    }
    
    println!();
    let builtin_status = StatusIndicator::new(StatusType::Info, "Built-in Workflows");
    println!("{}", builtin_status.render());
    
    let mut builtin_table = Table::new(vec!["Name", "Description"]).border_style(BorderStyle::None);
    builtin_table.add_row(vec!["rust-build-test", "Build and test Rust project"]);
    builtin_table.add_row(vec!["git-commit-push", "Git add, commit, and push workflow"]);
    
    println!("{}", builtin_table.render());
}

fn run_workflow_with_progress(runner: &mut WorkflowRunner, workflow_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start_status = StatusIndicator::new(StatusType::Processing, 
        &format!("Executing workflow: {}", workflow_name));
    println!("{}", start_status.render());
    println!();
    
    // Show a progress bar simulation (in real implementation this would track actual steps)
    let mut progress = EnhancedProgressBar::new(100, 40)
        .with_title(&format!("Running {}", workflow_name));
    
    print!("{}", TerminalControl::hide_cursor());
    
    // Simulate progress
    for i in 0..=100 {
        progress.set_progress(i);
        progress.set_status(&format!("Step {}/5", (i / 20) + 1));
        progress.set_rate(2.5);
        progress.set_eta(((100 - i) / 5) as u64);
        
        print!("{}", TerminalControl::clear_line());
        print!("{}", progress.render());
        io::stdout().flush()?;
        
        thread::sleep(Duration::from_millis(50));
    }
    
    print!("{}", TerminalControl::show_cursor());
    println!();
    println!();
    
    match runner.execute_workflow(workflow_name) {
        Ok(execution) => {
            if execution.overall_success {
                let success_status = StatusIndicator::new(StatusType::Success, 
                    "Workflow completed successfully!");
                println!("{}", success_status.render());
            } else {
                let warning_status = StatusIndicator::new(StatusType::Warning, 
                    "Workflow completed with some failures");
                println!("{}", warning_status.render());
            }
        }
        Err(e) => {
            let error_status = StatusIndicator::new(StatusType::Error, 
                &format!("Workflow execution failed: {}", e));
            println!("{}", error_status.render());
        }
    }
    
    Ok(())
}

fn run_demo_workflow(runner: &mut WorkflowRunner) -> Result<(), Box<dyn std::error::Error>> {
    let demo_status = StatusIndicator::new(StatusType::Info, "Running workflow demonstration");
    println!("{}", demo_status.render());
    println!();
    
    // Show loading spinner while preparing
    let mut spinner = Spinner::arrow().with_title("Preparing demo workflow...");
    print!("{}", TerminalControl::hide_cursor());
    
    for _ in 0..8 {
        print!("{}", TerminalControl::clear_line());
        print!("{}", spinner.next_frame());
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(250));
    }
    
    spinner.stop();
    print!("{}", TerminalControl::clear_line());
    print!("{}", spinner.render());
    print!("{}", TerminalControl::show_cursor());
    println!();
    println!();
    
    let demo_commands = vec![
        "echo 'Starting demo workflow...'",
        "echo 'Step 1: Checking system'",
        "date",
        "echo 'Step 2: Listing current directory'", 
        "ls -la",
        "echo 'Demo workflow completed!'",
    ];
    
    let demo_workflow = runner.create_simple_workflow("demo", demo_commands);
    runner.add_workflow(demo_workflow);
    
    // Show steps in a nice format
    let steps_box = BoxDrawing::rounded_border(60, 8, Some("Demo Workflow Steps"));
    for (i, line) in steps_box.iter().enumerate() {
        if i == 2 {
            println!("│ 1. Echo starting message{:>33} │", "");
        } else if i == 3 {
            println!("│ 2. Check system timestamp{:>32} │", "");
        } else if i == 4 {
            println!("│ 3. Display current date{:>34} │", "");
        } else if i == 5 {
            println!("│ 4. List directory contents{:>30} │", "");
        } else if i == 6 {
            println!("│ 5. Echo completion message{:>30} │", "");
        } else {
            println!("{}", line);
        }
    }
    println!();
    
    match runner.execute_workflow("demo") {
        Ok(_) => {
            let success_status = StatusIndicator::new(StatusType::Success, 
                "Demo workflow completed successfully!");
            println!("{}", success_status.render());
        }
        Err(e) => {
            let error_status = StatusIndicator::new(StatusType::Error, 
                &format!("Demo failed: {}", e));
            println!("{}", error_status.render());
        }
    }
    
    Ok(())
}