use engage::app::{BasicMenuItem_Attribute, IStructData_1Methods, SkillData};
use crate::dvc::DVC;
pub use super::*;
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq,Ord, PartialOrd)]
pub enum DVCFlags {
    RandomEnable = 0,
    ContinuousModeItems = 1,
    CustomEmblemsRecruit = 2,
    LueurJobSet = 3,
    Ironman = 4,
    RandomExplorationItems = 5,
    CustomClass = 6,
    Autolevel = 7,
    PostChapterAutolevel = 8,
    EquipLearnSkills = 9,
    CustomUnitRecruitDisable = 10,
    ContinuousExp = 11,
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
    WellItems = 42,
    WeaponStats = 43,
    Supports = 44,
    WeaponSkill = 45,
    PlayerAppearance = 50,
    CustomUnits = 51,
    OnlyRecruitmentSetting = 52,
    RRGenderUnitMatch = 53,
    ExcludeDLCUnitRR = 54,
    ExcludeDLCEmblemRR = 55,
    Initialized = 56,
}
#[allow(non_upper_case_globals)]
impl DVCFlags {
    pub const FlagSet1:  &'static str = "G_DVC_Status";
    pub const FlagSet2: &'static str = "G_DVC_Status2";
    pub fn get_i32() -> (i32, i32) {
        (GameVariableManager::get_number(Self::FlagSet1), GameVariableManager::get_number(Self::FlagSet2))
    }
    pub fn set_i32(v1: Option<i32>, v2: Option<i32>) {
        if let Some(v) = v1 { GameVariableManager::set_number(Self::FlagSet1, v); }
        if let Some(v2) = v2 { GameVariableManager::set_number(Self::FlagSet2, v2); }
    }
    pub fn from(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::RandomEnable),
            1 => Some(Self::ContinuousModeItems),
            2 => Some(Self::CustomEmblemsRecruit),
            3 => None, // LueurJobSet
            4 => Some(Self::Ironman),
            5 => Some(Self::RandomExplorationItems),
            6 => Some(Self::CustomClass),
            7 => Some(Self::Autolevel),
            8 => Some(Self::PostChapterAutolevel),
            9 => Some(Self::EquipLearnSkills),
            10 => Some(Self::CustomUnitRecruitDisable),
            11 => Some(Self::ContinuousExp),
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
            // 27   SingleJobEnable
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
            42 => Some(Self::WellItems),
            43 => Some(Self::WeaponStats),
            44 => Some(Self::Supports),
            45 => Some(Self::WeaponSkill),
            50 => Some(Self::PlayerAppearance),
            51 => Some(Self::CustomUnits),
            52 => Some(Self::OnlyRecruitmentSetting),
            53 => Some(Self::RRGenderUnitMatch),
            54 => Some(Self::ExcludeDLCUnitRR),
            55 => Some(Self::ExcludeDLCEmblemRR),
            _ => None,
        }
    }
    pub fn rebuild_menu_custom_call(&self) -> bool { *self == Self::RandomEnable || *self == Self::RandomExplorationItems || *self == Self::BondRing }
    pub fn is_adjustable(&self) -> bool {
        if DVCVariables::is_main_menu() { true }
        else {
            let sequence = DVCVariables::get_sequence();
            match self {
                Self::Ironman => false,
                Self::RandomDeploySpot|Self::Autolevel => sequence != 2 && sequence != 3,
                _ => { true }
            }
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItem_Attribute {
        if DVCVariables::is_main_menu() {
            let enable =
                match self {
                    Self::ContinuousModeItems|Self::ContinuousExp => DVCVariable::Continuous.get_setting_value() != 0,
                    Self::OnlyRecruitmentSetting => !DVCFlags::RandomEnable.get_from_config() && (DVCVariable::EmblemRecruitment.get_value() != 0 || DVCVariable::UnitRecruitment.get_value() != 0),
                    Self::ExcludeDLCUnitRR => DVCVariable::UnitRecruitment.get_value() == 1 && DVCVariables::has_dlc(),
                    Self::ExcludeDLCEmblemRR => DVCVariable::EmblemRecruitment.get_value() == 1 && DVCVariables::has_dlc(),
                    Self::RRGenderUnitMatch => DVCVariable::UnitRecruitment.get_value() == 1,
                    Self::CustomUnits => DVCVariable::UnitRecruitment.get_value() == 1 && gamedata::GameData::get().person_pool.playable.len() > 41,
                    Self::CustomUnitRecruitDisable =>  gamedata::GameData::get().person_pool.playable.len() > 41,
                    Self::CustomEmblemsRecruit => DVCVariable::EmblemRecruitment.get_value() == 1 && gamedata::GameData::get().emblem_pool.recruitment_list.len() > 19,
                    Self::CustomSkillEnemy => SkillData::get_count() > 1260,
                    _ => true
                };
            if enable { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::hide() }
        }
        else {
            match self {
                Self::ContinuousModeItems|Self::ContinuousExp => {
                    if DVCVariables::is_continuous() { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::hide() }
                },
                Self::Ironman => BasicMenuItem_Attribute::enable(),
                _ => {
                    if (*self as i32) >= 50 { BasicMenuItem_Attribute::hide() }
                    else if self.is_bypass_disable() { BasicMenuItem_Attribute::enable() }
                    else if DVCFlags::RandomEnable.get_value() { BasicMenuItem_Attribute::enable() }
                    else { BasicMenuItem_Attribute::disable() }
                }
            }
        }
    }
    pub fn get_setting_value(self) -> bool {
        let v = self as i32;
        if DVCVariables::is_main_menu() || v >= 50 { self.get_from_config() }
        else {
            if self == DVCFlags::Initialized { DVC::get().init }
            else {
                let set = if v < 32 { Self::FlagSet1 } else { Self::FlagSet2 };
                let v = v % 32;
                GameVariableManager::get_number(set) & (1 << v) != 0
            }
        }
    }
    pub fn get_value(self) -> bool {
        let v = self as i32;
        if DVCVariables::is_main_menu() || v >= 50 { self.get_from_config() }
        else if v < 50 && (self.is_bypass_disable() || GameVariableManager::get_number(Self::FlagSet1) & 1 != 0){
            self.get_setting_value()
        }
        else { false }
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
                if !GameVariableManager::is_exist(set) { GameVariableManager::make_entry_norewind(set, flag_value); }
                else { GameVariableManager::set_number(set, flag_value); }
            }
        }
    }
    pub fn is_bypass_disable(self) -> bool {
        let v = self as i64;
        (1 << v) & 0x2010000397i64 != 0
    }
    pub fn set_value_from_config(set_value: bool){
        let mut flag_value = 0i64;
        for x in 0..50 {
            if let Some(flag) = Self::from(x) {
                let v = flag.get_from_config();
                if v && (set_value || flag.is_bypass_disable()){ flag_value |= 1 << x; }
            }
        }
        let flag_1 = (flag_value & 0xFFFFFFFF) as i32;
        let flag_2 = ((flag_value >> 32) & 0xFFFFFFFF) as i32;
        GameVariableManager::set_number(Self::FlagSet1, flag_1);
        GameVariableManager::set_number(Self::FlagSet2, flag_2);
    }
    pub fn get_by_index(index: i32) -> bool { Self::from(index).map(|v| v.get_value()).unwrap_or(false) }
    pub fn set_by_index(index: i32, value: bool) { if let Some(v) = Self::from(index) { v.set_value(value); } }
    pub fn get_from_config(self) -> bool {
        let config = DVCConfig::get();
        match self {
            Self::RandomEnable => config.randomized,
            Self::Initialized | Self::LueurJobSet | Self::SingleJobEnabled => { false }
            _ => { config.flags & (1 << self as i64) != 0 }
        }
    }
    pub fn set_config(self, value: bool) {
        let config = DVCConfig::get();
        match self {
            Self::RandomEnable => { config.randomized = value; }
            Self::Initialized | Self::LueurJobSet | Self::SingleJobEnabled => {}
            _ => {
                let mask = 1 << (self as i64);
                if value { config.flags |= mask; } else { config.flags &= !mask; }
            }
        }
    }
}