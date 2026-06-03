pub mod bigram;
pub mod dict;
pub mod ime;
pub mod pinyin_dict;
pub mod rule;
pub mod rules;
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
