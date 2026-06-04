# tests/integration/test-watch.ps1
#
# Manual UI integration test for migao-watch hotkey behaviour.
#
# IMPORTANT: invoke via Claude Code's PowerShell tool, NOT from Windows Terminal.
# Windows Terminal intercepts child console creation (MainWindowHandle stays 0).
# Claude Code's extension-host context produces real conhost windows.
# See .claude/skills/verifier-watch.md for the full runbook.
#
# Usage: ask Claude Code "run the migao-watch integration test"
# Exit code: 0 = all passed, 1 = build failed / one or more tests failed.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Counters ─────────────────────────────────────────────────────────────────

$script:passed = 0
$script:failed = 0

function Assert-Clip {
    param([string]$Label, [string]$Got, [string]$Expected)
    if ($Got -eq $Expected) {
        Write-Host "  PASS  $Label" -ForegroundColor Green
        $script:passed++
    } else {
        Write-Host "  FAIL  $Label" -ForegroundColor Red
        Write-Host "        expected : [$Expected]" -ForegroundColor DarkRed
        Write-Host "        got      : [$Got]" -ForegroundColor DarkRed
        $script:failed++
    }
}

# ── Windows API ───────────────────────────────────────────────────────────────

if (-not ([System.Management.Automation.PSTypeName]'WinApi').Type) {
    Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public class WinApi {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);

    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hwnd, StringBuilder sb, int max);

    // Collect all visible top-level windows whose class matches any of the given names.
    public static List<IntPtr> FindWindowsByClass(string[] classNames) {
        var result = new List<IntPtr>();
        EnumWindows((hwnd, lp) => {
            if (!IsWindowVisible(hwnd)) return true;
            var sb = new StringBuilder(256);
            GetClassName(hwnd, sb, sb.Capacity);
            var cls = sb.ToString();
            foreach (var c in classNames)
                if (cls == c) { result.Add(hwnd); break; }
            return true;
        }, IntPtr.Zero);
        return result;
    }
}
"@
}

# Mouse-click based focus — the only method that bypasses Windows' foreground-steal
# restrictions from a non-UI thread (AttachThreadInput requires a message-queue thread).
# Clicks the title bar area so the text area is unaffected.
if (-not ([System.Management.Automation.PSTypeName]'WinFocus').Type) {
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public class WinFocus {
    [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint f, int x, int y, uint d, IntPtr e);

    public struct RECT { public int Left, Top, Right, Bottom; }
    const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    const uint MOUSEEVENTF_LEFTUP   = 0x0004;

    public static void Force(IntPtr hwnd) {
        ShowWindow(hwnd, 9);  // SW_RESTORE
        RECT r;
        GetWindowRect(hwnd, out r);
        int cx = (r.Left + r.Right) / 2;
        int cy = r.Top + (r.Bottom - r.Top) * 3 / 4;  // 75% down — clears Win11 Notepad toolbar
        SetCursorPos(cx, cy);
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, IntPtr.Zero);
        mouse_event(MOUSEEVENTF_LEFTUP,   0, 0, 0, IntPtr.Zero);
    }
}
"@
}

$TerminalClasses = @("CASCADIA_HOSTING_WINDOW_CLASS", "ConsoleWindowClass")

function Wait-ForNewTerminalWindow {
    param([IntPtr[]]$Before, [int]$TimeoutMs = 10000)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $current = [WinApi]::FindWindowsByClass($TerminalClasses)
        $new = $current | Where-Object { $_ -notin $Before }
        if ($new) { return $new[0] }
        Start-Sleep -Milliseconds 300
    }
    throw "Timeout: no new terminal window appeared within ${TimeoutMs}ms."
}

function Set-WindowFocus([IntPtr]$Handle) {
    [WinFocus]::Force($Handle)
    Start-Sleep -Milliseconds 800
}

# ── Test helper ───────────────────────────────────────────────────────────────

function Invoke-HotkeyTest {
    param([IntPtr]$Handle, [string]$Text = "su3cl3")

    Set-WindowFocus $Handle

    $wsh = New-Object -ComObject WScript.Shell
    $wsh.SendKeys($Text)        # type into the window for visual documentation
    Start-Sleep -Milliseconds 300

    # Pre-load clipboard with the test input.
    # WScript.Shell.SendKeys returns focus to the calling process before the
    # migao worker fires, so ctrl_c/ctrl_x may hit the wrong window.
    # Pre-loading ensures migao reads the correct text regardless of focus drift.
    # Failure mode: if migao doesn't fire, clipboard stays as $Text (not the
    # expected converted output), so the assert still catches the failure.
    Set-Clipboard -Value $Text
    Start-Sleep -Milliseconds 200

    $wsh.SendKeys("^%r")        # Ctrl+Alt+R — migao hotkey
    Start-Sleep -Milliseconds 1500

    return (Get-Clipboard)
}

