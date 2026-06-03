use crate::bigram;
use std::collections::HashMap;

/// Per-extra-syllable bonus added to compound spans.
///
/// Calibrated for the essay×100 frequency scale shared by all dictionary types:
/// - \> 23.2: common 2-syl compounds (e.g. 再說) beat high-freq single chars
/// - \< 26.3: rare spurious compounds (e.g. 旖旎) still lose to correct chars
/// - bigram scoring adds \~1.75 nats net advantage per split step; see dict.rs
pub const COMPOUND_BONUS: f32 = 24.0;

/// Weight applied to ln(bigram_count+1) at CJK word boundaries.
pub const BIGRAM_WEIGHT: f32 = 0.15;

/// Greedy best-entry lookup from a `HashMap<key, Vec<(word, freq)>>`.
pub fn best_entry<'a>(
    dict: &'a HashMap<String, Vec<(String, u32)>>,
    key: &str,
) -> Option<(&'a str, u32)> {
    dict.get(key)
        .and_then(|v| v.iter().max_by_key(|(_, f)| f))
        .map(|(w, f)| (w.as_str(), *f))
}

/// Returns up to `n` entries for `key`, sorted by frequency descending.
fn top_entries<'a>(
    dict: &'a HashMap<String, Vec<(String, u32)>>,
    key: &str,
    n: usize,
) -> Vec<(&'a str, u32)> {
    let Some(entries) = dict.get(key) else {
        return Vec::new();
    };
    let mut v: Vec<_> = entries.iter().map(|(w, f)| (w.as_str(), *f)).collect();
    v.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
    v.truncate(n);
    v
}

/// Remove the trailing tone marker (ˊ ˇ ˋ ˙ ˉ) from a Bopomofo syllable.
fn strip_tone(syl: &str) -> &str {
    for tone in &["ˊ", "ˇ", "ˋ", "˙", "ˉ"] {
        if let Some(s) = syl.strip_suffix(tone) {
            return s;
        }
    }
    syl
}

