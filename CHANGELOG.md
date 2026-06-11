<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2026-06-11

- **Fixed:** After correcting `bopomofo-daqian` garbled text, `migao-watch` now switches the foreground window's IME back to Chinese (native) input mode via `ImmSetConversionStatus`, so the next keystroke is typed in Chinese rather than English. Resolves [#1](https://github.com/winterdrive/migao/issues/1).

## [0.5.0] - 2026-06-04

### Added

- **Check for Updates in tray menu** — right-clicking the system tray icon now includes a "Check for Updates" item that opens `github.com/winterdrive/migao/releases/latest` in the default browser
- **Report Issue in tray menu** — right-clicking the system tray icon now includes a "Report Issue" item that opens `github.com/winterdrive/migao/issues/new` in the default browser
- **English landing page** — `docs/en.html` mirrors the Chinese landing page with full English copy; language toggle in nav and hero on both pages
- **Migao skill** — `skills/migao/SKILL.md` documents the CLI interface for AI agents and external tooling

### Changed

- **Landing page links overhauled** — nav now links GitHub · Changelog · language toggle; hero now links Download · README · ⭐ Star on GitHub; removed duplicate and raw-text links
- **Project agent skills restructured** — development skills (`test-smoke`, `test-e2e`) moved to `.agents/skills/`; `.claude/` added to `.gitignore`

### Fixed

- **E2E test reliability** — `tests/integration/test-watch.ps1` now uses mouse-click focus (`WinFocus` via P/Invoke) and pre-loaded clipboard to avoid `WScript.Shell` focus-drift; Win11 Notepad window lookup no longer relies on launcher PID; `Add-Type` guarded against duplicate definitions across PowerShell sessions

## [0.4.9] - 2026-06-04

### Fixed

- **PowerShell hotkey appending bug** — in terminals (Windows Terminal / conhost), `Ctrl+C` copies the selected text but clears the selection; `Ctrl+V` then appended the correction instead of replacing. `migao-watch` now detects the foreground window class at hotkey time and uses `Ctrl+X` (cut) in terminal contexts (`CASCADIA_HOSTING_WINDOW_CLASS` / `ConsoleWindowClass`) so the original text is removed before pasting the correction. In all other applications (text editors, browsers, etc.) the original `Ctrl+C` path is preserved, keeping the undo stack clean.

### Added

- UI integration test for the hotkey replacement behaviour — `tests/integration/test-watch.ps1` covering terminal (ctrl_x path) and non-terminal (ctrl_c path) scenarios; run via Claude Code per `.claude/skills/verifier-watch.md`

## [0.4.8] - 2026-06-04

### Fixed

- Restore macOS Intel build: replace retired `macos-13` runner with `macos-15-intel` — `macos-13` was officially retired by GitHub on 2025-12-04; the correct standard Intel runner is `macos-15-intel`, allowing `x86_64-apple-darwin` binary to be included in releases again without cross-compilation

### Added

- Release-facing documentation structure: localized READMEs, GitHub Pages landing page, LLM-readable docs, robots/sitemap files, public docs assets, contributing guide, I18N notes, and MIT license text

## [0.4.7] - 2026-06-03

### Changed

- Installer welcome screen now uses a large `MIGAO` ASCII wordmark

## [0.4.6] - 2026-06-03

### Changed

- Installer welcome screen now uses a compact text header instead of the rice-cake ASCII art

## [0.4.5] - 2026-06-03

### Fixed

- Installer suppresses PowerShell web request progress bars so the Migao welcome art is not overwritten during downloads

## [0.4.4] - 2026-06-03

### Added

- Installer now shows a compact Migao rice-cake ASCII welcome screen
- Windows release package now includes `migao.ico`; Start Menu shortcuts use the Migao logo instead of the default Windows executable icon

## [0.4.3] - 2026-06-03

### Added

- Windows installer now creates a Start Menu shortcut: `Migao > Migao Watch`

### Changed

- `migao-watch.exe` is now built as a Windows GUI subsystem binary, so launching it does not require keeping a terminal window open
- Installer messaging now points users to the Start Menu shortcut when auto-start is disabled

