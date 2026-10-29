# Stage Alts test builds

Target: Smash Ultimate 13.0.5, standard stage-select Lua.

| Package | Purpose |
| --- | --- |
| `stage-alts-0.1.12-form-patching-fix.zip` | Latest: patch only the loaded stage form; retain restoration of both forms. Only the NRO needs replacing if the previous matching Lua is installed. |
| `stage-alts-0.1.12-random-identity-fix.zip` | Previous: explicit Random preview detection, repeat-visit stage identity, and source/target diagnostic hashes. Install both the NRO and matching standard Lua. |
| `stage-alts-randomauto-0.1.12-path-fix.zip` | Previous: already-selected alt-path loading and stage-name diagnostics. |
| `stage-alts-randomauto-0.1.12.zip` | Original upstream integration baseline. |

Open a ZIP and use **Download raw file**. Older packages are retained for comparison
and rollback. The latest Lua is source text; use your existing installation and,
where required, compilation workflow. Keep only one Stage Alts NRO active.

The latest package was built from commit `3dee268abf5541401d45ac64795cc610c056ac3c`.
Nineteen Rust tests, twelve Lua 5.3 tests, the full Lua syntax check, and the Skyline
release build passed. In-game form switching and restoration still need testing.
The standard Lua and existing global cycling behavior are unchanged from the
previous package. Unknown Kalos/FD folders and PS2's missing model remain open.

Archive the previous log, follow `TESTING.txt`, and return the fresh `stage-alts.log`
with the selection buttons used and observations about visuals and loading.
