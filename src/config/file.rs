use std::fs;

use tracing::info;

use super::model::AppConfig;

pub const DEFAULT_CONFIG: &str = include_str!("config.toml");

fn config_file_path() -> String {
    let config_file_path = directories::ProjectDirs::from("com", "nicolaschan", "ocean")
        .unwrap()
        .config_dir()
        .join("config.toml")
        .to_str()
        .unwrap()
        .to_string();
    config_file_path
}

pub fn read_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let file_path = &config_file_path();
    info!("Reading config from: {}", file_path);
    let config_str = match fs::read_to_string(file_path) {
        Ok(config_str) => config_str,
        Err(_) => {
            // Create the config file if it doesn't exist
            let parent_dir = std::path::Path::new(file_path).parent().unwrap();
            fs::create_dir_all(parent_dir)?;
            fs::write(file_path, DEFAULT_CONFIG)?;
            DEFAULT_CONFIG.to_string()
        }
    };
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}
