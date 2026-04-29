use engage::{proc::ProcInst, titlebar::TitleBar, menu::{BasicMenuItemAttribute, BasicMenuMethods}};
use unity::prelude::MethodInfo;
use crate::{
    menus::ingame::draconic_vibe_name, utils::{can_rand, dlc_check},
    config::{DVCFlags::*, DVCVariables::*, menu::DVCMenuItemKind::Order},
};
use super::*;
use DVCMenuItemKind::*;

pub static mut MENU_SELECT: [i32; 21] = [0; 21];
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
}
impl DVCMenu {
    pub fn reset_select() {
        unsafe {
            MENU_SELECT = [0; 21];
            if !DVCVariables::is_main_menu() {
                MENU_SELECT[19] = BondRingSkillRate.get_value();
                MENU_SELECT[20] = ClassMode.get_value();
            }
        }
    }
    pub fn get_previous(&self) -> Option<DVCMenu> {
        match self {
            Self::Main => None,
            Self::CustomEmblemOrder|Self::CustomUnitOrder2|Self::CustomUnitOrder|Self::ViewUnitOrder|Self::ViewEmblemOrder => {
                if DVCVariables::is_main_menu() { Some(Self::Recruitment) } else { Some(Self::ReadOnly) }
            }
            _ => Some(Self::Main),
        }
    }
    pub fn save_select(item: &DVCConfigMenuItem) {
        let index = item.menu_kind as usize;
        if let Some(v) = unsafe { MENU_SELECT.get_mut(index) } { *v = item.menu.select_index | (item.menu.scroll_index << 8); }
    }
    pub fn get_select(&self) -> (i32, i32) {
        let i = *self as usize;
        if let Some(&v) = unsafe { MENU_SELECT.get(i) }{ (v & 255, (v >> 8) & 255) }
        else { (0, 0) }
    }
    pub fn get_items(&self) -> Vec<DVCMenuItemKind> {
        match self {
            DVCMenu::Main => {
                [   Command(DVCCommand::ReRandJob), Menu(Self::Recruitment), Menu(Self::Emblem), Menu(Self::Skill),
                    Menu(Self::UnitClass), Menu(Self::Item), Menu(Self::Growth), Menu(Self::Enemy),
                    Menu(Self::Asset), Menu(Self::Map), Menu(Self::Cutscene), Menu(Self::Other), Menu(Self::ReadOnly),
                    Command(DVCCommand::SetSeed)
                ].to_vec()
            }
            DVCMenu::Recruitment => {
                [Variable(UnitRecruitment), Variable(EmblemRecruitment), Flag(CustomUnitRecruitDisable), Flag(CustomEmblemsRecruit),
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
                [Flag(PersonalSkills), Variable(JobLearnMode), Flag(RandomSP), Flag(CustomSkillEnemy), Flag(EquipLearnSkills), 
                    Flag(RingStats), Flag(BondRing), Gauge(BondSkillS), Gauge(BondSkillA), Gauge(BondSkillB), Gauge(BondSkillC)].to_vec()
            }
            DVCMenu::UnitClass => {
                [Variable(ClassMode), DVCMenuItemKind::SingleJob, Flag(CustomClass), Variable(Reclassing), Variable(BattleStyles),
                    Flag(RandomClassAttrs)].to_vec()
            }
            DVCMenu::Item => {
                [Flag(RandomEventItems), Variable(ExplorationItem), Flag(AddedShopItems), Variable(UnitInventory),
                    Variable(InteractSetting), Flag(EvolveItems), Flag(RefineItem)].to_vec()
            }
            DVCMenu::Growth => {
                [Variable(PersonalGrowthMode), Flag(PersonalCaps), Flag(RandomClassGrowth),
                    Flag(AdaptiveGrowths), Flag(Autolevel), Flag(PostChapterAutolevel)].to_vec()
            }
            DVCMenu::Enemy => {
                [Gauge(EnemyJobGauge), Gauge(EnemySkillGauge), Gauge(EnemyItemDropGauge),
                    Gauge(EnemyRevivalStone), Gauge(EnemyEmblemGauge)
                ].to_vec()
            }
            DVCMenu::Map => {
                [Variable(UnitDeployment), Variable(EmblemDeployment), Flag(RandomDeploySpot),
                    Flag(Tile), Flag(BGM), Variable(TerrainEffect), Variable(FogOfWar)].to_vec()
            }
            DVCMenu::Asset => {
                [Variable(RandomJobOutfit), Flag(RandomWeaponAsset), Flag(RandomUnitInfo), Variable(GenericAppearance),
                    Variable(EmblemAppearance), Flag(PlayerAppearance), Variable(BodyScaling)].to_vec()
            }
            DVCMenu::Other => {
                [Flag(Ironman), Variable(Continuous), Flag(ContinuousDLC), Flag(ContinuousModeItems), Flag(MaxStatCaps)].to_vec()
            },
            DVCMenu::Cutscene => {
                [Flag(CutsceneBGM), Flag(CutsceneFacial), Flag(CutsceneMotion), Flag(CutsceneBackground)].to_vec()
            },
            DVCMenu::ReadOnly => {
                [Flag(Ironman), Variable(Continuous), Flag(ContinuousDLC), Flag(ContinuousModeItems),
                    Menu(DVCMenu::ViewUnitOrder), Menu(DVCMenu::ViewEmblemOrder)].to_vec()
            }
            DVCMenu::CustomEmblemOrder|DVCMenu::ViewEmblemOrder => {
                let n = if dlc_check() { 19 } else { 12 };
                vec![Order(RecruitmentOrder::Emblem); n]
            }
            DVCMenu::CustomUnitOrder|DVCMenu::ViewUnitOrder => {
                let n = if dlc_check() { 41 } else { 36 };
                vec![Order(RecruitmentOrder::Unit); n]
            }
            DVCMenu::CustomUnitOrder2 => {
                let n = if dlc_check() { 41 } else { 36 };
                vec![Order(RecruitmentOrder::UnitCustom); n]
            }
        }
    }
    pub fn rebuild_menu(&self, item: &mut DVCConfigMenuItem, _keep_index: bool) {
        item.menu.full_menu_item_list.clear();
        self.get_items().into_iter().enumerate().for_each(|(idx, k)| {
            match k {
                Order(order) => {
                    let i = DVCConfigMenuItem::new_recruitment_item(order, idx as i32);
                    i.menu_kind = *self;
                    item.menu.full_menu_item_list.add(i);
                }
                _ => {
                    let i = DVCConfigMenuItem::new_kind(k);
                    i.index = idx as i32;
                    i.menu_kind = *self;
                    item.menu.full_menu_item_list.add(i);
                }
            }
        });

        let s = BasicMenuSelect::instantiate().unwrap();
        let (index, scroll) = self.get_select();
        s.index = index;
        s.scroll = scroll;
        item.menu.rebuild_instant2(s);
        item.menu.restore_select(s);
        DVCConfigText::apply_menu_header(*self);
    }
    pub fn rebuild_menu_variable_change(item: &mut DVCConfigMenuItem) -> bool {
        let s = item.menu.full_menu_item_list.iter().map(|i| (i.menu_item_kind, i.dvc_value, i.is_command)).collect::<Vec<_>>();
        let menu = item.menu_kind;
        match item.menu_item_kind {
            Variable(UnitRecruitment)|Variable(EmblemRecruitment)|Variable(Continuous)|Flag(BondRing)|Variable(ClassMode) => {
                item.menu.full_menu_item_list.clear();
                menu.get_items().into_iter().for_each(|k| {
                    let i = DVCConfigMenuItem::new_kind(k);
                    if let Some((_, v, c)) = s.iter().find(|x| x.0 == k) {
                        i.dvc_value = *v;
                        i.is_command = *c;
                        i.menu_kind = menu.clone();
                    };
                    item.menu.full_menu_item_list.add(i);
                });
                item.menu.rebuild_instant(true);
                true
            }
            _ => false
        }
    }
}
impl DVCCMenuItem for DVCMenu {
    fn a_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        if item.attribute & 2 != 0 {
            GameMessage::create_key_wait(item.menu, "Access settings by setting the DVC seed.");
        }
        else {
            DVCMenu::save_select(item);
            self.rebuild_menu(item, false);
        }
        BasicMenuResult::se_cursor()

    }
    fn build_attribute(&self, _item: &DVCConfigMenuItem) -> BasicMenuItemAttribute {
        let enable =
        match self {
            Self::Recruitment => DVCVariables::is_main_menu(),
            Self::ReadOnly => !DVCVariables::is_main_menu(),
            Self::CustomUnitOrder => UnitRecruitment.get_value() == 3,
            Self::CustomUnitOrder2 => UnitRecruitment.get_value() == 4,
            Self::CustomEmblemOrder => EmblemRecruitment.get_value() == 3,
            _ => { true }
        };
        if enable {
            if can_rand() || DVCVariables::is_main_menu() { BasicMenuItemAttribute::Enable }
            else { BasicMenuItemAttribute::Disable }
        }
        else { BasicMenuItemAttribute::Hide }
    }
}
pub extern "C" fn open_anime_all_ondispose_to_dvc_main(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnime").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
    TitleBar::open_header(draconic_vibe_name(), super::super::super::VERSION, "");
}