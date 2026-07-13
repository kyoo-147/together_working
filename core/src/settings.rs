use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiSettings {
    pub theme_preset: String,
    pub custom_bg: Option<String>,
    pub custom_main: Option<String>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme_preset: "Together Classic".to_string(),
            custom_bg: None,
            custom_main: None,
        }
    }
}

pub fn is_valid_hex_color(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

impl UiSettings {
    pub fn sanitized(self) -> Self {
        Self {
            theme_preset: if self.theme_preset.trim().is_empty() {
                "Together Classic".to_string()
            } else {
                self.theme_preset
            },
            custom_bg: self
                .custom_bg
                .filter(|value| is_valid_hex_color(value.trim())),
            custom_main: self
                .custom_main
                .filter(|value| is_valid_hex_color(value.trim())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_hex_colors() {
        assert!(is_valid_hex_color("#237A3B"));
        assert!(is_valid_hex_color("#08141c"));
        assert!(!is_valid_hex_color("237A3B"));
        assert!(!is_valid_hex_color("#XYZXYZ"));
        assert!(!is_valid_hex_color("#12345"));
    }

    #[test]
    fn sanitizes_invalid_custom_colors() {
        let settings = UiSettings {
            theme_preset: "".to_string(),
            custom_bg: Some("bad".to_string()),
            custom_main: Some("#18B7A5".to_string()),
        }
        .sanitized();

        assert_eq!(settings.theme_preset, "Together Classic");
        assert_eq!(settings.custom_bg, None);
        assert_eq!(settings.custom_main.as_deref(), Some("#18B7A5"));
    }
}
