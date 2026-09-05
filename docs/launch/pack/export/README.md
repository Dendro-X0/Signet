# Pack / Desk export

| File | Use |
|------|-----|
| `release-trust-desk-v2-windows.zip` | **Current SKU** — upload to Gumroad |
| `install-trust-pack-v0-windows.zip` | Legacy Install Trust Pack (superseded) |
| `GUMROAD-LISTING.md` | Listing copy stub |

Rebuild Desk zip from repo root:

```bash
python -c "import zipfile; from pathlib import Path; z=zipfile.ZipFile('docs/launch/pack/export/release-trust-desk-v2-windows.zip','w',zipfile.ZIP_DEFLATED); z.write('docs/launch/pack/release-trust-desk-v2.md','release-trust-desk-v2.md')"
```
