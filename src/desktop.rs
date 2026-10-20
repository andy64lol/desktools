use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const COMMON_CATEGORIES: [&str; 16] = [
    "AudioVideo",
    "Development",
    "Education",
    "Game",
    "Graphics",
    "Network",
    "Office",
    "Science",
    "Settings",
    "System",
    "Utility",
    "TerminalEmulator",
    "WebBrowser",
    "TextEditor",
    "FileManager",
    "Player",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopEntry {
    pub path: Option<PathBuf>,
    pub name: String,
    pub generic_name: String,
    pub comment: String,
    pub exec: String,
    pub icon: String,
    pub working_dir: String,
    pub categories: String,
    pub keywords: String,
    pub startup_wm_class: String,
    pub terminal: bool,
    pub no_display: bool,
    pub startup_notify: bool,
    pub desktools: bool,
}

impl DesktopEntry {
    pub fn parse(path: &Path) -> io::Result<Self> {
        let contents = fs::read_to_string(path)?;

        let mut entry = DesktopEntry {
            path: Some(path.to_path_buf()),
            ..DesktopEntry::default()
        };

        let mut in_main_section = false;

        for raw in contents.lines() {
            let line = raw.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                in_main_section = line == "[Desktop Entry]";
                continue;
            }

            if !in_main_section {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => entry.name = unescape(value),
                "GenericName" => entry.generic_name = unescape(value),
                "Comment" => entry.comment = unescape(value),
                "Exec" => entry.exec = unescape(value),
                "Icon" => entry.icon = unescape(value),
                "Path" => entry.working_dir = unescape(value),
                "Categories" => entry.categories = value.to_string(),
                "Keywords" => entry.keywords = value.to_string(),
                "StartupWMClass" => entry.startup_wm_class = value.to_string(),
                "Terminal" => entry.terminal = value.eq_ignore_ascii_case("true"),
                "NoDisplay" => entry.no_display = value.eq_ignore_ascii_case("true"),
                "StartupNotify" => entry.startup_notify = value.eq_ignore_ascii_case("true"),
                "X-Desktools" => entry.desktools = value.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }

        Ok(entry)
    }

    pub fn render(&self) -> String {
        let mut out = String::from("[Desktop Entry]\n");
        out.push_str("Version=1.0\n");
        out.push_str("Type=Application\n");

        push_key(&mut out, "Name", &self.name);
        push_key(&mut out, "GenericName", &self.generic_name);
        push_key(&mut out, "Comment", &self.comment);
        push_key(&mut out, "Exec", &self.exec);
        push_key(&mut out, "Icon", &self.icon);
        push_key(&mut out, "Path", &self.working_dir);
        push_key(&mut out, "Categories", &self.categories);
        push_key(&mut out, "Keywords", &self.keywords);
        push_key(&mut out, "StartupWMClass", &self.startup_wm_class);

        out.push_str("X-Desktools=true\n");

        if self.terminal {
            out.push_str("Terminal=true\n");
        }

        if self.no_display {
            out.push_str("NoDisplay=true\n");
        }

        if self.startup_notify {
            out.push_str("StartupNotify=true\n");
        }

        out
    }

    pub fn file_name(&self) -> String {
        let slug = slugify(&self.name);

        if slug.is_empty() {
            "untitled.desktop".to_string()
        } else {
            format!("{slug}.desktop")
        }
    }

    pub fn save(&mut self, directory: &Path) -> io::Result<PathBuf> {
        fs::create_dir_all(directory)?;

        let target = match &self.path {
            Some(path) => path.clone(),
            None => {
                let base = self.file_name();
                let stem = base.strip_suffix(".desktop").unwrap_or(&base);
                let mut candidate = directory.join(&base);
                let mut counter = 2;

                while candidate.exists() {
                    candidate = directory.join(format!("{stem}-{counter}.desktop"));
                    counter += 1;
                }

                candidate
            }
        };

        fs::write(&target, self.render())?;
        self.path = Some(target.clone());
        self.desktools = true;

        Ok(target)
    }
}

pub fn applications_dir() -> PathBuf {
    data_home().join("applications")
}

pub fn desktop_dir() -> PathBuf {
    if let Some(dir) = xdg_user_dir("XDG_DESKTOP_DIR") {
        return dir;
    }

    home_dir().join("Desktop")
}

pub fn list_entries(directory: &Path, show_all: bool) -> Vec<DesktopEntry> {
    let Ok(read_dir) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut entries: Vec<DesktopEntry> = read_dir
        .filter_map(Result::ok)
        .map(|item| item.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "desktop")
        })
        .filter_map(|path| DesktopEntry::parse(&path).ok())
        .filter(|entry| show_all || entry.desktools)
        .collect();

    entries.sort_by_key(|entry| entry.name.to_lowercase());

    entries
}

