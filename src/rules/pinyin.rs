use crate::ime::pinyin::{self, Segment};
use crate::pinyin_dict;
use crate::rule::Rule;

/// Rule: recovers text garbled by forgetting to activate 拼音 (Pinyin) IME.
///
/// Input:  raw key presses intended for standard Pinyin layout, e.g. "nihao"
/// Output: Chinese text, e.g. "你好"
///
/// Pipeline: key sequence → tone-less pinyin syllables → Viterbi → Chinese text.
///
/// Confidence uses character-coverage ratio (fraction of input chars matched as valid
/// pinyin syllables). Threshold is 0.7 (higher than Bopomofo's 0.3) to reduce
/// false positives on English text that partially overlaps valid pinyin syllables.
pub struct PinyinRule;

impl Rule for PinyinRule {
    fn name(&self) -> &str {
        "pinyin"
    }

    fn apply(&self, input: &str) -> Option<String> {
        if self.confidence(input) < 0.7 {
            return None;
        }

        let segments = pinyin::segment(input);
        let mut out = String::new();
        let mut syl_buf: Vec<String> = Vec::new();

        for seg in &segments {
            match seg {
                Segment::Syllable(py) => {
                    syl_buf.push(py.clone());
                }
                Segment::Passthrough(ch) => {
                    if !syl_buf.is_empty() {
                        out.push_str(&pinyin_dict::to_chinese(&syl_buf));
                        syl_buf.clear();
                    }
                    out.push(*ch);
                }
            }
        }

        if !syl_buf.is_empty() {
            out.push_str(&pinyin_dict::to_chinese(&syl_buf));
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn confidence(&self, input: &str) -> f32 {
        pinyin::confidence(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> PinyinRule {
        PinyinRule
    }

    #[test]
    fn test_nihao() {
        assert_eq!(rule().apply("nihao"), Some("你好".into()));
    }

    #[test]
    fn test_zhongguo() {
        assert_eq!(rule().apply("zhongguo"), Some("中國".into()));
    }

    #[test]
    fn test_jintian() {
        assert_eq!(rule().apply("jintian"), Some("今天".into()));
    }

    #[test]
    fn test_women() {
        // 我們 — pinyin input that also looks like English
        assert_eq!(rule().apply("women"), Some("我們".into()));
    }

    #[test]
    fn test_passthrough_punctuation() {
        assert_eq!(rule().apply("nihao!"), Some("你好!".into()));
    }

    #[test]
    fn test_english_rejected() {
        // "hello world" — low char coverage, should be rejected
        assert_eq!(rule().apply("hello world"), None);
    }

    #[test]
    fn test_confidence_threshold() {
        assert!(rule().confidence("nihao") > 0.7);
        assert!(rule().confidence("zhongguo") > 0.7);
        assert!(rule().confidence("hello world") < 0.7);
    }
}
