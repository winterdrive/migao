pub mod bopomofo;
pub mod english_from_bopomofo;
pub mod pinyin;

use crate::rule::Rule;

/// Return the Rule for a given IME identifier string.
pub fn get_rule(ime: &str) -> Option<Box<dyn Rule>> {
    match ime.to_lowercase().as_str() {
        "bopomofo-daqian" | "zhuyin" | "注音" => Some(Box::new(bopomofo::BopomofoDaqianRule)),
        "english-from-bopomofo" | "reverse" => {
            Some(Box::new(english_from_bopomofo::EnglishFromBopomofoRule))
        }
        "pinyin" | "拼音" => Some(Box::new(pinyin::PinyinRule)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive() {
        assert!(get_rule("BOPOMOFO-DAQIAN").is_some());
        assert!(get_rule("ZhuYin").is_some());
        assert!(get_rule("注音").is_some());
        assert!(get_rule("PINYIN").is_some());
        assert!(get_rule("拼音").is_some());
        assert!(get_rule("ENGLISH-FROM-BOPOMOFO").is_some());
        assert!(get_rule("reverse").is_some());
    }
}
