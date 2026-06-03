/// 大千標準注音鍵盤佈局
/// Maps each ASCII key to its Bopomofo (Zhuyin) symbol.
/// Tone keys: 3=ˇ  4=ˋ  6=ˊ  7=˙  (space = tone 1, handled by the parser)

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum KeyCategory {
    Initial = 0, // 聲母
    Medial = 1,  // 介音
    Final = 2,   // 韻母
    Tone = 3,    // 聲調
}

pub fn key_category(key: char) -> Option<KeyCategory> {
    match key {
        '1' | 'q' | 'a' | 'z' | '2' | 'w' | 's' | 'x' | 'e' | 'd' | 'c' | 'r' | 'f' | 'v' | '5'
        | 't' | 'g' | 'b' | 'y' | 'h' | 'n' => Some(KeyCategory::Initial),
        'u' | 'j' | 'm' => Some(KeyCategory::Medial),
        '8' | 'i' | 'k' | ',' | '9' | 'o' | 'l' | '.' | '0' | 'p' | ';' | '/' | '-' => {
            Some(KeyCategory::Final)
        }
        '3' | '4' | '6' | '7' => Some(KeyCategory::Tone),
        _ => None,
    }
}

/// A syllable is valid if its keys appear in order: Initial → Medial → Final → Tone,
/// with at most one of each, and at least one phonetic component (non-tone).
pub fn is_valid_syllable(keys: &str) -> bool {
    let mut stage: Option<KeyCategory> = None;
    let mut has_phonetic = false;

    for ch in keys.chars() {
        let Some(cat) = key_category(ch) else {
            return false;
        };
        if let Some(prev) = stage {
            if cat <= prev {
                return false; // out of order or duplicate
            }
        }
        stage = Some(cat);
        if cat != KeyCategory::Tone {
            has_phonetic = true;
        }
    }

    has_phonetic
}

pub fn key_to_zhuyin(key: char) -> Option<&'static str> {
    match key {
        // Initials (聲母)
        '1' => Some("ㄅ"),
        'q' => Some("ㄆ"),
        'a' => Some("ㄇ"),
        'z' => Some("ㄈ"),
        '2' => Some("ㄉ"),
        'w' => Some("ㄊ"),
        's' => Some("ㄋ"),
        'x' => Some("ㄌ"),
        'e' => Some("ㄍ"),
        'd' => Some("ㄎ"),
        'c' => Some("ㄏ"),
        'r' => Some("ㄐ"),
        'f' => Some("ㄑ"),
        'v' => Some("ㄒ"),
        '5' => Some("ㄓ"),
        't' => Some("ㄔ"),
        'g' => Some("ㄕ"),
        'b' => Some("ㄖ"),
        'y' => Some("ㄗ"),
        'h' => Some("ㄘ"),
        'n' => Some("ㄙ"),
        // Medials (介音)
        'u' => Some("ㄧ"),
        'j' => Some("ㄨ"),
        'm' => Some("ㄩ"),
        // Finals (韻母)
        '8' => Some("ㄚ"),
        'i' => Some("ㄛ"),
        'k' => Some("ㄜ"),
        ',' => Some("ㄝ"),
        '9' => Some("ㄞ"),
        'o' => Some("ㄟ"),
        'l' => Some("ㄠ"),
        '.' => Some("ㄡ"),
        '0' => Some("ㄢ"),
        'p' => Some("ㄣ"),
        ';' => Some("ㄤ"),
        '/' => Some("ㄥ"),
        '-' => Some("ㄦ"),
        // Tone marks (聲調)
        '6' => Some("ˊ"),
        '3' => Some("ˇ"),
        '4' => Some("ˋ"),
        '7' => Some("˙"),
        _ => None,
    }
}

/// Reverse mapping: Bopomofo Unicode symbol → Daqian ASCII key.
pub fn zhuyin_to_key(sym: char) -> Option<char> {
    match sym {
        'ㄅ' => Some('1'),
        'ㄆ' => Some('q'),
        'ㄇ' => Some('a'),
        'ㄈ' => Some('z'),
        'ㄉ' => Some('2'),
        'ㄊ' => Some('w'),
        'ㄋ' => Some('s'),
        'ㄌ' => Some('x'),
        'ㄍ' => Some('e'),
        'ㄎ' => Some('d'),
        'ㄏ' => Some('c'),
        'ㄐ' => Some('r'),
        'ㄑ' => Some('f'),
        'ㄒ' => Some('v'),
        'ㄓ' => Some('5'),
        'ㄔ' => Some('t'),
        'ㄕ' => Some('g'),
        'ㄖ' => Some('b'),
        'ㄗ' => Some('y'),
        'ㄘ' => Some('h'),
        'ㄙ' => Some('n'),
        'ㄧ' => Some('u'),
        'ㄨ' => Some('j'),
        'ㄩ' => Some('m'),
        'ㄚ' => Some('8'),
        'ㄛ' => Some('i'),
        'ㄜ' => Some('k'),
        'ㄝ' => Some(','),
        'ㄞ' => Some('9'),
        'ㄟ' => Some('o'),
        'ㄠ' => Some('l'),
        'ㄡ' => Some('.'),
        'ㄢ' => Some('0'),
        'ㄣ' => Some('p'),
        'ㄤ' => Some(';'),
        'ㄥ' => Some('/'),
        'ㄦ' => Some('-'),
        'ˊ' => Some('6'),
        'ˇ' => Some('3'),
        'ˋ' => Some('4'),
        '˙' => Some('7'),
        _ => None,
    }
}

