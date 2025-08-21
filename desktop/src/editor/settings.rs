use iced::Color;
use serde::{Deserialize, Serialize};
use std::fmt;

mod serde_color {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [color.r, color.g, color.b].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [r, g, b] = <[f32; 3]>::deserialize(deserializer)?;
        Ok(Color::from_rgb(r, g, b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorTheme {
    Light,
    Dark,
    HighContrast,
    Custom,
}

impl EditorTheme {
    pub const ALL: [EditorTheme; 4] = [
        EditorTheme::Light,
        EditorTheme::Dark,
        EditorTheme::HighContrast,
        EditorTheme::Custom,
    ];
}

impl Default for EditorTheme {
    fn default() -> Self {
        EditorTheme::Light
    }
}

impl fmt::Display for EditorTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorTheme::Light => write!(f, "Light"),
            EditorTheme::Dark => write!(f, "Dark"),
            EditorTheme::HighContrast => write!(f, "High Contrast"),
            EditorTheme::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    #[serde(with = "serde_color")]
    pub background: Color,
    #[serde(with = "serde_color")]
    pub foreground: Color,
    #[serde(with = "serde_color")]
    pub current_line: Color,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            background: Color::from_rgb(1.0, 1.0, 1.0),
            foreground: Color::from_rgb(0.0, 0.0, 0.0),
            current_line: Color::from_rgb(0.9, 0.9, 0.9),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default = "default_tab_width")]
    pub tab_width: u8,
    #[serde(default = "default_true")]
    pub auto_indent: bool,
    #[serde(default)]
    pub line_wrapping: bool,
    #[serde(default = "default_true")]
    pub highlight_current_line: bool,
    #[serde(default)]
    pub theme: EditorTheme,
    #[serde(default)]
    pub custom_theme: CustomTheme,
}

fn default_font_size() -> u16 {
    14
}

fn default_tab_width() -> u8 {
    4
}

fn default_true() -> bool {
    true
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            tab_width: default_tab_width(),
            auto_indent: true,
            line_wrapping: false,
            highlight_current_line: true,
            theme: EditorTheme::Light,
            custom_theme: CustomTheme::default(),
        }
    }
}
