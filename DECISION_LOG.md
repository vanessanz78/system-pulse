# Decision Log

## 2026-07-02 - Data clarity visual pass

Vanessa approved the current Candle Pulse-style visual direction and asked for the next refinement to match the attached Today view reference:

- Companion score should be a clean green circle, not a heart outline.
- Today should keep the four visual status cards: Applications, Memory, Browser, Storage.
- Each Today card should show a compact metric footer with useful numbers.
- Memory should be framed as RAM, because it affects sluggishness and app switching.
- Storage should be framed as disk space, because it affects updates, caches, and reliability.
- Things worth knowing should show compact right-side metrics.
- Recommended care should become a short Needs attention action list: Application, Memory, Browser, Storage.

A local implementation commit was created but could not be pushed from this Mac because local GitHub auth is not configured and the connector upload path was not reliable for large files.

Local checkpoint, if still present:

- Path: /Users/vanessa/Documents/Codex/2026-07-01/git-5/work/system-pulse
- Commit: 52c98c0235a14b071c85d517870c5ab6f22f7bde
- Message: Clarify Today metrics and care actions
- Parent main at time of work: 4c130bdc1bd0efeff0625a2d932fe11d5b74395f

Changed locally in that checkpoint:

- CURRENT_SPRINT.md
- package.json
- app/desktop/package.json
- app/desktop/src-tauri/tauri.conf.json
- app/desktop/src-tauri/src/models.rs
- app/desktop/src-tauri/src/pulse_core.rs
- app/desktop/src/main.ts
- app/desktop/src/styles.css

Verification completed locally:

- git diff --check passed
- JSON parse passed for package.json, app/desktop/package.json, app/desktop/src-tauri/tauri.conf.json
- Non-ASCII scan passed for edited source/docs files

Verification not completed locally:

- cargo fmt/check/build could not run because Cargo is not installed on this Mac.
- The downloadable macOS artifact was not produced because the implementation commit was not pushed.
