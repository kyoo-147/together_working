use core::settings::UiSettings;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn settings_path(root: &Path) -> PathBuf {
    root.join(".together").join("settings.json")
}

pub fn load_settings(root: &Path) -> UiSettings {
    let path = settings_path(root);
    let Ok(contents) = fs::read_to_string(path) else {
        return UiSettings::default();
    };
    serde_json::from_str::<UiSettings>(&contents)
        .map(UiSettings::sanitized)
        .unwrap_or_default()
}

pub fn save_settings(root: &Path, settings: &UiSettings) -> io::Result<UiSettings> {
    let settings = settings.clone().sanitized();
    let path = settings_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(&settings)?;
    fs::write(path, contents)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_settings_falls_back_to_default() {
        let dir = std::env::temp_dir().join(format!("together-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(load_settings(&dir), UiSettings::default());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_persist_and_reload() {
        let dir =
            std::env::temp_dir().join(format!("together-settings-{}-persist", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let settings = UiSettings {
            theme_preset: "Ocean Blue".to_string(),
            custom_bg: Some("#08141C".to_string()),
            custom_main: Some("#3D9CFF".to_string()),
        };

        save_settings(&dir, &settings).unwrap();

        assert_eq!(load_settings(&dir), settings);

        let _ = fs::remove_dir_all(&dir);
    }
}