## [0.4.2] - 2026-06-03

### Fixed

- Modifier keys (Ctrl, Alt) no longer get stuck after `migao-watch` hotkey processing; apps like VS Code no longer enter shortcut mode or show a crosshair cursor after correction
- Drop `x86_64-apple-darwin` (macOS Intel) from release matrix — `macos-13` GitHub runners are unavailable for extended periods; Apple Silicon covers 99 %+ of active Mac users, and Intel Macs can run the ARM binary via Rosetta

## [0.4.1] - 2026-06-03

### Changed

- Tray icons now loaded from `assets/tray_active_16x16.png` and `assets/tray_paused_16x16.png` via `include_bytes!` + `image` crate; replaces the 150-line hardcoded RGBA arrays — updating icons now requires only swapping the PNG files
- Add `image 0.25` (png feature) as a direct Windows dependency; was previously a hidden transitive dependency via `tray-icon`

### Fixed

- CI accuracy gate binary path was stale after the project rename — now correctly references `target/release/migao`

## [0.4.0] - 2026-06-03

### Changed

- **Project rebranded as Migao (翻譯米糕)**
  - Crate: `migao`
  - Binaries: `migao` / `migao-watch`
  - Registry key: `MigaoWatch`
  - Release archives: `migao-*`
  - All user-facing messages and CLI command name updated

## [0.3.7] - 2026-06-03

### Changed

- Tray icon redesigned from a plain circle to a **konjac block** (rounded rectangle with darker border and lighter speckle dots), referencing Doraemon's Translation Konjac gadget
- `make_icon(bool)` replaced by `make_icon(IconState)` enum — three distinct visual states:
  - `Active` (green) — normal listening
  - `Corrected` (teal) — lights up for the 4 s feedback window after a fix
  - `Paused` (grey) — hotkey suspended

## [0.3.6] - 2026-06-03

### Added

- `install.ps1 -Uninstall` switch — stops the running daemon, removes the `MigaoWatch` registry entry, deletes both executables, and strips `installDir` from `PATH` if the directory is then empty
- `install.ps1` now runs `Stop-MigaoWatch` before extracting, so reinstalling over a running daemon no longer fails with a file-in-use error
- `install.ps1` calls `Unblock-File` on both executables after extraction to prevent Windows SmartScreen from blocking them
- `install.ps1` prints the uninstall command in the summary footer
- `release.yml` release body updated: added `migao-watch` usage section, uninstall command, and corrected hotkey from `Ctrl+Shift+K` to `Ctrl+Alt+R`

## [0.3.5] - 2026-06-03

### Changed

- Cycle mode now also posts tray tooltip feedback: `Candidate N/M: preview` while stepping through alternatives, and `Reverted to original` when wrapping back to the unmodified text; same 4 s auto-reset as the initial correction tooltip

## [0.3.4] - 2026-06-03

### Added

- **Correction feedback via tray tooltip** — after each successful fix, the tray tooltip shows `✓ original → corrected` for 4 seconds then restores the normal hint; implemented with `WM_APP+1` + `SetTimer`/`WM_TIMER` (zero new dependencies)
- `truncate()` helper caps original and corrected text at 25 characters each so the tooltip stays readable for long selections

### Changed

- `handle_hotkey` now returns `Option<String>` (correction summary) instead of `()`; cycle presses still return `None` since the corrected text is the feedback

## [0.3.3] - 2026-06-03

### Added

- **Launch at Login** toggle in tray right-click menu — reads and writes `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run\MigaoWatch`; checkmark reflects current state; no restart required
- `install.ps1` now prompts the user before enabling auto-start (default: Yes); explains how to change the setting later via the tray menu; also fixes the displayed hotkey from `Ctrl+Shift+K` to `Ctrl+Alt+R`

## [0.3.2] - 2026-06-03

### Added

- **System tray icon** for `migao-watch` — persistent green circle in the Windows taskbar notification area; right-click reveals a context menu
- **Pause / Resume** tray menu item — suspends hotkey processing without stopping the daemon; icon turns grey while paused; tooltip updates to reflect state
- **Exit** tray menu item — cleanly unregisters the hotkey and removes the tray icon before exiting

