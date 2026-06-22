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
}