/// Run the Viterbi DP and return the raw `(dp, back)` arrays.
fn run_dp(
    syllables: &[String],
    neutral_hints: &[bool],
    dict: &HashMap<String, Vec<(String, u32)>>,
) -> (Vec<f32>, Vec<Option<(usize, String)>>) {
    let n = syllables.len();
    let mut dp = vec![f32::NEG_INFINITY; n + 1];
    let mut back: Vec<Option<(usize, String)>> = vec![None; n + 1];
    dp[0] = 0.0;

    for i in 1..=n {
        for len in 1..=std::cmp::min(4, i) {
            let prev = dp[i - len];
            if prev == f32::NEG_INFINITY {
                continue;
            }
            let key: String = syllables[i - len..i].concat();
            let mut found = best_entry(dict, &key).map(|(w, f)| (w.to_string(), f));

            // For neutral-tone-origin syllables, also try ˊ / ˇ / ˉ tone variants
            // on the last syllable of the span (e.g. 覺得 stored as ˊ, typed as ˙→ˋ).
            if found.is_none() && neutral_hints.get(i - 1).copied().unwrap_or(false) {
                let last = &syllables[i - 1];
                let prefix: String = syllables[i - len..i - 1].concat();
                let stem = strip_tone(last);
                'alt: for tone in ["ˊ", "ˇ", "ˉ"] {
                    let alt = format!("{prefix}{stem}{tone}");
                    if let Some((w, f)) = best_entry(dict, &alt) {
                        found = Some((w.to_string(), f));
                        break 'alt;
                    }
                }
            }

            if let Some((word, freq)) = found {
                let bonus = if len > 1 {
                    (len - 1) as f32 * COMPOUND_BONUS
                } else {
                    0.0
                };
                let bigram_bonus = if i > len {
                    back[i - len]
                        .as_ref()
                        .and_then(|(_, prev_word)| {
                            let prev_last = prev_word.chars().last()?;
                            let curr_first = word.chars().next()?;
                            if bigram::is_cjk(prev_last) && bigram::is_cjk(curr_first) {
                                Some(bigram::log_count(prev_last, curr_first) * BIGRAM_WEIGHT)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let score = prev + (freq as f32).ln() + bonus + bigram_bonus;
                if score > dp[i] {
                    dp[i] = score;
                    back[i] = Some((len, word));
                }
            }
        }
        // Fallback: emit raw syllable key with a penalty when no dict entry exists.
        if back[i].is_none() && dp[i - 1] != f32::NEG_INFINITY {
            dp[i] = dp[i - 1] - 10.0;
            back[i] = Some((1, syllables[i - 1].clone()));
        }
    }

    (dp, back)
}

/// Reconstruct the path from the backpointer array.
/// Returns `Vec<(start_syllable_idx, span_len, word)>` in forward order.
fn collect_path(back: &mut [Option<(usize, String)>], n: usize) -> Vec<(usize, usize, String)> {
    let mut path = Vec::new();
    let mut i = n;
    while i > 0 {
        match back[i].take() {
            Some((len, word)) => {
                path.push((i - len, len, word));
                i -= len;
            }
            None => break,
        }
    }
    path.reverse();
    path
}

/// Viterbi decoder: convert a slice of phonetic syllable keys into Chinese text.
pub fn decode(syllables: &[String], dict: &HashMap<String, Vec<(String, u32)>>) -> String {
    decode_impl(syllables, &[], dict)
}

/// Like `decode`, but `neutral_hints[i]` marks syllable i as originally neutral-tone (˙).
///
/// For those positions, compound span lookups also try ˊ / ˇ / ˉ variants of the
/// tone-normalised syllable, enabling matches like 覺得 (dict key ends in ˊ) from
/// neutral-tone keystrokes that were normalised to ˋ (的) for single chars.
pub fn decode_with_hints(
    syllables: &[String],
    neutral_hints: &[bool],
    dict: &HashMap<String, Vec<(String, u32)>>,
) -> String {
    decode_impl(syllables, neutral_hints, dict)
}

fn decode_impl(
    syllables: &[String],
    neutral_hints: &[bool],
    dict: &HashMap<String, Vec<(String, u32)>>,
) -> String {
    if syllables.is_empty() {
        return String::new();
    }
    let (_, mut back) = run_dp(syllables, neutral_hints, dict);
    let path = collect_path(&mut back, syllables.len());
    path.iter().map(|(_, _, w)| w.as_str()).collect()
}

/// Return up to `n` candidate Chinese strings for the given phonetic syllable sequence.
///
/// The first element is always the highest-scoring Viterbi path. Additional candidates
/// are generated by substituting each word in the best path with its next-best dictionary
/// alternative for the same phonetic span, then re-scoring. Results are deduplicated and
/// sorted by approximate score (best first).
pub fn decode_candidates(
    n: usize,
    syllables: &[String],
    neutral_hints: &[bool],
    dict: &HashMap<String, Vec<(String, u32)>>,
) -> Vec<String> {
    if n == 0 || syllables.is_empty() {
        return Vec::new();
    }
    let (dp, mut back) = run_dp(syllables, neutral_hints, dict);
    let total_score = dp[syllables.len()];
    let path = collect_path(&mut back, syllables.len());

    let best: String = path.iter().map(|(_, _, w)| w.as_str()).collect();
    if n == 1 {
        return vec![best];
    }

    let mut candidates: Vec<(f32, String)> = vec![(total_score, best)];

    for (seg_idx, (seg_start, seg_len, seg_word)) in path.iter().enumerate() {
        let key: String = syllables[*seg_start..*seg_start + seg_len].concat();
        let alts = top_entries(dict, &key, 4);

        let orig_freq = alts
            .iter()
            .find(|(w, _)| *w == seg_word.as_str())
            .map(|(_, f)| *f)
            .unwrap_or(1);

        for (alt_word, alt_freq) in &alts {
            if *alt_word == seg_word.as_str() {
                continue;
            }
            let alt_text: String = path
                .iter()
                .enumerate()
                .map(|(i, (_, _, w))| if i == seg_idx { *alt_word } else { w.as_str() })
                .collect();
            let score_delta = (*alt_freq as f32).ln() - (orig_freq as f32).ln();
            candidates.push((total_score + score_delta, alt_text));
        }
    }

    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    candidates.dedup_by(|a, b| a.1 == b.1);
    candidates.truncate(n);
    candidates.into_iter().map(|(_, s)| s).collect()
}
