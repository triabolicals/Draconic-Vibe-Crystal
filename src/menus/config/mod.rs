use engage::{
    app::{
        GameUI, Proc, ProcBoolMethod, ProcDesc, ProcVoidMethod, ResourceManager_2,
        PersonData, UnitPool
    },
    root::{
        configroot::*, configsequence::*, configmenucontent::*,
        configbasicmenuitem::*,
    },
    system::collections::generic::IList_1Methods, ProcVoidMethodExt
};
use engage::app::GameSound;
use super::*;

mod text;
mod menuitem;

pub use {text::{CONFIG_TEXT, DVCConfigText}, menuitem::{DVCMenuItemKind, *}};
use crate::{dvc::unit::*, utils, gamedata::*, };

#[unity::inject(namespace="DVC", name="DVCSettingMenu", parent=BasicMenu)]
pub struct DVCSettingMenu {
    pub selects: Array<BasicMenuSelect>,
    pub current_menu_kind: i32,
    pub next_menu_kind: i32,
    pub init_variable: Array<i32>,
    pub init_flag: Array<bool>,
    pub current_var_values: Array<i32>,
    pub main_menu: bool,
}

#[unity::injected_methods]
impl DVCSettingMenu {
    #[override_virtual(name="BCall")]
    pub fn b_call(self) -> BasicMenu_Result {
        if let Some(v) = self.get_current_menu().get_previous() {
            self.rebuild_menu(v);
            BasicMenu_Result::se_cursor()
        }
        else { BasicMenu_Result::close_cancel() }
    }
    #[override_virtual(name="OnClose")]
    pub fn on_close(self) {
        if !DVCVariables::is_main_menu() {
            let mut emblem_change = false;
            let data = GameData::get();
            let randomized = RandomizedGameData::get_read();
            if self.is_flag_change(DVCFlags::RandomEnable){
                randomized.commit(data);
                return;
            }
            if !DVCFlags::RandomEnable.get_value() { return; }
            data.get_interaction().into_iter().enumerate().for_each(|(i, f)|{ InteractData::try_get_2(i as i32).get_flag().set_m_value(f); });
            if self.is_variable_change(DVCVariable::BattleStyles) { data.job_db.update_styles(); }
            if self.is_flag_change(DVCFlags::RandomClassAttrs) { data.job_db.update_attr(); }
            if self.is_variable_change(DVCVariable::EmblemWepProf) { randomized.emblem_aptitude_randomizer.commit(data); }
            if self.is_variable_change(DVCVariable::EmblemSyncSkill) || self.is_variable_change(DVCVariable::EmblemEngageSkill) {
                randomized.engage_skills.commit(data);
                emblem_change = true;
            }
            if self.is_flag_change(DVCFlags::EngageAttacks) || self.is_flag_change(DVCFlags::EngageWeapons) {
                randomized.engage_atks.commit(data);
                randomized.engage_weapons.commit(data);
                emblem_change = true;
            }
            if self.is_flag_change(DVCFlags::EmblemStats) {
                randomized.engage_skills.commit_stats(data);
                emblem_change = true;
            }
            if emblem_change { randomized.update_enemy_emblem(data); }
            if self.is_flag_change(DVCFlags::PersonalCaps)|| self.is_variable_change(DVCVariable::PersonalGrowthMode) ||
                self.is_flag_change(DVCFlags::RandomClassGrowth) || self.is_flag_change(DVCFlags::AdaptiveGrowths) || self.is_flag_change(DVCFlags::MaxStatCaps)
            {
                data.growth_data.update();
            }
            if self.is_variable_change(DVCVariable::EngraveLevel) { data.update_engraves(); }
            if self.is_variable_change(DVCVariable::JobLearn) || self.is_variable_change(DVCVariable::RandomLearnLevel) {
                for_each_unit(31, |unit|{ learn_skill::update_learn_skill(unit); })
            }
            if self.is_flag_change(DVCFlags::BondRing) || self.is_flag_change(DVCFlags::RingStats) { data.update_bond_ring(); }
            if self.is_flag_change(DVCFlags::PersonalSkills) { data.update_personals(); }
            if self.is_flag_change(DVCFlags::BGM) {
                if DVCVariables::get_sequence() == 3 {
                    if DVCFlags::BGM.get_value() { crate::dvc::randomizer::bgm::set_random_bgm(); }
                    else { GameSound::field_bgm_set_phase_bgm(DVCVariables::get_current_chapter(), false); }
                    GameSound::field_bgm_play(Force_Type::player());
                }
            }
            if self.is_flag_change(DVCFlags::WeaponStats) || self.is_flag_change(DVCFlags::WeaponSkill) {
                randomized.update_weapon_stat_skills(data);
            }
            if self.is_flag_change(DVCFlags::RandomExplorationItems) || self.is_variable_change(DVCVariable::ExplorationItem) {
                crate::dvc::randomizer::hub::hub_item_randomization();
            }
            if self.is_variable_change(DVCVariable::EnemyCapabilityOffset){
                for_each_unit(2, |unit|{
                    let current_hp = unit.get_hp();
                    let max_hp_before = unit.get_capability(0, true);
                    grow::enemy_unit_stat_scaling(unit, unit.get_level() + unit.get_internal_level(), get_enemy_adjustment(unit));
                    let max_hp = unit.get_capability(0, true);
                    if max_hp_before == current_hp || current_hp > max_hp { unit.set_hp(max_hp); }
                })
            }
            let mode = DVCVariable::ClassMode.get_value();
            if self.is_variable_change(DVCVariable::ClassMode) && mode > 1 && mode != 6 {
                let mut reclass_mode = None;
                match mode {
                    2 => {
                        if DVCVariable::SingleJob.get_value() == 0 { DVCVariable::ClassMode.set_value(self.init_variable().get(15)); }
                        else { reclass_mode = Some(ReclassType::PlayerSingle(false)); }
                    }
                    3|4 => {
                        let units = job::get_lookout_unit_reclass();
                        units.into_iter().map(|v| PersonData::try_get_from_hash(v))
                            .flat_map(|p| utils::to_opt(UnitPool::get_from_person(p, false)))
                            .for_each(|unit|{ job::unit_reclass(unit, ReclassType::PlayerLockout(false, mode == 4)); });
                        crate::dvc::unit::item::change_liberation_type();
                    }
                    5 => {
                        if DVCVariable::GenderClassM.get_value() == 0 && DVCVariable::GenderClassF.get_value() == 0 {
                            DVCVariable::ClassMode.set_value(self.init_variable().get(15));
                        }
                        else { reclass_mode = Some(ReclassType::GenderSingle(false)); }
                    }
                    _ => {}
                }
                if let Some(reclass_mode) = reclass_mode {
                    for_each_unit(31, |unit|{
                        if PersonPool::is_playable(unit.get_person()) {
                            let current_hp = unit.get_hp();
                            let max_hp_before = unit.get_capability(0, true);
                            job::unit_reclass(unit, reclass_mode);
                            let max_hp = unit.get_capability(0, true);
                            if max_hp_before == current_hp || current_hp > max_hp { unit.set_hp(max_hp); }
                        }
                    });
                    crate::dvc::unit::item::change_liberation_type();
                }

            }
        }
    }
}
impl DVCSettingMenu {
    pub fn create_settings_menu_item() -> DVCBasicMenuItem {
        DVCBasicMenuItem::from_type(DVCMenuItemKind::Command(DVCCommand::Menu), 0)
    }
    pub fn get_current_menu(self) -> DVCMenu { DVCMenu::from(self.current_menu_kind()).unwrap() }
    pub fn rebuild(self) {
        let kind = self.get_current_menu();
        self.rebuild_menu(kind);
    }
    pub fn set_current_value(self, var: DVCVariable, value: i32) {
        let v = var as i32;
        if v >= 0 && v < 60 { self.current_var_values().set(var as usize, value); }
    }
    pub fn is_variable_change(self, var: DVCVariable) -> bool { var.get_value() != self.init_variable().get(var as usize) }
    pub fn is_flag_change(self, flag: DVCFlags) -> bool { flag.get_value() != self.init_flag().get(flag as usize) }
    fn set_current_var_value(self, item: DVCBasicMenuItem) {
        let current = self.current_var_values();
        match item.get_menu_item_type() {
            DVCMenuItemKind::Variable(var) => { item.set_value(current.get(var as usize)); }
            DVCMenuItemKind::Job(job) => { if job != DVCJobItem::UnitJob { item.set_value(current.get(job.get_var() as usize)); } }
            _ => {}
        }
    }
    pub fn rebuild_menu(self, menu: DVCMenu) {
        let current_select = self.selects().get(self.get_current_menu() as usize);
        current_select.set_index(IBasicMenuMethods::get_select_index(self));
        current_select.set_scroll(IBasicMenuMethods::get_scroll_index(self));
        let select = self.selects().get(menu as usize);
        let list = self.m_full_menu_item_list();
        list.clear();
        self.set_current_menu_kind(menu as i32);
        let items =
            if menu == DVCMenu::UnitClass { menu.get_class_items(self.current_var_values().get(DVCVariable::ClassMode as usize)) }
            else { menu.get_items() };

        items.into_iter().enumerate()
            .for_each(|t| {
                let item = DVCBasicMenuItem::from_type(t.1, t.0 as i32);
                self.set_current_var_value(item);
                list.add(item.as_basic_menu_item());
            });

        self.set_show_row_auto();
        IBasicMenuMethods::rebuild_instant_2(self, select);
        IBasicMenuMethods::restore_select(self, select);
        self.apply_title();
    }
    pub fn create_bind(proc: impl Into<ProcInst>) {
        let config_sequence = ConfigSequence::new();
        let config = DVCConfig::get();
        config.adjust_custom_recruitment();
        config.save();
        if DVCVariables::get_sequence() == 0 {
            let descs = [
                Proc::label(0),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), ConfigSequence::load_resources_method_info().into())),
                Proc::wait_while_true_2(ProcBoolMethod::new(config_sequence.into(), ConfigSequence::is_loading_resources_method_info().into())),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), ConfigSequence::start_sequence_method_info().into())),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), create_dvc_setting_menu_method_info().into())),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), ConfigSequence::end_sequence_method_info().into())),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), ConfigSequence::destroy_config_menu_method_info().into())),
                Proc::label(1),
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), ConfigSequence::unload_resources_method_info().into())),
                Proc::label(2),
                Proc::end(),
            ];
            let descs_array: Array<ProcDesc> = Array::from_slice(&descs).unwrap();
            config_sequence.create_bind(proc, descs_array, "DVCConfigSequence");
        }
        else {
            let descs = [
                Proc::call_2(ProcVoidMethod::new(config_sequence.into(), create_dvc_setting_menu_method_info().into())),
                Proc::call_2(ProcVoidMethod::from_fn(config_sequence, Self::restore_previous_menu).unwrap()),
                Proc::end(),
            ];
            let descs_array: Array<ProcDesc> = Array::from_slice(&descs).unwrap();
            config_sequence.create_bind(proc, descs_array, "DVCConfigSequence");
        }
    }
    extern "C" fn restore_previous_menu(this: ConfigSequence, _method: OptionalMethod) {
        if let Some(menu) = this.get_super().try_cast::<BasicMenu>() {
            menu.set_active(true);
            let content = menu.m_menu_content();
            let suspend = IBasicMenu::m_suspend(menu) - 1;
            if suspend < 1 {
                IBasicMenu::set_m_suspend(menu, 0);
                if !content.is_null() { content.un_suspend(); }
                menu.on_resume();
            }
            else { IBasicMenu::set_m_suspend(menu, suspend); }
            engage::BasicMenuExt::open_anime_all(menu);
            engage::TitleBar::open_header(Mess::get("MID_MENU_CONFIG"), Mess::get("MID_MENU_CONFIG_HELP"), "KHID_環境設定_項目選択");
        }
    }
    pub fn apply_title(self) { DVCConfigText::apply_menu_header(self.get_current_menu()); }
}
#[unity::callback]
fn create_dvc_setting_menu(config: ConfigSequence, _: OptionalMethod) {
    let root = GameUI::get_root();
    let go = ResourceManager_2::instantiate("UI/Common/Config/Prefabs/ConfigRoot", root);
    let config_root = go.get_component::<ConfigRoot>();
    if config_root.is_null() { return; }
    let menu_content = config_root.m_config_menu_content_object().get_component::<ConfigMenuContent>();
    let list: List_1<BasicMenuItem> = List_1::new();
    DVCMenu::Main.get_items().iter().enumerate().for_each(|(i, v)|{ list.add(DVCBasicMenuItem::from_type(*v, i as i32).as_basic_menu_item()); });
    let menu = DVCSettingMenu::instantiate().unwrap();
    IBasicMenuMethods::ctor(menu, list, menu_content);
    let init_var: Array<i32> = Array::new(i32::class().raw(), 60).unwrap();
    let init_flag: Array<bool> = Array::new(bool::class().raw(), 60).unwrap();
    let current_var_values: Array<i32> = Array::new(i32::class().raw(), 60).unwrap();
    menu.set_main_menu(DVCVariables::is_main_menu());
    menu.set_current_menu_kind(DVCMenu::Main as i32);
    for i in 0..60 {
        if let Some(v) = DVCVariable::from(i) {
            let value = v.get_setting_value();
            init_var.set( v as usize, value);
            current_var_values.set(v as usize, value);
        }
        init_flag.set(i as usize, DVCFlags::from(i).map(|f| f.get_setting_value()).unwrap_or(false));
    }
    menu.set_current_var_values(current_var_values);
    menu.set_init_flag(init_flag);
    menu.set_init_variable(init_var);
    let selects: Array<BasicMenuSelect> = Array::new(BasicMenuSelect::class().raw(), 25).unwrap();
    for x in 0..25 {
        let select = BasicMenuSelect::new();
        select.set_index(0);
        select.set_scroll(0);
        selects.set(x, select);
    }
    menu.set_selects(selects);
    let descs = IBasicMenuMethods::create_default_desc(menu);
    IProcInstMethods::create_bind(menu, config, descs, "DVCSettingMenu");
    let row_count = IConfigMenuContentMethods::get_menu_item_content_max(menu_content);
    menu.set_show_row_num(row_count - 1);
}

