use crate::viterbi;
use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL: OnceLock<ZhuyinDict> = OnceLock::new();

pub struct ZhuyinDict {
    entries: HashMap<String, Vec<(String, u32)>>,
}

impl ZhuyinDict {
    fn load() -> Self {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        for src in &[
            include_str!("../data/bopomofo.tsv"),
            include_str!("../data/supplement.tsv"),
        ] {
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
        ZhuyinDict { entries }
    }

    /// Returns the highest-frequency word for a Bopomofo key (single or multi-syllable).
    pub fn lookup(&self, zhuyin: &str) -> Option<&str> {
        self.lookup_with_freq(zhuyin).map(|(w, _)| w)
    }

    /// Like `lookup` but also returns the frequency, used for Viterbi scoring.
    pub fn lookup_with_freq(&self, zhuyin: &str) -> Option<(&str, u32)> {
        if let Some(hit) = viterbi::best_entry(&self.entries, zhuyin) {
            return Some(hit);
        }
        // Neutral tone (˙, key 7) fallback: pypinyin stores most neutral-tone chars
        // under their base tone (usually ˋ). e.g. 個 is stored as ㄍㄜˋ, not ㄍㄜ˙.
        if zhuyin.ends_with('˙') {
            let stem = &zhuyin[..zhuyin.len() - '˙'.len_utf8()];
            if let Some(hit) = viterbi::best_entry(&self.entries, &format!("{stem}ˋ")) {
                return Some(hit);
            }
            if let Some(hit) = viterbi::best_entry(&self.entries, stem) {
                return Some(hit);
            }
        }
        None
    }
}

pub fn global() -> &'static ZhuyinDict {
    GLOBAL.get_or_init(ZhuyinDict::load)
}

/// Viterbi decoder: convert decoded Bopomofo syllables to Chinese text.
///
/// See viterbi::decode for the scoring details. Constants (COMPOUND_BONUS,
/// BIGRAM_WEIGHT) are shared with the pinyin decoder; calibration notes in
/// viterbi.rs explain the valid range.
///
/// Neutral-tone (˙) syllables are normalised to their ˋ equivalents before
/// Viterbi lookup because pypinyin stores most neutral-tone chars under their
/// base tone (e.g. 個 → ㄍㄜˋ, not ㄍㄜ˙).
/// Like `to_chinese`, but returns up to `n` candidate Chinese strings.
/// The best Viterbi path is first; subsequent entries substitute alternative
/// words at each ambiguous span.
pub fn to_chinese_candidates(n: usize, syllables: &[String]) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let dict = global();
    let mut neutral_hints = vec![false; syllables.len()];
    let normalised: Vec<String> = syllables
        .iter()
        .enumerate()
        .map(|(idx, syl)| {
            if !syl.ends_with('˙') {
                return syl.clone();
            }
            neutral_hints[idx] = true;
            if viterbi::best_entry(&dict.entries, syl).is_some() {
                return syl.clone();
            }
            let stem = &syl[..syl.len() - '˙'.len_utf8()];
            let with_falling = format!("{stem}ˋ");
            if viterbi::best_entry(&dict.entries, &with_falling).is_some() {
                return with_falling;
            }
            if viterbi::best_entry(&dict.entries, stem).is_some() {
                return stem.to_string();
            }
            syl.clone()
        })
        .collect();
    viterbi::decode_candidates(n, &normalised, &neutral_hints, &dict.entries)
}

pub fn to_chinese(syllables: &[String]) -> String {
    let dict = global();
    let mut neutral_hints = vec![false; syllables.len()];
    let normalised: Vec<String> = syllables
        .iter()
        .enumerate()
        .map(|(idx, syl)| {
            if !syl.ends_with('˙') {
                return syl.clone();
            }
            neutral_hints[idx] = true;
            if viterbi::best_entry(&dict.entries, syl).is_some() {
                return syl.clone();
            }
            let stem = &syl[..syl.len() - '˙'.len_utf8()];
            let with_falling = format!("{stem}ˋ");
            if viterbi::best_entry(&dict.entries, &with_falling).is_some() {
                return with_falling;
            }
            if viterbi::best_entry(&dict.entries, stem).is_some() {
                return stem.to_string();
            }
            syl.clone()
        })
        .collect();
    viterbi::decode_with_hints(&normalised, &neutral_hints, &dict.entries)
}
