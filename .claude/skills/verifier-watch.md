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

Use the **PowerShell tool** (not Bash):

```powershell
cd "c:/Users/kwz50/IdeaProjects/Migao"
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

| Symptom | Likely cause |
|---------|-------------|
| `__NOT_REPLACED__` in clipboard | Hotkey didn't fire — daemon not running or wrong context |
| Timeout on window handle | Running from user's terminal instead of Claude Code |
| Build failed | Run `cargo build --bin migao-watch` manually to see errors |
