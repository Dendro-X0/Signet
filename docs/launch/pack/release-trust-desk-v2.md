# Release Trust Desk — kit draft (v2) · Windows

**SKU:** Release Trust Desk (Windows)  
**Price:** $29–49 standard (intro optional)  
**As-of:** 2026-09-05 · **Status:** Working draft (EJR) — export after unlock  
**Pattern:** Evidence · Judgment · Ritual  
**GIFT:** Signet CLI stays free — [github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)  
**Quarry:** [install-trust-pack-v0.md](./install-trust-pack-v0.md) (legacy)

> Self-sign proves **integrity**. It does **not** buy Microsoft reputation. SmartScreen warnings are expected until you graduate (OV / Azure / Store).

---

## E — Evidence

### E1. Trust card template (buyer-facing)

Publish next to the installer (README section or `TRUST-CARD.md` on the release).

```markdown
# Trust card — 〈App〉 〈Version〉

| Field | Value |
|-------|-------|
| Publisher | 〈you〉 |
| Integrity tier | `self_signed_host` / `checksum_only` / … (must be true) |
| Artifact | 〈filename.exe〉 |
| SHA-256 | 〈hex〉 |
| Fingerprint (publisher) | 〈colons or hex〉 |
| How to verify | See commands below |
| OS reputation | **Not claimed** — SmartScreen may warn |

## Verify (Windows)

```powershell
certutil -hashfile .\〈filename.exe〉 SHA256
# Compare to SHA-256 above / SHA256SUMS
signet verify --sums .\SHA256SUMS --trust .\TRUST.md   # if Signet installed
signet inspect --file .\〈filename.exe〉
```

We never ask you to install certificates into Trusted Root.
```

**Worked sample:** adapt [demo/fixture/TRUST.md](../../../demo/fixture/TRUST.md) (HelloSignet) into a one-page Trust card for HOOK TRACK.

### E2. Verify transcript template (publisher archive)

After `signet verify` / `inspect`, paste into release notes or keep privately:

```text
Date:
App / version:
Artifact path:
signet version:
verify exit code:     (0 ok · 1 fail · 2 missing · 3 require-sig)
inspect signed?:
SHA-256 (certutil or verify):
TRUST.md tier declared:
Notes:
```

### E3. Worked Windows release fixture

Use Signet demo happy-path or a throwaway app:

1. `signet doctor` clean  
2. Identity fingerprint recorded  
3. Artifact built  
4. `TRUST.md` + `SHA256SUMS` published  
5. Trust card filled  
6. `signet verify` → exit 0  
7. Release notes include SmartScreen honesty (J2)

---

## J — Judgment

### J1. Integrity vs reputation

```text
Integrity (provable)          ≠     Reputation (Windows decides)
fingerprint · SHA-256 ·             SmartScreen · Smart App Control ·
host signature · signed sums        Store / CA reputation
```

| You can prove | You cannot buy with this desk |
|---------------|-------------------------------|
| Bytes match `SHA256SUMS` | Instant SmartScreen silence |
| Build matches published fingerprint | “Trusted by Microsoft” |
| Signed with *your* identity | Permission to install cert into Trusted Root |

### J2. OS-warn decision tree (SmartScreen)

```text
Windows shows “Windows protected your PC”
        │
        ├─ Did user download from YOUR release URL?
        │     NO  → stop · treat as untrusted source
        │     YES ↓
        ├─ Do SHA-256 / signet verify match published sums?
        │     NO  → stop · re-download or investigate tampering
        │     YES ↓
        ├─ Do they trust YOU as publisher?
        │     NO  → soft-no · do not pressure “Run anyway”
        │     YES → More info → Run anyway is THEIR choice
        │
        └─ NEVER: disable SmartScreen · Trusted Root install · “MS trusts this”
```

### J3. User-message bank (honest only)

**SmartScreen — good:**

> This build is **self-signed** (Windows does not yet reputate our publisher key). That is normal for indie downloads.  
> Check integrity: download `SHA256SUMS` from the same release, run `certutil -hashfile … SHA256` (or `signet verify`).  
> If the hash matches and you trust **us**, use **More info → Run anyway**. We do **not** ask for Trusted Root installs.

**“Is this a virus?” — good:**

> We publish checksums so you can prove the file was not altered in transit. Matching SHA-256 means **integrity**, not a Microsoft malware verdict.

