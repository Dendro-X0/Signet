# Indie Install Trust Pack (v0) — Windows

> **Superseded in scope (2026-09-05):** use [release-trust-desk-v2.md](./release-trust-desk-v2.md) for EJR / Desk deliverables. This file remains a content quarry (copy reusable sections into v2).

**SKU:** Indie Install Trust Pack · **OS lock:** Windows only  
**Price (checkout):** $19 intro / $29 standard  
**As-of:** 2026-09-05 · **Status:** Legacy working draft — merge into Desk v2 before checkout  
**Companion:** Free Signet CLI ([github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)) stays OSS — this pack is the **ritual**, not a license for the tool.

> Self-sign proves **integrity** of bits and publisher keys. It does **not** buy Microsoft reputation. SmartScreen warnings are expected until you graduate to OV / Azure Trusted Signing (or the Store).

---

## 1. Integrity vs reputation

```text
Integrity (what you can prove)     does not imply     Reputation (what Windows decides)
fingerprint · SHA-256 sums ·                       SmartScreen · Smart App Control ·
host signature · signed sums                       Store / CA reputation
```

| You can prove | You cannot buy with this pack |
|---------------|-------------------------------|
| “These bytes match `SHA256SUMS`” | Instant SmartScreen silence |
| “This build matches my published fingerprint” | “Trusted by Microsoft” |
| “I signed with *my* identity / cert” | Permission to install your cert into Trusted Root |

**Signet Check** (`signet verify` / `inspect`) answers integrity questions. Windows reputation is a separate ladder.

---

## 2. Trust tiers (cheat sheet)

Use the tier that is **true**. Do not declare a higher tier to look safer.

| Tier | Integrity meaning | Does **not** mean |
|------|-------------------|-------------------|
| `checksum_only` | `SHA256SUMS` present | Host crypto signature |
| `self_signed_host` | Signed with your Signet / self-issued identity | CA / Store trust |
| `community_signed_sums` | Sums attested (minisign / GPG) | Authenticode pass |
| `ca_authenticode` | Windows signature chains to a public CA | Instant SmartScreen silence |
| `unknown` | Not enough evidence | Safe to install |

Primary tier belongs in `TRUST.md` and matches reality. For Windows self-ship, `self_signed_host` or `checksum_only` is honest; `ca_authenticode` only after OV / Azure / EV signing.

---

## 3. Pre-release checklist (Windows)

Run before you tag or upload an installer.

### Identity & secrets

- [ ] Signet identity created (`signet identity create`) — fingerprint known
- [ ] Identity / PFX / keystore **not** in git
- [ ] No passwords or private keys in `TRUST.md`

### Build & prove

- [ ] Windows artifact built (e.g. `.exe` / `.msi` under your `dist/` or release folder)
- [ ] `signet trust` (or equivalent) wrote / refreshed `TRUST.md`
- [ ] `SHA256SUMS` lists the exact files you will publish
- [ ] Optional: sums signed (`SHA256SUMS.minisig` / `.asc`) if you publish signatures

### Check before publish

- [ ] `signet verify` passes on the release tree (or `--artifact` for the installer)
- [ ] `signet inspect --file path\to\setup.exe` reviewed (signed / unsigned / adhoc)
- [ ] README / release notes tell users how to verify (section 4)
- [ ] Release notes do **not** promise SmartScreen silence

### Publish

- [ ] Tag / GitHub Release (or other host) includes artifact **and** `SHA256SUMS` (+ `TRUST.md` when used)
- [ ] Filenames in `SHA256SUMS` match download basenames

---

## 4. Verify / inspect — Windows copy-paste

Install Signet first: [docs/install.md](../../install.md) (PowerShell one-liner from the repo README).

### Publisher: before you ship

```powershell
# From your app project (signet.toml present)
signet doctor
signet verify
signet verify --json
signet inspect --file .\dist\YourApp-Setup.exe
signet inspect --file .\dist\YourApp-Setup.exe --strict   # exit 1 if unsigned/error
```

With explicit paths:

```powershell
signet verify --sums .\SHA256SUMS --trust .\TRUST.md
signet verify --artifact .\dist\YourApp-Setup.exe
signet verify --require-sig    # only if you publish minisig/asc
```

