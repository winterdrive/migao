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
"@ -ErrorAction SilentlyContinue

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
    [WinApi]::ShowWindow($Handle, 9) | Out-Null   # SW_RESTORE
    [WinApi]::SetForegroundWindow($Handle) | Out-Null
    Start-Sleep -Milliseconds 600
}

# ── Test helper ───────────────────────────────────────────────────────────────

function Invoke-HotkeyTest {
    param([IntPtr]$Handle, [string]$Input = "su3cl3")

    Set-WindowFocus $Handle

    # Sentinel — lets us detect if the hotkey never fired
    Set-Clipboard -Value "__NOT_REPLACED__"

    $wsh = New-Object -ComObject WScript.Shell
    $wsh.SendKeys($Input)
    Start-Sleep -Milliseconds 400
    $wsh.SendKeys("^a")         # select all
    Start-Sleep -Milliseconds 400
    $wsh.SendKeys("^%r")        # Ctrl+Alt+R — migao hotkey
    Start-Sleep -Milliseconds 1500

    return (Get-Clipboard)
}

function Wait-ForProcessWindow {
    param([System.Diagnostics.Process]$Proc, [int]$TimeoutMs = 10000)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $Proc.Refresh()
        if ($Proc.MainWindowHandle -ne [IntPtr]::Zero) { return $Proc.MainWindowHandle }
        Start-Sleep -Milliseconds 300
    }
    throw "Timeout: '$($Proc.ProcessName)' (PID $($Proc.Id)) never got a window handle."
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

# --window new forces a separate WT window instead of a new tab in the current window
Start-Process wt.exe -ArgumentList "--window","new","powershell","-NoExit","-NoProfile"

try {
    $hwnd = Wait-ForNewTerminalWindow -Before $beforeHandles
    Write-Host "  found terminal window: $hwnd" -ForegroundColor DarkGray
    $clip = Invoke-HotkeyTest -Handle $hwnd -Input "su3cl3"
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

$npProc = Start-Process notepad -WindowStyle Normal -PassThru
try {
    $hwnd = Wait-ForProcessWindow $npProc
    $clip = Invoke-HotkeyTest -Handle $hwnd -Input "su3cl3"
    Assert-Clip "su3cl3 → 你好  (ctrl_c path — undo stack intact)" $clip "你好"
} finally {
    Stop-Process -Id $npProc.Id -Force -ErrorAction SilentlyContinue
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
