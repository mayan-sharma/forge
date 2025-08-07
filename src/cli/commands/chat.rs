use crate::http::client::OllamaClient;
use crate::config::client;
use crate::terminal::output::{
    StyledText, Color, error_text, success_text, info_text, warning_text, dim_text,
    Spinner, StatusIndicator, StatusType, TerminalControl, BoxDrawing
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", StyledText::new("💬 Forge Chat Interface")
        .fg(Color::BrightCyan)
        .bold());
    println!("{}", dim_text("   AI-powered coding assistance with conversation history"));
    println!();
    println!("{}", info_text("💡 Commands: /help, /clear, /history, exit"));
    println!("{}", dim_text("   Press Ctrl+C to interrupt at any time"));
    println!();

    let (client, default_model) = client::create_ollama_client()?;
    
    println!("{}", info_text("🔍 Checking available models..."));
    let model = match client.list_models() {
        Ok(models) => {
            if models.is_empty() {
                println!("{}", warning_text("⚠️  No models found"));
                println!("{}", dim_text("   Install a model with: ollama pull llama3"));
                return Ok(());
            }
            println!("{} {}", 
                success_text("✅ Found models:"),
                StyledText::new(&models.join(", ")).fg(Color::BrightCyan));
            
            let selected_model = if models.contains(&default_model) {
                &default_model
            } else {
                &models[0]
            };
            
            println!("{} {}", 
                info_text("🧪 Using:"),
                StyledText::new(selected_model).fg(Color::BrightGreen).bold());
            println!();
            selected_model.clone()
        }
        Err(e) => {
            println!("{}", warning_text(&format!("⚠️  Could not list models: {}", e)));
            println!("{} {}", 
                info_text("🧪 Using configured model:"),
                StyledText::new(&default_model).fg(Color::BrightGreen).bold());
            println!();
            default_model
        }
    };
    let mut conversation_history = String::new();

    loop {
        // Enhanced prompt with better styling
        print!("{}{} ", 
            StyledText::new("╭─[").fg(Color::BrightBlack),
            StyledText::new("forge").fg(Color::BrightCyan).bold());
        print!("{}{} ", 
            StyledText::new("]─[").fg(Color::BrightBlack),
            StyledText::new(&model).fg(Color::BrightYellow));
        print!("{}", StyledText::new("]\n╰─> ").fg(Color::BrightBlack));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => {
                println!("{}", success_text("👋 Goodbye! Thanks for using Forge!"));
                break;
            }
            "/help" => {
                show_help();
                continue;
            }
            "/clear" => {
                conversation_history.clear();
                println!("{}", success_text("✨ Conversation history cleared"));
                continue;
            }
            "/history" => {
                if conversation_history.is_empty() {
                    println!("{}", info_text("📜 No conversation history yet"));
                } else {
                    println!("{}", StyledText::new("📜 Conversation History:")
                        .fg(Color::BrightYellow).bold());
                    println!("{}", StyledText::new(&format!("─{}", "─".repeat(50)))
                        .fg(Color::BrightBlack));
                    println!("{}", conversation_history);
                    println!("{}", StyledText::new(&format!("─{}", "─".repeat(50)))
                        .fg(Color::BrightBlack));
                }
                continue;
            }
            _ => {}
        }

        // Add context to the prompt
        let full_prompt = if conversation_history.is_empty() {
            format!("You are Forge, a helpful coding assistant. Please provide clear, concise, and accurate responses.\n\nUser: {}", input)
        } else {
            format!("{}\n\nUser: {}", conversation_history, input)
        };

        print!("{} ", StyledText::new("🤖 Assistant:").fg(Color::BrightBlue).bold());
        io::stdout().flush()?;
        
        let mut response = String::new();
        match client.generate_stream(&model, &full_prompt, |chunk| {
            print!("{}", chunk);
            io::stdout().flush()?;
            response.push_str(chunk);
            Ok(())
        }) {
            Ok(()) => {
                println!(); // New line after streaming response
                
                // Update conversation history
                conversation_history.push_str(&format!("User: {}\nAssistant: {}\n", input, response));
                
                // Keep history manageable (last 2000 characters)
                if conversation_history.len() > 2000 {
                    let keep_from = conversation_history.len() - 1500;
                    if let Some(newline_pos) = conversation_history[keep_from..].find('\n') {
                        conversation_history = conversation_history[keep_from + newline_pos + 1..].to_string();
                    }
                }
            }
            Err(e) => {
                println!();
                println!("{}", error_text(&format!("❌ Connection error: {}", e)));
                println!("{}", dim_text("   • Make sure Ollama is running: ollama serve"));
                println!("{}", dim_text(&format!("   • Verify model is available: ollama list | grep {}", model)));
            }
        }
        
        println!();
    }

    Ok(())
}

