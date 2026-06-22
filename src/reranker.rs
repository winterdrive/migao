use crate::tokenizer::{self, Tokenizer};
use crate::user_data;
use ndarray::{Array2, Axis};
use ort::session::Session;
use ort::value::Tensor;
use std::sync::{Mutex, OnceLock};

static GLOBAL: OnceLock<Option<Reranker>> = OnceLock::new();

pub struct Reranker {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

impl Reranker {
    /// Load model and vocab from %APPDATA%\Migao\models\.
    ///
    /// Also loads `onnxruntime.dll` from the same directory (required for the
    /// `load-dynamic` build).  Returns None silently if any file is absent or
    /// fails to load — callers fall back to Viterbi-only ranking.
    pub fn load() -> Option<Self> {
        let base = user_data::user_supplement_path()?.parent()?.to_path_buf();
        let models_dir = base.join("models");
        let dll_path = models_dir.join("onnxruntime.dll");
        let model_path = models_dir.join("bert-tiny-chinese.onnx");
        let vocab_path = models_dir.join("vocab.txt");

        if !model_path.exists() || !vocab_path.exists() {
            return None;
        }

        // Load the ONNX Runtime DLL from the models directory.
        // Falls back to any onnxruntime.dll already in PATH if the local one is absent.
        if dll_path.exists() {
            ort::init_from(&dll_path).ok()?.commit();
        } else {
            ort::init().commit();
        }

        let tok = Tokenizer::load(&vocab_path).ok()?;
        let sess = Session::builder()
            .ok()?
            .commit_from_file(&model_path)
            .ok()?;

        Some(Reranker {
            session: Mutex::new(sess),
            tokenizer: tok,
        })
    }

    /// Compute pseudo-log-likelihood (PLH) for a sentence using masked scoring.
    ///
    /// For each content token at position i, we mask it ([MASK]) and run BERT
    /// to get P(token_i | surrounding context). This is true masked PLH.
    ///
    /// Each position is scored with a separate forward pass (batch_size=1) to
    /// avoid dimension mismatches from non-dynamic ONNX shapes.
    ///
    /// Returns the negative mean log-prob: lower = more natural.
    /// Returns f32::MAX on any inference error so the candidate sorts last.
    pub fn score(&self, sentence: &str) -> f32 {
        let token_ids = self.tokenizer.encode(sentence);
        let seq_len = token_ids.len();
        let content_len = seq_len - 2; // exclude [CLS] and [SEP]
        if content_len == 0 {
            return f32::MAX;
        }

        let Ok(mut session) = self.session.lock() else {
            return f32::MAX;
        };

        let mut total_log_prob = 0.0f32;

        for mask_pos in 1..seq_len - 1 {
            let mut masked = token_ids.clone();
            masked[mask_pos] = tokenizer::MASK_ID;

            let ids_arr = match Array2::from_shape_vec((1, seq_len), masked) {
                Ok(a) => a,
                Err(_) => return f32::MAX,
            };
            let mask_arr = Array2::<i64>::ones((1, seq_len));
            let type_arr = Array2::<i64>::zeros((1, seq_len));

            let (ids_t, mask_t, type_t) = match (
                Tensor::from_array(ids_arr),
                Tensor::from_array(mask_arr),
                Tensor::from_array(type_arr),
            ) {
                (Ok(a), Ok(b), Ok(c)) => (a, b, c),
                _ => return f32::MAX,
            };

            let Ok(outputs) = session.run(ort::inputs! {
                "input_ids"      => ids_t,
                "attention_mask" => mask_t,
                "token_type_ids" => type_t
            }) else {
                return f32::MAX;
            };

            // logits shape: [1, seq_len, vocab_size]
            let Ok(logits) = outputs[0].try_extract_array::<f32>() else {
                return f32::MAX;
            };

            let target_id = token_ids[mask_pos] as usize;

            // Characters not in BERT's vocab map to UNK. We can't compute a
            // meaningful PLH for them, so we apply a fixed large penalty that
            // ensures unknown-character candidates rank below known ones.
            if target_id == tokenizer::UNK_ID as usize {
                total_log_prob += -20.0;
                continue;
            }

            let row = logits
                .index_axis(Axis(0), 0) // remove batch dim
                .index_axis(Axis(0), mask_pos) // position we masked
                .to_owned();

            let max_logit = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_sum: f32 = row.iter().map(|&x| (x - max_logit).exp()).sum();
            let log_prob = (row[target_id] - max_logit) - exp_sum.ln();
            total_log_prob += log_prob;
        }

        -(total_log_prob / content_len as f32)
    }

    /// Re-rank candidates: most natural sentence first (lowest PLH score).
    ///
    /// Only displaces the Viterbi top candidate when PLH improvement exceeds
    /// MIN_RERANK_MARGIN. This prevents BERT-tiny from overriding correct
    /// Viterbi choices when its confidence is low (e.g. 遍 vs 變, diff ~0.2).
    /// The threshold is set above 0.2 (noisy) but below 0.57 (real signal).
    pub fn rerank(&self, mut candidates: Vec<String>) -> Vec<String> {
        if candidates.is_empty() {
            return candidates;
        }
        const MIN_RERANK_MARGIN: f32 = 0.40;

        let viterbi_top = candidates[0].clone();
        let mut scored: Vec<(f32, String)> =
            candidates.drain(..).map(|c| (self.score(&c), c)).collect();

        // Score of Viterbi's top candidate (might have been scored already above).
        let viterbi_score = scored
            .iter()
            .find(|(_, s)| *s == viterbi_top)
            .map(|(sc, _)| *sc)
            .unwrap_or(f32::MAX);

        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // If BERT prefers a different candidate but the margin is below threshold,
        // reinstate Viterbi's top-1 at position 0.
        if scored[0].1 != viterbi_top && viterbi_score - scored[0].0 < MIN_RERANK_MARGIN {
            if let Some(pos) = scored.iter().position(|(_, s)| *s == viterbi_top) {
                let entry = scored.remove(pos);
                scored.insert(0, entry);
            }
        }

        scored.into_iter().map(|(_, s)| s).collect()
    }
}

/// Global lazy-initialised reranker. Returns None if model files are absent.
pub fn global() -> &'static Option<Reranker> {
    GLOBAL.get_or_init(Reranker::load)
}

/// Pre-warm the reranker at startup (call from background thread).
pub fn init() {
    let _ = global();
}
