# Stage Alts test builds

Target: Smash Ultimate 13.0.5, standard stage-select Lua.

| Package | Purpose |
| --- | --- |
| `stage-alts-randomauto-0.1.12-path-fix.zip` | Latest: handles already-selected alt paths, preserves alternate source slots, and prints stage names in diagnostics. |
| `stage-alts-randomauto-0.1.12.zip` | Previous upstream integration baseline, retained for comparison and rollback. |

Open the desired ZIP on GitHub and use **Download raw file**. Each package contains
`libstage_alts.nro`, `TESTING.txt`, build metadata, the build dependency lockfile,
and a source patch. Replace the active Stage Alts NRO; keep only one copy active.

The latest package was built from commit `aae213c657eacec35335d40e60394d3447746416`. Nine host
regression tests and the Skyline release build passed. The path changes still
need in-game testing; successful compilation does not establish loading stability.
The original global cycling behavior and vanilla fallback are preserved.

Archive the previous log, follow `TESTING.txt`, and return `stage-alts.log` with
observations about stage visuals, cycling, and any loading failures.