fn show_help() {
    println!("{}", StyledText::new("💬 Forge Chat Commands:")
        .fg(Color::BrightYellow)
        .bold());
    println!();
    
    print_chat_command("/help", "Show this help message");
    print_chat_command("/clear", "Clear conversation history");
    print_chat_command("/history", "Show conversation history");
    print_chat_command("exit", "Exit the chat interface");
    print_chat_command("quit", "Exit the chat interface");
    
    println!();
    println!("{}", info_text("💡 Just type your message to chat with the AI assistant"));
    println!("{}", dim_text("   Example: \"Explain how to use Rust lifetimes\""));
}

fn print_chat_command(command: &str, description: &str) {
    println!("  {:<12} {}", 
        StyledText::new(command).fg(Color::BrightCyan).bold(),
        StyledText::new(description).fg(Color::White));
}

#[allow(dead_code)]
fn show_models(client: &OllamaClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut spinner = Spinner::new().with_title("Fetching available models...");
    print!("{}", TerminalControl::hide_cursor());
    
    for _ in 0..3 {
        print!("{}", TerminalControl::clear_line());
        print!("{}", spinner.next_frame());
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(200));
    }
    
    print!("{}", TerminalControl::clear_line());
    print!("{}", TerminalControl::show_cursor());
    
    match client.list_models() {
        Ok(models) => {
            if models.is_empty() {
                let status = StatusIndicator::new(StatusType::Warning, "No models available");
                println!("{}", status.render());
            } else {
                let status = StatusIndicator::new(StatusType::Success, 
                    &format!("Found {} available model(s)", models.len()));
                println!("{}", status.render());
                
                println!();
                for (i, model) in models.iter().enumerate() {
                    println!("  {}. {}", 
                        StyledText::new(&format!("{}", i + 1)).fg(Color::BrightYellow),
                        StyledText::new(model).fg(Color::BrightCyan));
                }
            }
        }
        Err(e) => {
            let status = StatusIndicator::new(StatusType::Error, 
                &format!("Failed to fetch models: {}", e));
            println!("{}", status.render());
        }
    }
    
    println!();
    Ok(())
}

#[allow(dead_code)]
fn show_history(conversation_history: &str) {
    if conversation_history.is_empty() {
        let status = StatusIndicator::new(StatusType::Info, "No conversation history yet");
        println!("{}", status.render());
    } else {
        // Create a bordered history display
        let header_box = BoxDrawing::single_border(80, 1, Some("Conversation History"));
        println!("{}", header_box[0]);
        
        // Display the history with proper formatting
        let lines: Vec<&str> = conversation_history.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("User: ") {
                println!("│ {} {:<74} │",
                    StyledText::new("👤").fg(Color::BrightGreen),
                    &line[6..]);
            } else if line.starts_with("Assistant: ") {
                println!("│ {} {:<74} │",
                    StyledText::new("🤖").fg(Color::BrightBlue),
                    &line[11..]);
            } else if !line.trim().is_empty() {
                println!("│   {:<76} │", line);
            }
            
            if i < lines.len() - 1 && !lines[i + 1].is_empty() {
                println!("│{:─<78}│", "");
            }
        }
        
        println!("└{:─<78}┘", "");
    }
    println!();
}