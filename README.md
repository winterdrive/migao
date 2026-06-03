# 翻譯米糕 Migao

[繁體中文](./docs/README.zh-TW.md) | [日本語](./docs/README.ja.md) | [한국어](./docs/README.ko.md) | [简体中文](./docs/README.zh-CN.md) | English

<div align="center">
  <img src="docs/assets/migao-banner.png" alt="翻譯米糕 Migao" width="100%">
  <br>
  <strong>忘了切換注音？翻譯米糕 Ctrl+Alt+R 一秒還原。</strong>
</div>

<br>

**IME garbled text recovery for Traditional Chinese (Bopomofo / Zhuyin).**

Typed the right keys but forgot to switch your input method? You get gibberish like `su3cl3` instead of `你好`. Migao converts it back — instantly, in-place, without retyping.

```text
Type:   su3cl3
Select: Ctrl+A
Fix:    Ctrl+Alt+R
Result: 你好
```

---

## Install

### Windows — one-liner (recommended)

```powershell
irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1 | iex
```

Installs both `migao` (CLI) and `migao-watch` (background daemon), adds them to `PATH`, creates a Start Menu shortcut, and optionally registers `migao-watch` to run on login.

To uninstall:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall
```

### Cargo

```sh
cargo install migao
```

### Build from source

```sh
git clone https://github.com/winterdrive/migao.git
cd Migao
cargo build --release
# binaries: target/release/migao  target/release/migao-watch
```

---

## Windows Daily Use — Migao Watch

`migao-watch` runs silently in the system tray and fixes garbled text anywhere on Windows with a single hotkey.

### Fix selected text in-place

1. Launch **Migao Watch** from Start Menu, or let it start at login.
2. Type normally. If you forgot to switch IME and produced garbled text, select it. In many editors, **Ctrl+A** selects the whole field.
3. Press **Ctrl+Alt+R**. The selected text is replaced with the correct Chinese text in-place.
4. Press **Ctrl+Alt+R** again within 3 seconds to cycle through alternative candidates.

Manual launch does not require keeping a terminal window open.

### Tray icon

A pixel-art rice cake appears in the taskbar notification area:

| Icon | Meaning |
|------|---------|
| Coloured | Active — hotkey listening |
| Grey | Paused — hotkey suspended |

Right-click for the context menu:

- **Pause / Resume** — suspend or re-enable the hotkey without exiting
- **Launch at Login** — toggle Windows auto-start (writes to `HKCU\...\Run`)
- **Exit** — unregisters the hotkey and removes the tray icon cleanly

### Tray tooltip feedback

After each correction the tooltip briefly shows what changed:

```text
✓  su3cl3 → 你好
```

While cycling through candidates:

```text
Candidate 2/3: 你好嗎
Reverted to original
```

---

## CLI — `migao`

### Fix garbled text

```sh
migao fix "su3cl3"           # 你好
migao fix "5j/ eji6"         # 中國
migao fix "ji3vu j;4su3dk3"  # 我希望你可以
```

### Pipe from clipboard or stdin

```sh
echo "su3cl3" | migao fix
pbpaste | migao fix      # macOS
```

### Multiple candidates

```sh
migao fix --top 3 "su3cl3"
# 1  你好
# 2  你號
# 3  你好嗎
# Pick [1-3] (default 1):
```

### Specify IME layout

```sh
migao fix --ime zhuyin "su3cl3"
migao fix --ime pinyin "nihao"
```

### List supported IMEs

```sh
migao list
```

```text
Supported IME identifiers:
  bopomofo-daqian  (aliases: zhuyin, 注音)  — 大千標準注音鍵盤
  pinyin           (alias: 拼音)             — 全拼（標準 QWERTY）
```

## How it works

```text
Raw key sequence
      │
      ▼
  Segment          Split into syllable candidates using Daqian ordering rules
      │             (Initial → Medial → Final → Tone). Non-Bopomofo chars pass through.
      ▼
  Decode           Each syllable → Bopomofo Unicode  (su3 → ㄋㄧˇ)
      │
      ▼
  Viterbi          DP over syllables finds the highest-probability Chinese word sequence
      │             using a 474,000-entry dictionary + bigram language model
      ▼
  Output           Traditional Chinese text
```

A **confidence score** (ratio of structurally valid Bopomofo syllables) gates the output — English text scores near 0.0 and is rejected automatically.

---

## Supported IMEs

| Identifier | Aliases | Layout |
|------------|---------|--------|
| `bopomofo-daqian` | `zhuyin`, `注音` | 大千標準注音鍵盤 (Standard Daqian) |
| `pinyin` | `拼音` | 全拼 QWERTY |

---

## Exit codes (CLI)

| Code | Meaning |
|------|---------|
| 0 | Recovery succeeded |
| 1 | No rule matched / confidence too low |

---

## Library usage

`migao` is also a Rust library:

```toml
[dependencies]
migao = "0.4"
```

```rust
// Best single recovery
let result = migao::recover("su3cl3", "bopomofo-daqian");
assert_eq!(result.unwrap(), "你好");

// Top-N candidates
let candidates = migao::recover_top_n("su3cl3", "bopomofo-daqian", 3);
```

---

## Dictionary sources

| Source | License |
|--------|---------|
| [RIME terra_pinyin](https://github.com/rime/rime-terra-pinyin) | Apache 2.0 |
| [rime-essay](https://github.com/rime/rime-essay) | Apache 2.0 |
| [pypinyin](https://github.com/mozillazg/python-pinyin) | MIT |

---

## License

MIT AND Apache-2.0

The Apache 2.0 component covers the RIME dictionary data embedded in the binary (`data/bopomofo.tsv`).
