use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const APP_NAME: &str = "zupo";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub location: LocationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationConfig {
    pub default_lat: Option<f64>,
    pub default_lng: Option<f64>,
    pub default_radius: Option<f64>,
    pub label: Option<String>,
}

impl Config {
    /// Load config from ~/.config/zupo/config.toml (returns default if missing)
    pub fn load() -> Self {
        let path = match config_path() {
            Some(p) => p,
            None => return Config::default(),
        };

        if !path.exists() {
            return Config::default();
        }

        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Config::default(),
        };

        toml::from_str(&contents).unwrap_or_default()
    }

    /// Save config to ~/.config/zupo/config.toml
    pub fn save(&self) -> Result<(), String> {
        let path = config_path().ok_or("could not determine config directory")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("could not create config directory: {}", e))?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|e| format!("could not serialize config: {}", e))?;

        fs::write(&path, contents).map_err(|e| format!("could not write config file: {}", e))?;

        Ok(())
    }

    /// Set default location
    pub fn set_location(
        &mut self,
        lat: f64,
        lng: f64,
        radius: Option<f64>,
        label: Option<String>,
    ) {
        self.location.default_lat = Some(lat);
        self.location.default_lng = Some(lng);
        self.location.default_radius = radius;
        self.location.label = label;
    }

    /// Clear default location
    pub fn clear_location(&mut self) {
        self.location = LocationConfig::default();
    }

    /// Get default location if set
    pub fn default_location(&self) -> Option<(f64, f64)> {
        match (self.location.default_lat, self.location.default_lng) {
            (Some(lat), Some(lng)) => Some((lat, lng)),
            _ => None,
        }
    }

    /// Get default radius (or fallback)
    pub fn default_radius(&self) -> f64 {
        self.location.default_radius.unwrap_or(1000.0)
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(APP_NAME).join("config.toml"))
}

pub fn config_file_path() -> String {
    config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/zupo/config.toml".to_string())
}
