use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::ProviderRuntimeService;

impl ProviderRuntimeService {
    pub(super) fn read_config(path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Ok(String::new());
        }
        fs::read_to_string(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))
    }

    pub(super) fn read_auth_preview(path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Ok(String::new());
        }
        let content = fs::read_to_string(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
        let Ok(mut value) = serde_json::from_str::<Value>(&content) else {
            return Ok("<existing auth.json hidden>\n".to_string());
        };
        if let Some(object) = value.as_object_mut() {
            if object.contains_key("OPENAI_API_KEY") {
                object.insert(
                    "OPENAI_API_KEY".to_string(),
                    Value::String("<existing credential>".to_string()),
                );
            }
        }
        serde_json::to_string_pretty(&value)
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }
    pub(super) fn auth_path_for_config(config_path: &Path) -> Result<PathBuf, String> {
        let Some(parent) = config_path.parent() else {
            return Err(format!(
                "Cannot resolve auth.json next to {}.",
                config_path.display()
            ));
        };
        Ok(parent.join("auth.json"))
    }
    pub(super) fn write_text(path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, content)
            .map_err(|error| format!("Failed to write {}: {error}", path.display()))
    }

    pub(super) fn build_diff(label: &str, before: &str, after: &str) -> String {
        format!("--- existing {label}\n{before}\n+++ generated {label}\n{after}")
    }
}
