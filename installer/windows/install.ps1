<#
.SYNOPSIS
    Automated DownSort Windows Installer (PowerShell)
.DESCRIPTION
    Installs DownSort to %LOCALAPPDATA%\Programs\DownSort, configures Start Menu
    and Desktop shortcuts, and registers an entry in Windows Add/Remove Programs.
#>

[CmdletBinding()]
param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$AppName = "DownSort"
$AppVersion = "0.1.0"
$Publisher = "DownSort"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$AppName"
$ExePath = Join-Path $InstallDir "downsort.exe"
$IconPath = Join-Path $InstallDir "downsort.ico"
$UninstallRegKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppName"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..")

function Remove-DownSort {
    Write-Host "Uninstalling $AppName..." -ForegroundColor Cyan
    
    # Kill running process if active
    Get-Process downsort -ErrorAction SilentlyContinue | Stop-Process -Force

    # Remove Desktop shortcut
    $DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "$AppName.lnk"
    if (Test-Path $DesktopShortcut) { Remove-Item $DesktopShortcut -Force }

    # Remove Start Menu shortcut
    $StartMenuDir = Join-Path ([Environment]::GetFolderPath("Programs")) $AppName
    if (Test-Path $StartMenuDir) { Remove-Item $StartMenuDir -Recurse -Force }

    # Remove registry uninstall entry
    if (Test-Path $UninstallRegKey) { Remove-Item $UninstallRegKey -Recurse -Force }

    # Remove install folder
    if (Test-Path $InstallDir) { Remove-Item $InstallDir -Recurse -Force }

    Write-Host "$AppName has been successfully uninstalled." -ForegroundColor Green
}

if ($Uninstall) {
    Remove-DownSort
    exit 0
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "       Installing $AppName v$AppVersion        " -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow

# Locate binaries
$SrcExe = Join-Path $ProjectRoot "target\release\downsort.exe"
$SrcIco = Join-Path $ProjectRoot "crates\app\downsort.ico"

if (-not (Test-Path $SrcExe)) {
    Write-Error "Could not find $SrcExe. Please run 'cargo build --release' first."
}

# Create installation directory
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item -Path $SrcExe -Destination $ExePath -Force
if (Test-Path $SrcIco) {
    Copy-Item -Path $SrcIco -Destination $IconPath -Force
}

# Create Shortcuts via WScript.Shell
$WshShell = New-Object -ComObject WScript.Shell

# 1. Desktop shortcut
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutDesktop = $WshShell.CreateShortcut((Join-Path $DesktopPath "$AppName.lnk"))
$ShortcutDesktop.TargetPath = $ExePath
$ShortcutDesktop.WorkingDirectory = $InstallDir
if (Test-Path $IconPath) { $ShortcutDesktop.IconLocation = "$IconPath,0" }
$ShortcutDesktop.Description = "DownSort - Downloads Folder Analyzer and Cleaner"
$ShortcutDesktop.Save()

# 2. Start Menu shortcut
$ProgramsDir = [Environment]::GetFolderPath("Programs")
$StartMenuDir = Join-Path $ProgramsDir $AppName
New-Item -ItemType Directory -Path $StartMenuDir -Force | Out-Null
$ShortcutStart = $WshShell.CreateShortcut((Join-Path $StartMenuDir "$AppName.lnk"))
$ShortcutStart.TargetPath = $ExePath
$ShortcutStart.WorkingDirectory = $InstallDir
if (Test-Path $IconPath) { $ShortcutStart.IconLocation = "$IconPath,0" }
$ShortcutStart.Description = "DownSort - Downloads Folder Analyzer and Cleaner"
$ShortcutStart.Save()

# 3. Register in Windows Add/Remove Programs
if (-not (Test-Path $UninstallRegKey)) {
    New-Item -Path $UninstallRegKey -Force | Out-Null
}
Set-ItemProperty -Path $UninstallRegKey -Name "DisplayName" -Value $AppName
Set-ItemProperty -Path $UninstallRegKey -Name "DisplayVersion" -Value $AppVersion
Set-ItemProperty -Path $UninstallRegKey -Name "Publisher" -Value $Publisher
Set-ItemProperty -Path $UninstallRegKey -Name "DisplayIcon" -Value "$IconPath,0"
Set-ItemProperty -Path $UninstallRegKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty -Path $UninstallRegKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$ScriptDir\install.ps1`" -Uninstall"

Write-Host "Installation completed successfully!" -ForegroundColor Green
Write-Host "Installed to: $InstallDir"
Write-Host "Created shortcuts on Desktop and in Start Menu."
