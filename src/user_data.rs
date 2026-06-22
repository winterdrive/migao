use std::io::Write;
use std::path::PathBuf;

/// Returns the path to the user-specific supplement file:
///   Windows: %APPDATA%\Migao\user_supplement.tsv
///   Other:   $HOME/.config/migao/user_supplement.tsv
pub fn user_supplement_path() -> Option<PathBuf> {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;
    Some(base.join("Migao").join("user_supplement.tsv"))
}

/// Reads the user supplement file, returning an empty string if it doesn't exist.
pub fn load() -> String {
    user_supplement_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// Appends a single correction entry to the user supplement file.
///
/// `bopomofo_key` is the concatenated Bopomofo syllables (e.g. "ㄅㄢˇㄅㄣˇ").
/// `word` is the correct Chinese text (e.g. "版本").
/// The entry is written with a high frequency so it beats the default dictionary.
pub fn append_entry(bopomofo_key: &str, word: &str) -> std::io::Result<()> {
    let path = user_supplement_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot determine user data directory",
        )
    })?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{}\t{}\t100000", bopomofo_key, word)?;
    Ok(())
}
