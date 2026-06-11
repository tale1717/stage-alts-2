use std::{collections::BTreeMap, ptr::NonNull};

use locks::RwLock;
use smash_arc::{FilePath, Hash40, HashToIndex};

use crate::{lua, music_fix::MusicCache, utils::ConcatHash};

pub static MANAGER: RwLock<AltManager> = RwLock::new(AltManager::new());

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct StageInfo {
    pub name: Hash40,
    pub normal_form: bool,
}

impl StageInfo {
    pub fn from_path(hash: Hash40) -> Option<Self> {
        let pretty = hash.pretty();

        let components = pretty.components();
        if components.len() != 3 {
            return None;
        }

        if components[0] != Hash40::from("stage") {
            return None;
        }

        let name = components[1];

        let normal_form = if components[2] == Hash40::from("normal") {
            true
        } else if components[2] == Hash40::from("battle") {
            false
        } else {
            return None;
        };

        Some(Self { name, normal_form })
    }
}

fn dlc_stages() -> [Hash40; 11] {
    [
        Hash40::from("jack_mementoes"),
        Hash40::from("brave_altar"),
        Hash40::from("buddy_spiral"),
        Hash40::from("dolly_stadium"),
        Hash40::from("fe_shrine"),
        Hash40::from("tantan_spring"),
        Hash40::from("pickel_world"),
        Hash40::from("ff_cave"),
        Hash40::from("xeno_alst"),
        Hash40::from("demon_dojo"),
        Hash40::from("trail_castle"),
    ]
}

#[derive(Copy, Clone, Debug)]
pub enum StageKind {
    Training,
    Battlefield,
    SmallBattlefield,
    BigBattlefield,
    FinalDestination,

    Normal(Hash40),
    DLC(Hash40),
}

impl StageKind {
    pub fn as_hash(&self) -> Hash40 {
        match self {
            Self::Training => Hash40::from("training"),
            Self::Battlefield => Hash40::from("battlefield"),
            Self::SmallBattlefield => Hash40::from("battlefield_s"),
            Self::BigBattlefield => Hash40::from("battlefield_l"),
            Self::FinalDestination => Hash40::from("end"),
            Self::Normal(normal) => *normal,
            Self::DLC(dlc) => *dlc,
        }
    }

    pub fn as_ui_hash(&self) -> Hash40 {
        match self {
            Self::SmallBattlefield => Hash40::from("battlefields"),
            Self::BigBattlefield => Hash40::from("battlefieldl"),
            _ => self.as_hash(),
        }
    }
}