### Downloader: after they fetch your release

```powershell
# Example: checksum the installer Microsoft gives them no help with
certutil -hashfile .\YourApp-Setup.exe SHA256
# Compare to the matching line in SHA256SUMS from the same release

# If they have Signet and a folder with installer + SHA256SUMS (+ TRUST.md):
signet verify --sums .\SHA256SUMS --trust .\TRUST.md
signet inspect --file .\YourApp-Setup.exe
```

**Exit codes (verify):** `0` ok · `1` mismatch / fail-stale · `2` missing inputs · `3` `--require-sig` unmet.

`verify` / `inspect` never evaluate SmartScreen and never recommend Trusted Root installs.

---

## 5. What to tell users when Windows warns

Use plain language. Prefer one short block in your README / download page.

### When SmartScreen says “Windows protected your PC”

**Good (honest):**

> This build is **self-signed** (or signed with our publisher key that Windows does not yet reputate). That is normal for indie downloads.  
> Check integrity: download `SHA256SUMS` from the same release, then run `certutil -hashfile … SHA256` (or `signet verify`) and compare.  
> If the hash matches and you trust **us**, use **More info → Run anyway**. We do **not** ask you to install certificates into Trusted Root.

**Bad (forbidden):**

> “Disable SmartScreen” · “Install our cert as Trusted Root” · “This removes all warnings” · “Microsoft trusts this”

### When they ask “Is this a virus?”

> We publish checksums and (if applicable) a signature so you can prove the file was not altered in transit. Virus scanners and SmartScreen use different signals. Matching our published SHA-256 means **integrity**, not a malware verdict from Microsoft.

### Optional one-liner for release notes

> Self-signed Windows builds may show SmartScreen. Verify with `SHA256SUMS` / Signet before running. We never ask for Trusted Root installs.

---

## 6. Anti-patterns

Do **not**:

1. Tell users to install your certificate into **Trusted Root** / the Root store  
2. Claim self-signing removes SmartScreen (or “guarantees silence”)  
3. Put private keys, PFX passwords, or keystore passwords in `TRUST.md` or git  
4. Sell “OS trust” or “bypass SmartScreen” as the product  
5. Declare `ca_authenticode` in `TRUST.md` while still self-signing  

Do:

- Publish matching sums + honest tier language  
- Document verify steps  
- Treat warnings as expected until you graduate  

---

## 7. When to graduate (pointer only)

This pack does **not** include buying certificates, Azure setup as a service, or Store submission.

When self-sign reputation is no longer acceptable:

| Step | Need | Signet helper (free CLI) |
|------|------|---------------------------|
| OV / EV Authenticode | CA-issued code-signing cert | `signet graduate ov-sign` |
| Azure Trusted Signing | Azure account + dlib + metadata | `signet graduate azure-sign` |

```powershell
signet graduate notes
```

Full ladder: Signet repo → `docs/graduation.md`. Declaring `ca_authenticode` in config is only honest after real CA / Azure signing.

---

## 8. Blank checklist (fill per release)

| Field | Yours |
|-------|-------|
| App name | |
| Version | |
| OS | **Windows** |
| Artifact path | |
| Artifact SHA-256 | |
| `TRUST.md` URL / path | |
| `SHA256SUMS` URL / path | |
| Declared trust tier | |
| Verify command documented? | ☐ |
| SmartScreen honesty in notes? | ☐ |
| Secrets out of git? | ☐ |

---

## 9. Disclaimer

This pack is educational process guidance for independent and OSS publishers. It is **not** legal advice, a security audit, malware certification, or affiliation with Apple, Microsoft, or Google.  

The Signet CLI remains free/open source under its project licenses. Purchase of this pack does not grant OS trust, SmartScreen silence, or permission to misrepresent signature status.  

**Never** install publisher certificates into Trusted Root because a checklist or vendor said so.

---

### Errata / as-of

| Date | Note |
|------|------|
| 2026-09-05 | v0 working draft · Windows lock · sections 1–9 filled from Signet trust/verify/graduation docs |
