use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_backend() -> String {
    "whisper".to_string()
}
fn default_device_pattern() -> String {
    "DJI".to_string()
}
fn default_sample_rate() -> u32 {
    16000
}
fn default_chunk_duration() -> f64 {
    0.1
}
fn default_silence_threshold() -> f64 {
    0.015
}
fn default_silence_duration() -> f64 {
    1.5
}
fn default_max_utterance_duration() -> f64 {
    30.0
}
fn default_whisper_model() -> String {
    "small".to_string()
}
fn default_whisper_device() -> String {
    "cpu".to_string()
}
fn default_whisper_compute_type() -> String {
    "int8".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_backend")]
    pub backend: String,

    #[serde(default = "default_device_pattern")]
    pub device_pattern: String,

    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_chunk_duration")]
    pub chunk_duration: f64,

    #[serde(default = "default_silence_threshold")]
    pub silence_threshold: f64,

    #[serde(default = "default_silence_duration")]
    pub silence_duration: f64,

    #[serde(default = "default_max_utterance_duration")]
    pub max_utterance_duration: f64,

    #[serde(default)]
    pub output_file: Option<String>,

    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,

    #[serde(default)]
    pub whisper_language: Option<String>,

    #[serde(default = "default_whisper_device")]
    pub whisper_device: String,

    #[serde(default = "default_whisper_compute_type")]
    pub whisper_compute_type: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            device_pattern: default_device_pattern(),
            sample_rate: default_sample_rate(),
            chunk_duration: default_chunk_duration(),
            silence_threshold: default_silence_threshold(),
            silence_duration: default_silence_duration(),
            max_utterance_duration: default_max_utterance_duration(),
            output_file: None,
            whisper_model: default_whisper_model(),
            whisper_language: None,
            whisper_device: default_whisper_device(),
            whisper_compute_type: default_whisper_compute_type(),
        }
    }
}

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

pub fn load_config() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            log::warn!("Failed to parse config.json: {e}. Using defaults.");
            Config::default()
        }),
        Err(_) => {
            log::info!("No config.json found at {:?}. Using defaults.", path);
            Config::default()
        }
    }
}

pub fn save_config(cfg: &Config) {
    let path = config_path();
    match serde_json::to_string_pretty(cfg) {
        Ok(contents) => {
            if let Err(e) = std::fs::write(&path, contents) {
                log::error!("Failed to write config.json: {e}");
            }
        }
        Err(e) => log::error!("Failed to serialize config: {e}"),
    }
}
