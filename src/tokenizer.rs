use std::collections::HashMap;
use std::path::Path;

// Standard BERT special token IDs (same across all BERT-Chinese variants).
pub const CLS_ID: i64 = 101;
pub const SEP_ID: i64 = 102;
pub const UNK_ID: i64 = 100;
pub const MASK_ID: i64 = 103;

pub struct Tokenizer {
    vocab: HashMap<String, i64>,
}

impl Tokenizer {
    /// Load vocab.txt from the given path.
    /// Each line is a token; the line number (0-indexed) is the token ID.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let vocab = content
            .lines()
            .enumerate()
            .map(|(id, token)| (token.to_string(), id as i64))
            .collect();
        Ok(Tokenizer { vocab })
    }

    /// Encode text into BERT input IDs.
    ///
    /// Returns: [CLS, char1, char2, ..., SEP]
    ///
    /// Each CJK character becomes its own token (character-level BERT).
    /// ASCII characters are looked up in vocab; unknown tokens → UNK_ID.
    pub fn encode(&self, text: &str) -> Vec<i64> {
        let mut ids = Vec::with_capacity(text.chars().count() + 2);
        ids.push(CLS_ID);
        for ch in text.chars() {
            let token = ch.to_string();
            let id = self.vocab.get(&token).copied().unwrap_or(UNK_ID);
            ids.push(id);
        }
        ids.push(SEP_ID);
        ids
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_tokens_present() {
        // Verify the constants match BERT convention (not vocab-file-dependent).
        assert_eq!(CLS_ID, 101);
        assert_eq!(SEP_ID, 102);
        assert_eq!(UNK_ID, 100);
    }

    #[test]
    fn test_encode_wraps_with_cls_sep() {
        // Build a minimal vocab map directly to test encode() logic.
        let vocab = [
            ("[PAD]", 0i64),
            ("[UNK]", 100),
            ("[CLS]", 101),
            ("[SEP]", 102),
            ("你", 872),
            ("好", 1962),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
        let tok = Tokenizer { vocab };
        let ids = tok.encode("你好");
        assert_eq!(ids, vec![101, 872, 1962, 102]);
    }

    #[test]
    fn test_encode_unknown_char_falls_back_to_unk() {
        let vocab = [("[CLS]", 101i64), ("[SEP]", 102), ("你", 872)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let tok = Tokenizer { vocab };
        // '好' is absent from vocab, so it must fall back to UNK_ID.
        let ids = tok.encode("你好");
        assert_eq!(ids, vec![101, 872, UNK_ID, 102]);
    }

    #[test]
    fn test_encode_empty_text_still_wraps_with_cls_sep() {
        let tok = Tokenizer {
            vocab: HashMap::new(),
        };
        let ids = tok.encode("");
        assert_eq!(ids, vec![CLS_ID, SEP_ID]);
    }

    #[test]
    fn test_vocab_size_matches_entry_count() {
        let vocab = [("[PAD]", 0i64), ("[UNK]", 100), ("你", 872)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let tok = Tokenizer { vocab };
        assert_eq!(tok.vocab_size(), 3);
    }

    #[test]
    fn test_load_missing_file_returns_err() {
        let result = Tokenizer::load(Path::new("does/not/exist/vocab.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_assigns_ids_by_line_number() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "migao_tokenizer_test_vocab_{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "[PAD]\n[UNK]\n[CLS]\n[SEP]\n你\n好\n").unwrap();

        let tok = Tokenizer::load(&path).expect("vocab file should load");
        std::fs::remove_file(&path).ok();

        assert_eq!(tok.vocab_size(), 6);
        // Line numbers (0-indexed) become token IDs: 你=4, 好=5.
        let ids = tok.encode("你好");
        assert_eq!(ids, vec![101, 4, 5, 102]);
    }
}