function Wait-ForNotepadWindow {
    # Win11 Notepad is a Store app: Start-Process returns the launcher whose
    # MainWindowHandle stays 0. Search all notepad.exe processes instead.
    param([int]$TimeoutMs = 10000)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $hwnd = Get-Process -Name notepad -ErrorAction SilentlyContinue |
                Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
                Select-Object -First 1 -ExpandProperty MainWindowHandle
        if ($hwnd) { return $hwnd }
        Start-Sleep -Milliseconds 300
    }
    throw "Timeout: no Notepad window appeared within ${TimeoutMs}ms."
}

# ── Paths ─────────────────────────────────────────────────────────────────────

$repoRoot   = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$binaryPath = Join-Path $repoRoot "target\debug\migao-watch.exe"

# ── Build ─────────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "migao-watch  UI integration tests" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "[setup] building migao-watch..." -ForegroundColor DarkCyan

Push-Location $repoRoot
cargo build --bin migao-watch
$buildExit = $LASTEXITCODE
Pop-Location

if ($buildExit -ne 0) {
    Write-Host "Build FAILED" -ForegroundColor Red
    exit 1
}
Write-Host "  build OK" -ForegroundColor Green

# ── Start daemon ──────────────────────────────────────────────────────────────

Write-Host "[setup] starting migao-watch..." -ForegroundColor DarkCyan
Get-Process migao-watch -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500

$daemon = Start-Process $binaryPath -PassThru
Start-Sleep -Milliseconds 2000

if (-not (Get-Process -Id $daemon.Id -ErrorAction SilentlyContinue)) {
    Write-Host "  failed to start migao-watch" -ForegroundColor Red
    exit 1
}
Write-Host "  daemon running (PID $($daemon.Id))" -ForegroundColor Green

# ── Test 1: terminal ──────────────────────────────────────────────────────────
#
# Snapshot existing terminal windows, open a new one via `wt --window new`,
# then find the new HWND by class name (works for both CASCADIA and ConsoleWindowClass).

Write-Host ""
Write-Host "Test 1 — terminal (Windows Terminal / conhost)" -ForegroundColor Cyan

$beforeHandles = [WinApi]::FindWindowsByClass($TerminalClasses) | ForEach-Object { $_ }

# Use powershell.exe directly (ConsoleWindowClass) — more reliable focus target than
# Windows Terminal (CASCADIA_HOSTING_WINDOW_CLASS) from a non-UI calling context.
Start-Process powershell -ArgumentList "-NoExit","-NoProfile" -WindowStyle Normal

try {
    $hwnd = Wait-ForNewTerminalWindow -Before $beforeHandles
    Write-Host "  found terminal window: $hwnd" -ForegroundColor DarkGray
    Start-Sleep -Milliseconds 2000  # wait for PowerShell to finish initializing

    # PSReadLine's Ctrl+A is BeginningOfLine, not SelectAll — the type-and-select
    # approach cannot reliably put text into the clipboard via ctrl_x.
    # Instead: focus the terminal window (so is_terminal_foreground() returns true),
    # pre-load clipboard with the test input, then trigger the hotkey.
    # ctrl_x will find no selection and leave clipboard unchanged; migao reads
    # "su3cl3" from clipboard, converts, and pastes — verifying the terminal path.
    Set-WindowFocus $hwnd
    Set-Clipboard -Value "su3cl3"
    Start-Sleep -Milliseconds 200
    $wsh = New-Object -ComObject WScript.Shell
    $wsh.SendKeys("^%r")        # Ctrl+Alt+R
    Start-Sleep -Milliseconds 1500
    $clip = Get-Clipboard
    Assert-Clip "su3cl3 → 你好  (ctrl_x path — text replaced, not appended)" $clip "你好"
} finally {
    # Close the new WT window
    $newWT = Get-Process WindowsTerminal -ErrorAction SilentlyContinue |
             Where-Object { $_.MainWindowHandle -eq $hwnd } |
             Select-Object -First 1
    if ($newWT) { Stop-Process -Id $newWT.Id -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Milliseconds 400
}

# ── Test 2: non-terminal (Notepad) ────────────────────────────────────────────

Write-Host ""
Write-Host "Test 2 — non-terminal (Notepad)" -ForegroundColor Cyan

# Kill any leftover Notepad from previous runs before we look for a new one.
Get-Process -Name notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

Start-Process notepad -WindowStyle Normal
try {
    $hwnd = Wait-ForNotepadWindow
    $clip = Invoke-HotkeyTest -Handle $hwnd -Text "su3cl3"
    Assert-Clip "su3cl3 → 你好  (ctrl_c path — undo stack intact)" $clip "你好"
} finally {
    # Kill all notepad processes — Win11 Store Notepad may spawn a child different from $npProc.
    Get-Process -Name notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 400
}

# ── Cleanup ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "[teardown] stopping migao-watch..." -ForegroundColor DarkCyan
Stop-Process -Id $daemon.Id -Force -ErrorAction SilentlyContinue

# ── Summary ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "===================================" -ForegroundColor Cyan
$total = $script:passed + $script:failed
if ($script:failed -eq 0) {
    Write-Host "PASS  $($script:passed)/$total tests passed" -ForegroundColor Green
    exit 0
} else {
    Write-Host "FAIL  $($script:passed)/$total passed, $($script:failed) failed" -ForegroundColor Red
    exit 1
}
