use core::settings::UiSettings;
use ratatui::style::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePreset {
    pub name: &'static str,
    pub bg: &'static str,
    pub main: &'static str,
    pub dark: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub ready: Color,
    pub warn: Color,
    pub danger: Color,
    pub dark: bool,
}

pub fn presets() -> Vec<ThemePreset> {
    vec![
        ThemePreset {
            name: "Together Classic",
            bg: "#E0E5F2",
            main: "#3667D6",
            dark: false,
        },
        ThemePreset {
            name: "Green Terminal",
            bg: "#F3F6F4",
            main: "#237A3B",
            dark: false,
        },
        ThemePreset {
            name: "Violet Terminal",
            bg: "#F5F4F8",
            main: "#7541B5",
            dark: false,
        },
        ThemePreset {
            name: "Amber Terminal",
            bg: "#F7F6F2",
            main: "#D99600",
            dark: false,
        },
        ThemePreset {
            name: "Orange Terminal",
            bg: "#F7F5F2",
            main: "#E85216",
            dark: false,
        },
        ThemePreset {
            name: "Ocean Blue",
            bg: "#08141C",
            main: "#3D9CFF",
            dark: true,
        },
        ThemePreset {
            name: "Deep Sea Teal",
            bg: "#071817",
            main: "#18B7A5",
            dark: true,
        },
        ThemePreset {
            name: "Forest Green",
            bg: "#101A13",
            main: "#78A94B",
            dark: true,
        },
        ThemePreset {
            name: "Neon Purple",
            bg: "#12101A",
            main: "#A56BFF",
            dark: true,
        },
        ThemePreset {
            name: "Amber Yellow",
            bg: "#19160D",
            main: "#E3A600",
            dark: true,
        },
        ThemePreset {
            name: "Warm Orange",
            bg: "#19120D",
            main: "#E96A19",
            dark: true,
        },
        ThemePreset {
            name: "Hot Pink / Magenta",
            bg: "#181017",
            main: "#EC4F9A",
            dark: true,
        },
        ThemePreset {
            name: "Cyan",
            bg: "#071719",
            main: "#35C9CC",
            dark: true,
        },
        ThemePreset {
            name: "Lime",
            bg: "#13190A",
            main: "#A1D119",
            dark: true,
        },
        ThemePreset {
            name: "Red",
            bg: "#1A1010",
            main: "#E64A3C",
            dark: true,
        },
        ThemePreset {
            name: "Rose Gold",
            bg: "#191313",
            main: "#D88978",
            dark: true,
        },
        ThemePreset {
            name: "Slate Gray",
            bg: "#15191C",
            main: "#8A949C",
            dark: true,
        },
        ThemePreset {
            name: "Electric Blue",
            bg: "#0C1220",
            main: "#527BFF",
            dark: true,
        },
    ]
}

pub fn theme_from_settings(settings: &UiSettings) -> Theme {
    let preset = presets()
        .into_iter()
        .find(|preset| preset.name == settings.theme_preset)
        .unwrap_or(ThemePreset {
            name: "Together Classic",
            bg: "#E0E5F2",
            main: "#3667D6",
            dark: false,
        });
    let bg_hex = settings.custom_bg.as_deref().unwrap_or(preset.bg);
    let main_hex = settings.custom_main.as_deref().unwrap_or(preset.main);
    derive_theme(preset.name, bg_hex, main_hex, preset.dark)
}

fn derive_theme(name: &str, bg_hex: &str, main_hex: &str, dark: bool) -> Theme {
    let bg = parse_hex(bg_hex).unwrap_or((224, 229, 242));
    let main = parse_hex(main_hex).unwrap_or((54, 103, 214));
    let panel = if dark {
        mix(bg, (255, 255, 255), 8)
    } else {
        mix(bg, (255, 255, 255), 55)
    };
    let panel_alt = if dark {
        mix(bg, (255, 255, 255), 14)
    } else {
        mix(bg, (75, 95, 145), 10)
    };
    let border = if dark {
        mix(main, (255, 255, 255), 35)
    } else {
        mix(main, (255, 255, 255), 52)
    };
    let text = if dark { (224, 232, 242) } else { (48, 57, 83) };
    let muted = if dark {
        (142, 153, 176)
    } else {
        (101, 113, 150)
    };

    Theme {
        name: name.to_string(),
        bg: rgb(bg),
        panel: rgb(panel),
        panel_alt: rgb(panel_alt),
        border: rgb(border),
        text: rgb(text),
        muted: rgb(muted),
        accent: rgb(main),
        ready: if dark {
            rgb((99, 202, 128))
        } else {
            rgb((55, 121, 82))
        },
        warn: if dark {
            rgb((230, 178, 70))
        } else {
            rgb((191, 105, 48))
        },
        danger: if dark {
            rgb((235, 91, 102))
        } else {
            rgb((191, 62, 75))
        },
        dark,
    }
}

pub fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), pct_b: u8) -> (u8, u8, u8) {
    let pct_b = pct_b as u16;
    let pct_a = 100 - pct_b;
    (
        ((a.0 as u16 * pct_a + b.0 as u16 * pct_b) / 100) as u8,
        ((a.1 as u16 * pct_a + b.1 as u16 * pct_b) / 100) as u8,
        ((a.2 as u16 * pct_a + b.2 as u16 * pct_b) / 100) as u8,
    )
}

fn rgb(value: (u8, u8, u8)) -> Color {
    Color::Rgb(value.0, value.1, value.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_color() {
        assert_eq!(parse_hex("#237A3B"), Some((35, 122, 59)));
        assert_eq!(parse_hex("237A3B"), None);
        assert_eq!(parse_hex("#XYZXYZ"), None);
    }

    #[test]
    fn includes_required_presets() {
        let names = presets().into_iter().map(|p| p.name).collect::<Vec<_>>();

        assert!(names.contains(&"Together Classic"));
        assert!(names.contains(&"Green Terminal"));
        assert!(names.contains(&"Electric Blue"));
        assert_eq!(names.len(), 18);
    }

    #[test]
    fn derives_light_and_dark_readable_themes() {
        let light = theme_from_settings(&UiSettings::default());
        let dark = theme_from_settings(&UiSettings {
            theme_preset: "Ocean Blue".to_string(),
            custom_bg: None,
            custom_main: None,
        });

        assert!(!light.dark);
        assert!(dark.dark);
        assert_ne!(light.bg, light.text);
        assert_ne!(dark.bg, dark.text);
    }
}
