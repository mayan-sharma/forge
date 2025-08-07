use crate::config::{ForgeConfig, ConfigError};
use crate::terminal::output::{StyledText, Color, error_text, success_text, info_text, dim_text};

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        show_config()?;
        return Ok(());
    }

    match args[0].as_str() {
        "show" => show_config()?,
        "init" => init_config()?,
        "set" => {
            if args.len() < 3 {
                eprintln!("{}", error_text("❌ Error: set requires key and value"));
                eprintln!("{}", dim_text("   Usage: forge config set <key> <value>"));
                return Err("Missing arguments".into());
            }
            set_config(&args[1], &args[2])?;
        }
        "get" => {
            if args.len() < 2 {
                eprintln!("{}", error_text("❌ Error: get requires a key"));
                eprintln!("{}", dim_text("   Usage: forge config get <key>"));
                return Err("Missing key".into());
            }
            get_config(&args[1])?;
        }
        "reset" => reset_config()?,
        _ => {
            eprintln!("{}", error_text(&format!("❌ Unknown config command: {}", args[0])));
            show_config_help();
            return Err("Unknown command".into());
        }
    }

    Ok(())
}

fn show_config() -> Result<(), ConfigError> {
    let config = ForgeConfig::load()?;
    
    println!("{}", StyledText::new("⚙️  Forge Configuration")
        .fg(Color::BrightCyan)
        .bold());
    println!();
    
    println!("{}", StyledText::new("🤖 LLM Settings:")
        .fg(Color::BrightYellow)
        .bold());
    println!("  {} {}", 
        StyledText::new("Provider:").fg(Color::White),
        StyledText::new(&config.llm.default_provider).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Model:").fg(Color::White),
        StyledText::new(&config.llm.default_model).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Temperature:").fg(Color::White),
        StyledText::new(&config.llm.temperature.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Max Tokens:").fg(Color::White),
        StyledText::new(&config.llm.max_tokens.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Timeout (s):").fg(Color::White),
        StyledText::new(&config.llm.timeout_seconds.to_string()).fg(Color::BrightGreen));
    println!();
    
    println!("{}", StyledText::new("🎨 UI Settings:")
        .fg(Color::BrightYellow)
        .bold());
    println!("  {} {}", 
        StyledText::new("Theme:").fg(Color::White),
        StyledText::new(&config.ui.theme).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Line Numbers:").fg(Color::White),
        StyledText::new(&config.ui.show_line_numbers.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Syntax Highlighting:").fg(Color::White),
        StyledText::new(&config.ui.syntax_highlighting.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Auto Save:").fg(Color::White),
        StyledText::new(&config.ui.auto_save.to_string()).fg(Color::BrightGreen));
    println!();
    
    println!("{}", StyledText::new("🛡️  Safety Settings:")
        .fg(Color::BrightYellow)
        .bold());
    println!("  {} {}", 
        StyledText::new("Enable Checks:").fg(Color::White),
        StyledText::new(&config.safety.enable_safety_checks.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Allow System Commands:").fg(Color::White),
        StyledText::new(&config.safety.allow_system_commands.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Max File Size (MB):").fg(Color::White),
        StyledText::new(&config.safety.max_file_size_mb.to_string()).fg(Color::BrightGreen));
    println!("  {} {}", 
        StyledText::new("Restricted Paths:").fg(Color::White),
        StyledText::new(&format!("{:?}", config.safety.restricted_paths)).fg(Color::BrightGreen));
    println!();
    
    println!("{}", StyledText::new("🔑 API Keys:")
        .fg(Color::BrightYellow)
        .bold());
    if config.api_keys.is_empty() {
        println!("  {} {}", 
            StyledText::new("Status:").fg(Color::White),
            StyledText::new("No API keys configured").fg(Color::Yellow));
    } else {
        for (provider, _) in &config.api_keys {
            println!("  {} {}", 
                StyledText::new(&format!("{}:", provider)).fg(Color::White),
                StyledText::new("***configured***").fg(Color::BrightGreen));
        }
    }
    
    Ok(())
}

fn init_config() -> Result<(), ConfigError> {
    let config = ForgeConfig::default();
    config.save()?;
    
    println!("{}", success_text("✅ Configuration initialized with defaults"));
    println!("{}", info_text("   Config saved to ~/.config/forge/config.toml"));
    
    Ok(())
}

fn set_config(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ForgeConfig::load()?;
    
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return Err("Invalid key format. Use format: section.key (e.g., llm.default_model)".into());
    }
    
    match (parts[0], parts[1]) {
        ("llm", "default_provider") => config.llm.default_provider = value.to_string(),
        ("llm", "default_model") => config.llm.default_model = value.to_string(),
        ("llm", "temperature") => config.llm.temperature = value.parse()?,
        ("llm", "max_tokens") => config.llm.max_tokens = value.parse()?,
        ("llm", "timeout_seconds") => config.llm.timeout_seconds = value.parse()?,
        ("ui", "theme") => config.ui.theme = value.to_string(),
        ("ui", "show_line_numbers") => config.ui.show_line_numbers = value.parse()?,
        ("ui", "syntax_highlighting") => config.ui.syntax_highlighting = value.parse()?,
        ("ui", "auto_save") => config.ui.auto_save = value.parse()?,
        ("safety", "enable_safety_checks") => config.safety.enable_safety_checks = value.parse()?,
        ("safety", "allow_system_commands") => config.safety.allow_system_commands = value.parse()?,
        ("safety", "max_file_size_mb") => config.safety.max_file_size_mb = value.parse()?,
        ("api_keys", provider) => {
            config.api_keys.insert(provider.to_string(), value.to_string());
        }
        _ => return Err(format!("Unknown configuration key: {}", key).into()),
    }
    
    config.save()?;
    println!("{} {} = {}", 
        success_text("✅ Set"),
        StyledText::new(key).fg(Color::BrightCyan),
        StyledText::new(value).fg(Color::BrightGreen));
    
    Ok(())
}

fn get_config(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = ForgeConfig::load()?;
    
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return Err("Invalid key format. Use format: section.key (e.g., llm.default_model)".into());
    }
    
    let value = match (parts[0], parts[1]) {
        ("llm", "default_provider") => config.llm.default_provider,
        ("llm", "default_model") => config.llm.default_model,
        ("llm", "temperature") => config.llm.temperature.to_string(),
        ("llm", "max_tokens") => config.llm.max_tokens.to_string(),
        ("llm", "timeout_seconds") => config.llm.timeout_seconds.to_string(),
        ("ui", "theme") => config.ui.theme,
        ("ui", "show_line_numbers") => config.ui.show_line_numbers.to_string(),
        ("ui", "syntax_highlighting") => config.ui.syntax_highlighting.to_string(),
        ("ui", "auto_save") => config.ui.auto_save.to_string(),
        ("safety", "enable_safety_checks") => config.safety.enable_safety_checks.to_string(),
        ("safety", "allow_system_commands") => config.safety.allow_system_commands.to_string(),
        ("safety", "max_file_size_mb") => config.safety.max_file_size_mb.to_string(),
        ("api_keys", provider) => {
            match config.api_keys.get(provider) {
                Some(_) => "***configured***".to_string(),
                None => "not set".to_string(),
            }
        }
        _ => return Err(format!("Unknown configuration key: {}", key).into()),
    };
    
    println!("{} {}", 
        StyledText::new(key).fg(Color::BrightCyan),
        StyledText::new(&value).fg(Color::BrightGreen));
    
    Ok(())
}

fn reset_config() -> Result<(), ConfigError> {
    let config = ForgeConfig::default();
    config.save()?;
    
    println!("{}", success_text("✅ Configuration reset to defaults"));
    
    Ok(())
}

fn show_config_help() {
    println!("{}", StyledText::new("⚙️  Forge Config Commands")
        .fg(Color::BrightCyan)
        .bold());
    println!();
    
    println!("{}", StyledText::new("USAGE:")
        .fg(Color::BrightYellow)
        .bold());
    println!("    {} {} {}", 
        StyledText::new("forge config").fg(Color::BrightGreen).bold(),
        StyledText::new("<COMMAND>").fg(Color::BrightMagenta),
        StyledText::new("[ARGS]").fg(Color::BrightBlue));
    println!();
    
    println!("{}", StyledText::new("COMMANDS:")
        .fg(Color::BrightYellow)
        .bold());
    
    println!("    {:<25} {}", 
        StyledText::new("show").fg(Color::BrightGreen).bold(),
        "Display current configuration");
    println!("    {:<25} {}", 
        StyledText::new("init").fg(Color::BrightGreen).bold(),
        "Initialize configuration with defaults");
    println!("    {:<25} {}", 
        StyledText::new("set <key> <value>").fg(Color::BrightGreen).bold(),
        "Set a configuration value");
    println!("    {:<25} {}", 
        StyledText::new("get <key>").fg(Color::BrightGreen).bold(),
        "Get a configuration value");
    println!("    {:<25} {}", 
        StyledText::new("reset").fg(Color::BrightGreen).bold(),
        "Reset configuration to defaults");
    
    println!();
    println!("{}", dim_text("Examples:"));
    println!("  {} {}", 
        StyledText::new("forge config set llm.default_model llama3.2").fg(Color::BrightGreen),
        dim_text("# Set default model"));
    println!("  {} {}", 
        StyledText::new("forge config set api_keys.openai your_api_key").fg(Color::BrightGreen),
        dim_text("# Set OpenAI API key"));
    println!("  {} {}", 
        StyledText::new("forge config get ui.theme").fg(Color::BrightGreen),
        dim_text("# Get current theme"));
}