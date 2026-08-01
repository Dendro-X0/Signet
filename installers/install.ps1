# Signet CLI installer (Windows PowerShell)
#   irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "Dendro-X0/Signet"
$Base = "https://github.com/$Repo/releases/latest/download"
$Root = Join-Path $env:LOCALAPPDATA "Signet"
$BinDir = Join-Path $Root "bin"
$Bin = Join-Path $BinDir "signet.exe"
$Asset = "signet-x86_64-pc-windows-msvc.exe"

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "signet-installer" }
$Tag = $release.tag_name
$Version = $Tag.TrimStart("v")

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$Tmp = Join-Path $env:TEMP "signet-install.exe"
Write-Host "Downloading $Asset ($Tag)…"
Invoke-WebRequest -Uri "$Base/$Asset" -OutFile $Tmp -UseBasicParsing
Move-Item -Force $Tmp $Bin

$Receipt = @"
# Signet CLI install receipt — do not edit
method = "installer"
repo = "$Repo"
installed_version = "$Version"
binary_path = '$Bin'
"@
Set-Content -Path (Join-Path $Root "install.toml") -Value $Receipt -Encoding UTF8

# User PATH — always put Signet bin first so it beats ~/.cargo/bin when both exist.
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $userPath) { $userPath = "" }
$parts = @($userPath -split ";" | Where-Object { $_ -and $_.Trim() -ne "" -and $_.TrimEnd('\') -ne $BinDir.TrimEnd('\') })
$newPath = (@($BinDir) + $parts) -join ";"
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")
$env:Path = "$BinDir;$env:Path"
Write-Host "User PATH: $BinDir is first (new terminals pick this up)."

Write-Host "Installed to $Bin"
try {
    $installedVer = & $Bin --version 2>$null
    if ($installedVer) { Write-Host "Managed binary: $installedVer" }
} catch {}

# Warn when another signet still wins on PATH (common: cargo install).
$shadow = $null
try {
    $cmd = Get-Command signet -ErrorAction SilentlyContinue
    if ($cmd -and $cmd.Source) {
        $resolved = [System.IO.Path]::GetFullPath($cmd.Source)
        $ours = [System.IO.Path]::GetFullPath($Bin)
        if ($resolved -ne $ours) {
            $shadow = $resolved
        }
    }
} catch {}

if ($shadow) {
    Write-Host ""
    Write-Host "WARNING: ``signet`` on PATH is not the installer binary:" -ForegroundColor Yellow
    Write-Host "  PATH resolves to: $shadow"
    Write-Host "  Installer binary: $Bin"
    if ($shadow -match '[\\/]\.cargo[\\/]bin[\\/]') {
        Write-Host "  This looks like a cargo install. Fix with:"
        Write-Host "    cargo uninstall signet"
        Write-Host "  Or remove that file, then open a new terminal."
    } else {
        Write-Host "  Remove or rename the shadowed binary, or put $BinDir earlier on PATH."
    }
    Write-Host "  Verify managed build:  & '$Bin' --version"
} else {
    Write-Host ""
    Write-Host "Then run:  signet --version   (open a new terminal if needed)"
}

Write-Host "Update:    signet self update"
Write-Host "Uninstall: signet self uninstall --yes"
Write-Host "Or open:   signet   (TUI → Update / Uninstall Signet)"
