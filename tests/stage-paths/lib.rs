// Compile the production path rules on the host without Skyline or game memory.
#[path = "../../src/stage_paths.rs"]
pub mod stage_paths;

#[cfg(test)]
mod tests {
    use super::stage_paths::{
        alternate_form, parent_stage_identity, patch_target, redirect_path,
        stage_identity, PatchTarget,
    };
    use hash40::Hash40;

    fn components(path: &str) -> Vec<Hash40> {
        path.split('/').map(Hash40::new).collect()
    }

    #[test]
    fn redirects_base_forms_and_nested_assets() {
        for form in ["normal", "battle"] {
            for tail in ["", "/model/example_set", "/model/normal/battle"] {
                let source = format!("stage/example/{form}{tail}");
                let expected = format!("stage/example/{form}_s02{tail}");
                assert_eq!(redirect_path(&components(&source), 2), Some(Hash40::new(expected.as_str())));
            }
        }
    }

    #[test]
    fn selected_alt_paths_are_idempotent() {
        for form in ["normal", "battle"] {
            for slot in [1, 2, 99, 100, 999] {
                for tail in ["", "/model/example_set"] {
                    let source = format!("stage/example/{form}_s{slot:02}{tail}");
                    assert_eq!(redirect_path(&components(&source), slot), Some(Hash40::new(source.as_str())));
                }
            }
        }
    }

    #[test]
    fn recognizes_the_hashed_alt_components_from_the_runtime_log() {
        for (slot, hash) in [(1, 0xa7f41108f), (2, 0xae6484135)] {
            let mut path = components("stage/battlefield_s/normal/model/example_set");
            path[2] = Hash40(hash);
            let expected = format!("stage/battlefield_s/normal_s{slot:02}/model/example_set");
            assert_eq!(redirect_path(&path, slot), Some(Hash40::new(expected.as_str())));
        }
    }

    #[test]
    fn another_slots_assets_are_not_redirected() {
        for source in ["stage/example/normal_s01", "stage/example/battle_s01/model/example_set"] {
            assert_eq!(redirect_path(&components(source), 2), None);
        }
    }

    #[test]
    fn vanilla_requests_do_not_adopt_an_alternate_slot() {
        for form in ["normal", "battle"] {
            let source = format!("stage/example/{form}/model/example_set");
            assert_eq!(redirect_path(&components(&source), 0), Some(Hash40::new(source.as_str())));
            let alternate = format!("stage/example/{form}_s01/model/example_set");
            assert_eq!(redirect_path(&components(&alternate), 0), None);
        }
    }

    #[test]
    fn rejects_missing_or_misplaced_stage_form_components() {
        for source in ["", "stage", "stage/example", "ui/example/normal", "stage/example/model/normal_s02", "stage/example/model/normal", "stage/example/unknown"] {
            assert_eq!(redirect_path(&components(source), 2), None, "{source}");
            assert_eq!(patch_target(&components(source), 2, true), PatchTarget::Unsupported, "{source}");
        }
    }

    #[test]
    fn patching_skips_both_selected_and_other_alternate_slots() {
        for form in ["normal", "battle"] {
            for slot in [1, 2, 99, 100, 999] {
                let source = format!("stage/example/{form}_s{slot:02}/model/example_set");
                assert_eq!(patch_target(&components(&source), 2, true), PatchTarget::Alternate);
            }
        }
    }

    #[test]
    fn patching_still_redirects_base_entries() {
        let source = components("stage/example/normal/model/example_set");
        assert_eq!(patch_target(&source, 2, true), PatchTarget::Redirect(Hash40::new("stage/example/normal_s02/model/example_set")));
    }

    #[test]
    fn patching_only_redirects_the_loaded_form_and_its_assets() {
        for (selected, other, normal) in [("normal", "battle", true), ("battle", "normal", false)] {
            for tail in ["", "/model/example_set", "/model/normal/battle"] {
                let selected_path = format!("stage/example/{selected}{tail}");
                let other_path = format!("stage/example/{other}{tail}");
                let target = format!("stage/example/{selected}_s08{tail}");
                assert_eq!(patch_target(&components(&selected_path), 8, normal), PatchTarget::Redirect(Hash40::new(&target)));
                assert_eq!(patch_target(&components(&other_path), 8, normal), PatchTarget::OtherForm);
            }
        }
    }

    #[test]
    fn battlefield_missing_targets_from_log_are_skipped_for_normal_loads() {
        let source = components("stage/battlefield/battle");
        for (slot, missing_hash) in [(1, 0x1c05487798), (8, 0x1c7c94cf3c)] {
            assert_eq!(patch_target(&source, slot, true), PatchTarget::OtherForm);
            // A battle-form request still checks the target instead of hiding a
            // genuinely missing asset in the requested form.
            assert_eq!(patch_target(&source, slot, false), PatchTarget::Redirect(Hash40(missing_hash)));
        }
    }

