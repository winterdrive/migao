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

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal Rule that does not override `apply_top_n`, so tests here
    /// exercise the trait's default implementation directly.
    struct AlwaysMatch;

    impl Rule for AlwaysMatch {
        fn name(&self) -> &str {
            "always-match"
        }

        fn apply(&self, input: &str) -> Option<String> {
            if input.is_empty() {
                None
            } else {
                Some(input.to_uppercase())
            }
        }

        fn confidence(&self, _input: &str) -> f32 {
            1.0
        }
    }

    #[test]
    fn test_default_apply_top_n_zero_returns_empty() {
        // n == 0 short-circuits before calling apply(), even for matching input.
        assert_eq!(AlwaysMatch.apply_top_n("hi", 0), Vec::<String>::new());
    }

    #[test]
    fn test_default_apply_top_n_delegates_to_apply() {
        assert_eq!(AlwaysMatch.apply_top_n("hi", 5), vec!["HI".to_string()]);
    }

    #[test]
    fn test_default_apply_top_n_none_returns_empty() {
        assert_eq!(AlwaysMatch.apply_top_n("", 5), Vec::<String>::new());
    }
}
