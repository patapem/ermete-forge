use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockConfig {
    pub pinned: Vec<String>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            pinned: vec![
                "org.gnome.Terminal.desktop".to_string(),
                "firefox.desktop".to_string(),
                "os.ermete.Settings.desktop".to_string(),
            ],
        }
    }
}

pub fn get_dock_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/ermete-shell/dock.json")
}

pub fn load_dock_config() -> DockConfig {
    let path = get_dock_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<DockConfig>(&content) {
            return config;
        }
    }
    let default_config = DockConfig::default();
    let _ = save_dock_config(&default_config);
    default_config
}

pub fn save_dock_config(config: &DockConfig) -> Result<(), String> {
    let path = get_dock_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Impossibile serializzare configurazione Dock: {}", e))?;
    fs::write(&path, json_str)
        .map_err(|e| format!("Impossibile scrivere file {}: {}", path.display(), e))?;
    Ok(())
}

pub fn add_pin(desktop_id: &str) -> Result<DockConfig, String> {
    let mut config = load_dock_config();
    if !config.pinned.contains(&desktop_id.to_string()) {
        config.pinned.push(desktop_id.to_string());
        save_dock_config(&config)?;
    }
    Ok(config)
}

pub fn remove_pin(desktop_id: &str) -> Result<DockConfig, String> {
    let mut config = load_dock_config();
    config.pinned.retain(|id| id != desktop_id);
    save_dock_config(&config)?;
    Ok(config)
}

pub fn is_pinned(desktop_id: &str) -> bool {
    let config = load_dock_config();
    config.pinned.contains(&desktop_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_pin_logic() {
        let mut config = DockConfig {
            pinned: vec!["app1.desktop".to_string()],
        };
        
        // Test add
        if !config.pinned.contains(&"app2.desktop".to_string()) {
            config.pinned.push("app2.desktop".to_string());
        }
        assert_eq!(config.pinned, vec!["app1.desktop", "app2.desktop"]);

        // Test add duplicate (should not add)
        if !config.pinned.contains(&"app2.desktop".to_string()) {
            config.pinned.push("app2.desktop".to_string());
        }
        assert_eq!(config.pinned.len(), 2);

        // Test remove
        config.pinned.retain(|id| id != "app1.desktop");
        assert_eq!(config.pinned, vec!["app2.desktop"]);
    }

    #[test]
    fn test_api_add_remove_and_is_pinned() {
        let tmp_dir = std::env::temp_dir().join("ermete_test_dock_config_api");
        let _ = fs::remove_dir_all(&tmp_dir);
        std::env::set_var("HOME", tmp_dir.to_str().unwrap());

        // Initial load should create default config
        let initial = load_dock_config();
        assert_eq!(initial, DockConfig::default());
        assert!(get_dock_config_path().exists(), "load_dock_config should save default if file didn't exist");

        // Test add_pin
        let added = add_pin("custom.app.desktop").expect("add_pin failed");
        assert!(added.pinned.contains(&"custom.app.desktop".to_string()));
        assert!(is_pinned("custom.app.desktop"));

        // Test adding duplicate
        let added_again = add_pin("custom.app.desktop").expect("add_pin duplicate failed");
        let count = added_again.pinned.iter().filter(|id| *id == "custom.app.desktop").count();
        assert_eq!(count, 1, "duplicate pin should not be added");

        // Test remove_pin
        let removed = remove_pin("custom.app.desktop").expect("remove_pin failed");
        assert!(!removed.pinned.contains(&"custom.app.desktop".to_string()));
        assert!(!is_pinned("custom.app.desktop"));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
