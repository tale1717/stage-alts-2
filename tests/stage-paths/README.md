# Host path-rule tests

Run `cargo test --locked --manifest-path tests/stage-paths/Cargo.toml` from the repository root.
This harness compiles the production `src/stage_paths.rs` directly, using the same
Hash40 implementation, without requiring Switch memory or the Skyline runtime.

It checks base and already-selected paths, nested assets, normal/battle forms,
other-slot rejection, path boundaries, source-slot preservation during patching,
and the alternate-folder hashes reported in the runtime log. It does not validate
the game's live filesystem tables, resource reference counts, or gameplay.

Stage-identity cases also cover the five failed repeat visits from the log,
normal/battle forms, and parent-directory recovery using the supplied form.

Form-filter cases cover the exact missing Battlefield target hashes from the log,
both directions of normal/battle filtering, nested assets, form recovery from
revisited alt paths, and continued reporting of unknown Kalos/FD folders. The
filter does not prove live restoration across matches; test that in the game.
