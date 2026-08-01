# Signet CLI installer (Windows PowerShell)
#   irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Repo = "Dendro-X0/Signet"
$Base = "https://github.com/$Repo/releases/latest/download"
$Root = Join-Path $env:LOCALAPPDATA "Signet"
$BinDir = Join-Path $Root "bin"
$Bin = Join-Path $BinDir "signet.exe"
$Asset = "signet-x86_64-pc-windows-msvc.exe"
# Many Git Bash / Cursor sessions already have %USERPROFILE%\bin on PATH (and won't
# reload the User PATH registry until the IDE restarts). Mirror there too.
$HomeBin = Join-Path $env:USERPROFILE "bin"
$HomeShim = Join-Path $HomeBin "signet.exe"

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
home_shim = '$HomeShim'
"@
# Avoid UTF-8 BOM (breaks some TOML/shell parsers)
[System.IO.File]::WriteAllText((Join-Path $Root "install.toml"), $Receipt)

# User PATH — always put Signet bin first so it beats ~/.cargo/bin when both exist.
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $userPath) { $userPath = "" }
$parts = @($userPath -split ";" | Where-Object { $_ -and $_.Trim() -ne "" -and $_.TrimEnd('\') -ne $BinDir.TrimEnd('\') })
$newPath = (@($BinDir) + $parts) -join ";"
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")
$env:Path = "$BinDir;$env:Path"
Write-Host "User PATH: $BinDir is first."

# Notify Windows that PATH changed (new Explorer windows; does not refresh Cursor/Git Bash).
try {
    Add-Type -Namespace Win32 -Name Native -MemberDefinition @"
[DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@ -ErrorAction SilentlyContinue
    $HWND_BROADCAST = [IntPtr]0xffff
    $WM_SETTINGCHANGE = 0x1a
    $result = [UIntPtr]::Zero
    [void][Win32.Native]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, "Environment", 2, 5000, [ref]$result)
} catch {}

# Mirror into %USERPROFILE%\bin for shells that already have it on PATH (Git Bash, Cursor).
New-Item -ItemType Directory -Force -Path $HomeBin | Out-Null
Copy-Item -Force $Bin $HomeShim
Write-Host "Also mirrored to $HomeShim (helps Git Bash / Cursor without restart)."

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
        $shim = [System.IO.Path]::GetFullPath($HomeShim)
        if ($resolved -ne $ours -and $resolved -ne $shim) {
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
    Write-Host "Try now:     signet --version"
    Write-Host "If bash says command not found: fully quit and reopen Cursor (or run):"
    Write-Host '  export PATH="$LOCALAPPDATA/Signet/bin:$HOME/bin:$PATH"'
    Write-Host '  # or:  cp "$LOCALAPPDATA/Signet/bin/signet.exe" "$HOME/bin/signet.exe"'
}

Write-Host "Update:    signet self update"
Write-Host "Uninstall: signet self uninstall --yes"
Write-Host "Or open:   signet   (TUI → Update / Uninstall Signet)"
