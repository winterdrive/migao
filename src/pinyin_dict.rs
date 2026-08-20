use crate::viterbi;
use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL: OnceLock<PinyinDict> = OnceLock::new();

pub struct PinyinDict {
    pub entries: HashMap<String, Vec<(String, u32)>>,
}

fn parse_tsv_into(src: &str, entries: &mut HashMap<String, Vec<(String, u32)>>) {
    for line in src.lines() {
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
}

impl PinyinDict {
    fn load() -> Self {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        parse_tsv_into(include_str!("../data/pinyin.tsv"), &mut entries);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skips_blank_and_comment_lines() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        parse_tsv_into("\n# a comment\nni\n\ni\tyou\t500\n", &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["i"], vec![("you".to_string(), 500)]);
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        // Fewer than 3 tab-separated fields is malformed and must be dropped.
        parse_tsv_into("hao\thao\nma\tma\t100", &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["ma"], vec![("ma".to_string(), 100)]);
    }

    #[test]
    fn parse_defaults_unparseable_frequency_to_one() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        parse_tsv_into("ta\tta\tnot-a-number", &mut entries);
        assert_eq!(entries["ta"], vec![("ta".to_string(), 1)]);
    }

    #[test]
    fn parse_accumulates_multiple_entries_under_same_key() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        parse_tsv_into("shi\t是\t900\nshi\t事\t300\n", &mut entries);
        assert_eq!(
            entries["shi"],
            vec![("是".to_string(), 900), ("事".to_string(), 300)]
        );
    }

    #[test]
    fn global_loads_nonempty_dict_from_embedded_tsv() {
        assert!(!global().entries.is_empty());
    }
}
