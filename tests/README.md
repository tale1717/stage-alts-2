# Regression checks

Run from the repository root with the environment's Rust tools activated:

```sh
cargo test --locked --manifest-path tests/stage-paths/Cargo.toml
texlua tests/random-selection.lua
cargo skyline build --release -- --locked
```

The Lua check requires Lua 5.3; `lua5.3 tests/random-selection.lua` is also suitable.
It compiles the full standard stage-select script and executes that script's
actual selection helper and dispatch block with mocked game APIs. It covers
Random reported on panel zero, negative and positive panels, manual panel zero,
normal/Battlefield/Omega forms, cancellation, custom-stage handling, and capture
before finalization. The nine-argument Rust interface stays unchanged.

The Rust harness compiles the production path rules without Skyline. These checks
do not load game assets or prove that the live preview API still reports Random
at capture time. Test the matching NRO and standard Lua together in Smash 13.0.5,
using both A and Start, and inspect the explicit selection diagnostic in the log.
