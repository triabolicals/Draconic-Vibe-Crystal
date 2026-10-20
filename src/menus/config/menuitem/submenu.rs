use super::{*, DVCMenuItemKind::*};
use config::{DVCVariable::*, DVCFlags::*};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DVCMenu {
    Recruitment = 0,
    Item = 1,
    Emblem = 2,
    UnitClass = 3,
    Skill = 4,
    Asset = 5,
    Map = 6,
    ReadOnly = 7,
    Cutscene = 8,
    Growth = 9,
    Enemy = 10,
    Other = 11,
    Main = 12,
    CustomUnitOrder = 13,
    CustomEmblemOrder = 14,
    ViewUnitOrder = 15,
    ViewEmblemOrder = 16,
    CustomUnitOrder2 = 17,
    CustomUnitJob = 18,
    #[cfg(feature = "debug")] DebugMenu = 19,
}
impl DVCMenu {
    pub fn from(i: i32) -> Option<Self> {
        match i { 
            0 => Some(Self::Recruitment),
            1 => Some(Self::Item),
            2 => Some(Self::Emblem),
            3 => Some(Self::UnitClass),
            4 => Some(Self::Skill),
            5 => Some(Self::Asset),
            6 => Some(Self::Map),
            7 => Some(Self::ReadOnly),
            8 => Some(Self::Cutscene),
            9 => Some(Self::Growth),
            10 => Some(Self::Enemy),
            11 => Some(Self::Other),
            12 => Some(Self::Main),
            13 => Some(Self::CustomUnitOrder),
            14 => Some(Self::CustomEmblemOrder),
            15 => Some(Self::ViewUnitOrder),
            16 => Some(Self::ViewEmblemOrder),
            17 => Some(Self::CustomUnitOrder2),
            18 => Some(Self::CustomUnitJob),
            #[cfg(feature = "debug")] 19 => Some(Self::DebugMenu),
            _ => None,
        }
    }
    pub fn get_previous(&self) -> Option<DVCMenu> {
        match self {
            Self::Main => None,
            Self::CustomUnitJob => if DVCVariables::is_main_menu() { Some(Self::UnitClass) } else { Some(Self::ReadOnly) }
            Self::CustomEmblemOrder|Self::CustomUnitOrder2|Self::CustomUnitOrder|Self::ViewUnitOrder|Self::ViewEmblemOrder => {
                if DVCVariables::is_main_menu() { Some(Self::Recruitment) } else { Some(Self::ReadOnly) }
            }
            _ => Some(Self::Main),
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItem_Attribute {
        let enable =
            match self {
                Self::Recruitment => DVCVariables::is_main_menu(),
                Self::ReadOnly => !DVCVariables::is_main_menu(),
                Self::CustomUnitOrder => UnitRecruitment.get_value() == 3,
                Self::CustomUnitOrder2 => UnitRecruitment.get_value() == 4,
                Self::CustomEmblemOrder => EmblemRecruitment.get_value() == 3,
                Self::CustomUnitJob => ClassMode.get_value() == 6,
                #[cfg(feature = "debug")] Self::DebugMenu => DVCConfig::get().debug,
                _ => { true }
            };
        if enable { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::hide() }
    }
    pub fn get_class_items(&self, mode: i32) -> Vec<DVCMenuItemKind> {
        let mut v = vec![];
        v.push(Variable(ClassMode));
        if mode == 6 && DVCVariables::is_main_menu() { v.push(Menu(Self::CustomUnitJob)); }
        if mode == 5 {
            v.push(Job(DVCJobItem::GenderM));
            v.push(Job(DVCJobItem::GenderF));
        }
        else if mode == 2 { v.push(Job(DVCJobItem::SingleClass)); }
        v.extend([Flag(CustomClass), Variable(Reclassing), Variable(BattleStyles), Flag(RandomClassAttrs)]);
        v
    }
    pub fn get_items(&self) -> Vec<DVCMenuItemKind> {
        match self {
            #[cfg(feature = "debug")]
            DVCMenu::DebugMenu => {
                [
                    Debug(DVCDebug::ClearMap), 
                    Debug(DVCDebug::OpenWorldMap), 
                    Debug(DVCDebug::ChangeChapterState), 
                    Debug(DVCDebug::AverageLevel),
                ].to_vec()
            }
            DVCMenu::Main => { 
                [
                    #[cfg(feature = "debug")] Menu(Self::DebugMenu),
                    Flag(RandomEnable), Flag(OnlyRecruitmentSetting),
                    Command(DVCCommand::ReRandJob),
                    Menu(Self::Recruitment), Menu(Self::Emblem), Menu(Self::Skill),
                    Menu(Self::UnitClass), Menu(Self::Item), Menu(Self::Growth), Menu(Self::Enemy),
                    Menu(Self::Asset), Menu(Self::Map), Menu(Self::Cutscene), Menu(Self::Other), Menu(Self::ReadOnly),
                    Command(DVCCommand::SetSeed)
                ].to_vec()
            }
            DVCMenu::Recruitment => {
                [Variable(UnitRecruitment), Variable(EmblemRecruitment), Flag(CustomUnits), Flag(CustomEmblemsRecruit),
                    Flag(RRGenderUnitMatch), Flag(ExcludeDLCUnitRR), Flag(ExcludeDLCEmblemRR),
                    Menu(Self::CustomUnitOrder), Menu(Self::CustomUnitOrder2), Menu(Self::CustomEmblemOrder)
                ].to_vec()
            }
            DVCMenu::Emblem => {
                [Flag(EngageAttacks), Flag(EngageWeapons), Variable(EmblemWepProf), Flag(EmblemStats),
                    Variable(EmblemSyncSkill), Variable(EmblemEngageSkill),
                    Variable(EmblemInherit), Variable(EngraveLevel)].to_vec()
            }
            DVCMenu::Skill => {
                [Flag(PersonalSkills), Variable(JobLearn), Variable(RandomLearnLevel), Flag(RandomSP), Flag(CustomSkillEnemy), Flag(EquipLearnSkills),
                    Flag(RingStats), Flag(BondRing), Gauge(BondSkillS), Gauge(BondSkillA), Gauge(BondSkillB), Gauge(BondSkillC)].to_vec()
            }
            DVCMenu::UnitClass => {
                let mut v = vec![];
                v.push(Variable(ClassMode));
                let mode = ClassMode.get_setting_value();
                if mode == 6 && DVCVariables::is_main_menu() { v.push(Menu(Self::CustomUnitJob)); }
                if mode == 5 {
                    v.push(Job(DVCJobItem::GenderM));
                    v.push(Job(DVCJobItem::GenderF));
                }
                else if mode == 2 { v.push(Job(DVCJobItem::SingleClass)); }
                v.extend([Flag(CustomClass), Variable(Reclassing), Variable(BattleStyles), Flag(RandomClassAttrs)]);
                v
            }
            DVCMenu::Item => {
                [Flag(RandomEventItems), Flag(RandomExplorationItems), Variable(ExplorationItem), Flag(AddedShopItems), Variable(RandomGifts),
                    Flag(WellItems), Variable(UnitInventory), Variable(InteractSetting), Flag(EvolveItems), Flag(RefineItem), Flag(WeaponStats),
                    Flag(WeaponSkill)
                ].to_vec()
            }
            DVCMenu::Growth => {
                [Variable(PersonalGrowthMode), Flag(RandomStartingApt), Flag(RandomClassGrowth),
                    Flag(AdaptiveGrowths), Flag(PersonalCaps), Flag(Autolevel), Flag(PostChapterAutolevel),
                ].to_vec()
            }
            DVCMenu::Enemy => {
                [Variable(EnemyCapabilityOffset), Gauge(EnemyJobGauge), Gauge(EnemySkillGauge), Gauge(EnemyItemDropGauge), Gauge(EnemyRevivalStone), Gauge(EnemyEmblemGauge)].to_vec()
            }
            DVCMenu::Map => {
                [Variable(UnitDeployment), Variable(EmblemDeployment), Flag(RandomDeploySpot), Flag(BGM), Variable(TerrainEffect), Variable(FogOfWar)].to_vec()
            }
            DVCMenu::Asset => {
                [Variable(RandomJobOutfit), Flag(RandomWeaponAsset), Flag(RandomUnitInfo), Variable(GenericAppearance),
                    Flag(RandomBossesNPCs), Flag(GodNames), Variable(EmblemAppearance), Flag(PlayerAppearance), Variable(BodyScaling)
                ].to_vec()
            }
            DVCMenu::Other => {
                [Flag(Ironman), Variable(Continuous), Flag(ContinuousExp), Flag(ContinuousModeItems), Flag(MaxStatCaps)].to_vec()
            },
            DVCMenu::Cutscene => {
                [Flag(Supports), Flag(CutsceneBGM), Flag(CutsceneFacial), Flag(CutsceneMotion), Flag(CutsceneBackground)].to_vec()
            },
            DVCMenu::ReadOnly => {
                [
                    Flag(Ironman), Variable(Continuous), Flag(ContinuousExp), Flag(ContinuousModeItems),
                    Menu(DVCMenu::ViewUnitOrder), Menu(DVCMenu::ViewEmblemOrder), Menu(Self::CustomUnitJob)
                ].to_vec()
            }
            DVCMenu::CustomEmblemOrder => {
                let n = if DVCVariables::has_dlc() { 19 } else { 12 };
                vec![Order(RecruitmentOrder::Emblem); n]
            }
            DVCMenu::ViewEmblemOrder => {
                let n = EmblemPool::get().emblem_data.len();
                vec![Order(RecruitmentOrder::Emblem); n]
            }
            DVCMenu::ViewUnitOrder => {
                let n = PersonPool::get().playable.len();
                vec![Order(RecruitmentOrder::Unit); n]
            }
            DVCMenu::CustomUnitOrder => {
                let n = if DVCVariables::has_dlc() { 41 } else { 36 };
                vec![Order(RecruitmentOrder::Unit); n]
            }
            DVCMenu::CustomUnitOrder2 => {
                let n = if DVCVariables::has_dlc() { 41 } else { 36 };
                vec![Order(RecruitmentOrder::UnitCustom); n]
            }
            DVCMenu::CustomUnitJob => {
                let n = if DVCVariables::has_dlc() { 41 } else { 36 };
                vec![Job(DVCJobItem::UnitJob); n]
            }
        }
    }
}