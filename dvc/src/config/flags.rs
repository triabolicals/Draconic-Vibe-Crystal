use engage::{
    gameuserdata::GameUserData, gamevariable::GameVariableManager, menu::BasicMenuItemAttribute,
    gamedata::{Gamedata, skill::SkillData},
};
use crate::{
    DVCVariables, DVCConfig, utils::dlc_check,
    randomizer::{status::RandomizerStatus, data::GameData},
};
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
    RandomStartingApt = 41,
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
                Self::CustomUnits => DVCVariables::UnitRecruitment.get_value() == 1 && GameData::get().playables.len() > 41,
                Self::CustomUnitRecruitDisable => GameData::get().playables.len() > 41,
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
#[allow(non_upper_case_globals)]
impl DVCFlags {
    pub const FlagSet1:  &'static str = "G_DVC_Status";
    pub const FlagSet2: &'static str = "G_DVC_Status2";
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
            // 27
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
            41 => Some(Self::RandomStartingApt),
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
            else {
                let set = if v < 32 { Self::FlagSet1 } else { Self::FlagSet2 };
                let v = v % 32;
                GameVariableManager::get_number(set) & (1 << v) != 0
            }
        }
    }
    pub fn set_value(self, value: bool) {
        if DVCVariables::is_main_menu() { self.set_config(value); }
        else {
            let v = self as i32;
            if v < 50 {
                let set = if v < 32 { Self::FlagSet1 } else { Self::FlagSet2 };
                let bit = 1 << (v % 32);
                let mut flag_value =  GameVariableManager::get_number(set);
                if value { flag_value |= bit; } else { flag_value &= !bit; }
                GameVariableManager::set_number(set, flag_value);
            }
        }
    }
    pub fn set_value_from_config() {
        for x in 1..50 {
            if let Some(flag) = Self::from(x) {
                let v = flag.get_from_config();
                let set = if x < 32 { Self::FlagSet1 } else { Self::FlagSet2 };
                let bit = 1 << (x % 32);
                let mut flag_value =  GameVariableManager::get_number(set);
                if v { flag_value |= bit; } else { flag_value &= !bit; }
                GameVariableManager::set_number(set, flag_value);
            }
        }
    }
    pub fn get_by_index(index: i32) -> bool { Self::from(index).map(|v| v.get_value()).unwrap_or(false) }
    pub fn set_by_index(index: i32, value: bool) { if let Some(v) = Self::from(index) { v.set_value(value); } }
    pub fn get_from_config(self) -> bool {
        let config = DVCConfig::get();
        match self {
            Self::Initialized | Self::LueurJobSet | Self::SingleJobEnabled => { false }
            _ => { config.flags & (1 << self as i64) != 0 }
        }
    }
    pub fn set_config(self, value: bool) {
        let config = DVCConfig::get();
        match self {
            Self::Initialized | Self::LueurJobSet | Self::SingleJobEnabled => {}
            _ => {
                let mask = 1 << (self as i64);
                if value { config.flags |= mask; } else { config.flags &= !mask; }
            }
        }
    }
}