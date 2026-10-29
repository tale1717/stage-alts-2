# Stage Alts test builds

Target: Smash Ultimate 13.0.5, standard stage-select Lua.

| Package | Purpose |
| --- | --- |
| `stage-alts-0.1.12-form-diagnostics.zip` | Latest diagnostic build: cached/fixed Lua forms and original/resolved load hashes. Install BOTH the NRO and diagnostic standard Lua. Selection behavior is unchanged. |
| `stage-alts-0.1.12-form-patching-fix.zip` | Previous: patch only the loaded stage form; retain restoration of both forms. Only the NRO needs replacing if the previous matching Lua is installed. |
| `stage-alts-0.1.12-random-identity-fix.zip` | Previous: explicit Random preview detection, repeat-visit stage identity, and source/target diagnostic hashes. Install both the NRO and matching standard Lua. |
| `stage-alts-randomauto-0.1.12-path-fix.zip` | Previous: already-selected alt-path loading and stage-name diagnostics. |
| `stage-alts-randomauto-0.1.12.zip` | Original upstream integration baseline. |

Open a ZIP and use **Download raw file**. Older packages are retained for comparison
and rollback. The latest Lua is source text; use your existing installation and,
where required, compilation workflow. Keep only one Stage Alts NRO active.

The latest package was built from commit `ab5eb91fe3bbf38a67ccb6f17b619b21218fbd72`.
Nineteen Rust tests, sixteen Lua 5.3 tests, the full Lua syntax check, and the
Skyline release build passed. This build adds diagnostics without changing alt
selection, the form-mismatch guard, or form patching. It requires both updated
components and still needs in-game testing. Existing missing-asset issues remain.

Archive the previous log, follow `TESTING.txt`, and return the fresh `stage-alts.log`
with the selection buttons used and observations about visuals and loading.
