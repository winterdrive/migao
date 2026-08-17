pub mod bigram;
pub mod config;
pub mod dict;
pub mod ime;
pub mod pinyin_dict;
pub mod reranker;
pub mod rule;
pub mod rules;
pub mod tokenizer;
pub mod user_data;
pub mod viterbi;

/// Recover garbled text produced by the given IME.
///
/// `ime` accepts: "bopomofo-daqian", "zhuyin", "注音"
/// Returns None if no rule recognises the input or confidence is too low.
pub fn recover(input: &str, ime: &str) -> Option<String> {
    let rule = rules::get_rule(ime)?;
    rule.apply(input)
}

/// Like `recover`, but returns up to `n` candidate recoveries (best first).
/// Returns an empty Vec if no rule matches.
pub fn recover_top_n(input: &str, ime: &str, n: usize) -> Vec<String> {
    let Some(rule) = rules::get_rule(ime) else {
        return Vec::new();
    };
    rule.apply_top_n(input, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_unknown_ime_returns_none() {
        assert_eq!(recover("su3cl3", "not-a-real-ime"), None);
    }

    #[test]
    fn test_recover_bopomofo_daqian() {
        assert_eq!(recover("su3cl3", "bopomofo-daqian"), Some("你好".into()));
    }

    #[test]
    fn test_recover_pinyin() {
        assert_eq!(recover("nihao", "pinyin"), Some("你好".into()));
    }

    #[test]
    fn test_recover_top_n_unknown_ime_returns_empty() {
        assert!(recover_top_n("su3cl3", "not-a-real-ime", 3).is_empty());
    }

    #[test]
    fn test_recover_top_n_zero_returns_empty() {
        assert!(recover_top_n("su3cl3", "bopomofo-daqian", 0).is_empty());
    }

    #[test]
    fn test_recover_top_n_default_impl_delegates_to_apply() {
        // "pinyin" and "english-from-bopomofo" rules don't override apply_top_n,
        // so recover_top_n must fall back to the trait's default (apply -> 0 or 1 result).
        let candidates = recover_top_n("nihao", "pinyin", 3);
        assert_eq!(candidates, vec!["你好".to_string()]);
    }
}
