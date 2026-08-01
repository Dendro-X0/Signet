# Signet demo kit — CLI happy path (Sign → Prove → Check) against demo/fixture.
# Prereqs: `signet` on PATH, or $env:SIGNET = 'cargo run -q -p signet --'
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Fixture = Join-Path $Root "fixture"
$RepoRoot = Split-Path -Parent $Root
$Signet = if ($env:SIGNET) { $env:SIGNET } else { "signet" }

function Invoke-Signet {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)
    if ($Signet -match "\s") {
        # e.g. cargo run -q -p signet --
        $parts = $Signet -split "\s+"
        & $parts[0] @($parts[1..($parts.Length - 1)]) @Args
    } else {
        & $Signet @Args
    }
}

Write-Host "==> demo fixture: $Fixture"
Write-Host "==> using: $Signet"
Set-Location $Fixture

try {
    Invoke-Signet --version | Out-Null
} catch {
    Write-Error @"
Cannot run Signet. Install from README, or from repo root:
  `$env:SIGNET = 'cargo run -q -p signet --'
  pwsh ./demo/scripts/happy-path.ps1
"@
}

Write-Host ""
Write-Host "==> Doctor"
try { Invoke-Signet doctor } catch { Write-Host "doctor reported issues (continuing)" }

Write-Host ""
Write-Host "==> Identity (Sign)"
if (Test-Path ".signet/identity/active") {
    Invoke-Signet identity show
} else {
    Invoke-Signet identity create --name default --cn "HelloSignet Demo" --org "Signet Demo" --days 825
}

Write-Host ""
Write-Host "==> Trust (Prove)"
Invoke-Signet trust

Write-Host ""
Write-Host "==> Build --skip-build --no-sign (Prove checksums; fake PE is not host-signed)"
Invoke-Signet build --skip-build --no-sign --no-sums-sign

Write-Host ""
Write-Host "==> Verify (Check)"
try { Invoke-Signet verify } catch { Write-Host "verify exit non-zero (see output)" }

Write-Host ""
Write-Host "==> Inspect (Check)"
try { Invoke-Signet inspect --file dist/HelloSignet.exe } catch { Write-Host "inspect exe: $_" }
try { Invoke-Signet inspect --file dist/HelloSignet.AppImage } catch { Write-Host "inspect appimage: $_" }

Write-Host ""
Write-Host "==> Graduate notes (official path hint)"
Invoke-Signet graduate notes

Write-Host ""
Write-Host "OK — CLI happy path finished."
Write-Host "Visual:  cd `"$Fixture`"; signet   # TUI → Guided setup"
Write-Host "Docs:    $RepoRoot/docs/demo.md"
Write-Host "Release: irm https://github.com/Dendro-X0/Signet/releases/latest/download/SHA256SUMS"