    #[test]
    fn revisited_alt_paths_select_their_form_without_reusing_the_old_slot() {
        for (loaded, normal) in [("normal_s01", true), ("battle_s08", false), ("normal_s03", true)] {
            let (_, loaded_normal) = stage_identity(&components(&format!("stage/example/{loaded}"))).unwrap();
            assert_eq!(loaded_normal, normal);
            for (form, matches) in [("normal", normal), ("battle", !normal)] {
                let source = components(&format!("stage/example/{form}"));
                let expected = if matches {
                    PatchTarget::Redirect(Hash40::new(&format!("stage/example/{form}_s02")))
                } else {
                    PatchTarget::OtherForm
                };
                assert_eq!(patch_target(&source, 2, loaded_normal), expected);
            }
        }
    }

    #[test]
    fn filtering_forms_does_not_hide_unknown_runtime_folders() {
        for (stage, unknown) in [("poke_kalos", 0xa2502e7a8), ("end", 0xe34681cb6)] {
            let path = [Hash40::new("stage"), Hash40::new(stage), Hash40(unknown)];
            for normal in [true, false] {
                assert_eq!(patch_target(&path, 1, normal), PatchTarget::Unsupported);
            }
        }
    }

    #[test]
    fn discovery_retains_existing_slot_and_form_rules() {
        for (form, normal) in [("normal", true), ("battle", false)] {
            for slot in [1, 2, 9, 10, 99, 100, 999] {
                let name = format!("{form}_s{slot:02}");
                assert_eq!(alternate_form(Hash40::new(name.as_str())), Some((slot, normal)));
            }
        }
        for invalid in ["normal", "battle", "normal_s00", "normal_s1", "normal_s1000", "model_s01"] {
            assert_eq!(alternate_form(Hash40::new(invalid)), None);
        }
    }

    #[test]
    fn stage_identity_accepts_base_and_alternate_forms_without_retaining_slots() {
        for (form, normal) in [("normal", true), ("battle", false)] {
            let expected = Some((Hash40::new("example"), normal));
            assert_eq!(stage_identity(&components(&format!("stage/example/{form}"))), expected);
            for slot in [1, 2, 3, 12, 18, 99, 100, 999] {
                let path = format!("stage/example/{form}_s{slot:02}");
                assert_eq!(stage_identity(&components(&path)), expected);
            }
        }
    }

    #[test]
    fn recovers_all_five_failed_stage_identities_from_the_runtime_log() {
        for (stage, raw_form) in [
            ("battlefield", 0xae6484135),
            ("end", 0xa7f41108f),
            ("battlefield_s", 0xa7f41108f),
            ("poke_stadium2", 0xae6484135),
            ("poke_kalos", 0xa914f71a3),
        ] {
            let path = [Hash40::new("stage"), Hash40::new(stage), Hash40(raw_form)];
            assert_eq!(stage_identity(&path), Some((Hash40::new(stage), true)));
        }
    }

    #[test]
    fn repeated_stage_visits_keep_identity_across_different_slots() {
        for stage in ["battlefield", "end", "poke_kalos"] {
            let expected = Some((Hash40::new(stage), true));
            for form in ["normal", "normal_s02", "normal_s01", "normal_s03", "normal"] {
                let path = format!("stage/{stage}/{form}");
                assert_eq!(stage_identity(&components(&path)), expected);
            }
        }
    }

    #[test]
    fn invalid_stage_paths_do_not_guess_a_stage_or_form() {
        for path in ["", "stage", "stage/example", "ui/example/normal", "stage/example/model/normal", "stage/example/unknown", "stage/example/normal/model", "stage/example/normal_s00"] {
            assert_eq!(stage_identity(&components(path)), None, "{path}");
        }
        assert_eq!(stage_identity(&[Hash40::new("stage"), Hash40(0), Hash40::new("normal")]), None);
    }

    #[test]
    fn parent_fallback_uses_stage_name_and_the_supplied_form() {
        for normal in [true, false] {
            assert_eq!(
                parent_stage_identity(&components("stage/battlefield"), normal),
                Some((Hash40::new("battlefield"), normal))
            );
        }
    }

    #[test]
    fn parent_fallback_rejects_non_stage_or_ambiguous_paths() {
        for path in ["", "stage", "battlefield", "ui/battlefield", "stage/battlefield/normal", "stage/battlefield/model"] {
            assert_eq!(parent_stage_identity(&components(path), true), None, "{path}");
        }
        assert_eq!(parent_stage_identity(&[Hash40::new("stage"), Hash40(0)], true), None);
    }
}
