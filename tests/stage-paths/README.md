# Host path-rule tests

Run `cargo test --locked --manifest-path tests/stage-paths/Cargo.toml` from the repository root.
This harness compiles the production `src/stage_paths.rs` directly, using the same
Hash40 implementation, without requiring Switch memory or the Skyline runtime.

It checks base and already-selected paths, nested assets, normal/battle forms,
other-slot rejection, path boundaries, source-slot preservation during patching,
and the alternate-folder hashes reported in the runtime log. It does not validate
the game's live filesystem tables, resource reference counts, or gameplay.
