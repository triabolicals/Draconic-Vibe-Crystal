use engage::gamedata::Gamedata;
use engage::gamedata::skill::SkillData;
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::menu::BasicMenuItemAttribute;
use crate::config::DVCVariables;
use crate::DVCConfig;
use crate::randomizer::data::GameData;
use crate::randomizer::status::RandomizerStatus;
use crate::utils::dlc_check;

pub const FLAGNAME: &'static str = "G_DVC_Status";
pub const FLAGNAME2: &'static str = "G_DVC_Status2";

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq,Ord, PartialOrd)]
pub enum DVCFlags {
    ContinuousModeItems = 1,
    CustomEmblemsRecruit = 2,
    LueurJobSet = 3,
    Ironman = 4,
    Tile = 5,
    CustomClass = 6,
    Autolevel = 7,
    PostChapterAutolevel = 8,
    EquipLearnSkills = 9,
    CustomUnitRecruitDisable = 10,
    ContinuousDLC = 11,
    RandomBossesNPCs = 12,
    RandomWeaponAsset = 13,
    RandomUnitInfo = 14,
    CustomSkillEnemy = 15,
    BGM = 16,
    GodNames = 17,
    EngageWeapons = 18,
    AddedShopItems = 19,
    RandomEventItems = 20,
    BondRing = 21,
    RandomDeploySpot = 22,
    EngageAttacks = 23,
    EmblemStats = 24,
    RandomSP = 25,
    RandomClassAttrs = 26,
    SingleJobEnabled = 27,
    AdaptiveGrowths = 28,
    RandomClassGrowth = 29,
    PersonalSkills = 30,
    EvolveItems = 31,
    CutsceneBGM = 32,
    CutsceneFacial = 33,
    CutsceneMotion = 34,
    CutsceneBackground = 35,
    RefineItem = 36,
    MaxStatCaps = 37,
    CutsceneSFX = 38,
    PersonalCaps = 39,
    RingStats = 40,
    PlayerAppearance = 50,
    CustomUnits = 51,
    Randomized = 52,
    RRGenderUnitMatch = 53,
    ExcludeDLCUnitRR = 54,
    ExcludeDLCEmblemRR = 55,
    Initialized = 56,
}
impl DVCFlags {
    pub fn need_confirm_to_change(&self) -> bool {
        if DVCVariables::is_main_menu() { false }
        else {
            match self {
                Self::RandomSP| Self::RandomClassAttrs  | Self::AdaptiveGrowths| Self::PersonalSkills  |
                Self::EmblemStats | Self::EngageAttacks| Self::BondRing | Self::EngageWeapons | Self::RandomBossesNPCs |
                Self::EquipLearnSkills | Self::RandomClassGrowth | Self::EvolveItems | Self::MaxStatCaps | Self::PersonalCaps |
                Self::RingStats
                => { true }
                _ => false,
            }
        }
    }
    pub fn can_a_call(&self, current: bool) -> bool {  // Require to ACall to Change Value
        if DVCVariables::is_main_menu() { false }
        else {
            match self {
                Self::BGM => { GameUserData::get_sequence() == 3 }
                Self::PostChapterAutolevel => {
                    current && (GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 5)
                }
                _ => { self.need_confirm_to_change() && current != self.get_value() }
            }
        }
    }
    pub fn build_menu_item(&self) -> BasicMenuItemAttribute {
        if DVCVariables::is_main_menu() {
            let enable =
            match self {
                Self::ExcludeDLCUnitRR => DVCVariables::UnitRecruitment.get_value() == 1 && dlc_check(),
                Self::ExcludeDLCEmblemRR => DVCVariables::EmblemRecruitment.get_value() == 1 && dlc_check(),
                Self::RRGenderUnitMatch => DVCVariables::UnitRecruitment.get_value() == 1,
                Self::ContinuousDLC => dlc_check(),
                Self::CustomUnits|Self::CustomUnitRecruitDisable => GameData::get().playables.len() > 41,
                Self::CustomEmblemsRecruit => DVCVariables::EmblemRecruitment.get_value() == 1 && GameData::get_playable_emblem_hashes().len() > 19,
                Self::CustomSkillEnemy => SkillData::get_count() > 1260,
                _ => true
            };
            if enable { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
        }
        else {
            let sequence = GameUserData::get_sequence();
            match self {
                Self::Ironman => BasicMenuItemAttribute::Disable,
                Self::RandomDeploySpot =>
                    if sequence == 2 || sequence == 3 { BasicMenuItemAttribute::Disable }
                    else { BasicMenuItemAttribute::Enable },

                Self::Tile =>
                    if sequence == 3 { BasicMenuItemAttribute::Disable }
                    else { BasicMenuItemAttribute::Enable },

                Self::ContinuousModeItems|Self::ContinuousDLC =>
                    if DVCVariables::is_continuous() { BasicMenuItemAttribute::Disable }
                    else { BasicMenuItemAttribute::Hide },
                _ => {
                    if (*self as i32) >= 50 { BasicMenuItemAttribute::Hide }
                    else { BasicMenuItemAttribute::Enable }
                }
            }
        }
    }
}

impl DVCFlags {
    pub fn from(v: i32) -> Option<Self> {
        match v {
            // 0 => Some(Self::Initialized),
            1 => Some(Self::ContinuousModeItems),
            2 => Some(Self::CustomEmblemsRecruit),
            // 3 => Some(Self::LueurJobSet),
            4 => Some(Self::Ironman),
            5 => Some(Self::Tile),
            6 => Some(Self::CustomClass),
            7 => Some(Self::Autolevel),
            8 => Some(Self::PostChapterAutolevel),
            9 => Some(Self::EquipLearnSkills),
            10 => Some(Self::CustomUnitRecruitDisable),
            11 => Some(Self::ContinuousDLC),
            12 => Some(Self::RandomBossesNPCs),
            13 => Some(Self::RandomWeaponAsset),
            14 => Some(Self::RandomUnitInfo),
            15 => Some(Self::CustomSkillEnemy),
            16 => Some(Self::BGM),
            17 => Some(Self::GodNames),
            18 => Some(Self::EngageWeapons),
            19 => Some(Self::AddedShopItems),
            20 => Some(Self::RandomEventItems),
            21 => Some(Self::BondRing),
            22 => Some(Self::RandomDeploySpot),
            23 => Some(Self::EngageAttacks),
            24 => Some(Self::EmblemStats),
            25 => Some(Self::RandomSP),
            26 => Some(Self::RandomClassAttrs),
            27 => None,
            28 => Some(Self::AdaptiveGrowths),
            29 => Some(Self::RandomClassGrowth),
            30 => Some(Self::PersonalSkills),
            31 => Some(Self::EvolveItems),
            32 => Some(Self::CutsceneBGM),
            33 => Some(Self::CutsceneFacial),
            34 => Some(Self::CutsceneMotion),
            35 => Some(Self::CutsceneBackground),
            36 => Some(Self::RefineItem),
            37 => Some(Self::MaxStatCaps),
            39 => Some(Self::PersonalCaps),
            40 => Some(Self::RingStats),
            // 37
            // 38
            50 => Some(Self::PlayerAppearance),
            51 => Some(Self::CustomUnits),
            52 => Some(Self::Randomized),
            53 => Some(Self::RRGenderUnitMatch),
            54 => Some(Self::ExcludeDLCUnitRR),
            55 => Some(Self::ExcludeDLCEmblemRR),
            _ => None,
        }

    }
    pub fn get_value(self) -> bool {
        let v = self as i32;
        if DVCVariables::is_main_menu() || v >= 50 { self.get_from_config() }
        else {
            if self == DVCFlags::Initialized { RandomizerStatus::is_init() }
            else if v < 32 { GameVariableManager::get_number(FLAGNAME) & (1 << v) != 0 }
            else {
                let num = GameVariableManager::get_number(FLAGNAME2);
                num & (1 << (v - 32)) != 0
            }
        }
    }
    pub fn set_value(self, value: bool) {
        if DVCVariables::is_main_menu() { self.set_config(value); }
        else {
            let v = self as i32;
            if v < 50 {
                let mut flag_value =
                    if v < 32 { GameVariableManager::get_number(FLAGNAME) }
                    else { GameVariableManager::get_number(FLAGNAME2) };

                let bit = if v < 32 { 1 << v } else { 1 << (v-32) };
                if value { flag_value |= bit; } else { flag_value &= !bit; }

                if v < 32 { GameVariableManager::set_number(FLAGNAME, flag_value); }
                else { GameVariableManager::set_number(FLAGNAME2, flag_value); }
            }
        }
    }
    pub fn set_value_from_config() {
        for x in 1..50 {
            if let Some(flag) = Self::from(x) {
                let v = flag.get_from_config();
                let mut flag_value =
                    if x < 32 { GameVariableManager::get_number(FLAGNAME) }
                    else { GameVariableManager::get_number(FLAGNAME2) };

                let bit = if x < 32 { 1 << x } else { 1 << (x-32) };
                if v { flag_value |= bit; } else { flag_value &= !bit; }

                if x < 32 { GameVariableManager::set_number(FLAGNAME, flag_value); }
                else { GameVariableManager::set_number(FLAGNAME2, flag_value); }
            }
        }
    }
    pub fn get_by_index(index: i32) -> bool { Self::from(index).map(|v| v.get_value()).unwrap_or(false) }
    pub fn set_by_index(index: i32, value: bool) { if let Some(v) = Self::from(index) { v.set_value(value); } }
    pub fn get_from_config(self) -> bool {
        let config = DVCConfig::get();
        match self {
            Self::Initialized | Self::LueurJobSet => { false }
            Self::RandomEventItems => { config.random_item }
            Self::ContinuousModeItems => config.continuous_items,
            Self::CustomEmblemsRecruit => get_bit_from_bool(config.recruitment_option, 0),
            Self::Ironman => config.ironman,
            Self::Tile => config.tile,
            Self::CustomClass => config.custom_jobs,
            Self::Autolevel => config.autolevel,
            Self::PostChapterAutolevel => config.post_chapter_scaling,
            Self::EquipLearnSkills => config.equip_learn_skill,
            Self::CustomUnitRecruitDisable => get_bit_from_bool(config.recruitment_option, 1),
            Self::ContinuousDLC => config.continuous_dlc,
            Self::RandomBossesNPCs => config.bosses,
            Self::RandomWeaponAsset => config.weapon_asset,
            Self::RandomUnitInfo => config.unit_info_asset,
            Self::CustomSkillEnemy => config.enemy_custom_skill,
            Self::BGM => config.random_map_bgm,
            Self::GodNames => config.random_names,
            Self::EngageWeapons => config.random_engage_weapon,
            Self::AddedShopItems => config.random_shop_items,
            Self::BondRing => config.bond_ring_skill,
            Self::RandomDeploySpot => config.random_deploy_spots,
            Self::EngageAttacks => config.random_engage_attacks,
            Self::EmblemStats => config.emblem_stats,
            Self::RandomSP => config.random_skill_cost,
            Self::RandomClassAttrs => config.random_job_attrs,
            Self::AdaptiveGrowths => config.adaptive_growth,
            Self::RandomClassGrowth => config.class_growth,
            Self::PersonalSkills => config.personal_skills,
            Self::EvolveItems => config.random_evolve_items,
            Self::RefineItem => { config.random_refine }
            Self::PlayerAppearance => get_bit_from_bool(config.recruitment_option, 6),
            Self::CustomUnits => get_bit_from_bool(config.recruitment_option, 2),
            Self::Randomized => config.randomized,
            Self::CutsceneBGM => get_bit_from_bool(config.cutscene_options, 0),
            Self::CutsceneFacial => get_bit_from_bool(config.cutscene_options, 1),
            Self::CutsceneMotion => get_bit_from_bool(config.cutscene_options, 2),
            Self::CutsceneBackground => get_bit_from_bool(config.cutscene_options, 3),
            Self::CutsceneSFX => get_bit_from_bool(config.cutscene_options, 4),
            Self::RRGenderUnitMatch => get_bit_from_bool(config.recruitment_option, 3),
            Self::ExcludeDLCUnitRR => get_bit_from_bool(config.recruitment_option, 4),
            Self::ExcludeDLCEmblemRR => get_bit_from_bool(config.recruitment_option, 5),
            Self::MaxStatCaps => config.max_stat_caps,
            Self::PersonalCaps => { config.personal_caps  }
            Self::SingleJobEnabled => false,
            Self::RingStats => { config.bond_ring_stat }
        }
    }
    pub fn set_config(self, value: bool) {
        let config = DVCConfig::get();
        match self {
            // Self::Initialized|Self::LueurJobSet => {}
            Self::RandomEventItems => { config.random_item = value }
            Self::ContinuousModeItems => { config.continuous_items = value; },
            Self::CustomEmblemsRecruit => set_bit_from_bool(&mut config.recruitment_option, 0, value),
            Self::Ironman => { config.ironman = value; }
            Self::Tile => { config.tile = value;}
            Self::CustomClass => { config.custom_jobs = value; }
            Self::Autolevel => { config.autolevel = value; }
            Self::PostChapterAutolevel => { config.post_chapter_scaling = value; }
            Self::EquipLearnSkills => { config.equip_learn_skill = value; }
            Self::CustomUnitRecruitDisable => set_bit_from_bool(&mut config.recruitment_option, 1, value),
            Self::ContinuousDLC => { config.continuous_dlc = value; }
            Self::RandomBossesNPCs => { config.bosses = value; }
            Self::RandomWeaponAsset  => { config.weapon_asset = value; }
            Self::RandomUnitInfo => { config.unit_info_asset = value; }
            Self::CustomSkillEnemy => { config.enemy_custom_skill = value;}
            Self::BGM => { config.random_map_bgm = value; }
            Self::GodNames => { config.random_names = value; }
            Self::EngageWeapons => { config.random_engage_weapon = value; }
            Self::AddedShopItems => { config.random_shop_items = value;}
            Self::BondRing => { config.bond_ring_skill = value;}
            Self::RandomDeploySpot => { config.random_deploy_spots = value; }
            Self::EngageAttacks => { config.random_engage_attacks = value; }
            Self::EmblemStats => { config.emblem_stats = value; }
            Self::RandomSP => { config.random_skill_cost = value;}
            Self::RandomClassAttrs => { config.random_job_attrs = value; }
            Self::AdaptiveGrowths => { config.adaptive_growth = value; }
            Self::RandomClassGrowth => { config.class_growth = value; }
            Self::PersonalSkills => { config.personal_skills = value; }
            Self::EvolveItems => { config.random_evolve_items = value; }
            Self::RefineItem => { config.random_refine = value; }
            Self::CustomUnits => set_bit_from_bool(&mut config.recruitment_option, 2, value),
            Self::Randomized => config.randomized = value,
            Self::CutsceneBGM => set_bit_from_bool(&mut config.cutscene_options, 0, value),
            Self::CutsceneFacial => set_bit_from_bool(&mut config.cutscene_options, 1, value),
            Self::CutsceneMotion => set_bit_from_bool(&mut config.cutscene_options, 2, value),
            Self::CutsceneBackground => set_bit_from_bool(&mut config.cutscene_options, 3, value),
            Self::CutsceneSFX => set_bit_from_bool(&mut config.cutscene_options, 4, value),
            Self::RRGenderUnitMatch => set_bit_from_bool(&mut config.recruitment_option, 3, value),
            Self::ExcludeDLCUnitRR => set_bit_from_bool(&mut config.recruitment_option, 4, value),
            Self::ExcludeDLCEmblemRR => set_bit_from_bool(&mut config.recruitment_option, 5, value),
            Self::PlayerAppearance => set_bit_from_bool(&mut config.recruitment_option, 6, value),
            Self::MaxStatCaps => { config.max_stat_caps = value; }
            Self::PersonalCaps => { config.personal_caps = value; }
            Self::RingStats => { config.bond_ring_stat = value; }
            _ => {}
        }
    }
}
fn set_bit_from_bool(v: &mut i32, bit: usize, value: bool) {
    let mask = 1 << bit;
    if value { *v |= mask; } else { *v &= !mask; }
}
fn get_bit_from_bool(v: i32, bit: usize) -> bool {
    let mask = 1 << bit;
    v & mask != 0
}