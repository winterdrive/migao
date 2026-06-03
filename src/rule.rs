/// Core trait for all recovery rules (Heuristic Learning unit).
/// Each Rule handles one specific garbling pattern.
/// New patterns discovered via user feedback → new Rule implementations.
pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    /// Returns Some(recovered) if this rule applies, None if it doesn't recognise the input.
    fn apply(&self, input: &str) -> Option<String>;
    /// 0.0–1.0 estimate of how confident this rule is about the input.
    fn confidence(&self, input: &str) -> f32;

    /// Returns up to `n` candidate recoveries, highest-confidence first.
    ///
    /// Default: delegates to `apply`, returning 0 or 1 result.
    /// Override to provide richer top-N output (e.g. Bopomofo Viterbi candidates).
    fn apply_top_n(&self, input: &str, n: usize) -> Vec<String> {
        if n == 0 {
            return Vec::new();
        }
        self.apply(input).into_iter().collect()
    }
}
