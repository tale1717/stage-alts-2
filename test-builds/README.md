# Stage Alts test builds

Target: Smash Ultimate 13.0.5, standard stage-select Lua.

| Package | Purpose |
| --- | --- |
| `stage-alts-0.1.12-random-form-tracking.zip` | Latest: Random sends the last form passed to the game setter. Update Lua; the included NRO is unchanged from form-diagnostics. The NRO mismatch guard remains active. |
| `stage-alts-0.1.12-form-diagnostics.zip` | Previous diagnostic build: cached/fixed Lua forms and original/resolved load hashes. Install BOTH the NRO and diagnostic standard Lua. Selection behavior is unchanged. |
| `stage-alts-0.1.12-form-patching-fix.zip` | Previous: patch only the loaded stage form; retain restoration of both forms. Only the NRO needs replacing if the previous matching Lua is installed. |
| `stage-alts-0.1.12-random-identity-fix.zip` | Previous: explicit Random preview detection, repeat-visit stage identity, and source/target diagnostic hashes. Install both the NRO and matching standard Lua. |
| `stage-alts-randomauto-0.1.12-path-fix.zip` | Previous: already-selected alt-path loading and stage-name diagnostics. |
| `stage-alts-randomauto-0.1.12.zip` | Original upstream integration baseline. |

Open a ZIP and use **Download raw file**. Older packages are retained for comparison
and rollback. The latest Lua is source text; use your existing installation and,
where required, compilation workflow. Keep only one Stage Alts NRO active.

The latest Lua is from commit `1e64c510cab2b1ba4c6d2cf7d663d648400bdefc` and passed 24 Lua 5.3
checks including the full script syntax check. The package reuses the identical,
previously built and tested form-diagnostics NRO; no Rust code changed. Install
only the Lua if that NRO is already active. Manual selections retain their current
behavior. Applied-form tracking still needs in-game testing, and the existing
form-mismatch guard can still skip an alt after a correctly reported Omega choice.

Archive the previous log, follow `TESTING.txt`, and return the fresh `stage-alts.log`
with the selection buttons used and observations about visuals and loading.
