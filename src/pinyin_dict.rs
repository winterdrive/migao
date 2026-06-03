use crate::viterbi;
use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL: OnceLock<PinyinDict> = OnceLock::new();

pub struct PinyinDict {
    pub entries: HashMap<String, Vec<(String, u32)>>,
}

impl PinyinDict {
    fn load() -> Self {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        for line in include_str!("../data/pinyin.tsv").lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() != 3 {
                continue;
            }
            let freq: u32 = parts[2].trim().parse().unwrap_or(1);
            entries
                .entry(parts[0].to_string())
                .or_default()
                .push((parts[1].to_string(), freq));
        }
        PinyinDict { entries }
    }
}

pub fn global() -> &'static PinyinDict {
    GLOBAL.get_or_init(PinyinDict::load)
}

/// Viterbi decoder: convert tone-less pinyin syllables to Chinese text.
pub fn to_chinese(syllables: &[String]) -> String {
    viterbi::decode(syllables, &global().entries)
}
