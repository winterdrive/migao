use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL: OnceLock<BigramTable> = OnceLock::new();

struct BigramTable {
    counts: HashMap<(char, char), u32>,
}

impl BigramTable {
    fn load() -> Self {
        let mut counts = HashMap::new();
        for line in include_str!("../data/bigram.tsv").lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(3, '\t');
            let a = match parts.next().and_then(|s| s.chars().next()) {
                Some(c) => c,
                None => continue,
            };
            let b = match parts.next().and_then(|s| s.chars().next()) {
                Some(c) => c,
                None => continue,
            };
            let count: u32 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(1);
            counts.insert((a, b), count);
        }
        BigramTable { counts }
    }
}

fn global() -> &'static BigramTable {
    GLOBAL.get_or_init(BigramTable::load)
}

/// Returns ln(count+1) for a seen character bigram (a → b), or 0.0 for unseen pairs.
/// The caller applies a weight factor to control the bigram's influence on Viterbi scoring.
pub fn log_count(a: char, b: char) -> f32 {
    let count = global().counts.get(&(a, b)).copied().unwrap_or(0);
    if count == 0 {
        return 0.0;
    }
    (count as f32 + 1.0).ln()
}

/// True for CJK Unified Ideographs (excludes Bopomofo, ASCII, etc.).
pub fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // Extension A
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
    )
}
