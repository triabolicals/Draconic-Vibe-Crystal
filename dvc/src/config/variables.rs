use engage::{
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
    gamedata::{Gamedata, GodData, JobData, PersonData},
    menu::BasicMenuItemAttribute, random::Random,
    unit::{UnitPool, Unit},
    god::{GodPool}
};
use crate::{
    utils, DVCConfig, config::DVCFlags,
    utils::{clamp_value, dlc_check},
    randomizer::data::GameData,
    enums::{EMBLEM_GIDS, PIDS}
};
use std::cmp::PartialEq;
use unity::prelude::Il2CppString;



/// Structure that contains and manages DVC-Related GameVariables
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum DVCVariables {
    Continuous = 0, // "G_Continuous"
    UnitRecruitment = 1,    //"G_Random_Recruitment",
    EmblemRecruitment = 2,  // "G_Emblem_Mode"
    EmblemWepProf = 3,  //"G_EmblemWepProf"
    EmblemSyncSkill = 4,    //"G_Random_God_Sync"
    EngraveLevel = 5,   //"G_EngraveSetting"
    EmblemEngageSkill = 6,  //"G_EngageSkill"
    EmblemInherit = 7,  //"G_EmblemInherit"
    EmblemAppearance = 8,   //"G_EmblemApp"
    BodyScaling = 9,  //
    JobLearnMode = 10,  // "G_LearnMode"
    Reclassing = 11,    // "G_RandomCC"
    BattleStyles = 12,  // "G_BattleStyles"
    SingleJob = 13, // "G_DVC_SingleJob"
    InteractSetting = 14,   // G_InteractSetting
    ClassMode = 15,    //  "G_Random_Item"
    RandomGifts = 16,   // G_RngGifts"
    ExplorationItem = 17,   //  G_ExplorationItems
    UnitInventory = 18, // G_PRW"
    PersonalGrowthMode = 19,    // "G_Random_Grow_Mode"
    RandomJobOutfit = 20,   // "G_RandomJobOutfit"
    TerrainEffect = 21, // G_RandomEnergy
    FogOfWar = 22,  // "G_FOW"
    UnitDeployment = 23,    // G_DeploymentMode"
    EmblemDeployment = 24,  // "G_EmblemDeployMode"
    GenericAppearance = 25, // "G_GenericMode"
    Seed = 26,
    EnemyRevivalStone = 30, // G_EnemyRevivalStone"
    EnemySkillGauge = 31,   // "G_EnemySkillGauge"
    EnemyJobGauge = 32, // "G_EnemyJobGauge"
    EnemyEmblemGauge = 33,  // "G_EnemyEmblemGauge"
    EnemyItemDropGauge = 34,    // "G_ItemDropGauge"
    BondSkillS = 35,
    BondSkillA = 36,
    BondSkillB = 37,
    BondSkillC = 38,
    BondRingSkillRate = 39,    // G_BRRS
}
impl DVCVariables {
    pub fn from(i: i32) -> Option<Self> {
        match i {
            0 => Some(Self::Continuous),
            1 => Some(Self::UnitRecruitment),
            2 => Some(Self::EmblemRecruitment),
            3 => Some(Self::EmblemWepProf),
            4 => Some(Self::EmblemSyncSkill),
            5 => Some(Self::EngraveLevel),
            6 => Some(Self::EmblemEngageSkill),
            7 => Some(Self::EmblemInherit),
            8 => Some(Self::EmblemAppearance),
            9 => Some(Self::BodyScaling),
            10 => Some(Self::JobLearnMode),
            11 => Some(Self::Reclassing),
            12 => Some(Self::BattleStyles),
            13 => Some(Self::SingleJob),
            14 => Some(Self::InteractSetting),
            15 => Some(Self::ClassMode),
            16 => Some(Self::RandomGifts),
            17 => Some(Self::ExplorationItem),
            18 => Some(Self::UnitInventory),
            19 => Some(Self::PersonalGrowthMode),
            20 => Some(Self::RandomJobOutfit),
            21 => Some(Self::TerrainEffect),
            22 => Some(Self::FogOfWar),
            23 => Some(Self::UnitDeployment),
            24 => Some(Self::EmblemDeployment),
            25 => Some(Self::GenericAppearance),
            26 => Some(Self::Seed),
            30 => Some(Self::EnemyRevivalStone),
            31 => Some(Self::EnemySkillGauge),
            32 => Some(Self::EnemyJobGauge),
            33 => Some(Self::EnemyEmblemGauge),
            34 => Some(Self::EnemyItemDropGauge),
            35 => Some(Self::BondSkillS),
            36 => Some(Self::BondSkillA),
            37 => Some(Self::BondSkillB),
            38 => Some(Self::BondSkillC),
            39 => Some(Self::BondRingSkillRate),
            _ => None,
        }
    }
    pub fn init_var(&self, value: i32, overwrite: bool) {
        let key = self.get_key();
        if GameVariableManager::exist(key) {
            if overwrite { GameVariableManager::set_number(key, value); }
        }
        else { GameVariableManager::make_entry_norewind(key, value); }
    }
    pub fn get_value(&self) -> i32 {
        if DVCVariables::is_main_menu() { DVCConfig::get().get_value(*self) }
        else if self.is_gauge() {
            let v = (*self as i32) - 30;
            let value = Self::get_by_variable(self.get_key());
            if v < 5 { ((value >> v*5) & 0x1F) * 10 }
            else {  // Bond Ring Rates
                let v = v - 5;
                (value >> v*8) & 0xFF
            }
        }
        else { Self::get_by_variable(self.get_key()) }
    }
    pub fn create_variable(self, value: i32) -> i32 {
        GameVariableManager::make_entry_norewind(self.get_key(), value);
        value
    }
    pub fn set_by_variable(key: &str, value: i32) { GameVariableManager::set_number(key, value); }
    pub fn get_by_variable(key: &str) -> i32 { GameVariableManager::get_number(key) }
    pub fn set_value(&self, value: i32) {
        if DVCVariables::is_main_menu() { DVCConfig::get().set_value(*self, value); }
        else if self.is_gauge() {
            let value = clamp_value(value, 0, 100);
            let v = (*self as i32) - 30;
            let old_var = Self::get_by_variable(self.get_key());
            if v < 5 {  // Enemy Gauges
                let new_value = (old_var & !(0x1F << v*5)) | ((value/10) << v*5);
                Self::set_by_variable(self.get_key(), new_value);
            }
            else {  // Bond Ring Rates
                let v = v - 5;
                let new_value = (old_var & !(0xFF << 8*v)) | (value << 8*v);
                Self::set_by_variable(self.get_key(), new_value);
            }
        }
        else {
            println!("Set Value #{}: {}", self.get_key(), value);
            match self {
                Self::Seed => { Self::set_by_variable(self.get_key(), value); }
                Self::SingleJob => {
                    if (value >= 0 && value < 3) || JobData::try_get_hash(value).is_some(){
                        Self::set_by_variable(self.get_key(), value);
                    }
                    else { Self::set_by_variable(self.get_key(), 0); }
                }
                _ => {
                    let value = clamp_value(value, 0, self.get_max());
                    Self::set_by_variable(self.get_key(), value);
                }
            }
        }
    }
    pub fn get_key(&self) -> &'static str {
        match self {
            Self::Continuous => "G_Continuous",
            Self::UnitRecruitment => "G_Random_Recruitment",
            Self::EmblemRecruitment => "G_Emblem_Mode",
            Self::EmblemWepProf => "G_EmblemWepProf",
            Self::EmblemSyncSkill => "G_Random_God_Sync",
            Self::EngraveLevel => "G_EngraveSetting",
            Self::EmblemEngageSkill => "G_EngageSkill",
            Self::EmblemInherit => "G_EmblemInherit",
            Self::EmblemAppearance => "G_EmblemApp",
            Self::BodyScaling => "G_BodyScale",
            Self::JobLearnMode => "G_LearnMode",
            Self::Reclassing => "G_RandomCC",
            Self::BattleStyles => "G_BattleStyles",
            Self::SingleJob => "G_DVC_SingleJob",
            Self::InteractSetting => "G_InteractSetting",
            Self::ClassMode => "G_ClassMode",
            Self::RandomGifts => "G_RngGifts",
            Self::ExplorationItem => "G_ExplorationItems",
            Self::UnitInventory => "G_PRW",
            Self::PersonalGrowthMode => "G_Random_Grow_Mode",
            Self::EnemySkillGauge|Self::EnemyRevivalStone|Self::EnemyJobGauge|Self::EnemyEmblemGauge|Self::EnemyItemDropGauge => "G_EnemyGauge",
            Self::RandomJobOutfit => "G_RandomJobOutfit",
            Self::TerrainEffect => "G_RandomEnergy",
            Self::FogOfWar => "G_FOW",
            Self::UnitDeployment => "G_DeploymentMode",
            Self::EmblemDeployment => "G_EmblemDeployMode",
            Self::GenericAppearance => "G_GenericMode",
            Self::BondRingSkillRate|Self::BondSkillS|Self::BondSkillA|Self::BondSkillB|Self::BondSkillC => "G_BRRS",
            Self::Seed => "G_Random_Seed",
        }
    }
    pub fn can_a_call(&self, menu_value: i32) -> bool {  // Require to ACall to Change Value
        if DVCVariables::is_main_menu() { false }
        else {
            match self {
                Self::ClassMode => {
                    let v = self.get_value();
                    (v > 2 ) != (menu_value > 2)
                }
                Self::BattleStyles | Self::SingleJob | Self::JobLearnMode | Self::EmblemInherit | Self::EngraveLevel |
                Self::EmblemSyncSkill | Self::EmblemWepProf | Self::InteractSetting | Self::PersonalGrowthMode |
                Self::EmblemEngageSkill => true,
                _ => false,
            }
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItemAttribute {
        let index = *self as i32;
        if DVCVariables::is_main_menu() {
            if index < 35 || (index >= 35 && DVCFlags::BondRing.get_value()) { BasicMenuItemAttribute::Enable }
            else { BasicMenuItemAttribute::Hide }
        }
        else if index >= 35 && DVCFlags::BondRing.get_value() { BasicMenuItemAttribute::Enable }
        else {
            let sequence = GameUserData::get_sequence();
            match self {
                Self::FogOfWar|Self::EnemySkillGauge|Self::TerrainEffect|Self::UnitDeployment|Self::EmblemDeployment => {
                    if sequence == 2 || sequence == 3 { BasicMenuItemAttribute::Disable }
                    else { BasicMenuItemAttribute::Enable }
                }
                Self::Continuous|Self::UnitRecruitment|Self::EmblemRecruitment => { BasicMenuItemAttribute::Disable }
                _ => { BasicMenuItemAttribute::Enable }
            }
        }
    }
    pub fn get_max(&self) -> i32 {
        match self {
            Self::SingleJob => { JobData::get_count() }
            Self::UnitDeployment => 8,
            Self::InteractSetting => 7,
            Self::Continuous|Self::UnitRecruitment|Self::FogOfWar|Self::EngraveLevel|Self::ClassMode => 5,
            Self::Reclassing|Self::EmblemDeployment|Self::EmblemInherit|Self::EmblemRecruitment|
            Self::ExplorationItem|Self::UnitInventory|Self::EmblemAppearance|Self::GenericAppearance => 4,
            _ => { if self.is_gauge() { 100 } else { 3 } }
        }
    }
    pub fn is_gauge(&self) -> bool {
        match self {
            Self::EnemyItemDropGauge|Self::EnemyEmblemGauge|Self::EnemyJobGauge|Self::EnemySkillGauge|
            Self::EnemyRevivalStone|Self::BondSkillS|Self::BondSkillA|Self::BondSkillB|Self::BondSkillC
            => true,
            _ => false,
        }
    }
    pub fn increment(&self, mut value: i32, increase: bool) -> i32 {
        if self.is_gauge() {
            if increase { value += 10; } else { value -= 10; }
            value = clamp_value(value, 0, 100);
            self.set_value(value);
        }
        else {
            let max = self.get_max();
            value = if increase { (value + 1) % max } else { (value + max - 1) % max };
            if !self.can_a_call(value) { self.set_value(value); }
        }
        value
    }
}
impl DVCVariables {
    pub const SEED: &'static str = "G_Random_Seed";
    pub const DVC_STATUS: &'static str = "G_DVC_Status";

    pub const LUEUR_NAME: &'static str = "G_Lueur_Name";    //
    pub const LUEUR_GENDER: &'static str = "G_Lueur_Gender2";   //
    pub const MISERCODE_TYPE: &'static str = "G_Misercode_Type";
    pub const LIBERATION_TYPE: &'static str = "G_Liberation_Type";
    pub const CONTINUOUS: &'static str = "G_Continuous";

    pub const PLAYER_AVERAGE_CAP: &'static str = "G_Player_Rating_Average";
    pub const EMBLEM_PARALOGUE_LEVEL: &'static str = "G_Paralogue_Level";
    pub const SINGLE_CLASS: &'static str = "G_DVC_SingleJob";
    pub const RECRUITMENT_KEY: &'static str =  "G_Random_Recruitment";
    pub const EMBLEM_RECRUITMENT_KEY: &'static str =  "G_Emblem_Mode";
    pub const TILE_RNG: &'static str = "G_TileRNG";
    
    pub fn set_variable_key_string<'a>(key: impl Into<&'a Il2CppString>, value: impl Into<&'a Il2CppString>) {
        let key = key.into();
        if GameVariableManager::exist(key) { GameVariableManager::set_string(key, value.into()); }
        else { GameVariableManager::set_string(key, value.into()); }
    }

    // pub const RANDOM_CLASS_OUTFITS: &'static str = "G_RandomJobOutfit";
    pub fn init_tile_rng(init: bool) -> &'static Random {
        if !GameVariableManager::exist(Self::TILE_RNG) { GameVariableManager::make_entry(Self::TILE_RNG, 0); }
        let mut v = GameVariableManager::get_number(Self::TILE_RNG);
        if init || v == 0 {
            v = Random::get_system().value();
            GameVariableManager::set_number(Self::TILE_RNG, v);
        }
        Random::new(v as u32)
    }
    pub fn get_seed() -> i32 {  GameVariableManager::get_number(Self::SEED) }
    pub fn is_main_menu() -> bool { GameUserData::get_sequence() == 0 && !GameVariableManager::exist(Self::SEED) }
    pub fn random_enabled() -> bool { Self::get_seed() != 0 }
    pub fn is_continuous() -> bool {
        let v = Self::Continuous.get_value();
        v == 1 || v == 2
    }
    pub fn is_changed_recruitment_order(emblem: bool) -> bool {
        if emblem { DVCVariables::EmblemRecruitment.get_value()  != 0 }
        else { DVCVariables::UnitRecruitment.get_value()  != 0 }
    }
    pub fn is_lueur_female() -> bool { GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 }

    pub fn get_dvc_person(pid_index: i32, reverse: bool) -> &'static Il2CppString {
        let key = if reverse {format!("G_R2_{}", PIDS[pid_index as usize]) } else { format!("G_R_{}", PIDS[pid_index as usize]) };
        if GameVariableManager::exist(key.as_str()) { GameVariableManager::get_string(key.as_str()) }
        else { PIDS[pid_index as usize].into() }
    }
    pub fn get_dvc_recruitment_index(person_index: i32) -> i32 {
        let key = format!("G_R_{}", PIDS[person_index as usize]);
        if GameVariableManager::exist(key.as_str()) {
            let s1 = GameVariableManager::get_string(key.as_str()).to_string();
            PIDS.iter().position(|&x| s1 == *x).map(|p| p as i32)
        }
        else { None }.unwrap_or(-1)
    }
    pub fn get_dvc_person_data(pid_index: i32, reverse: bool) -> Option<&'static mut PersonData> {
        if pid_index < 41 {
            PersonData::get_mut(Self::get_dvc_person(pid_index, reverse))
        }
        else {
            GameData::get().units.iter()
                .filter(|x| *x.1 == pid_index)
                .find_map(|x| PersonData::try_get_hash_mut(*x.0).filter(|x| x.name.is_some_and(|x| !x.str_contains("Hide") && !x.str_contains("Unknown"))))
        }

    }
    pub fn get_dvc_unit(dvc_person_index: i32, reverse: bool) -> Option<&'static mut Unit> {
        if dvc_person_index < 41 {
            UnitPool::get_from_pid(Self::get_dvc_person(dvc_person_index, reverse), false)
        }
        else {
            GameData::get().units.iter()
                .filter(|x| *x.1 == dvc_person_index)
                .flat_map(|x| PersonData::try_get_hash(*x.0))
                .find_map(|person| UnitPool::get_from_person_force_mask(person, 102))
        }
    }
    pub fn get_single_class(is_base: bool, female: bool) -> Option<&'static JobData> {
        let hash = DVCVariables::SingleJob.get_value();
        if hash == 1 && dlc_check() { JobData::get(if female { "JID_裏邪竜ノ娘" } else { "JID_裏邪竜ノ子" }) }
        else if let Some(job) = JobData::try_get_hash(hash) {
            if job.max_level == 40 || !is_base { Some(job) }
            else {
                let bases = utils::get_base_classes(job);
                let base_count = bases.len();
                if base_count == 0 { Some(job) }
                else if base_count > 1 {
                    let selection = Random::get_system().get_value(base_count as i32) as usize;
                    Some(bases[selection])
                }
                else { Some(bases[0]) }
            }
        }
        else { None }
    }

    pub fn get_dvc_emblem(index: i32, reverse: bool) -> &'static Il2CppString {
        // for extra supporting emblems swapping
        let emblem_index =
            match index {
                20|21 => { 12 }    // Dimitri, Claude to Edelgard Index
                22 => { 18 }    // Robin -> Chrom
                23 => { 11 }    // Ephiram  -> Eirika
                _ => { index }
            };
        let key =
            if reverse {format!("G_R2_{}", EMBLEM_GIDS[emblem_index as usize]) }
            else { format!("G_R_{}", EMBLEM_GIDS[emblem_index as usize]) };

        if GameVariableManager::exist(key.as_str()) { GameVariableManager::get_string(key.as_str()) }
        else { EMBLEM_GIDS[emblem_index as usize].into() }
    }
    pub fn get_current_god(recruitment_index: i32) -> Option<&'static GodData> {
        let god = Self::get_god_from_index(recruitment_index, true)?;
        if let Some(g_unit) = GodPool::try_get(god, true) { Some(g_unit.data) }
        else { Some(god) }
        
    }
    pub fn get_dvc_emblem_index(recruitment_index: i32, reverse: bool) -> usize {
        // for extra supporting emblems swapping
        let emblem_index =
            match recruitment_index {
                20|21 => { 12 }    // Dimitri, Claude to Edelgard Index
                22 => { 18 }    // Robin -> Chrom
                23 => { 11 }    // Ephiram  -> Eirika
                _ => { recruitment_index }
            };
        let key =
            if reverse {format!("G_R2_{}", EMBLEM_GIDS[emblem_index as usize]) }
            else { format!("G_R_{}", EMBLEM_GIDS[emblem_index as usize]) };
        if GameVariableManager::exist(key.as_str()) {
            let str = GameVariableManager::get_string(key.as_str()).to_string();
           GodData::get(str.as_str()).map(|v| v.parent.hash).and_then(|hash|{
                GameData::get_playable_emblem_hashes().iter().position(|v| *v == hash)
            }).unwrap_or( emblem_index as usize)
            // EMBLEM_GIDS.iter().position(|gid| gid == &str).unwrap_or(recruitment_index as usize)
        }
        else { emblem_index as usize }
    }
    /// Non-Custom Emblems Only
    pub fn set_emblem_recruitment(emblem_index: i32, replace_emblem_index: i32) {
        if emblem_index > 18 || replace_emblem_index > 18 || emblem_index == replace_emblem_index { return; }
        GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[emblem_index as usize]), EMBLEM_GIDS[replace_emblem_index as usize]);
        GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[replace_emblem_index as usize]), EMBLEM_GIDS[emblem_index as usize]);
    }

    pub fn set_person_recruitment(pid_index: i32, replace_pid_index: i32) {
        if pid_index > 40 || replace_pid_index > 40 { return; }
        GameVariableManager::set_string(&format!("G_R_{}",PIDS[pid_index as usize]), PIDS[replace_pid_index as usize]);
        GameVariableManager::set_string(&format!("G_R2_{}",PIDS[replace_pid_index as usize]), PIDS[pid_index as usize]);
    }
    pub fn get_god_from_index(index: i32, randomized: bool) -> Option<&'static mut GodData> {
        let list = GameData::get_playable_emblem_hashes();
        if index as usize >=  list.len() { return None; }
        let hash = list[index as usize];
        if !randomized { GodData::try_get_hash_mut(hash) }
        else {
            let key = format!("G_R_{}", GodData::try_get_hash(hash).unwrap().gid);
            if GameVariableManager::exist(&key){ GodData::get_mut(GameVariableManager::get_string(&key)) }
            else { None }
        }
    }
    pub fn is_recruitment_set(emblem: bool) -> bool {
        if emblem { GameVariableManager::find_starts_with("G_R_GID").len() >= 12 }
        else { GameVariableManager::find_starts_with("G_R_PID").len() >= 36 }
    }
    pub fn create_recruitment_variables(emblem: bool) {
        if emblem {
            for i in 0..19 {
                let key = format!("G_R_{}",EMBLEM_GIDS[i as usize]);
                GameVariableManager::make_entry_str(key.as_str(), EMBLEM_GIDS[i as usize]);
                GameVariableManager::make_entry_str(&format!("G_R2_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[i as usize]);
            }
        }
        else {
            for i in 0..41 {
                GameVariableManager::make_entry_str(&format!("G_R_{}",PIDS[i as usize]), PIDS[i as usize]);
                GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[i as usize]), PIDS[i as usize]);
            }
        }
    }
    pub fn is_random_map() -> bool { DVCVariables::Continuous.get_value() == 2 }
    pub fn is_main_chapter_complete(main_index: i32) -> bool {
        GameVariableManager::get_bool(if main_index < 10 { format!("G_Cleared_M00{}", main_index) }
        else { format!("G_Cleared_M0{}", main_index) })
    }
    pub fn log_variable(var: &str) {
        if !GameVariableManager::exist(var) {
            println!("[DVC Variable] {}: Does not Exist", var.trim_start_matches("G_"));
        }
        else {
            let value = GameVariableManager::get_number(var);
            println!("[DVC Variable] {}: {}", var.trim_start_matches("G_"), value);
        }
    }
}