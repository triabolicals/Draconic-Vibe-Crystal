use engage::app::{BasicMenu_Result, IBitField32, IForceMethods, IJobDataMethods, IUnit, Mess_IconCategory, SoftwareKeyboard, SoftwareKeyboard_Preset, Unit};
use crate::{
    dvc::unit,
    utils::to_opt
};
use crate::dvc::autolevel::try_autoscale_unit;
use super::*;
const WEP_KEY: [&str; 4] = ["Sword", "Lance", "Ax", "Bow"];
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum DVCCommand {
    SetSeed,
    ReRandJob,
    Menu,
}
impl DVCCommand {
    pub fn from(i: i32) -> Option<Self> {
        match i {
            0 => Some(DVCCommand::SetSeed),
            1 => Some(DVCCommand::ReRandJob),
            2 => Some(DVCCommand::Menu),
            _ => None,
        }
    }
    pub fn a_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        match self {
            DVCCommand::SetSeed => {
                let mut v = if DVCVariables::is_main_menu() { DVCConfig::get().seed } else { DVCVariable::Seed.get_setting_value() as u32 };
                if v == 0 { v = DVCVariables::get_random_number_for_seed(); }
                let action: Action_1<Il2CppString> = Action_1::<Il2CppString>::new(menu_item.into(), set_new_dvc_seed_method_info().into());
                SoftwareKeyboard::create_bind(
                    menu_item.get_dvc_menu(),
                    10,
                    v.to_string(),
                    "Enter text / numeric value for DVC seed.",
                    format!("Number Range: 1 to {}.", u32::MAX-1),
                    SoftwareKeyboard_Preset::default(),
                    action
                );
            }
            DVCCommand::ReRandJob => {
                let message =
                    if GameVariableManager::get_bool("G_Cleared_M003") { "Re-Randomize unrecruited unit classes?" }
                    else { "Re-Randomize player unit classes?" };
                let action = Action::new(menu_item.into(), re_randomize_class_method_info().into());
                dialog::create_dialog_confirm(menu_item.get_dvc_menu(), message, action);
            }
            DVCCommand::Menu => {
                IBasicMenuMethods::close_anime_all(menu_item.get_menu());
                DVCSettingMenu::create_bind(menu_item.get_menu());
                return BasicMenu_Result::se_cursor();
            }
        }
        BasicMenu_Result::se_cursor()
    }
    pub fn plus_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        if self == DVCCommand::SetSeed {
            let seed = DVCVariables::get_random_number_for_seed();
            menu_item.set_value(seed as i32);
            let message =
                if DVCVariable::Seed.get_value() == 0 || DVCVariables::is_main_menu() { format!("Set Randomizer Seed to:\n{}?", seed) }
                else { format!("Change seed to {}?\nGame data will be re-randomize.", seed) };
            let action = Action::new(menu_item.into(), set_seed_yes_method_info().into());
            dialog::create_dialog_confirm(menu_item.get_dvc_menu(), message, action);
            BasicMenu_Result::se_cursor()
        }
        else { BasicMenu_Result::pass() }
    }
    pub fn build_attribute(self) -> BasicMenuItem_Attribute {
        match self {
            Self::ReRandJob => {
                if DVCVariables::is_main_menu() { BasicMenuItem_Attribute::hide()  } else { re_randomize_jobs_build_attr() }
            }
            Self::SetSeed => {
                if DVCVariables::is_main_menu(){
                    if DVCConfig::get().randomized { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::hide() }
                }
                else if DVCFlags::RandomEnable.get_value() { BasicMenuItem_Attribute::enable() }
                else { BasicMenuItem_Attribute::hide() }
            }
            _ => { BasicMenuItem_Attribute::enable() }
        }
    }
}
#[unity::callback]
fn set_new_dvc_seed(item: DVCBasicMenuItem, value: Il2CppString, _: OptionalMethod) {
    if value.is_null() { return; }
    let v = value.to_string();
    if v.len() == 0 { return; }
    let seed = if let Ok(seed) = v.parse::<u32>() { seed } else { engage::app::Ut::hash_fnv_1_5(value) as u32 };
    if seed == 0 || seed == DVCVariable::Seed.get_setting_value() as u32 { return; }
    item.set_value(seed as i32);
    let message =
        if DVCVariable::Seed.get_setting_value() == 0 || DVCVariables::is_main_menu() { format!("Set Randomizer Seed to:\n{}?", seed) }
        else { format!("Change seed to {}?\nGame data will re-randomize.", seed) };
    let action = Action::new(item.into(), set_seed_yes_method_info().into());
    dialog::create_dialog_confirm(item.get_dvc_menu(), message, action);
}

