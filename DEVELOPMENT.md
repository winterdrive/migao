# Migao — Development Guide

---

## Requirements

- Rust stable (1.75+)
- Windows 10/11 for building `migao-watch` (the daemon is Windows-only)
- Python 3.9+ (optional, for dictionary rebuilds via `scripts/`)

---

## Build

```sh
# All targets
cargo build

# Release build (stripped, LTO, size-optimised)
cargo build --release

# Windows daemon only
cargo build --bin migao-watch
```

---

## Test

```sh
# Full suite (52 unit tests across IME rules and Viterbi decoder)
cargo test

# Library tests only
cargo test --lib

# Single test by name
cargo test test_ni_hao
```

### Accuracy gate

The CI runs an accuracy regression script after each build:

```sh
python scripts/accuracy_gate.py --threshold 90
```

This script samples canonical garbled→correct pairs and fails if the recovery rate drops below 90 %.

---

## Lint & format

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
```

Both are required to pass before merging.

---

## Project structure

```
migao/
├── src/
│   ├── lib.rs                  # Public API: recover(), recover_top_n()
│   ├── main.rs                 # migao CLI (clap subcommands: fix, list)
│   ├── bin/
│   │   └── watch.rs            # migao-watch Windows daemon (tray icon, hotkey)
│   ├── dict.rs                 # Zhuyin dictionary (OnceLock, 474 k entries)
│   ├── pinyin_dict.rs          # Pinyin dictionary
│   ├── bigram.rs               # Bigram language model
│   ├── viterbi.rs              # Viterbi decoder + top-N candidate generation
│   ├── rule.rs                 # Rule trait: confidence(), apply(), apply_top_n()
│   ├── rules/
│   │   ├── mod.rs              # get_rule(ime: &str) dispatcher
│   │   ├── bopomofo.rs         # bopomofo-daqian rule (Daqian layout → 注音 → 漢字)
│   │   ├── english_from_bopomofo.rs  # reverse rule (注音 → original English keys)
│   │   └── pinyin.rs           # pinyin rule (全拼 → 漢字)
│   └── ime/
│       ├── daqian.rs           # Daqian key-to-Bopomofo mapping + segmenter
│       └── pinyin.rs           # Pinyin syllable table + segmenter
├── data/
│   ├── bopomofo.tsv            # Primary Zhuyin→word dictionary
│   ├── supplement.tsv          # Manually curated overrides / compound fixes
│   ├── pinyin.tsv              # Pinyin dictionary
│   └── bigram.tsv              # Bigram frequency table
├── assets/                     # Product/build assets used by Rust and release packaging
│   ├── tray_active_16x16.png    # Embedded into migao-watch.exe
│   ├── tray_paused_16x16.png    # Embedded into migao-watch.exe
│   └── migao.ico                # Packaged into Windows release zip for Start Menu shortcut
├── scripts/
│   ├── build_dict.py           # Rebuild dictionaries from upstream RIME sources
│   └── accuracy_gate.py        # Regression accuracy check
├── docs/
│   ├── index.html              # GitHub Pages landing page
│   ├── llms.txt                # Short LLM-readable project index
│   ├── llms-full.txt           # Full LLM-readable project context
│   ├── robots.txt
│   ├── sitemap.xml
│   ├── README.zh-TW.md         # Traditional Chinese README
│   ├── README.zh-CN.md         # Simplified Chinese README
│   ├── README.ja.md            # Japanese README
│   ├── README.ko.md            # Korean README
│   └── assets/                 # Public docs assets served by GitHub Pages
├── .github/
│   └── workflows/
│       ├── ci.yml              # Tests + clippy + fmt + accuracy gate (on PR)
│       └── release.yml         # Build all platforms + publish GitHub Release (on tag)
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
├── I18N.md
├── LICENSE                    # MIT license text
├── LICENSE-APACHE             # Apache 2.0 license text for dictionary components
├── ROADMAP.md
├── README.md
├── DEVELOPMENT.md              # This file
└── install.ps1                 # Windows one-liner installer / uninstaller
```

### Module responsibilities

| File | Role |
|------|------|
| `lib.rs` | Public API surface — `recover()` and `recover_top_n()` |
| `main.rs` | CLI entry point — argument parsing via `clap` |
| `bin/watch.rs` | Windows daemon — `RegisterHotKey`, tray icon, worker thread, clipboard I/O |
| `dict.rs` | Lazy-loaded (OnceLock) Zhuyin dict; `to_chinese()` and `to_chinese_candidates()` |
| `viterbi.rs` | DP decoder — `run_dp()` scores syllable sequences, `decode_candidates()` returns top-N |
| `rule.rs` | `Rule` trait — `confidence()`, `apply()`, `apply_top_n()` |
| `rules/bopomofo.rs` | Segments Daqian keys → Bopomofo → calls dict + Viterbi |

---

## Adding a new IME rule

1. Create `src/rules/my_ime.rs` implementing `Rule`:

```rust
use crate::rule::Rule;

