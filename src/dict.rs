use crate::user_data;
use crate::viterbi;
use std::collections::HashMap;
use std::sync::OnceLock;

static GLOBAL: OnceLock<ZhuyinDict> = OnceLock::new();

pub struct ZhuyinDict {
    entries: HashMap<String, Vec<(String, u32)>>,
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

/// Modernise archaic 喫 → 吃 for all dictionary entries, in place.
///
/// bopomofo.tsv was built from a corpus with high 喫 frequency; in modern
/// Traditional Chinese (Taiwan), 吃 is the standard character. We inject
/// 吃-variants at very high frequency so they outrank the 喫 originals.
///
/// Some keys (e.g. ㄔ) already carry both 吃 and 喫 as separate entries; for
/// those we bump the existing 吃 entry's frequency instead of pushing a
/// duplicate, so n-best candidate lists don't surface the same word twice.
fn modernise_archaic_chi(entries: &mut HashMap<String, Vec<(String, u32)>>) {
    let chi_keys: Vec<(String, String)> = entries
        .iter()
        .flat_map(|(key, vals)| {
            vals.iter().filter_map(|(word, _)| {
                if word.contains('喫') {
                    Some((key.clone(), word.replace('喫', "吃")))
                } else {
                    None
                }
            })
        })
        .collect();
    for (key, new_word) in chi_keys {
        let vals = entries.entry(key).or_default();
        if let Some(existing) = vals.iter_mut().find(|(w, _)| *w == new_word) {
            existing.1 = existing.1.max(10_000_000);
        } else {
            vals.push((new_word, 10_000_000));
        }
    }
}

impl ZhuyinDict {
    fn load() -> Self {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        for src in &[
            include_str!("../data/bopomofo.tsv"),
            include_str!("../data/supplement.tsv"),
        ] {
            parse_tsv_into(src, &mut entries);
        }
        let user_src = user_data::load();
        if !user_src.is_empty() {
            parse_tsv_into(&user_src, &mut entries);
        }
        modernise_archaic_chi(&mut entries);
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
    // Request extra candidates so the reranker has room to reorder.
    let fetch_n = if crate::reranker::global().is_some() {
        n.max(5)
    } else {
        n
    };
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
    let candidates =
        viterbi::decode_candidates(fetch_n, &normalised, &neutral_hints, &dict.entries);

    // Rerank with neural model when available; truncate to requested n.
    let reranked = crate::reranker::global()
        .as_ref()
        .map(|r| r.rerank(candidates.clone()))
        .unwrap_or(candidates);

    reranked.into_iter().take(n).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modernise_chi_bumps_existing_entry_instead_of_duplicating() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        entries.insert(
            "ㄔ".to_string(),
            vec![("吃".to_string(), 81_500), ("喫".to_string(), 6_560_700)],
        );

        modernise_archaic_chi(&mut entries);

        let words = &entries["ㄔ"];
        let chi_matches: Vec<&(String, u32)> = words.iter().filter(|(w, _)| w == "吃").collect();
        assert_eq!(
            chi_matches.len(),
            1,
            "modernisation must not create a duplicate 吃 entry, got {words:?}"
        );
        assert_eq!(chi_matches[0].1, 10_000_000);
    }

    #[test]
    fn modernise_chi_adds_new_entry_when_absent() {
        let mut entries: HashMap<String, Vec<(String, u32)>> = HashMap::new();
        entries.insert("ㄏㄠˇㄔ".to_string(), vec![("好喫".to_string(), 811_200)]);

        modernise_archaic_chi(&mut entries);

        let words = &entries["ㄏㄠˇㄔ"];
        assert_eq!(words.len(), 2);
        assert!(words.iter().any(|(w, f)| w == "好吃" && *f == 10_000_000));
        assert!(words.iter().any(|(w, f)| w == "好喫" && *f == 811_200));
    }
}
