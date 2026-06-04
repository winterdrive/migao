---
name: test-smoke
description: run the migao CLI smoke test — build, fix, pipe, list, exit codes
---

# run-migao

Migao has two binaries:
- `migao` — CLI tool; stdin/args in, corrected text out. Primary programmable surface.
- `migao-watch` — Windows tray daemon; hotkey-driven, tested separately via `test-e2e`.

The CLI driver is `.agents/skills/test-smoke/smoke.sh`. Run it with Bash tool.
For `migao-watch` UI tests see `.agents/skills/test-e2e/SKILL.md` (use PowerShell tool).

## Prerequisites

Rust toolchain (stable). No other system dependencies.

```bash
rustup show   # verify stable toolchain present
```

## Build

```bash
# CLI only (fast)
cargo build --bin migao

# Both binaries
cargo build --bin migao --bin migao-watch

# Release
cargo build --release --bin migao --bin migao-watch
```

Debug binaries land in `target/debug/`, release in `target/release/`.

## Run — agent path (smoke script)

```bash
bash .agents/skills/test-smoke/smoke.sh [binary]
# default binary: ./target/debug/migao
```

Runs 8 representative cases and exits 0/1:

| Case | Input | Expected |
|------|-------|----------|
| bopomofo basic | `su3cl3` | `你好` |
| bopomofo sentence | `rup wu0 wu0 fu4cp3cl3` | `今天天氣很好` |
| stdin pipe | `echo "su3cl3" \| migao fix` | `你好` |
| top-N first | `--top 3 su3cl3` | `你好` (line 1) |
| pinyin | `--ime pinyin nihao` | `你好` |
| explicit zhuyin alias | `--ime zhuyin su3cl3` | `你好` |
| unrecognised input | `hello world` | exit 1 |
| list command | `migao list` | contains `bopomofo-daqian` |

## Run — direct invocation

```bash
# Single correction
./target/debug/migao fix "su3cl3"
# → 你好

# Multiple candidates (interactive picker on TTY, one-per-line in pipe)
./target/debug/migao fix --top 3 "su3cl3"

# Explicit IME
./target/debug/migao fix --ime pinyin "nihao"

# List supported IMEs
./target/debug/migao list

# Pipe mode
echo "su3cl3" | ./target/debug/migao fix
```

## Tests

```bash
cargo test
```

## Gotchas

- `migao fix "nihao"` exits 1 — auto-detection uses bopomofo-daqian; pinyin overlaps
  too much with English. Pass `--ime pinyin` explicitly for pinyin input.
- `migao-watch` is Windows-only and requires a real desktop session; running the binary
  on Linux/macOS prints an error and exits. The CLI (`migao fix`) is cross-platform.
- `--top N` in TTY mode shows an interactive numbered picker; in pipe mode it prints one
  candidate per line. Scripts should pipe to avoid the prompt.