fn push_key(out: &mut String, key: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }

    out.push_str(key);
    out.push('=');
    out.push_str(&escape(value));
    out.push('\n');
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
}

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('s') => out.push(' '),
            Some('\\') => out.push('\\'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    out
}

fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_separator = true;

    for c in name.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('-');
            last_was_separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn data_home() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home_dir().join(".local").join("share"))
}

fn xdg_user_dir(key: &str) -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home_dir().join(".config"));

    let contents = fs::read_to_string(config.join("user-dirs.dirs")).ok()?;

    for line in contents.lines() {
        let line = line.trim();

        if !line.starts_with(key) {
            continue;
        }

        let (_, value) = line.split_once('=')?;
        let value = value.trim().trim_matches('"');
        let expanded = value.replace("$HOME", &home_dir().to_string_lossy());

        return Some(PathBuf::from(expanded));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_desktop_file() {
        let dir = std::env::temp_dir().join("desktools-parse-test");

        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("sample.desktop");

        fs::write(
            &path,
            "[Desktop Entry]\nType=Application\nName=Sample\nExec=sample --flag\nIcon=sample\nTerminal=true\nNoDisplay=false\nCategories=Utility;System;\nX-Desktools=true\n",
        )
        .unwrap();

        let entry = DesktopEntry::parse(&path).unwrap();

        assert_eq!(entry.name, "Sample");
        assert_eq!(entry.exec, "sample --flag");
        assert_eq!(entry.categories, "Utility;System;");
        assert!(entry.terminal);
        assert!(!entry.no_display);
        assert!(entry.desktools);
    }

    #[test]
    fn only_lists_launchers_made_by_desktools() {
        let dir = std::env::temp_dir().join("desktools-list-test");

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("pre-existing.desktop"),
            "[Desktop Entry]\nName=Firefox\n",
        )
        .unwrap();
        fs::write(
            dir.join("mine.desktop"),
            "[Desktop Entry]\nName=Mine\nX-Desktools=true\n",
        )
        .unwrap();

        let entries = list_entries(&dir, false);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Mine");

        let all = list_entries(&dir, true);

        assert_eq!(all.len(), 2);
    }

    #[test]
    fn renders_and_roundtrips() {
        let entry = DesktopEntry {
            name: "My App".to_string(),
            exec: "my-app %u".to_string(),
            icon: "my-app".to_string(),
            categories: "Utility;".to_string(),
            terminal: true,
            ..DesktopEntry::default()
        };

        let rendered = entry.render();

        assert!(rendered.contains("Name=My App\n"));
        assert!(rendered.contains("Terminal=true\n"));
        assert!(rendered.contains("X-Desktools=true\n"));
        assert!(!rendered.contains("NoDisplay"));
    }

    #[test]
    fn slugs_names() {
        let mut entry = DesktopEntry::default();

        entry.name = "  Hello, World!  ".to_string();

        assert_eq!(entry.file_name(), "hello-world.desktop");
    }
}
