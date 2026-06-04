---
name: migao
description: "Recover garbled Traditional Chinese text caused by wrong IME — converts bopomofo/zhuyin or pinyin shorthand back to correct Chinese characters."
---

# Migao

Use `migao fix` when the user has typed with the wrong input method and got garbled text (e.g. `su3cl3`) instead of Chinese characters (e.g. `你好`).

## When to use

- User pastes garbled text that looks like random ASCII keys (e.g. `rup wu0 wu0 fu4cp3cl3`)
- User says they forgot to switch IME / input method
- User asks to convert bopomofo, zhuyin, or pinyin shorthand to Chinese

Do **not** use for: already-correct Chinese text, Japanese kana/kanji, Simplified Chinese (migao targets Traditional Chinese output only).

## Quick start

```bash
# Single correction (bopomofo-daqian by default)
migao fix "su3cl3"
# → 你好

# Pipe mode
echo "rup wu0 wu0 fu4cp3cl3" | migao fix
# → 今天天氣很好

# Explicit IME
migao fix --ime pinyin "nihao"
# → 你好

migao fix --ime zhuyin "su3cl3"
# → 你好  (zhuyin is an alias for bopomofo-daqian)

# Multiple candidates (prints one per line in pipe/non-TTY mode)
migao fix --top 3 "su3cl3"
```

## Supported IMEs

```bash
migao list
```

| Identifier | Aliases | Description |
|---|---|---|
| `bopomofo-daqian` | `zhuyin`, `注音` | 大千標準注音鍵盤 (default) |
| `pinyin` | `拼音` | 全拼（標準 QWERTY） |

## Gotchas

- **Pinyin requires `--ime pinyin`** — auto-detection defaults to bopomofo-daqian. `migao fix "nihao"` exits 1 because `nihao` is ambiguous with English.
- **Exit codes:** 0 = recovered successfully, 1 = input unrecognised or no candidates found.
- **`--top N` in TTY** shows an interactive picker; in pipe/non-TTY mode it prints one candidate per line. Always pipe when scripting.
- `migao-watch` is the Windows tray daemon for in-place hotkey replacement (Ctrl+Alt+R); it is separate from this CLI skill.

## Install

```powershell
# Windows (installs both migao and migao-watch)
irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1 | iex
```

```sh
# Cross-platform (CLI only)
cargo install migao
```
