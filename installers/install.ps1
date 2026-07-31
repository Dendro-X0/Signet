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

# User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $userPath) { $userPath = "" }
$parts = $userPath -split ";" | Where-Object { $_ -and $_.Trim() -ne "" }
if ($parts -notcontains $BinDir) {
    $newPath = if ($userPath.Trim() -eq "") { $BinDir } else { "$BinDir;$userPath" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$BinDir;$env:Path"
    Write-Host "Added $BinDir to your user PATH (new terminals pick this up)."
}

Write-Host "Installed to $Bin"
Write-Host ""
Write-Host "Then run:  signet --version"
Write-Host "Update:    signet self update"
Write-Host "Uninstall: signet self uninstall --yes"
Write-Host "Or open:   signet   (TUI → Update / Uninstall Signet)"
