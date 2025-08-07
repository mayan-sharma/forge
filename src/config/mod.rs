use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod client;

#[derive(Debug, Clone)]
pub struct ForgeConfig {
    pub llm: LLMConfig,
    pub ui: UIConfig,
    pub safety: SafetyConfig,
    pub api_keys: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub default_provider: String,
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct UIConfig {
    pub theme: String,
    pub show_line_numbers: bool,
    pub syntax_highlighting: bool,
    pub auto_save: bool,
}

#[derive(Debug, Clone)]
pub struct SafetyConfig {
    pub enable_safety_checks: bool,
    pub allow_system_commands: bool,
    pub restricted_paths: Vec<String>,
    pub max_file_size_mb: u32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            ui: UIConfig::default(),
            safety: SafetyConfig::default(),
            api_keys: HashMap::new(),
        }
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            default_model: "llama3.2".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            timeout_seconds: 30,
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_line_numbers: true,
            syntax_highlighting: true,
            auto_save: false,
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enable_safety_checks: true,
            allow_system_commands: false,
            restricted_paths: vec![
                "/etc".to_string(),
                "/usr/bin".to_string(),
                "/sbin".to_string(),
            ],
            max_file_size_mb: 10,
        }
    }
}

impl ForgeConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| ConfigError::IoError(format!("Failed to read config file: {}", e)))?;

        let parsed: TomlConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Invalid TOML: {}", e)))?;

        Ok(parsed.into())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::IoError(format!("Failed to create config directory: {}", e)))?;
        }

        let toml_config: TomlConfig = self.clone().into();
        let content = toml::to_string_pretty(&toml_config)
            .map_err(|e| ConfigError::SerializeError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, content)
            .map_err(|e| ConfigError::IoError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::PathError("Could not determine home directory".to_string()))?;
        
        Ok(home.join(".config").join("forge").join("config.toml"))
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    PathError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialization error: {}", msg),
            ConfigError::PathError(msg) => write!(f, "Path error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TomlConfig {
    llm: Option<TomlLLMConfig>,
    ui: Option<TomlUIConfig>,
    safety: Option<TomlSafetyConfig>,
    api_keys: Option<HashMap<String, String>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TomlLLMConfig {
    default_provider: Option<String>,
    default_model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    timeout_seconds: Option<u64>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TomlUIConfig {
    theme: Option<String>,
    show_line_numbers: Option<bool>,
    syntax_highlighting: Option<bool>,
    auto_save: Option<bool>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TomlSafetyConfig {
    enable_safety_checks: Option<bool>,
    allow_system_commands: Option<bool>,
    restricted_paths: Option<Vec<String>>,
    max_file_size_mb: Option<u32>,
}

impl From<TomlConfig> for ForgeConfig {
    fn from(toml: TomlConfig) -> Self {
        let default = ForgeConfig::default();
        
        ForgeConfig {
            llm: LLMConfig {
                default_provider: toml.llm.as_ref()
                    .and_then(|l| l.default_provider.clone())
                    .unwrap_or(default.llm.default_provider),
                default_model: toml.llm.as_ref()
                    .and_then(|l| l.default_model.clone())
                    .unwrap_or(default.llm.default_model),
                temperature: toml.llm.as_ref()
                    .and_then(|l| l.temperature)
                    .unwrap_or(default.llm.temperature),
                max_tokens: toml.llm.as_ref()
                    .and_then(|l| l.max_tokens)
                    .unwrap_or(default.llm.max_tokens),
                timeout_seconds: toml.llm.as_ref()
                    .and_then(|l| l.timeout_seconds)
                    .unwrap_or(default.llm.timeout_seconds),
            },
            ui: UIConfig {
                theme: toml.ui.as_ref()
                    .and_then(|u| u.theme.clone())
                    .unwrap_or(default.ui.theme),
                show_line_numbers: toml.ui.as_ref()
                    .and_then(|u| u.show_line_numbers)
                    .unwrap_or(default.ui.show_line_numbers),
                syntax_highlighting: toml.ui.as_ref()
                    .and_then(|u| u.syntax_highlighting)
                    .unwrap_or(default.ui.syntax_highlighting),
                auto_save: toml.ui.as_ref()
                    .and_then(|u| u.auto_save)
                    .unwrap_or(default.ui.auto_save),
            },
            safety: SafetyConfig {
                enable_safety_checks: toml.safety.as_ref()
                    .and_then(|s| s.enable_safety_checks)
                    .unwrap_or(default.safety.enable_safety_checks),
                allow_system_commands: toml.safety.as_ref()
                    .and_then(|s| s.allow_system_commands)
                    .unwrap_or(default.safety.allow_system_commands),
                restricted_paths: toml.safety.as_ref()
                    .and_then(|s| s.restricted_paths.clone())
                    .unwrap_or(default.safety.restricted_paths),
                max_file_size_mb: toml.safety.as_ref()
                    .and_then(|s| s.max_file_size_mb)
                    .unwrap_or(default.safety.max_file_size_mb),
            },
            api_keys: toml.api_keys.unwrap_or_default(),
        }
    }
}

impl From<ForgeConfig> for TomlConfig {
    fn from(config: ForgeConfig) -> Self {
        TomlConfig {
            llm: Some(TomlLLMConfig {
                default_provider: Some(config.llm.default_provider),
                default_model: Some(config.llm.default_model),
                temperature: Some(config.llm.temperature),
                max_tokens: Some(config.llm.max_tokens),
                timeout_seconds: Some(config.llm.timeout_seconds),
            }),
            ui: Some(TomlUIConfig {
                theme: Some(config.ui.theme),
                show_line_numbers: Some(config.ui.show_line_numbers),
                syntax_highlighting: Some(config.ui.syntax_highlighting),
                auto_save: Some(config.ui.auto_save),
            }),
            safety: Some(TomlSafetyConfig {
                enable_safety_checks: Some(config.safety.enable_safety_checks),
                allow_system_commands: Some(config.safety.allow_system_commands),
                restricted_paths: Some(config.safety.restricted_paths),
                max_file_size_mb: Some(config.safety.max_file_size_mb),
            }),
            api_keys: if config.api_keys.is_empty() { None } else { Some(config.api_keys) },
        }
    }
}