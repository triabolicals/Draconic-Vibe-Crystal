use std::sync::OnceLock;
use engage::dialog::BasicDialogItem;
use engage::gamedata::{Gamedata, JobData};
use engage::menu::{BasicMenu, BasicMenuContent, BasicMenuMethods, BasicMenuResult};
use engage::mess::Mess;
use engage::proc::{Bindable, ProcInstFields};
use engage::unit::UnitPool;
use num_traits::FromPrimitive;
use unity::prelude::*;
use unity::system::{Il2CppString, List};
use dvc_local::{DVCCommandText, DVCLocalizer};
use crate::config::DVCFlags;
use crate::{randomizer, DVCVariables};
use crate::menus::items::{DVCConfigMenuItem, MenuType};
use crate::randomizer::data::GameData;
use crate::randomizer::{get_rand_data_read, interact};
use crate::randomizer::job::unit_change_to_random_class;

pub static DVC_CONFIRM_DIALOG: OnceLock<&'static Il2CppClass> = OnceLock::new();
#[unity::class("App", "YesNoDialog")]
pub struct DVCConfirmDialog {
    pub proc: ProcInstFields,
    pub menu_content: &'static mut BasicMenuContent,
    pub menu_item_list: &'static mut List<BasicDialogItem>,
    pub full_menu_item_list: &'static mut List<BasicDialogItem>,
    status_field: *const u8,
    pub result: i32,
    scroll_preced_input_a: bool,
    pub row_num: i32,
    pub show_row_num: i32,
    pub select_index: i32,
    pub select_index_old: i32,
    pub scroll_index: i32,
    pub scroll_index_old: i32,
    pub reserved_select_index: i32,
    pub reserved_scroll_index: i32,
    pub reserved_show_row_num: i32,
    pub memory_display_index: i32,
    pub suspend: i32,
    pub menu_index: i32,
    content: u64,
    b_bind_bg: bool,
    b_is_not_bind_bg: bool,
    pub not_config: bool,
    pad: bool,
    pub dvc_index: i32,
}
impl DVCConfirmDialog {
    pub fn create_class() -> &'static Il2CppClass {
        let klass = Il2CppClass::from_name("App", "YesNoDialog").unwrap().clone();
        let vtable = klass.get_vtable_mut();
        vtable[50].method_ptr = Self::a_call as _;
        vtable[51].method_ptr = Self::b_call as _;
        klass
    }
    pub fn new_confirm_cancel<B: Bindable>(parent: &B, message: &Il2CppString) {
        let dialog = DVC_CONFIRM_DIALOG
            .get_or_init(||Self::create_class())
            .instantiate_as::<BasicMenu<BasicDialogItem>>().unwrap();
        
        let list = unsafe { create_menu_item(None) };
        list.clear();
        list.add(BasicDialogItem::new(DVCCommandText::Confirm.get().to_string()));
        list.add(BasicDialogItem::new(DVCCommandText::Cancel.get().to_string()));
        unsafe { basic_dialog_ctor(dialog, list, basic_dialog_content(None), None); }

        let desc = dialog.create_default_desc();
        dialog.create_bind(parent, desc, "DVCDialog");
        dialog.bind_parent_menu();
        unsafe {
            basic_dialog_set_text(dialog, message, None);
            let s = std::mem::transmute::<_, &mut Self>(dialog);
            s.not_config = true;
        }
    }
    pub fn new_confirm(this: &DVCConfigMenuItem, single_job: bool) -> &'static Self {
        let dialog = DVC_CONFIRM_DIALOG.get_or_init(||Self::create_class()).instantiate_as::<BasicMenu<BasicDialogItem>>().unwrap();
        let list = this.menu.full_menu_item_list.get_class().instantiate_as::<List<BasicDialogItem>>().unwrap();
        list.items = Il2CppArray::new(2).unwrap();
        let message = 
            if single_job { 
                let new_setting =
                    match this.dvc_value {
                        0 => { DVCCommandText::Off.get() }
                        1 => { Mess::get("MJID_ShadowLord")}
                        _ => { JobData::try_get_hash(this.dvc_value).map(|v| Mess::get(v.name)).unwrap_or(Mess::get("MPID_Unknown")) }
                };
                DVCLocalizer::change_text(this.title.to_string().as_str(), new_setting).into()
            }
            else if this.menu_id == -3 { DVCLocalizer::get_config_title(-309) }
            else { DVCLocalizer::change_text(this.title.to_string().as_str(), this.command_text).into() };
        
        list.add(BasicDialogItem::new(Mess::get("MID_SELECTRING_GOD_CHANGE_YES").to_string()));
        list.add(BasicDialogItem::new(Mess::get("MID_KEYHELP_MENU_RETURN").to_string()));
        unsafe { basic_dialog_ctor(dialog, list, basic_dialog_content(None), None); }
        
        let desc = dialog.create_default_desc();
        dialog.create_bind(this.menu, desc, "DVCDialog");
        dialog.bind_parent_menu();
        unsafe {
            basic_dialog_set_text(dialog, message, None);
            std::mem::transmute::<_, &Self>(dialog)
        }
    }
    pub fn a_call(this: &mut DVCConfirmDialog, _optional_method: OptionalMethod) -> BasicMenuResult {
        if this.not_config { BasicMenuResult::new() }
        else {
            if this.select_index == 0 {
                let previous_menu = this.proc.parent.as_mut().unwrap().cast_mut::<BasicMenu<DVCConfigMenuItem>>();
                let index = previous_menu.select_index;
                if let Some(item) = previous_menu.get_item(index){
                    if let Some(data) = item.get_menu_data() {
                        match data.menu_type {
                            MenuType::Job => {
                                if JobData::try_get_hash(item.dvc_value).is_some(){
                                    DVCVariables::SingleJob.set_value(item.dvc_value);
                                    for x in 1..250 {
                                        if let Some(unit) = UnitPool::get(x).filter(|u| u.force.is_some_and(|f| (1 << f.force_type) & 57 != 0)){
                                            unit_change_to_random_class(unit, false);
                                            if unit.force.is_some_and(|f| f.force_type < 3) { unit.reload_actor(); }
                                        }
                                    }
                                }
                                else { DVCVariables::SingleJob.set_value(0) }
                            }
                            MenuType::Flag => {
                                let v = item.dvc_value != 0;
                                DVCFlags::set_by_index(data.var_index, v);
                                Self::execute_a_call_action(true, data.var_index);
                            }
                            _ => {
                                let v = item.dvc_value;
                                DVCVariables::set_variable_value(data.var_index, v);
                                Self::execute_a_call_action(false, data.var_index);
                            }
                        }
                        item.is_command = false;
                        item.update_text();
                        BasicMenuResult::se_decide()
                    }
                    else { BasicMenuResult::se_cursor() }
                }
                else { BasicMenuResult::se_cursor() }
            }
            else { BasicMenuResult::new().with_se_cancel(true) }.with_close_this(true)
        }
    }
    pub fn b_call(_this: &DVCConfirmDialog, _optional_method: OptionalMethod) -> BasicMenuResult {
        BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
    }
    pub fn execute_a_call_action(is_flag: bool, index: i32) {
        if is_flag {
            if let Some(flag) = DVCFlags::from(index) {
                match flag {
                    DVCFlags::EngageWeapons|DVCFlags::EngageAttacks => {
                        let data = GameData::get();
                        let random = get_rand_data_read();
                        random.update_engage_atk_items(data);
                        random.update_enemy_emblem(data);
                    }
                    DVCFlags::RandomSP|DVCFlags::EmblemStats => {
                        let data = GameData::get();
                        let random = get_rand_data_read();
                        random.engage_skills.commit(data);
                    }
                    DVCFlags::BondRing => { randomizer::data::randomize_bond_ring_skills(); }
                    DVCFlags::AdaptiveGrowths|DVCFlags::RandomClassGrowth => {
                        randomizer::grow::random_grow();
                    }
                    DVCFlags::EvolveItems => {
                        let random = get_rand_data_read();
                        let data = GameData::get();
                        random.update_evolve_items(data);
                    }
                    _ => {}
                }
            }
        }
        else {
            if let Some(key) = DVCVariables::from(index) {
                let value = key.get_value();
                match key {
                    DVCVariables::InteractSetting => { interact::change_interaction_data(value, false); }
                    DVCVariables::BattleStyles => { randomizer::styles::randomize_job_styles(); }
                    DVCVariables::EmblemEngageSkill|DVCVariables::EmblemSyncSkill => {
                        let data = GameData::get();
                        let random = get_rand_data_read();
                        random.engage_skills.commit(data);
                        random.update_enemy_emblem(data);
                    }
                    DVCVariables::JobLearnMode => { randomizer::skill::learn::update_learn_skills(true); }
                    _ => {}
                }
            }
            else {
                match index {
                    53 => { randomizer::job::rerandomize_jobs(); }
                    _ => {}
                }
            }
        }
    }
}

#[unity::from_offset("App", "BasicDialogContent", "Create")]
fn basic_dialog_content(method_info: OptionalMethod) -> &'static BasicMenuContent;

#[unity::from_offset("App", "YesNoDialog", ".ctor")]
fn basic_dialog_ctor(this: &BasicMenu<BasicDialogItem>, list: &List<BasicDialogItem>, content: &BasicMenuContent, method_info: OptionalMethod);

#[skyline::from_offset(0x2453d90)]
fn basic_dialog_set_text(this: &BasicMenu<BasicDialogItem>, text: &Il2CppString, optional_method: OptionalMethod);

#[unity::from_offset("App", "AchievementMenu", "CreateMenuItem")]
fn create_menu_item(method_info: OptionalMethod) -> &'static mut List<BasicDialogItem>;


