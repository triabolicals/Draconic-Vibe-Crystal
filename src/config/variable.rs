use engage::{
    app::{
        dlcmanager::*, gameuserdata::*, random_2::*, Gender,
        unit::*, jobdata::*, unitedit::*, chapterdata::*, BasicMenuItem_Attribute,
        IGameVariableMethods, ISingletonClass_1Methods, IStructBase, IStructData_1Methods,
    },
    GameVariableManager,
    List_1Ext
};
use outfit_core::Randomizer;
use unity::Cast;
use crate::{
    config::DVCConfig,DVCFlags,
    gamedata::{self, EmblemPool, ENCHANTER, MAGE_CANNON},
    dvc::{DVC, PersonPool},
    enums::PIDS,
    utils::{clamp_value, to_opt},
};

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum DVCVariable {
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
    JobLearn = 10,  // "G_LearnMode"
    Reclassing = 11,    // "G_RandomCC"
    BattleStyles = 12,  // "G_BattleStyles"
    SingleJob = 13,
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
    RandomLearnLevel = 26,
    EnemyCapabilityOffset = 27,
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
    GenderClassM = 40,
    GenderClassF = 41,
    // Gameplay tracking variables
    LueurGender = 50,
    LiberationKind = 51,
    MisercodeKind = 52,

    // Hashes, RNG, or custom range variables
    Version = 100,
    Seed = 101,
    NextChapter = 102,
    TileRNG = 103,
    PlayerCapabilityRating = 104,
    HeroPerson = 105,
    AverageLevel = 106,
    EmblemParalogueGenericLevel = 107,
}
impl DVCVariable {
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
            10 => Some(Self::JobLearn),
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
            26 => Some(Self::RandomLearnLevel),
            27 => Some(Self::EnemyCapabilityOffset),
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
            40 => Some(Self::GenderClassM),
            41 => Some(Self::GenderClassF),
            50 => Some(Self::LueurGender),
            51 => Some(Self::LiberationKind),
            52 => Some(Self::MisercodeKind),

