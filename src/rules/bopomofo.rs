use crate::dict;
use crate::ime::daqian::{self, Segment};
use crate::rule::Rule;

/// Rule: recovers text garbled by forgetting to activate 注音 (Bopomofo) IME.
///
/// Input:  raw key presses intended for 大千 layout, e.g. "su3cl3"
/// Output: Traditional Chinese text, e.g. "你好"
///
/// Pipeline: key sequence → Bopomofo symbols → dictionary lookup → Chinese text.
/// Falls back to raw Bopomofo symbols for syllables not found in the dictionary.
pub struct BopomofoDaqianRule;

impl Rule for BopomofoDaqianRule {
    fn name(&self) -> &str {
        "bopomofo-daqian"
    }

    fn apply(&self, input: &str) -> Option<String> {
        if self.confidence(input) < 0.3 {
            return None;
        }

        let segments = daqian::segment(input);
        let mut out = String::new();
        let mut syl_buf: Vec<String> = Vec::new();

        for seg in &segments {
            match seg {
                Segment::Syllable(keys) => {
                    if daqian::is_valid_syllable(keys) {
                        let zhuyin = daqian::keys_to_bopomofo(keys);
                        if !zhuyin.is_empty() {
                            syl_buf.push(zhuyin);
                        }
                    } else {
                        // Structurally invalid syllable (e.g. English word): flush and passthrough
                        if !syl_buf.is_empty() {
                            out.push_str(&dict::to_chinese(&syl_buf));
                            syl_buf.clear();
                        }
                        out.push_str(keys);
                    }
                }
                Segment::Passthrough(ch) => {
                    if !syl_buf.is_empty() {
                        out.push_str(&dict::to_chinese(&syl_buf));
                        syl_buf.clear();
                    }
                    out.push(*ch);
                }
            }
        }

        if !syl_buf.is_empty() {
            out.push_str(&dict::to_chinese(&syl_buf));
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn apply_top_n(&self, input: &str, n: usize) -> Vec<String> {
        if n == 0 || self.confidence(input) < 0.3 {
            return Vec::new();
        }
        if n == 1 {
            return self.apply(input).into_iter().collect();
        }

        let segments = daqian::segment(input);

        // Top-N only works cleanly on continuous phonetic runs; degrade for mixed content.
        let has_passthrough = segments
            .iter()
            .any(|s| matches!(s, Segment::Passthrough(_)));
        if has_passthrough {
            return self.apply(input).into_iter().collect();
        }

        let syllables: Vec<String> = segments
            .iter()
            .filter_map(|s| {
                if let Segment::Syllable(keys) = s {
                    if daqian::is_valid_syllable(keys) {
                        let z = daqian::keys_to_bopomofo(keys);
                        if !z.is_empty() {
                            return Some(z);
                        }
                    }
                }
                None
            })
            .collect();

        if syllables.is_empty() {
            return Vec::new();
        }

        dict::to_chinese_candidates(n, &syllables)
    }

    /// Confidence = ratio of structurally valid Bopomofo syllables to total syllables.
    /// Uses ordering rules (Initial→Medial→Final→Tone) so English text scores near 0.
    fn confidence(&self, input: &str) -> f32 {
        let segments = daqian::segment(input);
        let syllables: Vec<&str> = segments
            .iter()
            .filter_map(|s| {
                if let Segment::Syllable(k) = s {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            .collect();

        if syllables.is_empty() {
            return 0.0;
        }

        let valid = syllables
            .iter()
            .filter(|&&k| daqian::is_valid_syllable(k))
            .count();

        valid as f32 / syllables.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> BopomofoDaqianRule {
        BopomofoDaqianRule
    }

    #[test]
    fn test_ni_hao() {
        assert_eq!(rule().apply("su3cl3"), Some("你好".into()));
    }

    #[test]
    fn test_zhongguo() {
        assert_eq!(rule().apply("5j/ eji6"), Some("中國".into()));
    }

    #[test]
    fn test_word_separator_preserved() {
        let result = rule().apply("su3cl3 su3cl3");
        assert!(result.unwrap().contains(' '));
    }

    #[test]
    fn test_passthrough_punctuation() {
        assert_eq!(rule().apply("su3cl3!"), Some("你好!".into()));
    }

    #[test]
    fn test_english_rejected() {
        // "hello world": h+e = two Initials → both syllables invalid → confidence 0
        assert_eq!(rule().apply("hello world"), None);
    }

    #[test]
    fn test_mixed_chinese_english() {
        // 你好 followed by a literal space and English passthrough
        let result = rule().apply("su3cl3 world");
        let r = result.unwrap();
        assert!(r.starts_with("你好"));
        assert!(r.contains("world"));
    }

    #[test]
    fn test_taipei_sentence() {
        // 我想今天去台北搭火車 — real-world sentence with multi-syllable words and spaces
        // This caught a bug where terra_pinyin's flat 100% weight caused rare chars
        // (e.g. 𧋲) to win over common chars (北) for ㄅㄟˇ.
        let result = rule().apply("ji3vu;3rup wu0 fm4w961o328 cji3tk");
        let r = result.unwrap();
        assert!(r.contains("我"), "missing 我: {r}");
        assert!(r.contains("想"), "missing 想: {r}");
        assert!(r.contains("今天"), "missing 今天: {r}");
        assert!(r.contains("去"), "missing 去: {r}");
        assert!(r.contains("北"), "missing 北 (was bug: 𧋲): {r}");
        assert!(r.contains("搭"), "missing 搭: {r}");
        assert!(r.contains("火車"), "missing 火車: {r}");
    }

    #[test]
    fn test_henhao_sentence() {
        // 很好，這個有成為單元測試的一環嗎?
        // Caught two bugs: ①為 secondary-reading vs 維 (supplement fix);
        // ②有成 greedily consumed 有+成 preventing 成為 compound match (有成為 added to supplement).
        let result = rule().apply("cp3cl3，5k4ek7u.3t/6jo620 m06hk4g42k7u4cj06a87?");
        let r = result.unwrap();
        assert!(r.contains("很好"), "missing 很好: {r}");
        assert!(r.contains("成為"), "missing 成為 (was bug: 維): {r}");
        assert!(r.contains("單元"), "missing 單元: {r}");
        assert!(r.contains("測試"), "missing 測試: {r}");
        assert!(r.contains("一環"), "missing 一環: {r}");
    }

    #[test]
    fn test_next_step_is_what() {
        // 很好，下一步是甚麼? — 甚麼 essay uses 什麼 variant (same word)
        // Note: 甚=ㄕㄣˊ→gp6, 麼=ㄇㄜ˙→ak7 (not gk6/ai7).
        let result = rule().apply("cp3cl3，vu84u61j4g4gp6ak7?");
        let r = result.unwrap();
        assert!(r.contains("很好"), "missing 很好: {r}");
        assert!(r.contains("下一步"), "missing 下一步: {r}");
        assert!(r.contains("是"), "missing 是: {r}");
        assert!(r.contains("什麼"), "missing 什麼: {r}");
    }

    #[test]
    fn test_generate_content_sentence() {
        // 我希望你可以一次生成很多內容，然後讓我打一次字試試看
        // Bugs fixed: ①一次(ㄧˊㄘˋ)→宜次 (ㄧˊ sandhi; supplement);
        //             ②刺字(ㄘˋㄗˋ,481) greedy beat 次+字 (字試試看 added);
        //             ③是(118M) beat 試(747K) (試試看 compound added).
        let result =
            rule().apply("ji3vu j;4su3dk3u3u6h4g/ t/6cp32ji so4bj/6，b06c.4b;4ji3283u6h4y4g4g4d04");
        let r = result.unwrap();
        assert!(r.contains("我希望"), "missing 我希望: {r}");
        assert!(r.contains("可以"), "missing 可以: {r}");
        assert!(r.contains("一次生成"), "missing 一次生成: {r}");
        assert!(r.contains("很多內容"), "missing 很多內容: {r}");
        assert!(r.contains("然後"), "missing 然後: {r}");
        assert!(r.contains("讓我打"), "missing 讓我打: {r}");
        assert!(r.contains("一次字"), "missing 一次字: {r}");
        assert!(r.contains("試試看"), "missing 試試看: {r}");
    }

    // ── Mass regression tests (auto-generated via scripts/test_gen.py) ──────────

    #[test]
    fn test_mass_jintian() {
        assert_eq!(
            rule().apply("rup wu0 wu0 fu4cp3cl3"),
            Some("今天天氣很好".into())
        );
    }
    #[test]
    fn test_mass_wuzhidao() {
        assert_eq!(
            rule().apply("ji31j45 2l4yp3ak7104"),
            Some("我不知道怎麼辦".into())
        );
    }
    #[test]
    fn test_mass_fuza() {
        assert_eq!(
            rule().apply("5k4ek4jp4wu6cp3zj4y86"),
            Some("這個問題很複雜".into())
        );
    }
    #[test]
    fn test_mass_mingtian() {
        assert_eq!(
            rule().apply("au/6wu0 ji3ap7d9 cjo4"),
            Some("明天我們開會".into())
        );
    }
    #[test]
    fn test_mass_juede() {
        assert_eq!(
            rule().apply("ji3rm,62k65k4u;4yji41u3rul4cl3"),
            Some("我覺得這樣做比較好".into())
        );
    }
    #[test]
    fn test_mass_xiexie() {
        assert_eq!(
            rule().apply("vu,4vu,4su32k71; a;6"),
            Some("謝謝你的幫忙".into())
        );
    }
    #[test]
    fn test_mass_buhao() {
        assert_eq!(
            rule().apply("1j4cl3u4n 283bl3su3"),
            Some("不好意思打擾你".into())
        );
    }
    #[test]
    fn test_mass_diannao() {
        assert_eq!(
            rule().apply("2u04sl3wj b062; 2ul4xk7"),
            Some("電腦突然當掉了".into())
        );
    }
    #[test]
    fn test_mass_xuyao() {
        assert_eq!(
            rule().apply("ji3vm ul4e/42ji g6ru0"),
            Some("我需要更多時間".into())
        );
    }
    #[test]
    fn test_mass_zaishuo() {
        assert_eq!(
            rule().apply("su3dk3u3y94gji u 1u04a87"),
            Some("你可以再說一遍嗎".into())
        );
    }
    #[test]
    fn test_mass_chuangyi() {
        assert_eq!(
            rule().apply("5k4ek4vu;3z83cp3u.3tj;4u4"),
            Some("這個想法很有創意".into())
        );
    }

    // Known limitations (same-syllable disambiguation requires context/LM):
    // 請問廁所在哪裡 → 裸→裏 (variant), 你吃飯了嗎 → 喫/犯,
    // 這個功能還沒實作 → 十/座, 這個版本有很多問題 → 板

    #[test]
    fn test_appendix_sentence() {
        // 這個放附錄吧
        assert_eq!(
            rule().apply("5k4ek7z;4zj4xj4187"),
            Some("這個放附錄吧".into())
        );
    }

    #[test]
    fn test_how_did_you_fix() {
        // 那所以你是怎麼改的?
        // Bugs fixed: ①旖旎(freq 896) greedily beat 以你 → added 以你 to supplement.
        // Note: 麼=ㄇㄜ˙ → ak7 (not ai7=ㄇㄛ˙).
        let result = rule().apply("s84nji3u3su3g4yp3ak7e932k7?");
        let r = result.unwrap();
        assert!(r.contains("那所"), "missing 那所: {r}");
        assert!(r.contains("以你"), "missing 以你 (was bug: 旖旎): {r}");
        assert!(r.contains("怎麼"), "missing 怎麼: {r}");
        assert!(r.contains("改的"), "missing 改的: {r}");
    }

    #[test]
    fn test_next_step() {
        // 我想知道下一步要做什麼?
        // Bug fixed: 下移(freq 760) greedily beat 下一步 → added 下一步 to supplement.
        // Note: 甚=ㄕㄣˊ → gp6; essay stores compound as 什麼 not 甚麼 (same word).
        let result = rule().apply("ji3vu;35 2l4vu84u61j4ul4yji4gp6ak7?");
        let r = result.unwrap();
        assert!(r.contains("我想"), "missing 我想: {r}");
        assert!(r.contains("知道"), "missing 知道: {r}");
        assert!(r.contains("下一步"), "missing 下一步 (was bug: 下移): {r}");
        assert!(r.contains("要做"), "missing 要做: {r}");
        assert!(r.contains("什麼"), "missing 什麼: {r}");
    }

    #[test]
    fn test_ssot_passthrough() {
        // 這是SSOT嗎? — ASCII passthrough in the middle of Chinese
        assert_eq!(rule().apply("5k4g4SSOTa87?"), Some("這是SSOT嗎?".into()));
    }

    #[test]
    fn test_no_problem_sentence() {
        // 每一項我都沒問題，
        // Note: 一 (tone-1) is keyed as "u" + space in daqian; 項=vu;4.
        let result = rule().apply("ao3u vu;4ji32. ao6jp4wu6，");
        let r = result.unwrap();
        assert!(r.contains("每"), "missing 每: {r}");
        assert!(r.contains("一項"), "missing 一項: {r}");
        assert!(r.contains("我都"), "missing 我都: {r}");
        assert!(r.contains("沒問題"), "missing 沒問題: {r}");
    }
}
