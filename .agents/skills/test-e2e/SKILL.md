---
name: test-e2e
description: run the migao-watch E2E UI test — hotkey replacement in terminal and editor
---

# verifier-watch

Use this skill when asked to test, verify, or run the migao-watch integration test.

## Important: execution context

`tests/integration/test-watch.ps1` **must be run via Claude Code's PowerShell tool**,
not from the user's Windows Terminal.

Windows Terminal intercepts new console windows and leaves `MainWindowHandle = 0` on
child processes, breaking the window automation. Claude Code's extension-host context
runs in a separate conhost session where `Start-Process powershell.exe` creates real
windows the script can find and control.

## How to run

Use the **PowerShell tool** (not Bash). Ensure your working directory is the Migao project root (for example: `cd "$env:USERPROFILE/Migao"`), or run the script with a path relative to the repo root:

```powershell
cd "$env:USERPROFILE/Migao"
.\tests\integration\test-watch.ps1
```

## What the test covers

1. Builds `migao-watch` (debug)
2. Starts the daemon
3. **Test 1 — terminal (conhost):** opens a PowerShell window, types `su3cl3`,
   Ctrl+A, Ctrl+Alt+R → asserts clipboard = `你好`
   (verifies `is_terminal_foreground()` detection + `ctrl_x` cut path)
4. **Test 2 — non-terminal (Notepad):** same flow in Notepad → asserts clipboard = `你好`
   (verifies `ctrl_c` copy path, undo stack not polluted)
5. Stops the daemon, reports PASS / FAIL

After running the test, summarize the PASS/FAIL results for the user. If the test fails, include the likely cause from the Failure reference table.

## Expected output

```
migao-watch  UI integration tests
===================================

[setup] building migao-watch...
  build OK
[setup] starting migao-watch...
  daemon running (PID ...)

Test 1 — terminal (Windows Terminal / conhost)
  PASS  su3cl3 → 你好  (ctrl_x path — text replaced, not appended)

Test 2 — non-terminal (Notepad)
  PASS  su3cl3 → 你好  (ctrl_c path — undo stack intact)

[teardown] stopping migao-watch...

===================================
PASS  2/2 tests passed
```

## Failure reference

| Symptom | Likely cause | Action to take |
|---------|-------------|-----------------|
| `__NOT_REPLACED__` in clipboard | Hotkey didn't fire — daemon not running or wrong context | Ensure no orphaned daemon processes are lingering, then execute the test script again so it can start the daemon as intended. If the problem persists, capture the daemon process list and recent daemon logs and include them when reporting. |
| Timeout on window handle | Running from user's terminal instead of Claude Code | Confirm the test was executed by Claude Code's PowerShell tool (not Windows Terminal or bash). If it was executed in the correct tool and the timeout still occurs, collect environment diagnostics (which tool ran the script, relevant PIDs, and `MainWindowHandle` values) and include them when reporting. |
| Build failed | Run `cargo build --bin migao-watch` manually to see errors | Run the manual build command, capture the output, and share the build error output so the underlying compilation issue can be diagnosed. |
| Any other error or script hangs | Uncategorized failure or UI lockup | Cancel execution if the script is hanging, capture the last 20 lines of console output (and any relevant logs), and report the unknown failure to the user for further investigation. |
