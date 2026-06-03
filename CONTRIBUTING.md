# Contributing to Migao

Migao is a Rust CLI and Windows tray app for recovering text typed with the wrong IME.

## Before opening a PR

Run the checks that match your change:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python scripts/accuracy_gate.py --threshold 90
```

Windows tray changes should also be tested manually on Windows 10/11:

1. Install or run `migao-watch`.
2. Type a garbled Bopomofo sequence such as `su3cl3`.
3. Select the text, usually with `Ctrl+A`.
4. Press `Ctrl+Alt+R`.
5. Confirm the selected text is replaced and Ctrl/Alt do not remain stuck.

## Dictionary changes

Dictionary data lives in `data/`.

- `data/bopomofo.tsv` is the main Traditional Chinese dictionary.
- `data/bigram.tsv` is the bigram language model.
- `data/supplement.tsv` contains curated overrides and compound fixes.
- `data/pinyin.tsv` supports the pinyin rule.

If you regenerate dictionaries, document the upstream source and run the accuracy gate.

## Adding an IME rule

1. Add a rule module under `src/rules/`.
2. Implement the `Rule` trait.
3. Register it in `src/rules/mod.rs`.
4. Add focused tests for segmentation, confidence, and recovery output.
5. Update README, localized READMEs, and `docs/llms-full.txt`.

## Release notes

User-facing behavior changes should update `CHANGELOG.md`.

Installer or release workflow changes should also be reflected in `README.md`, `docs/README.zh-TW.md`, and the GitHub release body template in `.github/workflows/release.yml`.
