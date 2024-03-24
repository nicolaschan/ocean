use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfig {
    pub font: FontConfig,
    pub window: WindowConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f64,
}

#[derive(Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub transparency: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize)]
pub struct DefaultsConfig {
    pub shell: Option<String>,
}
