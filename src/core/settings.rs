use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use log::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub auto_save_interval: u32, // in minutes
    pub performance: PerformanceSettings,
    pub save: SaveSettings,
    pub display: DisplaySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub memory_usage_percent: u32,
    pub undo_levels: u32,
    pub use_gpu: bool,
    pub thread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSettings {
    pub default_format: String,
    pub jpeg_quality: u32,
    pub png_compression: u32,
    pub include_metadata: bool,
    pub default_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub color_space: String,
    pub color_depth: u8,
    pub use_checkerboard: bool,
    pub checkerboard_color1: String,
    pub checkerboard_color2: String,
}

impl Default for Settings {
    fn default() -> Self {
        info!("Creating default settings");
        Self {
            theme: "system".to_string(),
            language: "en-US".to_string(),
            auto_save: true,
            auto_save_interval: 10,
            performance: PerformanceSettings::default(),
            save: SaveSettings::default(),
            display: DisplaySettings::default(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            memory_usage_percent: 50,
            undo_levels: 30,
            use_gpu: true,
            thread_count: num_cpus::get() as u32,
        }
    }
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            default_format: "png".to_string(),
            jpeg_quality: 85,
            png_compression: 6,
            include_metadata: true,
            default_location: dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            color_space: "srgb".to_string(),
            color_depth: 8,
            use_checkerboard: true,
            checkerboard_color1: "#CCCCCC".to_string(),
            checkerboard_color2: "#999999".to_string(),
        }
    }
}

impl Settings {
    /// Load settings from the configuration file
    pub fn load() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            info!("Settings file not found, creating default settings");
            let settings = Self::default();
            settings.save()?;
            return Ok(settings);
        }
        
        info!("Loading settings from {}", config_path.display());
        let mut file = File::open(&config_path)
            .map_err(|e| format!("Failed to open settings file: {}", e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings file: {}", e))
    }
    
    /// Save settings to the configuration file
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path()?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
        
        info!("Saving settings to {}", config_path.display());
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        let mut file = File::create(&config_path)
            .map_err(|e| format!("Failed to create settings file: {}", e))?;
        
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        Ok(())
    }
    
    /// Get the path to the configuration file
    fn get_config_path() -> Result<PathBuf, String> {
        let proj_dirs = ProjectDirs::from("com", "example", "rust_photo")
            .ok_or_else(|| "Failed to determine project directories".to_string())?;
        
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("settings.json");
        
        Ok(config_path)
    }
}

/// Settings manager for the application
#[derive(Debug, Clone)]
pub struct SettingsManager {
    pub settings: Settings,
}

impl SettingsManager {
    /// Create a new settings manager with default settings
    pub fn new() -> Self {
        info!("Creating settings manager with default settings");
        Self {
            settings: Settings::default(),
        }
    }
    
    /// Load settings from the configuration file
    pub fn load() -> Result<Self, String> {
        info!("Loading settings manager");
        let settings = Settings::load()?;
        Ok(Self {
            settings,
        })
    }
    
    /// Save current settings to the configuration file
    pub fn save(&self) -> Result<(), String> {
        info!("Saving settings from manager");
        self.settings.save()
    }
    
    /// Get a reference to the settings
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }
    
    /// Get a mutable reference to the settings
    pub fn get_settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
} 