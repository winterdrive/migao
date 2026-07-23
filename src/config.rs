use std::fmt;
use std::path::PathBuf;

pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+R";
pub const DEFAULT_IME: &str = "bopomofo-daqian";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub hotkey: String,
    pub default_ime: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY.to_string(),
            default_ime: DEFAULT_IME.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hotkey {
    pub label: String,
    pub key_code: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfigError(String);

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

pub fn config_dir() -> Option<PathBuf> {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;
    Some(base.join("Migao"))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Config::default();
    };

    let mut cfg = Config::default();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "hotkey" => cfg.hotkey = value,
            "default_ime" => cfg.default_ime = value,
            _ => {}
        }
    }

    cfg
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let path = config_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot determine config directory",
        )
    })?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        path,
        format!(
            "hotkey = \"{}\"\ndefault_ime = \"{}\"\n",
            cfg.hotkey, cfg.default_ime
        ),
    )
}

pub fn parse_hotkey(input: &str) -> Result<Hotkey, ConfigError> {
    let parts: Vec<String> = input
        .split('+')
        .map(|part| part.trim().to_ascii_uppercase())
        .filter(|part| !part.is_empty())
        .collect();

    if parts.len() != 3 || parts[0] != "CTRL" || parts[1] != "ALT" {
        return Err(ConfigError::new(
            "hotkey must use the form Ctrl+Alt+<A-Z>, for example Ctrl+Alt+R",
        ));
    }

    let key = &parts[2];
    if key.len() != 1 {
        return Err(ConfigError::new("hotkey key must be a single A-Z letter"));
    }
    let ch = key.chars().next().unwrap();
    if !ch.is_ascii_alphabetic() {
        return Err(ConfigError::new("hotkey key must be a single A-Z letter"));
    }

    Ok(Hotkey {
        label: format!("Ctrl+Alt+{}", ch.to_ascii_uppercase()),
        key_code: ch.to_ascii_uppercase() as u32,
    })
}

pub fn normalized_hotkey_or_default(input: &str) -> Hotkey {
    parse_hotkey(input).unwrap_or_else(|_| parse_hotkey(DEFAULT_HOTKEY).unwrap())
}

#[cfg(windows)]
mod autostart {
    use std::path::PathBuf;

    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    const REG_RUN: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    const REG_KEY: &str = "MigaoWatch";

    pub fn is_enabled() -> bool {
        let Ok(run) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(REG_RUN, KEY_READ)
        else {
            return false;
        };
        run.get_raw_value(REG_KEY).is_ok()
    }

    pub fn set_enabled(enable: bool) -> std::io::Result<()> {
        let run =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(REG_RUN, KEY_SET_VALUE)?;
        if enable {
            let path = watch_exe_path()?;
            run.set_value(REG_KEY, &format!("\"{}\"", path.display()))?;
        } else {
            let _ = run.delete_value(REG_KEY);
        }
        Ok(())
    }

    fn watch_exe_path() -> std::io::Result<PathBuf> {
        let exe = std::env::current_exe()?;
        if exe
            .file_stem()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("migao-watch"))
        {
            return Ok(exe);
        }
        Ok(exe.with_file_name("migao-watch.exe"))
    }
}

#[cfg(not(windows))]
mod autostart {
    pub fn is_enabled() -> bool {
        false
    }

    pub fn set_enabled(_enable: bool) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "migao-watch autostart is currently Windows-only",
        ))
    }
}

pub fn is_autostart_enabled() -> bool {
    autostart::is_enabled()
}

pub fn set_autostart_enabled(enable: bool) -> std::io::Result<()> {
    autostart::set_enabled(enable)
}

#[cfg(test)]
mod tests {
    use super::{parse_hotkey, DEFAULT_HOTKEY};

    #[test]
    fn parses_ctrl_alt_letter_hotkey() {
        let hotkey = parse_hotkey("ctrl+alt+k").unwrap();
        assert_eq!(hotkey.label, "Ctrl+Alt+K");
        assert_eq!(hotkey.key_code, 'K' as u32);
    }

    #[test]
    fn rejects_unsupported_hotkey_shape() {
        assert!(parse_hotkey("Ctrl+Shift+R").is_err());
        assert!(parse_hotkey("Ctrl+Alt+F12").is_err());
        assert!(parse_hotkey(DEFAULT_HOTKEY).is_ok());
    }
}
