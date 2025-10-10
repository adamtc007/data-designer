use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub port: u16,
    pub auto_start: bool,
    pub reconnect_attempts: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarConfig {
    pub auto_refresh: bool,
    pub cache_enabled: bool,
    pub validation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub application: ApplicationConfig,
    pub lsp: LspConfig,
    pub grammar: GrammarConfig,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "data_designer".to_string(),
            username: "adamtc007".to_string(),
            password: None,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
        }
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            name: "Data Designer IDE".to_string(),
            version: "1.0.0".to_string(),
            debug_mode: true,
        }
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        LspConfig {
            port: 3030,
            auto_start: false,
            reconnect_attempts: 3,
            timeout_ms: 5000,
        }
    }
}

impl Default for GrammarConfig {
    fn default() -> Self {
        GrammarConfig {
            auto_refresh: false,
            cache_enabled: true,
            validation_enabled: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database: DatabaseConfig::default(),
            application: ApplicationConfig::default(),
            lsp: LspConfig::default(),
            grammar: GrammarConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file with environment variable overrides
    pub fn load() -> Result<Self, String> {
        let mut config = Self::load_from_file().unwrap_or_else(|_| {
            println!("⚠️ Could not load config.toml, using defaults");
            Config::default()
        });

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Load configuration from config.toml file
    fn load_from_file() -> Result<Self, String> {
        let config_path = Path::new("config.toml");

        if !config_path.exists() {
            return Err("config.toml not found".to_string());
        }

        let config_content = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config.toml: {}", e))?;

        toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config.toml: {}", e))
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Database URL override (highest priority)
        if let Ok(database_url) = env::var("DATABASE_URL") {
            if let Ok(parsed) = Self::parse_database_url(&database_url) {
                self.database = parsed;
                return;
            }
        }

        // Individual database settings
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.database.port = port_num;
            }
        }
        if let Ok(database) = env::var("DB_NAME") {
            self.database.database = database;
        }
        if let Ok(username) = env::var("DB_USER") {
            self.database.username = username;
        }
        if let Ok(password) = env::var("DB_PASSWORD") {
            self.database.password = Some(password);
        }
        if let Ok(password) = env::var("PGPASSWORD") {
            self.database.password = Some(password);
        }

        // LSP settings
        if let Ok(port) = env::var("LSP_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.lsp.port = port_num;
            }
        }

        // Application settings
        if let Ok(debug) = env::var("DEBUG") {
            self.application.debug_mode = debug.to_lowercase() == "true" || debug == "1";
        }
    }

    /// Parse a PostgreSQL connection URL
    fn parse_database_url(url: &str) -> Result<DatabaseConfig, String> {
        // Simple URL parsing for postgresql://user:password@host:port/database
        if !url.starts_with("postgresql://") {
            return Err("Invalid database URL scheme".to_string());
        }

        let url = &url[13..]; // Remove "postgresql://"
        let mut config = DatabaseConfig::default();

        // Split by @ to separate user info from host info
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid database URL format".to_string());
        }

        // Parse user info (user:password)
        let user_info = parts[0];
        if let Some(colon_pos) = user_info.find(':') {
            config.username = user_info[..colon_pos].to_string();
            config.password = Some(user_info[colon_pos + 1..].to_string());
        } else {
            config.username = user_info.to_string();
        }

        // Parse host info (host:port/database)
        let host_info = parts[1];
        if let Some(slash_pos) = host_info.find('/') {
            config.database = host_info[slash_pos + 1..].to_string();
            let host_port = &host_info[..slash_pos];

            if let Some(colon_pos) = host_port.find(':') {
                config.host = host_port[..colon_pos].to_string();
                if let Ok(port) = host_port[colon_pos + 1..].parse::<u16>() {
                    config.port = port;
                }
            } else {
                config.host = host_port.to_string();
            }
        } else {
            return Err("Database name not found in URL".to_string());
        }

        Ok(config)
    }

    /// Generate a PostgreSQL connection URL from the config
    pub fn database_url(&self) -> String {
        let auth = if let Some(ref password) = self.database.password {
            format!("{}:{}", self.database.username, password)
        } else {
            self.database.username.clone()
        };

        format!(
            "postgresql://{}@{}:{}/{}",
            auth, self.database.host, self.database.port, self.database.database
        )
    }

    /// Save current configuration to file
    pub fn save_to_file(&self) -> Result<(), String> {
        let config_content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write("config.toml", config_content)
            .map_err(|e| format!("Failed to write config.toml: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_database_url() {
        let url = "postgresql://user:pass@localhost:5432/mydb";
        let config = Config::parse_database_url(url).unwrap();

        assert_eq!(config.username, "user");
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "mydb");
    }

    #[test]
    fn test_database_url_generation() {
        let config = Config::default();
        let url = config.database_url();
        assert!(url.starts_with("postgresql://"));
        assert!(url.contains("@localhost:5432/data_designer"));
    }
}