pub struct MyImeRule;

impl Rule for MyImeRule {
    fn name(&self) -> &str { "my-ime" }
    fn confidence(&self, input: &str) -> f32 { /* 0.0–1.0 */ }
    fn apply(&self, input: &str) -> Option<String> { /* best recovery */ }
}
```

2. Register it in `src/rules/mod.rs`:

```rust
pub fn get_rule(ime: &str) -> Option<Box<dyn Rule>> {
    match ime.to_lowercase().as_str() {
        "my-ime" => Some(Box::new(my_ime::MyImeRule)),
        // …
    }
}
```

3. Add at least one integration test in `src/rules/my_ime.rs`.

---

## CI / Release flow

```
feature branch
      │
      ▼
Pull Request to main
      │
      ├── cargo test --all-features       (Ubuntu / Windows / macOS)
      ├── cargo clippy -- -D warnings     (Ubuntu)
      ├── cargo fmt --check               (Ubuntu)
      └── accuracy_gate.py --threshold 90 (Ubuntu)
      │
      ▼
merge to main
      │
      ▼
git tag vX.Y.Z && git push origin vX.Y.Z
      │
      ▼
release.yml builds three platform archives:
      ├── migao-windows-x64.zip   (migao.exe + migao-watch.exe + migao.ico)
      ├── migao-macos-arm64.tar.gz
      └── migao-linux-x64.tar.gz
      │
      ▼
GitHub Release published automatically with install instructions
```

### Pre-release checklist

```sh
# 1. Bump version in Cargo.toml
# 2. Update CHANGELOG.md
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release
```

### Version bump convention

| Change | Bump |
|--------|------|
| New IME rule, new CLI flag, new tray feature | `minor` |
| Bug fix, accuracy improvement, dependency update | `patch` |
| Breaking API or binary rename | `major` |

---

## Dictionary rebuild

If you need to regenerate the dictionary from upstream RIME sources:

```sh
pip install pypinyin opencc-python-reimplemented
python scripts/build_dict.py
```

This fetches `rime-terra-pinyin` and `rime-essay`, converts entries to Bopomofo, and writes `data/bopomofo.tsv` and `data/bigram.tsv`.

---

## Common issues

**`RegisterHotKey` fails on `migao-watch` startup**

Another application has already registered Ctrl+Alt+R globally. Identify it with Spy++ or AutoHotkey's `ListHotkeys`, or temporarily reassign the other app's shortcut.

**First hotkey press is slow (~500 ms)**

Dictionary pre-warming runs on the worker thread at startup. If you start `migao-watch` and immediately press Ctrl+Alt+R before pre-warming finishes, the first press will be slower. This is expected and resolves itself after the first few seconds.

**Clipboard access fails**

`arboard` initialises COM on the worker thread. If another application holds an exclusive clipboard lock, `clipboard.get_text()` returns an error and `migao-watch` silently skips that hotkey press. Retry the hotkey.
