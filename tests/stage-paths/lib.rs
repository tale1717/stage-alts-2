// Compile the production path rules on the host without Skyline or game memory.
#[path = "../../src/stage_paths.rs"]
pub mod stage_paths;

#[cfg(test)]
mod tests {
    use super::stage_paths::{alternate_form, patch_target, redirect_path, PatchTarget};
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
            assert_eq!(patch_target(&components(source), 2), PatchTarget::Unsupported, "{source}");
        }
    }

    #[test]
    fn patching_skips_both_selected_and_other_alternate_slots() {
        for form in ["normal", "battle"] {
            for slot in [1, 2, 99, 100, 999] {
                let source = format!("stage/example/{form}_s{slot:02}/model/example_set");
                assert_eq!(patch_target(&components(&source), 2), PatchTarget::Alternate);
            }
        }
    }

    #[test]
    fn patching_still_redirects_base_entries() {
        let source = components("stage/example/normal/model/example_set");
        assert_eq!(patch_target(&source, 2), PatchTarget::Redirect(Hash40::new("stage/example/normal_s02/model/example_set")));
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
}
