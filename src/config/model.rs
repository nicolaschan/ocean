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
    pub size: f32,
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub transparency: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
pub struct DefaultsConfig {
    pub shell: Option<String>,
}