impl From<Hash40> for StageKind {
    fn from(value: Hash40) -> Self {
        if value == Hash40::from("training") {
            Self::Training
        } else if value == Hash40::from("battlefield") {
            Self::Battlefield
        } else if value == Hash40::from("battlefield_s") {
            Self::SmallBattlefield
        } else if value == Hash40::from("battlefield_l") {
            Self::BigBattlefield
        } else if value == Hash40::from("end") {
            Self::FinalDestination
        } else if dlc_stages().contains(&value) {
            Self::DLC(value)
        } else {
            Self::Normal(value)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UiPaths {
    pub normal: Hash40,
    pub battle: Hash40,
    pub end: Hash40,
}

impl UiPaths {
    pub fn new(value: StageKind, alt_id: usize) -> Self {
        let extension = if alt_id == 0 {
            Hash40::from(".bntx")
        } else {
            Hash40::from(format!("_s{alt_id:02}.bntx").as_str())
        };

        let normal = match value {
            kind @ (StageKind::DLC(_) | StageKind::SmallBattlefield) => {
                Hash40::from("ui/replace_patch/stage/stage_2/stage_2_")
                    .concat(kind.as_ui_hash())
                    .concat(extension)
            }
            kind => Hash40::from("ui/replace/stage/stage_2/stage_2_")
                .concat(kind.as_ui_hash())
                .concat(extension),
        };

        let battle = match value {
            StageKind::Training => normal,
            StageKind::Battlefield | StageKind::BigBattlefield | StageKind::SmallBattlefield => {
                Hash40::from("ui/replace/stage/stage_4/stage_4_battlefield").concat(extension)
            }
            StageKind::DLC(hash) => Hash40::from("ui/replace_patch/stage/stage_4/stage_4_")
                .concat(hash)
                .concat(extension),
            other => Hash40::from("ui/replace/stage/stage_4/stage_4_")
                .concat(other.as_hash())
                .concat(extension),
        };

        let end = match value {
            StageKind::Training | StageKind::FinalDestination => normal,
            StageKind::Battlefield | StageKind::BigBattlefield | StageKind::SmallBattlefield => {
                Hash40::from("ui/replace/stage/stage_3/stage_3_battlefield_")
                    .concat(format!("s{alt_id:02}.bntx").as_str())
            }
            StageKind::DLC(hash) => Hash40::from("ui/replace_patch/stage/stage_3/stage_3_")
                .concat(hash)
                .concat(extension),
            other => Hash40::from("ui/replace/stage/stage_3/stage_3_")
                .concat(other.as_hash())
                .concat(extension),
        };

        Self {
            normal,
            battle,
            end,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AltInfo {
    pub slot_value: usize,
    pub wifi_safe: bool,
    pub ui_paths: UiPaths,
}

#[derive(Copy, Clone, Debug)]
pub struct SelectedAltInfo {
    pub index: usize,
    pub stage_info: StageInfo,
}

impl Default for SelectedAltInfo {
    fn default() -> Self {
        Self {
            index: 0,
            stage_info: StageInfo {
                name: Hash40::from("battlefield"),
                normal_form: true,
            },
        }
    }
}

pub enum PlayableAlts {
    OneStage(SelectedAltInfo),
    TwoStages([SelectedAltInfo; 2]),
    ThreeStages([SelectedAltInfo; 3]),
    RandomAuto {
        normal_form: bool,
    },
}

pub struct SelectedAlts {
    pub playable: PlayableAlts,
    pub current_index: usize,
}

fn log_selected_alt(label: &str, info: SelectedAltInfo) {
    log::info!(
        "[stage-alts] {label}: stage={} normal_form={} selected_index={}",
        info.stage_info.name.pretty(),
        info.stage_info.normal_form,
        info.index
    );
}

fn log_resolved_alt(label: &str, info: SelectedAltInfo, resolved: Option<AltInfo>) {
    match resolved {
        Some(alt) => log::info!(
            "[stage-alts] {label}: stage={} normal_form={} selected_index={} resolved_slot={}",
            info.stage_info.name.pretty(),
            info.stage_info.normal_form,
            info.index,
            alt.slot_value
        ),
        None => log::warn!(
            "[stage-alts] {label}: stage={} normal_form={} selected_index={} resolved_slot=None",
            info.stage_info.name.pretty(),
            info.stage_info.normal_form,
            info.index
        ),
    }
}

pub struct AltManager {
    pub alts: BTreeMap<StageInfo, Vec<AltInfo>>,
    pub selected_alts: Option<SelectedAlts>,

    pub backup_filepaths: BTreeMap<Hash40, u32>,
    pub backup_searchpaths: BTreeMap<Hash40, u32>,

    // For lua
    pub index_to_hash: BTreeMap<usize, Hash40>,
    pub ui_to_place: BTreeMap<Hash40, Hash40>,

    pub current_singleton: Option<NonNull<()>>,

    pub music_cache: Option<MusicCache>,

    pub stage_data: Option<Vec<u8>>,
    pub bgm_data: Option<Vec<u8>>,
}

impl AltManager {
    fn try_create_singleton_backups(&mut self) {
        let Some(stage) = self.stage_data.as_ref() else {
            return;
        };

        let Some(bgm) = self.bgm_data.as_ref() else {
            return;
        };

        self.music_cache = Some(MusicCache::new(stage, bgm));
        self.ui_to_place = lua::get_ui_hash_to_stage_hash(stage);

        self.stage_data = None;
        self.bgm_data = None;
    }

    pub const fn new() -> Self {
        Self {
            alts: BTreeMap::new(),
            selected_alts: None,
            backup_filepaths: BTreeMap::new(),
            backup_searchpaths: BTreeMap::new(),
            index_to_hash: BTreeMap::new(),
            ui_to_place: BTreeMap::new(),
            current_singleton: None,
            music_cache: None,
            stage_data: None,
            bgm_data: None,
        }
    }

    pub fn set_stage_data(&mut self, stage: &[u8]) {
        self.stage_data = Some(stage.to_vec());
        self.try_create_singleton_backups();
    }

    pub fn set_bgm_data(&mut self, bgm: &[u8]) {
        self.bgm_data = Some(bgm.to_vec());
        self.try_create_singleton_backups();
    }

    pub fn add_alt(&mut self, stage_info: StageInfo, alt: usize, kind: StageKind) {
        self.alts.entry(stage_info).or_default().push(AltInfo {
            slot_value: alt,
            wifi_safe: true,
            ui_paths: UiPaths::new(kind, alt),
        });
    }

    pub fn nth_alt(&self, info: StageInfo, index: usize) -> Option<AltInfo> {
        if index == 0 {
            return None;
        }

        self.alts
            .get(&info)
            .and_then(|list| list.get(index - 1))
            .copied()
    }

    pub fn set_alts(
        &mut self,
        first: SelectedAltInfo,
        second: Option<SelectedAltInfo>,
        third: Option<SelectedAltInfo>,
    ) {

        log::info!("[stage-alts] set_alts called");
        log_selected_alt("set_alts first", first);

        if let Some(second) = second {
            log_selected_alt("set_alts second", second);
        } else {
            log::info!("[stage-alts] set_alts second=None");
        }

        if let Some(third) = third {
            log_selected_alt("set_alts third", third);
        } else {
            log::info!("[stage-alts] set_alts third=None");
        }

        let playable = match (second, third) {
            (Some(second), Some(third)) => PlayableAlts::ThreeStages([first, second, third]),
            (Some(second), None) => PlayableAlts::TwoStages([first, second]),
            (None, Some(third)) => PlayableAlts::TwoStages([first, third]),
            (None, None) => PlayableAlts::OneStage(first),
        };

        self.selected_alts = Some(SelectedAlts {
            playable,
            current_index: 0,
        });
    }

    pub fn clear_alts(&mut self) {
        log::info!("[stage-alts] clear_alts called");
        self.selected_alts = None;
    }

    pub fn set_random_auto(&mut self, normal_form: bool) {
        let current_index = match &self.selected_alts {
            Some(SelectedAlts {
                     playable: PlayableAlts::RandomAuto {
                         normal_form: existing_normal_form,
                     },
                     current_index,
                 }) if *existing_normal_form == normal_form => *current_index,
            _ => 0,
        };

        log::info!(
            "[stage-alts] set_random_auto called: normal_form={} current_index={}",
            normal_form,
            current_index
    );

        self.selected_alts = Some(SelectedAlts {
            playable: PlayableAlts::RandomAuto { normal_form },
            current_index,
        });
    }

    pub fn is_random_auto(&self) -> bool {
        matches!(
        self.selected_alts,
        Some(SelectedAlts {
            playable: PlayableAlts::RandomAuto { .. },
            ..
        })
    )
    }

    pub fn stage_info_from_parent_path(&self, parent_path: Hash40) -> Option<StageInfo> {
        self.alts
            .keys()
            .copied()
            .find(|stage| stage.name == parent_path)
    }

    pub fn fetch_advance(&mut self) -> Option<usize> {
        let Some(alts) = self.selected_alts.as_mut() else {
            log::warn!("[stage-alts] fetch_advance selected_alts=None");
            return None;
        };

        match alts.playable {
            PlayableAlts::OneStage(info) => {
                let resolved = self.nth_alt(info.stage_info, info.index);
                log_resolved_alt("fetch_advance OneStage", info, resolved);
                resolved.map(|info| info.slot_value)
            }
            PlayableAlts::TwoStages(infos) => {
                let cycle_index = alts.current_index % 2;
                let info = infos[cycle_index];
                alts.current_index += 1;
                let resolved = self.nth_alt(info.stage_info, info.index);
                log::info!("[stage-alts] fetch_advance TwoStages cycle_index={cycle_index}");
                log_resolved_alt("fetch_advance TwoStages", info, resolved);
                resolved.map(|info| info.slot_value)
            }
            PlayableAlts::ThreeStages(infos) => {
                let cycle_index = alts.current_index % 3;
                let info = infos[cycle_index];
                alts.current_index += 1;
                let resolved = self.nth_alt(info.stage_info, info.index);
                log::info!("[stage-alts] fetch_advance ThreeStages cycle_index={cycle_index}");
                log_resolved_alt("fetch_advance ThreeStages", info, resolved);
                resolved.map(|info| info.slot_value)
            }

            PlayableAlts::RandomAuto { normal_form } => {
                let candidates: Vec<(StageInfo, usize)> = self
                    .alts
                    .iter()
                    .filter(|(stage, alts)| stage.normal_form == normal_form && !alts.is_empty())
                    .flat_map(|(stage, alts)| {
                        alts.iter()
                            .enumerate()
                            .map(move |(index, _)| (*stage, index + 1))
                    })
                    .collect();

                if candidates.is_empty() {
                    log::warn!(
                        "[stage-alts] fetch_advance RandomAuto: no candidates normal_form={}",
                        normal_form
                    );
                    return None;
                }

                let candidate_index = alts.current_index % candidates.len();
                alts.current_index += 1;

                let (stage_info, selected_index) = candidates[candidate_index];
                let resolved = self.nth_alt(stage_info, selected_index);

                log::info!(
                    "[stage-alts] fetch_advance RandomAuto: candidate_index={} candidate_count={} stage={} normal_form={} selected_index={}",
                    candidate_index,
                    candidates.len(),
                    stage_info.name.pretty(),
                    stage_info.normal_form,
                    selected_index
                );

                log_resolved_alt(
                    "fetch_advance RandomAuto",
                    SelectedAltInfo {
                        index: selected_index,
                        stage_info,
                    },
                    resolved,
                );

                resolved.map(|info| info.slot_value)
            }

        }
    }

    pub fn fetch_advance_for_stage(&mut self, stage_info: StageInfo) -> Option<usize> {
        let Some(alts) = self.selected_alts.as_mut() else {
            log::warn!("[stage-alts] fetch_advance_for_stage selected_alts=None");
            return None;
        };

        match alts.playable {
            PlayableAlts::RandomAuto { normal_form } => {
                if stage_info.normal_form != normal_form {
                    log::warn!(
                        "[stage-alts] fetch_advance_for_stage RandomAuto form mismatch: stage={} stage_normal_form={} random_normal_form={}",
                        stage_info.name.pretty(),
                        stage_info.normal_form,
                        normal_form
                    );
                    return None;
                }

                let Some(stage_alts) = self.alts.get(&stage_info) else {
                    log::info!(
                        "[stage-alts] fetch_advance_for_stage RandomAuto: default_vanilla no_alt_entry actual_stage={} normal_form={}",
                        stage_info.name.pretty(),
                        stage_info.normal_form
                );
                    return None;
                };

                if stage_alts.is_empty() {
                    log::info!(
                        "[stage-alts] fetch_advance_for_stage RandomAuto: default_vanilla empty_alt_list actual_stage={} normal_form={}",
                        stage_info.name.pretty(),
                        stage_info.normal_form
                        );
                    return None;
                }

                let selected_index = (alts.current_index % stage_alts.len()) + 1;
                alts.current_index += 1;

                let resolved = self.nth_alt(stage_info, selected_index);

                log::info!(
                    "[stage-alts] fetch_advance_for_stage RandomAuto: actual_stage={} normal_form={} selected_index={} alt_count={}",
                    stage_info.name.pretty(),
                    stage_info.normal_form,
                    selected_index,
                    stage_alts.len()
                );

                log_resolved_alt(
                    "fetch_advance_for_stage RandomAuto",
                    SelectedAltInfo {
                        index: selected_index,
                        stage_info,
                    },
                    resolved,
                );

                resolved.map(|info| info.slot_value)
            }
            _ => self.fetch_advance(),
        }
    }

    pub fn fetch_alt_for_stage(&self, stage: Hash40, alt: usize) -> Option<usize> {
        self.nth_alt(
            StageInfo {
                name: stage,
                normal_form: true,
            },
            alt,
        )
        .map(|alt| alt.slot_value)
    }
}
