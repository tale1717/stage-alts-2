# Stage Alts test builds

Target: Smash Ultimate 13.0.5, standard stage-select Lua.

| Package | Purpose |
| --- | --- |
| `stage-alts-0.1.12-random-identity-fix.zip` | Latest: explicit Random preview detection, repeat-visit stage identity, and source/target diagnostic hashes. Install both the NRO and matching standard Lua. |
| `stage-alts-randomauto-0.1.12-path-fix.zip` | Previous: already-selected alt-path loading and stage-name diagnostics. |
| `stage-alts-randomauto-0.1.12.zip` | Original upstream integration baseline. |

Open a ZIP and use **Download raw file**. Older packages are retained for comparison
and rollback. The latest Lua is source text; use your existing installation and,
where required, compilation workflow. Keep only one Stage Alts NRO active.

The latest package was built from commit `ffadfaa4a7d57d0fc9400f4ed13d6c2ebb48f6e7`.
Fifteen Rust tests, twelve Lua 5.3 tests, the full Lua syntax check, and the Skyline
release build passed. Live gameplay and the timing of the game's Random-preview
API still need validation. The existing global cycling behavior is preserved.

Archive the previous log, follow `TESTING.txt`, and return the fresh `stage-alts.log`
with the selection buttons used and observations about visuals and loading.
