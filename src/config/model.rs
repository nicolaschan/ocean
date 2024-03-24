use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub font: FontConfig,
    pub window: WindowConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f64,
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub transparency: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize)]
pub struct DefaultsConfig {
    pub shell: Option<String>,
}
