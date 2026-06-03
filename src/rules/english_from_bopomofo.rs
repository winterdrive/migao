use crate::ime::daqian;
use crate::rule::Rule;

/// Rule: recovers text garbled by typing English while 注音 IME was active.
///
/// Input:  Bopomofo Unicode symbols produced by the IME, e.g. "ㄘㄍㄠㄠㄟ"
/// Output: original English keystrokes, e.g. "hello"
///
/// Non-Bopomofo characters (Chinese text, punctuation, spaces) pass through unchanged.
pub struct EnglishFromBopomofoRule;

impl Rule for EnglishFromBopomofoRule {
    fn name(&self) -> &str {
        "english-from-bopomofo"
    }

    fn apply(&self, input: &str) -> Option<String> {
        if self.confidence(input) < 0.3 {
            return None;
        }

        let out: String = input
            .chars()
            .map(|ch| match daqian::zhuyin_to_key(ch) {
                Some(k) => k,
                None => ch,
            })
            .collect();

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn confidence(&self, input: &str) -> f32 {
        let total = input.chars().count();
        if total == 0 {
            return 0.0;
        }
        let bopomofo_count = input
            .chars()
            .filter(|&ch| daqian::zhuyin_to_key(ch).is_some())
            .count();
        bopomofo_count as f32 / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> EnglishFromBopomofoRule {
        EnglishFromBopomofoRule
    }

    #[test]
    fn test_hello() {
        // h→ㄘ e→ㄍ l→ㄠ l→ㄠ o→ㄟ
        assert_eq!(rule().apply("ㄘㄍㄠㄠㄟ"), Some("hello".into()));
    }

    #[test]
    fn test_mixed_bopomofo_chinese() {
        let result = rule().apply("ㄘㄍㄠㄠㄟ 你好");
        assert_eq!(result, Some("hello 你好".into()));
    }

    #[test]
    fn test_empty_rejected() {
        assert_eq!(rule().apply(""), None);
    }

    #[test]
    fn test_pure_chinese_rejected() {
        // No Bopomofo symbols → confidence 0.0 → None
        assert_eq!(rule().apply("你好世界"), None);
    }

    #[test]
    fn test_confidence_pure_bopomofo() {
        assert!((rule().confidence("ㄘㄍㄠㄠㄟ") - 1.0).abs() < f32::EPSILON);
    }
}
