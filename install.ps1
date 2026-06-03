#!/usr/bin/env pwsh
# Migao Windows Installer / Uninstaller
#
# Install:   irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1 | iex
# Uninstall: & ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall
#            (or: .\install.ps1 -Uninstall)

param([switch]$Uninstall)

$ErrorActionPreference = "Stop"
$repo        = "winterdrive/migao"
$installDir  = "$env:USERPROFILE\.local\bin"
$regRunPath  = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"
$regKeyName  = "MigaoWatch"
$watchExe    = Join-Path $installDir "migao-watch.exe"
$migaoExe   = Join-Path $installDir "migao.exe"
$iconPath   = Join-Path $installDir "migao.ico"
$startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Migao"
$shortcutPath = Join-Path $startMenuDir "Migao Watch.lnk"

# ── Shared helper ─────────────────────────────────────────────────────────────

function Stop-MigaoWatch {
    $procs = Get-Process -Name "migao-watch" -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "Stopping running migao-watch..." -ForegroundColor Yellow
        $procs | Stop-Process -Force
        Start-Sleep -Milliseconds 500
    }
}

function Show-MigaoWelcome {
    $banner = @'
 __  __ ___ ____    _    ___
|  \/  |_ _/ ___|  / \  / _ \
| |\/| || | |  _  / _ \| | | |
| |  | || | |_| |/ ___ \ |_| |
|_|  |_|___\____/_/   \_\___/
'@
    Write-Host ""
    Write-Host $banner -ForegroundColor Cyan
    Write-Host "CLI + tray hotkey for fixing text typed with the wrong IME" -ForegroundColor DarkGray
    Write-Host ""
}

function New-MigaoShortcut {
    if (-not (Test-Path $watchExe)) {
        return
    }

    New-Item -ItemType Directory -Force $startMenuDir | Out-Null

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $watchExe
    $shortcut.WorkingDirectory = $installDir
    if (Test-Path $iconPath) {
        $shortcut.IconLocation = "$iconPath,0"
    } else {
        $shortcut.IconLocation = "$watchExe,0"
    }
    $shortcut.Description = "Launch Migao Watch in the system tray"
    $shortcut.Save()

    Write-Host "Created Start Menu shortcut: Migao > Migao Watch" -ForegroundColor Green
}

function Remove-MigaoShortcut {
    if (Test-Path $shortcutPath) {
        Remove-Item $shortcutPath -Force
        Write-Host "Removed Start Menu shortcut." -ForegroundColor Green
    }

    $remaining = Get-ChildItem $startMenuDir -ErrorAction SilentlyContinue
    if (-not $remaining) {
        Remove-Item $startMenuDir -Force -ErrorAction SilentlyContinue
    }
}

# ── Uninstall ─────────────────────────────────────────────────────────────────

if ($Uninstall) {
    Write-Host "Uninstalling Migao..." -ForegroundColor Cyan

    Stop-MigaoWatch
    Remove-MigaoShortcut

    # Remove auto-start registry entry
    if (Get-ItemProperty -Path $regRunPath -Name $regKeyName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $regRunPath -Name $regKeyName -Force
        Write-Host "Removed auto-start entry." -ForegroundColor Green
    }

    # Remove executables
    foreach ($file in @($watchExe, $migaoExe, $iconPath)) {
        if (Test-Path $file) {
            Remove-Item $file -Force
            Write-Host "Removed $file" -ForegroundColor Green
        }
    }

    # Remove installDir from PATH if it is now empty or contains only Migao files
    $remaining = Get-ChildItem $installDir -ErrorAction SilentlyContinue
    if (-not $remaining) {
        Remove-Item $installDir -Force -ErrorAction SilentlyContinue
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        $newPath  = ($userPath -split ";" | Where-Object { $_ -ne $installDir }) -join ";"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Host "Removed $installDir from PATH." -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "Migao uninstalled." -ForegroundColor Green
    exit 0
}

# ── Install ───────────────────────────────────────────────────────────────────

Write-Host "Installing Migao..." -ForegroundColor Cyan
Show-MigaoWelcome

# Stop any running daemon before overwriting its exe
Stop-MigaoWatch

# Fetch latest release. PowerShell's built-in progress bar overwrites the
# welcome art, so keep downloads quiet and use explicit status lines instead.
$previousProgressPreference = $ProgressPreference
$ProgressPreference = "SilentlyContinue"
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
} finally {
    $ProgressPreference = $previousProgressPreference
}
$asset = $release.assets | Where-Object { $_.name -like "*windows*" } | Select-Object -First 1

if (-not $asset) {
    Write-Error "No Windows release found. Check https://github.com/$repo/releases"
    exit 1
}

Write-Host "Downloading $($asset.name) ($($release.tag_name))..." -ForegroundColor Yellow
$tmpZip = Join-Path $env:TEMP "migao-install.zip"
$previousProgressPreference = $ProgressPreference
$ProgressPreference = "SilentlyContinue"
try {
    Invoke-WebRequest $asset.browser_download_url -OutFile $tmpZip
} finally {
    $ProgressPreference = $previousProgressPreference
}

New-Item -ItemType Directory -Force $installDir | Out-Null
Expand-Archive $tmpZip $installDir -Force
Remove-Item $tmpZip

# Unblock executables so Windows SmartScreen does not prevent them from running
foreach ($exe in @($watchExe, $migaoExe)) {
    if (Test-Path $exe) { Unblock-File $exe }
}

# Add installDir to PATH if missing
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    $env:PATH = "$env:PATH;$installDir"
    Write-Host "Added $installDir to PATH." -ForegroundColor Yellow
}

# ── migao-watch daemon setup ─────────────────────────────────────────────────

if (Test-Path $watchExe) {
    Write-Host ""
    New-MigaoShortcut

    # Ask about auto-start (default: Yes)
    $answer    = Read-Host "Launch migao-watch automatically at login? [Y/n]"
    $autoStart = ($answer -eq "" -or $answer -match "^[Yy]")

    if ($autoStart) {
        Set-ItemProperty -Path $regRunPath -Name $regKeyName -Value "`"$watchExe`"" -Force
        Write-Host "Auto-start enabled." -ForegroundColor Green
        Write-Host "(Change this later: right-click the tray icon → Launch at Login)" -ForegroundColor DarkGray
    } else {
        # Clean up any previous auto-start entry (e.g. from a prior install)
        if (Get-ItemProperty -Path $regRunPath -Name $regKeyName -ErrorAction SilentlyContinue) {
            Remove-ItemProperty -Path $regRunPath -Name $regKeyName -Force
        }
        Write-Host "Auto-start skipped. Start later from Start Menu: Migao > Migao Watch." -ForegroundColor Yellow
        Write-Host "(Enable later: right-click the tray icon → Launch at Login)" -ForegroundColor DarkGray
    }

    # Start the daemon now
    Start-Process $watchExe -WindowStyle Hidden
    Write-Host "migao-watch started." -ForegroundColor Green
}

# ── Summary ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "Migao $($release.tag_name) installed!" -ForegroundColor Green
Write-Host ""
Write-Host "  CLI      migao fix `"su3cl3`"  →  你好" -ForegroundColor Cyan
Write-Host "  Hotkey   Select garbled text → Ctrl+Alt+R → fixed!" -ForegroundColor Cyan
Write-Host "  Tray     Right-click taskbar icon: Pause · Launch at Login · Exit" -ForegroundColor Cyan
Write-Host "  Run later Start Menu → Migao → Migao Watch" -ForegroundColor Cyan
Write-Host ""
Write-Host "  To uninstall: & ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall" -ForegroundColor DarkGray