            -1|100 => Some(Self::Version),
            101 => Some(Self::Seed),
            102 => Some(Self::NextChapter),
            103 => Some(Self::TileRNG),
            104 => Some(Self::PlayerCapabilityRating),
            105 => Some(Self::HeroPerson),
            106 => Some(Self::EmblemParalogueGenericLevel),
            _ => None,
        }
    }
    pub fn is_valid_single_job() -> bool {
        let single_job = DVCVariable::SingleJob.get_value();
        (single_job == 1000 && DVCVariables::has_dlc()) || !JobData::try_get_from_hash(single_job).is_null()
    }
    pub fn init_var(&self, value: i32, overwrite: bool) {
        let key = self.get_key();
        if GameVariableManager::is_exist(key) { if overwrite { GameVariableManager::set_number(key, value); } }
        else { GameVariableManager::make_entry_norewind(key, value); }
    }
    pub fn get_setting_value(&self) -> i32 {
        if DVCVariables::is_main_menu() { DVCConfig::get().get_value(*self) }
        else if self.is_gauge() {
            let v = (*self as i32) - 30;
            let value = Self::get_by_key(self.get_key());
            if v < 5 { ((value >> v*5) & 0x1F) * 10 }
            else {
                let v = v - 5;
                (value >> v*8) & 0xFF
            }
        }
        else { Self::get_by_key(self.get_key()) }
    }
    pub fn is_bypass_disable(&self) -> bool {
        let v = *self as i32;
        v < 3 || v == 23 || v == 24 || v >= 50
    }
    pub fn is_adjustable(self) -> bool {
        let sequence = DVCVariables::get_sequence();
        let index = self as i32;
        if DVCVariables::is_main_menu() { index < 35 || (index >= 35 && DVCFlags::BondRing.get_value()) }
        else {
            match self {
                Self::TerrainEffect|Self::UnitDeployment|Self::EmblemDeployment => { sequence != 2 && sequence != 3 }
                Self::Continuous|Self::UnitRecruitment|Self::EmblemRecruitment => { false }
                _ => true
            }
        }
    }
    pub fn get_value(&self) -> i32 {
        if DVCVariables::is_main_menu() { DVCConfig::get().get_value(*self) }
        else if self.is_bypass_disable() || DVCFlags::RandomEnable.get_value() { self.get_setting_value() }
        else { 0 }
    }
    pub fn create_variable(self, value: i32) -> i32 {
        GameVariableManager::make_entry_norewind(self.get_key(), value);
        value
    }
    pub fn set_by_key(key: &str, value: i32) { GameVariableManager::set_number(key, value); }
    pub fn get_by_key(key: &str) -> i32 { GameVariableManager::get_number(key) }
    pub fn set_value(&self, mut value: i32) {
        if DVCVariables::is_main_menu() { DVCConfig::get().set_value(*self, value); }
        else if self.is_gauge() {
            let value = clamp_value(value, 0, 100);
            let v = (*self as i32) - 30;
            let old_var = Self::get_by_key(self.get_key());
            if v < 5 {  // Enemy Gauges
                let new_value = (old_var & !(0x1F << v*5)) | ((value/10) << v*5);
                Self::set_by_key(self.get_key(), new_value);
            }
            else {  // Bond Ring Rates
                let v = v - 5;
                let new_value = (old_var & !(0xFF << 8*v)) | (value << 8*v);
                Self::set_by_key(self.get_key(), new_value);
            }
        }
        else {
            match self {
                Self::SingleJob => {
                    if (value >= 0 && value < 3) || !JobData::try_get_from_hash(value).is_null(){
                        Self::set_by_key(self.get_key(), value);
                    }
                    else { Self::set_by_key(self.get_key(), 0); }
                }
                _ => {
                    let idx = *self as i32;
                    if idx < 100 && idx != 27 { value = clamp_value(value, 0, self.get_max()); }
                    self.init_var(value, true);
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
            Self::JobLearn => "G_LearnMode",
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
            Self::LueurGender => "G_Lueur_Gender2",
            Self::LiberationKind => "G_Liberation_Type",
            Self::MisercodeKind => "G_Misercode_Type",
            Self::Seed => "G_Random_Seed",
            Self::NextChapter => "G_DVC_Next",
            Self::TileRNG => "G_TileRNG",
            Self::Version => "G_DVC_Version",
            Self::PlayerCapabilityRating => "G_Player_Rating_Average",
            Self::HeroPerson => "G_HeroHash",
            Self::RandomLearnLevel => "G_LearnLevel",
            Self::AverageLevel => "G_AvgLvl",
            Self::EnemyCapabilityOffset => "G_CapOff",
            Self::EmblemParalogueGenericLevel => "RingGenericLevel",
            Self::GenderClassF => "G_SJF",
            Self::GenderClassM => "G_SJM",
        }
    }
    pub fn can_a_call(&self) -> bool {  // Require to ACall to Change Value
        if DVCVariables::is_main_menu() { false }
        else {
            match self {
                Self::ClassMode|Self::GenderClassM|Self::GenderClassF => { true }
                _ => false,
            }
        }
    }
    pub fn rebuild_menu_custom_call(&self) -> bool {
        match self {
            Self::Continuous|Self::UnitRecruitment|Self::EmblemRecruitment|Self::ClassMode => { true }
            _ => false
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItem_Attribute {
        let index = *self as i32;
        if DVCVariables::is_main_menu() {
            if *self == DVCVariable::ExplorationItem {
                if DVCFlags::RandomExplorationItems.get_setting_value() { BasicMenuItem_Attribute::enable() }
                else { BasicMenuItem_Attribute::hide() }
            }
            else if index < 35 || (index >= 35 && DVCFlags::BondRing.get_setting_value()) { BasicMenuItem_Attribute::enable() }
            else { BasicMenuItem_Attribute::hide() }
        }
        else if self.is_bypass_disable() { BasicMenuItem_Attribute::enable() }
        else if Self::ClassMode.get_setting_value() == 6 { BasicMenuItem_Attribute::hide() }
        else {
            let attr = if DVCFlags::RandomEnable.get_value() { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::disable() };
            if index >= 35 {
                if DVCFlags::BondRing.get_setting_value() { attr }
                else { BasicMenuItem_Attribute::hide() }
            }
            else if *self == DVCVariable::ExplorationItem {
                if DVCFlags::RandomExplorationItems.get_setting_value() { attr }
                else { BasicMenuItem_Attribute::hide()  }
            }
            else { attr }
        }
    }
    pub fn get_max(&self) -> i32 {
        match self {
            Self::SingleJob => { JobData::get_count() }
            Self::LiberationKind|Self::MisercodeKind => 10,
            Self::UnitDeployment => 8,
            Self::InteractSetting => 7,
            Self::ClassMode => 6 + DVCVariables::is_main_menu() as i32,
            Self::PersonalGrowthMode => 6,
            Self::Continuous|Self::FogOfWar|Self::EngraveLevel => 5,
            Self::UnitRecruitment => { 4 + (PersonPool::get().playable.len() > 41) as i32 }
            Self::Reclassing => 4,
            Self::EmblemInherit|Self::EmblemRecruitment|Self::EmblemDeployment|Self::TerrainEffect|Self::ExplorationItem|Self::UnitInventory|Self::EmblemAppearance|Self::GenericAppearance => 4,
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
            value = if increase { value + 10 } else { value - 10 };
            value = crate::utils::wrap_value(value, 0, 100);
            self.set_value(value);
        }
        else if *self == Self::EnemyCapabilityOffset {
            value = clamp_value(value + if increase { 1 } else { -1 }, -10, 10);
            self.set_value(value);
        }
        else {
            let max = self.get_max();
            value = if increase { (value + 1) % max } else { (value + max - 1) % max };
            if !self.can_a_call() { self.set_value(value); }
        }
        value
    }
}
pub struct DVCVariables;
impl DVCVariables {
    pub const LUEUR_NAME: &'static str = "G_Lueur_Name";
    pub fn set_lueur_name_gender(lueur_unit: Unit) {
        DVCVariables::set_variable_key_string(Self::LUEUR_NAME, lueur_unit.m_edit().get_name());
        DVCVariable::LueurGender.init_var(lueur_unit.m_edit().m_gender().value, true);
    }
    pub fn get_rng() -> Random_2 {
        let seed = GameVariableManager::get_number(DVCVariable::Seed.get_key());
        Random_2::new_2(seed as u32)
    }
    pub fn create_hash_rng(hash: i32, m: i32) -> Random_2 {
        let r_seed = GameVariableManager::get_number(DVCVariable::Seed.get_key());
        let rng = Random_2::new_2(r_seed as u32);
        rng.ctor_2((( hash >> 2) + (r_seed >> 2)) as u32);
        for _ in 0..m { rng.get_value(); }
        rng
    }
    pub fn user_data() -> GameUserData { GameUserData::get_instance() }
    pub fn get_sequence() -> i32 { Self::user_data().get_sequence().value }
    pub fn get_current_chapter() -> ChapterData { Self::user_data().m_chapter() }
    pub fn has_dlc() -> bool { DLCManager::has_content(DLCManager_Content::e0()) }

    pub fn set_variable_key_string(key: impl Into<unity::Il2CppString>, value: impl Into<unity::Il2CppString>) {
        let key = key.into();
        if GameVariableManager::is_exist(key) { GameVariableManager::set_string(key, value.into()); }
        else { Self::user_data().m_variable().entry_no_rewind_2(key, value); }
    }
    pub fn get_seed() -> i32 { DVCVariable::Seed.get_setting_value() }
    pub fn is_main_menu() -> bool { Self::get_sequence() == 0 && !DVC::get().enabled }
    pub fn is_continuous() -> bool {
        let v = DVCVariable::Continuous.get_value();
        v == 1 || v == 2
    }
    pub fn is_changed_recruitment_order(emblem: bool) -> bool {
        if emblem { DVCVariable::EmblemRecruitment.get_value() != 0 }
        else { DVCVariable::UnitRecruitment.get_value() != 0 }
    }
    pub fn get_random_number_for_seed() -> u32 {
        let frames = engage::unity_engine::Time::get_frame_count();
        let rng = Random_2::new_2(frames as u32);
        rng.get_value() as u32
    }
    pub fn is_lueur_female() -> bool { DVCVariable::LueurGender.get_value() == 2 }
    pub fn get_dvc_recruitment_index(person_index: i32) -> i32 {
        let key = format!("G_R_{}", PIDS[person_index as usize]);
        if GameVariableManager::is_exist(key.as_str()) {
            let s1 = GameVariableManager::get_string(key.as_str()).to_string();
            PIDS.iter().position(|&x| s1 == *x).map(|p| p as i32)
        }
        else { None }.unwrap_or(-1)
    }
    pub fn get_single_class(is_base: bool, female: bool) -> Option<JobData> {
        let hash = DVCVariable::SingleJob.get_value();
        if hash == 1000 && DVCVariables::has_dlc() { to_opt(JobData::get(if female { "JID_裏邪竜ノ娘" } else { "JID_裏邪竜ノ子" }.into())) }
        else if (hash == ENCHANTER || hash == MAGE_CANNON) && DVCVariables::has_dlc() { to_opt(JobData::try_get_from_hash(hash)) }
        else {
            let job = JobData::try_get_from_hash(hash);
            if job.is_null() { None }
            else if is_base && job.is_high() {
                let classes = gamedata::JobDataBase::get_base_classes(job);
                if classes.is_empty() { Some(job) }
                else { classes.get_random_element(Random_2::get_system()).map(|&v| JobData::try_get_from_hash(v)) }
            }
            else { Some(job) } 
        }
    }
    pub fn get_single_class_gender(is_base: bool, gender: Gender) -> Option<JobData> {
        let hash =
            if gender.value == 1 { DVCVariable::GenderClassM.get_value() }
            else if gender.value == 2 { DVCVariable::GenderClassF.get_value() }
            else { 0 };
        if DVCConfig::is_valid_single_class(hash, gender) {
            let job = JobData::try_get_from_hash(hash);
            if job.is_null() { None }
            else if is_base && job.is_high() {
                let classes = gamedata::JobDataBase::get_base_classes(job);
                if classes.is_empty() { Some(job) }
                else { classes.get_random_element(Random_2::get_system()).map(|&v| JobData::try_get_from_hash(v)) }
            }
            else { Some(job) }
        }
        else { None }
    }
    /// Non-Custom Emblems Only
    pub fn set_person_recruitment(pid_index: i32, replace_pid_index: i32) {
        if pid_index > 40 || replace_pid_index > 40 { return; }
        DVCVariables::set_variable_key_string(format!("G_R_{}",PIDS[pid_index as usize]), PIDS[replace_pid_index as usize]);
        DVCVariables::set_variable_key_string(format!("G_R2_{}",PIDS[replace_pid_index as usize]), PIDS[pid_index as usize]);
    }
    pub fn create_recruitment_variables(emblem: bool) {
        if emblem {
            for i in 0..19 {
                let gid = format!("GID_{}", EmblemPool::GIDS[i as usize]);
                let key = format!("G_R_{}", gid);
                GameVariableManager::make_entry_str(key.as_str(), gid.as_str());
                GameVariableManager::make_entry_str(format!("G_R2_GID_{}", EmblemPool::GIDS[i as usize]), gid.as_str());
            }
        }
        else {
            for i in 0..41 {
                GameVariableManager::make_entry_str(format!("G_R_{}",PIDS[i as usize]), PIDS[i as usize]);
                GameVariableManager::make_entry_str(format!("G_R2_{}",PIDS[i as usize]), PIDS[i as usize]);
            }
        }
    }
    pub fn is_random_map() -> bool { DVCVariable::Continuous.get_value() == 2 }
    pub fn get_next_main_chapter() -> Option<ChapterData> {
        for i in 2..27 {
            if !Self::is_main_chapter_complete(i) { return to_opt(ChapterData::get(format!("CID_M{:03}", i).into())); }
        }
        None
    }
    pub fn get_next_chapter() -> Option<ChapterData>{ to_opt(ChapterData::try_get_from_hash(DVCVariable::NextChapter.get_value())) }
    pub fn set_next_chapter(chapter: ChapterData) { DVCVariable::NextChapter.set_value(to_opt(chapter).map(|v| v.hash() ).unwrap_or(0)); }
    pub fn is_main_chapter_complete(main_index: i32) -> bool { GameVariableManager::get_bool(format!("G_Cleared_{}", get_chapter_from_idx(main_index))) }
    pub fn log_variable(var: &str) {
        let value = GameVariableManager::get_number(var);
        println!("[DVC Variable] {}: {}", var.trim_start_matches("G_"), value);
    }
    pub fn get_chapter_index() -> i32 {
        if DVCVariables::user_data().is_null() { return -1; }
        let prefixless = Self::get_current_chapter().m_prefixless_cid().to_rust_string();
        let p = [("M0", 0), ("S0", 30), ("G00", 50), ("E00", 60)];
        p.iter()
            .find(|(p, _)| prefixless.starts_with(p))
            .and_then(|(p, offset)| {
                prefixless.trim_start_matches(p).parse::<i32>().ok().map(|i| i + offset)
            }).unwrap_or(-1)
    }
    pub fn chapter_number_complete(with_side: bool) -> i32 {
        let main =
            GameVariableManager::find_starts_with("G_Cleared_M0")
                .iter().map(|v| GameVariableManager::get_bool(v.to_string().as_str()) as i32)
                .sum::<i32>();
        if with_side {
            GameVariableManager::find_starts_with("G_Cleared_S0")
                .iter().map(|v| GameVariableManager::get_bool(v.to_string().as_str()) as i32)
                .sum::<i32>() + main
        }
        else { main }
    }
}

pub fn get_chapter_from_idx(idx: i32) -> String {
    match idx {
        0..27 => { format!("M{:03}", idx) }
        31..46 => { format!("S{:03}", idx - 30) }
        51..57 => { format!("G{:03}", idx - 50) }
        61..68 => { format!("E{:03}", idx - 60) }
        _ => "".to_string()
    }
}