#[unity::callback]
fn set_seed_yes(item: DVCBasicMenuItem, _: OptionalMethod) {
    update_seed(item.value() as u32);
    item.update_text();
}
fn update_seed(new_seed: u32) {
    if new_seed == DVCVariable::Seed.get_value() as u32 { return; }
    if !DVCVariables::is_main_menu() {
        DVCVariable::Seed.init_var(new_seed as i32, true);
        GameVariableManager::find_starts_with("G_P_PID").iter()
            .for_each(|person_key| GameVariableManager::set_number(person_key.to_string().as_str(), 0));

        GameVariableManager::find_starts_with("G_L_JID").iter()
            .for_each(|job_key| GameVariableManager::set_number(job_key.to_string().as_str(), 0));

        let data = GameData::get();
        let mut rando = RandomizedGameData::get_write();
        data.emblem_pool.reset_all();
        rando.randomize(data);
        rando.commit(data);
    }
    else {
        DVCVariable::Seed.set_value(new_seed as i32);
        DVCConfig::get().seed = new_seed;
    }
}

#[unity::callback]
fn re_randomize_class(this: DVCBasicMenuItem, _: OptionalMethod) {
    let units = rerandomize_jobs();
    if units.len() == 0 { return; }
    else {
        let mut str = String::new();
        units.iter().for_each(|unit| {
            if !str.is_empty() { str += "\n"; }
            str += unit.as_str();
        });
        GameMessage::create_key_wait(this.get_dvc_menu(), str);
    }
}
fn re_randomize_jobs_build_attr() -> BasicMenuItem_Attribute {
    let sequence = DVCVariables::get_sequence();
    if DVCVariables::is_main_menu() || (sequence != 2 && sequence != 3) || DVCVariable::ClassMode.get_value() != 1 { BasicMenuItem_Attribute::hide() }
    else {
        let m003 = DVCVariables::is_main_chapter_complete(3);
        let units = UnitPool::s_units().iter()
            .filter(|u| to_opt(u.get_person()).is_some_and(|p| PersonPool::is_playable(p)) && to_opt(u.m_force()).is_some_and(|v| v.get_force_type().value < 3) && (!m003 || can_re_rand_job_check(*u)))
            .count() > 0;
        if units { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::hide() }
    }
}
pub fn rerandomize_jobs() -> Vec<String> {
    let mut out = vec![];
    let m003 = DVCVariables::is_main_chapter_complete(3);
    UnitPool::s_units().iter()
        .filter(|u| to_opt(u.get_person()).is_some_and(|p| PersonPool::is_playable(p)) && to_opt(u.m_force()).is_some_and(|v| v.get_force_type().value < 3) && (!m003 || can_re_rand_job_check(*u)))
        .for_each(|unit|{
            unit::job::unit_reclass(unit, ReclassType::Recruitment(true));
            try_autoscale_unit(unit);
            unit::item::adjust_unit_items(unit);
            let mut spr = String::new();
            let mask = unit.m_selected_weapon_mask().m_value();
            for x in 1..5 {
                if mask & (1 << x) != 0 {
                    spr += Mess::create_sprite_tag(Mess_IconCategory::system(), WEP_KEY[x-1]).to_string().as_str();
                }
            }
            out.push(format!("{}: {} {}", unit.get_name(), Mess::get(unit.get_job().get_name()), spr));
            unit.item_equip();
            unit.reload_actor();
        });
    if !m003 { unit::item::change_liberation_type(); }
    out
}
fn can_re_rand_job_check(unit: Unit) -> bool { !unit.is_vision() && !unit.is_summon() && unit.get_force_type().value != 0 }