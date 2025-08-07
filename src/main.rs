use std::env;
use std::process;

mod cli;
mod http;
mod fs;
mod terminal;
mod forge_process;
mod config;

use terminal::output::{StyledText, Color, error_text, success_text, info_text, warning_text, dim_text};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        show_help();
        return;
    }
    
    match args[1].as_str() {
        "chat" => {
            println!("{}", info_text("🚀 Starting chat interface..."));
            if let Err(e) = cli::commands::chat::run() {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "edit" => {
            if args.len() < 3 {
                eprintln!("{}", error_text("❌ Error: edit command requires a file path"));
                eprintln!("{}", dim_text("   Usage: forge edit <file> [instruction]"));
                process::exit(1);
            }
            let instruction = if args.len() > 3 {
                Some(args[3..].join(" "))
            } else {
                None
            };
            if let Err(e) = cli::commands::edit::run(&args[2], instruction.as_deref()) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("{}", error_text("❌ Error: search command requires a query"));
                eprintln!("{}", dim_text("   Usage: forge search <query> [path]"));
                process::exit(1);
            }
            let path = if args.len() > 3 { Some(args[3].as_str()) } else { None };
            if let Err(e) = cli::commands::search::run(&args[2], path) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "exec" => {
            if args.len() < 3 {
                eprintln!("{}", error_text("❌ Error: exec command requires a command"));
                eprintln!("{}", dim_text("   Usage: forge exec <command>"));
                process::exit(1);
            }
            let command_args = args[2..].to_vec();
            if let Err(e) = cli::commands::exec::run(&command_args) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "shell" => {
            println!("{}", info_text("🐚 Starting interactive shell..."));
            if let Err(e) = cli::commands::shell::run() {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "workflow" => {
            let workflow_args = if args.len() > 2 { &args[2..] } else { &[] };
            if let Err(e) = cli::commands::workflow::run(workflow_args) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "status" => {
            let status_args = if args.len() > 2 { &args[2..] } else { &[] };
            if let Err(e) = cli::commands::status::run_with_args(&status_args.iter().map(|s| s.clone()).collect::<Vec<String>>()) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "config" => {
            let config_args = if args.len() > 2 { &args[2..] } else { &[] };
            if let Err(e) = cli::commands::config::run(&config_args.iter().map(|s| s.clone()).collect::<Vec<String>>()) {
                eprintln!("{}", error_text(&format!("❌ Error: {}", e)));
                process::exit(1);
            }
        }
        "test-ollama" => {
            println!("{}", info_text("🔍 Testing Ollama connection..."));
            if let Err(e) = test_ollama() {
                eprintln!("{}", error_text(&format!("❌ Ollama test failed: {}", e)));
                process::exit(1);
            }
            println!("{}", success_text("✅ Ollama test passed!"));
        }
        "test-files" => {
            println!("{}", info_text("📁 Testing file operations..."));
            if let Err(e) = test_files() {
                eprintln!("{}", error_text(&format!("❌ File test failed: {}", e)));
                process::exit(1);
            }
            println!("{}", success_text("✅ File test passed!"));
        }
        "--help" | "-h" => show_help(),
        "--version" | "-v" => show_version(),
        _ => {
            eprintln!("{}", error_text(&format!("❌ Unknown command: {}", args[1])));
            eprintln!();
            show_help();
            process::exit(1);
        }
    }
}

fn show_help() {
    println!("{}", StyledText::new("⚡ Forge - Autonomous CLI Agent Tool")
        .fg(Color::BrightCyan)
        .bold());
    println!("{}", dim_text("   Built in Rust with minimal dependencies for maximum performance"));
    println!();
    
    println!("{}", StyledText::new("USAGE:")
        .fg(Color::BrightYellow)
        .bold());
    println!("    {} {} {}", 
        StyledText::new("forge").fg(Color::BrightGreen).bold(),
        StyledText::new("<COMMAND>").fg(Color::BrightMagenta),
        StyledText::new("[OPTIONS]").fg(Color::BrightBlue));
    println!();
    
    println!("{}", StyledText::new("COMMANDS:")
        .fg(Color::BrightYellow)
        .bold());
    
    print_command_help("💬", "chat", "", "Start interactive chat with AI");
    print_command_help("✏️ ", "edit", "<file> [instruction]", "Edit a file with AI assistance");
    print_command_help("🔍", "search", "<query> [path]", "Search files for text or patterns");
    print_command_help("⚡", "exec", "<command>", "Execute commands with enhanced safety");
    print_command_help("🐚", "shell", "", "Start interactive shell with safety features");
    print_command_help("📋", "workflow", "[subcommand]", "Manage and execute command workflows");
    print_command_help("📊", "status", "[--clear|--demo]", "Show background tasks and notifications");
    print_command_help("⚙️ ", "config", "[subcommand]", "Manage configuration settings");
    print_command_help("🧪", "test-ollama", "", "Test Ollama API connection");
    print_command_help("📁", "test-files", "", "Test file operations");
    
    println!();
    println!("{}", StyledText::new("OPTIONS:")
        .fg(Color::BrightYellow)
        .bold());
    print_option_help("-h, --help", "Show this help message");
    print_option_help("-v, --version", "Show version information");
    
    println!();
    println!("{}", dim_text("Examples:"));
    println!("  {} {}", 
        StyledText::new("forge chat").fg(Color::BrightGreen),
        dim_text("# Start AI chat session"));
    println!("  {} {}", 
        StyledText::new("forge edit main.rs \"add error handling\"").fg(Color::BrightGreen),
        dim_text("# Edit file with AI"));
    println!("  {} {}", 
        StyledText::new("forge search \"fn main\" src/").fg(Color::BrightGreen),
        dim_text("# Search for functions"));
    println!();
    println!("{}", info_text("💡 Tip: Make sure Ollama is running with 'ollama serve'"));
}

fn print_command_help(icon: &str, command: &str, args: &str, description: &str) {
    let command_part = if args.is_empty() {
        format!("{}  {}", icon, command)
    } else {
        format!("{}  {} {}", icon, command, StyledText::new(args).fg(Color::BrightBlue))
    };
    
    println!("    {:<25} {}", 
        StyledText::new(&command_part).fg(Color::BrightGreen).bold(),
        StyledText::new(description).fg(Color::White));
}

fn print_option_help(option: &str, description: &str) {
    println!("    {:<18} {}", 
        StyledText::new(option).fg(Color::BrightBlue),
        StyledText::new(description).fg(Color::White));
}

fn show_version() {
    println!("{} {}", 
        StyledText::new("⚡ Forge").fg(Color::BrightCyan).bold(),
        StyledText::new(&format!("v{}", env!("CARGO_PKG_VERSION"))).fg(Color::BrightGreen));
    println!("{}", dim_text("Autonomous CLI Agent Tool built in Rust"));
    println!("{}", dim_text("https://github.com/yourusername/forge"));
}

fn test_ollama() -> Result<(), Box<dyn std::error::Error>> {
    let (client, default_model) = config::client::create_ollama_client()?;
    
    println!("{}", info_text("🔍 Checking available models..."));
    match client.list_models() {
        Ok(models) => {
            if models.is_empty() {
                println!("{}", warning_text("⚠️  No models found"));
                println!("{}", dim_text("   Install a model with: ollama pull llama3"));
                return Err("No models available".into());
            }
            
            println!("{}", success_text(&format!("✅ Found {} model(s):", models.len())));
            for model in &models {
                println!("    {} {}", 
                    StyledText::new("•").fg(Color::BrightGreen),
                    StyledText::new(model).fg(Color::BrightCyan));
            }
            println!();
            
            let test_model = if models.contains(&default_model) {
                &default_model
            } else {
                &models[0]
            };
            println!("{}", info_text(&format!("🧪 Testing with model: {}", test_model)));
            
            let response = client.generate(test_model, "Hello, this is a test message. Please respond briefly.", false)?;
            println!("{}", success_text("✅ Connection successful!"));
            println!("{} {}", 
                StyledText::new("Response:").fg(Color::BrightYellow),
                StyledText::new(&response).fg(Color::White));
        }
        Err(e) => {
            println!("{}", error_text(&format!("❌ Failed to connect to Ollama: {}", e)));
            println!("{}", dim_text("   Make sure Ollama is running: ollama serve"));
            return Err(format!("Failed to list models: {}", e).into());
        }
    }
    
    Ok(())
}

fn test_files() -> Result<(), Box<dyn std::error::Error>> {
    let test_content = "Hello, this is a test file!";
    let test_path = "test_forge_file.txt";
    
    println!("{}", info_text("📝 Creating test file..."));
    fs::operations::write_file(test_path, test_content)?;
    println!("{} {}", 
        success_text("✅ Created:"), 
        StyledText::new(test_path).fg(Color::BrightCyan));
    
    println!("{}", info_text("📄 Reading file content..."));
    let read_content = fs::operations::read_file(test_path)?;
    println!("{} {}", 
        success_text("✅ Content:"),
        StyledText::new(&format!("\"{}\"", read_content)).fg(Color::BrightGreen));
    
    if read_content != test_content {
        println!("{}", error_text("❌ File content mismatch!"));
        println!("{} {}", 
            StyledText::new("Expected:").fg(Color::Yellow),
            StyledText::new(&format!("\"{}\"", test_content)).fg(Color::White));
        println!("{} {}", 
            StyledText::new("Got:").fg(Color::Yellow),
            StyledText::new(&format!("\"{}\"", read_content)).fg(Color::White));
        return Err("File content mismatch".into());
    }
    
    println!("{}", info_text("🗺️ Cleaning up..."));
    fs::operations::delete_file(test_path)?;
    println!("{} {}", 
        success_text("✅ Deleted:"), 
        StyledText::new(test_path).fg(Color::BrightCyan));
    
    Ok(())
}