**Forbidden:** “Disable SmartScreen” · “Install our cert as Trusted Root” · “This removes all warnings” · “Microsoft trusts this”

**Release notes one-liner:**

> Self-signed Windows builds may show SmartScreen. Verify with `SHA256SUMS` / Signet before running. We never ask for Trusted Root installs.

### J4. Trust tiers (declare only what is true)

| Tier | Integrity meaning | Does **not** mean |
|------|-------------------|-------------------|
| `checksum_only` | `SHA256SUMS` present | Host crypto signature |
| `self_signed_host` | Signed with your Signet / self-issued identity | CA / Store trust |
| `community_signed_sums` | Sums attested (minisign / GPG) | Authenticode pass |
| `ca_authenticode` | Chains to a public CA | Instant SmartScreen silence |
| `unknown` | Not enough evidence | Safe to install |

### J5. Anti-patterns + cost of mistake

| Anti-pattern | Cost |
|--------------|------|
| Trusted Root install advice | User machine compromise · your reputation as scamware |
| “Removes SmartScreen” claim | False advertising · support storm · possible takedown |
| Secrets in `TRUST.md` / git | Key theft · supply-chain incident |
| Declaring `ca_authenticode` while self-signing | Buyers misled · integrity story collapses |

### J6. When to graduate (pointer)

Desk does **not** include buying certs or Azure as a service.

| Need | Signet helper (free CLI) |
|------|---------------------------|
| OV / EV | `signet graduate ov-sign` |
| Azure Trusted Signing | `signet graduate azure-sign` |
| Notes | `signet graduate notes` |

→ Signet `docs/graduation.md`

### J7. As-of sheet

| Date | Can claim | Cannot claim |
|------|-----------|--------------|
| 2026-09-05 | Integrity via sums/Signet; honest user scripts | OS silence; MS partnership; multi-OS desk |

---

## R — Ritual

### R1. Pre-tag → post-release (Windows)

**Identity & secrets**

- [ ] `signet identity create` — fingerprint known  
- [ ] Keys / PFX **not** in git  
- [ ] No secrets in `TRUST.md`

**Build & prove**

- [ ] Windows artifact built  
- [ ] `TRUST.md` refreshed  
- [ ] `SHA256SUMS` lists exact publish filenames  
- [ ] Optional signed sums  

**Check before publish**

- [ ] `signet verify` passes  
- [ ] `signet inspect --file path\to\setup.exe` reviewed  
- [ ] Trust card filled (E1)  
- [ ] Release notes: SmartScreen honesty (J3) — **no silence promise**

**Publish**

- [ ] Release includes artifact + `SHA256SUMS` (+ `TRUST.md`)  
- [ ] Filenames match  

**Post-release**

- [ ] Verify transcript archived (E2)  
- [ ] Spot-check download path as a stranger would  

### R2. Publisher copy-paste

```powershell
signet doctor
signet verify
signet verify --json
signet inspect --file .\dist\YourApp-Setup.exe
signet inspect --file .\dist\YourApp-Setup.exe --strict
signet verify --sums .\SHA256SUMS --trust .\TRUST.md
signet verify --artifact .\dist\YourApp-Setup.exe
```

Downloader:

```powershell
certutil -hashfile .\YourApp-Setup.exe SHA256
signet verify --sums .\SHA256SUMS --trust .\TRUST.md
signet inspect --file .\YourApp-Setup.exe
```

`verify` / `inspect` never evaluate SmartScreen and never recommend Trusted Root.

### R3. Blank checklist

| Field | Yours |
|-------|-------|
| App name | |
| Version | |
| OS | **Windows** |
| Artifact path | |
| Artifact SHA-256 | |
| `TRUST.md` path/URL | |
| `SHA256SUMS` path/URL | |
| Trust tier | |
| Trust card published? | ☐ |
| Verify documented? | ☐ |
| SmartScreen honesty? | ☐ |
| Secrets out of git? | ☐ |
| Transcript archived? | ☐ |

---

## Disclaimer

Educational process guidance for indie/OSS publishers. **Not** legal advice, security audit, malware certification, or MS/Apple/Google affiliation. CLI remains free. Purchase does not grant OS trust or SmartScreen silence. **Never** install publisher certs into Trusted Root because a kit said so.

| Date | Note |
|------|------|
| 2026-09-05 | v2 EJR Desk draft from v0 quarry + redesign |
