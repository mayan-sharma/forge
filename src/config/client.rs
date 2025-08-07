use crate::config::ForgeConfig;
use crate::http::client::OllamaClient;

pub fn create_ollama_client() -> Result<(OllamaClient, String), Box<dyn std::error::Error>> {
    let config = ForgeConfig::load()?;
    
    let base_url = match config.llm.default_provider.as_str() {
        "ollama" => "http://localhost:11434",
        _ => return Err(format!("Unsupported LLM provider: {}", config.llm.default_provider).into()),
    };
    
    let client = OllamaClient::new(base_url)?;
    let model = config.llm.default_model.clone();
    
    Ok((client, model))
}

pub fn get_config_or_default() -> ForgeConfig {
    ForgeConfig::load().unwrap_or_else(|_| ForgeConfig::default())
}