## [0.3.1] - 2026-06-03

### Fixed

- `migao-watch` message loop blocked during hotkey handling — all sleeps, clipboard I/O, and dictionary decoding now run on a dedicated worker thread; the Windows message loop stays responsive at all times
- `migao-watch` missing `UnregisterHotKey` on exit — hotkey is now properly released so the daemon can be restarted without the "already in use" error

### Added

- Graceful shutdown for `migao-watch` via Ctrl+C, Ctrl+Break, or window close — posts WM_QUIT to the message loop, unregisters the hotkey, then exits cleanly
- Dictionary pre-warming in `migao-watch` — both dictionaries are loaded on daemon startup so the first hotkey press has no cold-start latency

### Changed

- `migao-watch` hotkey changed from Ctrl+Alt+K to Ctrl+Alt+R for single-hand ergonomics (left hand: Ctrl → Alt → R; works naturally after Ctrl+A selection)

## [0.3.0] - 2026-05-26

- **Added:** `migao fix --top N` flag — outputs up to N candidate recoveries; on TTY shows a numbered picker, in pipe mode prints one per line
- **Added:** `recover_top_n(input, ime, n)` public API for library consumers
- **Added:** `Rule::apply_top_n()` trait method with default single-result implementation; `bopomofo-daqian` provides full multi-candidate Viterbi substitution
- **Added:** `migao-watch` Windows daemon (Ctrl+Alt+K hotkey) — select garbled text, press hotkey to fix; press again within 3 s to cycle through alternative candidates; wraps back to first after exhausting all options
- **Added:** Neutral-tone hint propagation in Viterbi — ˙-syllables also try ˊ/ˇ/ˉ tone variants in compound span lookups, fixing words like 覺得 that are stored under non-ˋ dict keys
- **Fixed:** Cycling paste appended instead of replacing — previous candidate is re-selected via Shift+Left after each paste so the next cycle directly replaces the selection
- **Fixed:** Physical Ctrl+Alt modifier contamination — `GetAsyncKeyState` polling waits for keys to be fully released before injecting any synthetic events

## [0.2.0] - 2026-05-25

### Added

- Full Bopomofo dictionary (463,000+ entries) generated from RIME terra_pinyin + rime-essay via `scripts/build_dict.py`
- `--ime english-from-bopomofo` (alias: `reverse`) rule: converts Bopomofo Unicode symbols back to original English keystrokes
- `zhuyin_to_key()` function in `daqian` module — strict reverse mapping of the Daqian layout
- Case-insensitive `--ime` flag: `zhuyin`, `注音`, `BOPOMOFO-DAQIAN` all accepted
- CI pipeline (GitHub Actions) running tests on Ubuntu, Windows, macOS + clippy + rustfmt
- `scripts/build_dict.py` — reproducible dictionary build from upstream RIME sources

### Changed

- `main()` now returns `Result` instead of panicking on stdin errors
- `bopomofo-daqian` rule outputs Traditional Chinese text (dictionary lookup) instead of raw Bopomofo symbols
- Confidence scoring uses syllable structure validation (Initial→Medial→Final→Tone ordering) — English text now scores ~0.0
- Invalid syllables pass through as original key text in mixed Chinese/English input
- `Cargo.toml` updated with crates.io metadata (repository, keywords, categories, readme)
- License updated to `MIT AND Apache-2.0` to reflect RIME data attribution

### Fixed

- English text incorrectly scored high confidence (0.9+) due to character-counting approach
- Panics on stdin read failure replaced with proper error propagation

## [0.1.0] - 2026-05-01

### Added

- Initial implementation of `migao fix` command
- `bopomofo-daqian` rule: recovers text typed with 大千 Bopomofo layout active
- `migao list` command listing available IME rules
- Cross-platform support (Windows, macOS, Linux)
- Pipe mode: `echo "su3cl3" | migao fix`
- Embedded starter dictionary with common phrases
