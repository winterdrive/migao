# Migao Roadmap

## UX Lifecycle Analysis — `migao-watch`

Before prioritising features, we mapped the full user journey: **install → first use → daily use → close → reopen**. The gaps below are ordered by impact.

---

### 1. Install

| Step | Current experience | Problem |
|------|--------------------|---------|
| Download `migao-watch.exe` | ✅ | — |
| Double-click to launch | Black console window flashes and disappears | No indication the daemon is running |
| Confirm it's active | No tray icon, no notification, no feedback | User has no way to know it worked |
| Set up auto-start on boot | Not possible | User must remember to re-launch every session |

---

### 2. First use

| Step | Current experience | Problem |
|------|--------------------|---------|
| Type garbled text | ✅ | — |
| Select all (Ctrl+A) | ✅ | — |
| Press Ctrl+Alt+R | Text is replaced silently | No visual confirmation the tool acted |
| Result is wrong / unwanted | Press Ctrl+Alt+R again to cycle | User doesn't know how many candidates exist or where they are in the cycle |
| Abandon correction | Keep pressing until original text comes back | No "cancel" gesture; cycle count is invisible |

---

### 3. Daily use (ongoing)

| Step | Current experience | Problem |
|------|--------------------|---------|
| Check if daemon is running | Must open Task Manager and look for `migao-watch.exe` | No ambient indicator |
| See correction history | Not available | Can't review what was changed |
| Pause/disable temporarily | Must kill the process | No toggle |

---

### 4. Close

| Step | Current experience | Problem |
|------|--------------------|---------|
| Close the tool | Must kill via Task Manager | No obvious exit point |
| Ctrl+C in console (if open) | ✅ Works after v0.3.1 | Only works if the console window is visible |

---

### 5. Reopen

| Step | Current experience | Problem |
|------|--------------------|---------|
| After reboot | Must manually re-launch | No auto-start |
| After accidental close | Same as above | No self-healing |

---

## Prioritised Action Items

### P0 — Minimum viable UX

- [x] **System tray icon** — pixel-art rice cake icon in Windows system tray; right-click menu (Pause / Resume, Launch at Login, Exit); green = active, grey = paused.

### P1 — Makes the tool feel trustworthy

- [x] **Windows startup registration** — "Launch at Login" `CheckMenuItem` reads/writes `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run\MigaoWatch`. No installer or elevated permissions required.
- [x] **Correction feedback** — tray tooltip shows `✓ original → corrected` for 4 s after each fix; cycle presses show `Candidate 2/3: 你好` / `Reverted to original`.

### P2 — Quality of life

- [x] **Cycle progress indicator** — tray tooltip shows candidate position while cycling.
- [x] **One-click installer / uninstaller** — `install.ps1 -Uninstall` cleanly stops the daemon, removes registry entry, deletes executables, and strips `installDir` from PATH if empty; `Unblock-File` prevents SmartScreen blocking.
- [ ] **Undo last correction** — dedicated hotkey or tray menu item that restores the original text without cycling through all candidates.

### P3 — Nice to have

- [ ] **Configurable hotkey** — stored in `%APPDATA%\Migao\config.toml`; changeable from tray menu without restart.
- [ ] **Correction log** — rolling log of the last N corrections viewable from the tray menu.
- [ ] **Korean 두벌식 IME rule** — highest-priority language expansion (see analysis below).
- [ ] **Pinyin accuracy improvement** — `bopomofo-daqian` has 90 %+ accuracy; `pinyin` rule needs similar Viterbi tuning for Simplified Chinese users.
- [ ] **macOS / Linux port** — equivalent background agent using platform-native APIs (Accessibility API on macOS, X11/Wayland on Linux).
- [ ] **Personalised selection memory** — record which candidate the user chose for each garbled input and bias future corrections toward their historical preference. Design considerations: (1) privacy — local-only storage under `%APPDATA%\Migao\history.db`, never transmitted; (2) architecture — a lightweight frequency table keyed on garbled input → chosen candidate sits alongside the existing Viterbi output and re-ranks the top-N list; (3) cold-start — falls back to Viterbi ranking until enough history exists (suggested threshold: 3 confirmations per input pattern).

---

## Language Expansion Analysis

Migao's recovery technique works when: (1) keyboard keys map deterministically to IME input, and (2) the garbled output is structurally distinguishable from plain English.

| Language | IME type | Garbled = English? | Deterministic mapping | Demand | Priority |
|----------|----------|-------------------|-----------------------|--------|----------|
| 繁中 注音 (Bopomofo) | Daqian key sequence | No — structurally distinct | ✅ Yes | ★★★★★ | **Shipped** |
| 簡中 拼音 (Pinyin) | Latin romanisation | Partially — low confidence | ✅ Yes | ★★★★ | Needs tuning |
| 韓文 두벌식 (Dubeolsik) | Fixed consonant/vowel keys | No — structurally distinct | ✅ Yes | ★★★★ | **Top next target** |
| 韓文 세벌식 (Sebeolsik) | Fixed key layout | No | ✅ Yes | ★★ | Low (minority layout) |
| 日文 かな直打 | Fixed kana keys | No | ✅ Yes | ★★ | Low (minority input method) |
| 日文 Romaji | Latin romanisation | Yes — indistinguishable | ❌ No | — | Not feasible |

### Why Korean (두벌식) is the top next target

- Each key maps to exactly one consonant (자음) or vowel (모음) — the same deterministic structure as Bopomofo.
- Garbled output like `gksrnrdl` is structurally unlike English and easily detected.
- Demand is documented: Korean users already use third-party tools for this ("한글 IME 오타 수정").
- Implementation mirrors `bopomofo.rs`: key→jamo mapping + syllable segmenter + dictionary lookup.

### Why Pinyin needs different treatment

Pinyin romanisation (`nihao` for 你好) overlaps heavily with English (`hi`, `la`, `ma`). The confidence gate must be tuned more aggressively — short inputs are unreliable. Long sentences (5+ syllables) work well; 1–2 syllable inputs should be rejected unless confidence is very high.

### Why Japanese Romaji is not feasible

`nihao` in Romaji IME mode is just the literal string `nihao` — no transformation occurs. There is nothing to reverse. Japanese Romaji users who forget to switch IME simply see their English keystrokes as-is.

---

## Implementation notes

- **P2 undo**: simplest implementation is a second `Ctrl+Z` hotkey that fires `ctrl_z()` once; needs a flag to track whether the last action was a migao correction.
- **P3 configurable hotkey**: `UnregisterHotKey` → update config → `RegisterHotKey`; store in `%APPDATA%\Migao\config.toml` via the `dirs` crate.
- **P3 Korean rule**: add `src/rules/korean_dubeolsik.rs`; key table is ~40 entries; syllable segmenter follows the Unicode Hangul composition algorithm (Initial + Medial + Final → precomposed block).
- **P3 macOS port**: `migao-watch` equivalent using `CGEventTap` (requires Accessibility permission); clipboard via `NSPasteboard`; tray via `objc`/`cocoa` crates.
