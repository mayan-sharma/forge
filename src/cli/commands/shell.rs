use crate::forge_process::executor::ProcessExecutor;
use crate::terminal::output::{success_text, info_text};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", info_text("🐚 Starting Forge Interactive Shell"));
    println!("Enhanced shell with safety checks, history, and AI integration");
    println!("Type 'help' for available commands, 'exit' to return to forge");
    println!();

    let mut executor = ProcessExecutor::new();
    executor.run_interactive_shell()?;

    println!("{}", success_text("Returned to Forge CLI"));
    Ok(())
}