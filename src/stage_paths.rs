//! Stage path rules independent of the game's mutable filesystem tables.

use hash40::Hash40;

/// Recognize the same numbered forms used when discovering installed alts.
pub fn alternate_form(hash: Hash40) -> Option<(usize, bool)> {
    let slots = match hash.0 >> 32 {
        10 => 1..100,
        11 => 100..1000,
        _ => return None,
    };

    for slot in slots {
        let mut name = format!("normal_s{slot:02}");
        if Hash40::new(name.as_str()) == hash {
            return Some((slot, true));
        }
        name.replace_range(..6, "battle");
        if Hash40::new(name.as_str()) == hash {
            return Some((slot, false));
        }
    }
    None
}

fn stage_form(components: &[Hash40]) -> Option<Hash40> {
    if components.len() >= 3 && components[0] == Hash40::new("stage") {
        Some(components[2])
    } else {
        None
    }
}

/// Redirect a base form, or retain a path already inside the selected form.
/// Only the third component of stage/<name>/<form>/... can be replaced.
pub fn redirect_path(components: &[Hash40], alt: usize) -> Option<Hash40> {
    let form = stage_form(components)?;
    let normal = Hash40::new("normal");
    let battle = Hash40::new("battle");
    let (selected_normal, selected_battle) = if alt == 0 {
        (normal, battle)
    } else {
        (
            Hash40::new(format!("normal_s{alt:02}").as_str()),
            Hash40::new(format!("battle_s{alt:02}").as_str()),
        )
    };

    let target_form = if form == normal {
        selected_normal
    } else if form == battle {
        selected_battle
    } else if form == selected_normal || form == selected_battle {
        form
    } else {
        // Do not redirect another slot's assets or guess an unknown form.
        return None;
    };

    let mut path = components[0];
    for (index, component) in components.iter().copied().enumerate().skip(1) {
        path = path.concat(Hash40::new("/"));
        path = path.concat(if index == 2 { target_form } else { component });
    }
    Some(path)
}

#[derive(Debug, PartialEq, Eq)]
pub enum PatchTarget {
    Redirect(Hash40),
    Alternate,
    Unsupported,
}

/// Alternate-slot assets are sources, not entries to overwrite when patching
/// the base stage's directory/search tables.
pub fn patch_target(components: &[Hash40], alt: usize) -> PatchTarget {
    if stage_form(components).and_then(alternate_form).is_some() {
        return PatchTarget::Alternate;
    }
    match redirect_path(components, alt) {
        Some(path) => PatchTarget::Redirect(path),
        None => PatchTarget::Unsupported,
    }
}
