use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn key(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        };

        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    pub applications_dir: Option<PathBuf>,
    pub desktop_dir: Option<PathBuf>,
    pub show_all: bool,
    pub theme: ThemeMode,
}

impl Settings {
    pub fn load() -> Self {
        fs::read_to_string(config_file())
            .map(|contents| parse(&contents))
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let file = config_file();

        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(file, self.render())
    }

    pub fn applications_dir(&self) -> PathBuf {
        self.applications_dir
            .clone()
            .unwrap_or_else(crate::desktop::applications_dir)
    }

    pub fn desktop_dir(&self) -> PathBuf {
        self.desktop_dir
            .clone()
            .unwrap_or_else(crate::desktop::desktop_dir)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();

        if let Some(dir) = &self.applications_dir {
            out.push_str(&format!("applications_dir={}\n", dir.display()));
        }

        if let Some(dir) = &self.desktop_dir {
            out.push_str(&format!("desktop_dir={}\n", dir.display()));
        }

        out.push_str(&format!("show_all={}\n", self.show_all));
        out.push_str(&format!("theme={}\n", self.theme.key()));

        out
    }
}

fn parse(contents: &str) -> Settings {
    let mut settings = Settings::default();

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let value = value.trim();

        match key.trim() {
            "applications_dir" if !value.is_empty() => {
                settings.applications_dir = Some(PathBuf::from(value));
            }
            "desktop_dir" if !value.is_empty() => {
                settings.desktop_dir = Some(PathBuf::from(value));
            }
            "show_all" => {
                settings.show_all = value.eq_ignore_ascii_case("true");
            }
            "theme" => {
                settings.theme = ThemeMode::parse(value);
            }
            _ => {}
        }
    }

    settings
}

fn config_file() -> PathBuf {
    config_dir().join("config")
}

pub fn config_path() -> PathBuf {
    config_file()
}

fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("desktools")
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_and_parses_roundtrip() {
        let settings = Settings {
            applications_dir: Some(PathBuf::from("/tmp/apps")),
            desktop_dir: Some(PathBuf::from("/tmp/desk")),
            show_all: true,
            theme: ThemeMode::Dark,
        };

        let parsed = parse(&settings.render());

        assert_eq!(parsed, settings);
    }

    #[test]
    fn defaults_when_empty() {
        let settings = parse("");

        assert_eq!(settings.applications_dir, None);
        assert_eq!(settings.desktop_dir, None);
        assert!(!settings.show_all);
        assert_eq!(settings.theme, ThemeMode::System);
    }

    #[test]
    fn parses_theme_values() {
        assert_eq!(parse("theme=light\n").theme, ThemeMode::Light);
        assert_eq!(parse("theme=DARK\n").theme, ThemeMode::Dark);
        assert_eq!(parse("theme=weird\n").theme, ThemeMode::System);
    }
}
