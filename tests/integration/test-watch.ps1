# tests/integration/test-watch.ps1
#
# Manual UI integration test for migao-watch hotkey behaviour.
#
# Requirements:
#   - Run from a STANDALONE PowerShell window (not VS Code integrated terminal)
#   - Windows desktop session (no headless / CI)
#
# Usage (from repo root):
#   .\tests\integration\test-watch.ps1
#
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
using System.Runtime.InteropServices;
public class WinApi {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
}
"@ -ErrorAction SilentlyContinue

function Wait-ForWindow {
    param([System.Diagnostics.Process]$Proc, [int]$TimeoutMs = 15000)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $p = Get-Process -Id $Proc.Id -ErrorAction SilentlyContinue
        if ($p -and $p.MainWindowHandle -ne [IntPtr]::Zero) {
            return $p.MainWindowHandle
        }
        Start-Sleep -Milliseconds 300
    }
    throw @"
Timeout: '$($Proc.ProcessName)' (PID $($Proc.Id)) never got a window handle.
Run this script from a standalone PowerShell window, not from VS Code's integrated terminal.
"@
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

    # Sentinel so we can detect if the hotkey never fired
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
Start-Sleep -Milliseconds 2000   # wait for hotkey registration

if (-not (Get-Process -Id $daemon.Id -ErrorAction SilentlyContinue)) {
    Write-Host "  failed to start migao-watch" -ForegroundColor Red
    exit 1
}
Write-Host "  daemon running (PID $($daemon.Id))" -ForegroundColor Green

# ── Test 1: terminal (conhost PowerShell) ─────────────────────────────────────

Write-Host ""
Write-Host "Test 1 — terminal (conhost PowerShell)" -ForegroundColor Cyan

$psProc = Start-Process powershell.exe `
    -ArgumentList "-NoExit","-NoProfile" `
    -WindowStyle Normal `
    -PassThru
try {
    $hwnd  = Wait-ForWindow $psProc
    $clip  = Invoke-HotkeyTest -Handle $hwnd -Input "su3cl3"
    Assert-Clip "su3cl3 → 你好  (ctrl_x path — text replaced, not appended)" $clip "你好"
} finally {
    Stop-Process -Id $psProc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 400
}

# ── Test 2: non-terminal (Notepad) ────────────────────────────────────────────

Write-Host ""
Write-Host "Test 2 — non-terminal (Notepad)" -ForegroundColor Cyan

$npProc = Start-Process notepad -WindowStyle Normal -PassThru
try {
    $hwnd  = Wait-ForWindow $npProc
    $clip  = Invoke-HotkeyTest -Handle $hwnd -Input "su3cl3"
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