/// Returns true if the key is an explicit tone marker (excludes space/tone-1).
pub fn is_tone_key(key: char) -> bool {
    matches!(key, '3' | '4' | '6' | '7')
}

/// Returns true if this key maps to any Bopomofo symbol (including tone keys).
pub fn is_zhuyin_key(key: char) -> bool {
    key_to_zhuyin(key).is_some()
}

/// Segment a raw key sequence into individual Bopomofo syllables.
///
/// Rules:
/// - Tone keys 3/4/6/7 terminate and are included in the current syllable.
/// - Space after a non-empty phonetic buffer = tone-1 syllable (space consumed).
/// - Space with empty buffer = literal word separator (kept as " ").
/// - A new Initial key arriving after the buffer already holds a Medial or Final
///   auto-terminates the current syllable (ordering rule: I→M→F→T, no reset).
/// - Unrecognised keys are passed through as-is.
pub fn segment(input: &str) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut buf = String::new();
    let mut max_cat: Option<KeyCategory> = None;

    for ch in input.chars() {
        if is_tone_key(ch) {
            buf.push(ch);
            result.push(Segment::Syllable(buf.clone()));
            buf.clear();
            max_cat = None;
        } else if ch == ' ' {
            if !buf.is_empty() {
                // Non-empty buffer + space = tone-1 character
                result.push(Segment::Syllable(buf.clone()));
                buf.clear();
                max_cat = None;
            } else {
                result.push(Segment::Passthrough(' '));
            }
        } else if is_zhuyin_key(ch) {
            let cat = key_category(ch).unwrap();
            // Auto-terminate when a new Initial arrives after the buffer already holds a
            // Medial. In valid 大千 input this signals a new syllable; restricting to
            // Initial-after-Medial (not Initial-after-Final) preserves English passthrough
            // for words like "world" which have consonant→vowel→consonant patterns.
            if cat == KeyCategory::Initial && max_cat == Some(KeyCategory::Medial) {
                result.push(Segment::Syllable(buf.clone()));
                buf.clear();
            }
            buf.push(ch);
            max_cat = Some(cat);
        } else {
            // Non-Bopomofo character: flush current buffer first
            if !buf.is_empty() {
                result.push(Segment::Syllable(buf.clone()));
                buf.clear();
                max_cat = None;
            }
            result.push(Segment::Passthrough(ch));
        }
    }

    if !buf.is_empty() {
        result.push(Segment::Syllable(buf));
    }

    result
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    /// A sequence of phonetic keys (+ optional tone key) forming one syllable.
    Syllable(String),
    /// A character that passes through unchanged (punctuation, spaces, etc.).
    Passthrough(char),
}

/// Convert a syllable's key sequence to Bopomofo Unicode symbols.
pub fn keys_to_bopomofo(keys: &str) -> String {
    let mut out = String::new();
    for ch in keys.chars() {
        if let Some(sym) = key_to_zhuyin(ch) {
            out.push_str(sym);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ni_hao() {
        // 你好 → ㄋㄧˇ ㄏㄠˇ
        let segs = segment("su3cl3");
        assert_eq!(
            segs,
            vec![
                Segment::Syllable("su3".into()),
                Segment::Syllable("cl3".into()),
            ]
        );
        assert_eq!(keys_to_bopomofo("su3"), "ㄋㄧˇ");
        assert_eq!(keys_to_bopomofo("cl3"), "ㄏㄠˇ");
    }

    #[test]
    fn test_tone1_space() {
        // 他 = ㄊㄚ (tone 1) → "w8 "
        let segs = segment("w8 ");
        assert_eq!(segs, vec![Segment::Syllable("w8".into())]);
        assert_eq!(keys_to_bopomofo("w8"), "ㄊㄚ");
    }

    #[test]
    fn test_passthrough() {
        let segs = segment("su3!cl3");
        assert_eq!(
            segs,
            vec![
                Segment::Syllable("su3".into()),
                Segment::Passthrough('!'),
                Segment::Syllable("cl3".into()),
            ]
        );
    }

    #[test]
    fn test_initial_after_medial_auto_terminates() {
        // ㄧ (Medial u) followed immediately by ㄋ (Initial s): two syllables.
        // "u" alone = ㄧ tone-1, "su3" = ㄋㄧˇ (你).
        let segs = segment("usu3");
        assert_eq!(
            segs,
            vec![
                Segment::Syllable("u".into()),
                Segment::Syllable("su3".into()),
            ]
        );
    }

    #[test]
    fn test_initial_after_final_no_auto_terminate() {
        // Initial after Final is NOT auto-terminated: preserves English passthrough
        // for consonant-vowel-consonant words ("world": w=Initial, o=Final, r=Initial).
        // "ksu3" = ㄜ+ㄋㄧˇ accumulated as one (invalid) segment.
        let segs = segment("ksu3");
        assert_eq!(segs, vec![Segment::Syllable("ksu3".into()),]);
    }

    #[test]
    fn test_two_initials_no_auto_terminate() {
        // Two consecutive Initials accumulate as one (invalid) segment so that
        // English words like "he" (h=Initial, e=Initial) remain invalid syllables
        // and get passed through rather than decoded as Chinese.
        let segs = segment("sc");
        assert_eq!(segs, vec![Segment::Syllable("sc".into()),]);
    }
}
