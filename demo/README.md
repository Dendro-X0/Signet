# Signet demo kit

Fixed fixture + scripts for local smoke tests and GIF/video recording.

**Full guide:** [`docs/demo.md`](../docs/demo.md)

```bash
# from repo root (Signet on PATH, or: export SIGNET="cargo run -q -p signet --")
./demo/scripts/happy-path.sh
# Windows:
pwsh ./demo/scripts/happy-path.ps1
```

Then open the TUI for a visual pass: `cd demo/fixture && signet` → **Guided setup**.
