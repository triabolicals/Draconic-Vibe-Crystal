use unity::prelude::*;
use engage::{
    mess::Mess,
    proc::{desc::ProcDesc, ProcVoidMethod, ProcInst},
    sequence::mapsequence::human::MapSequenceHumanLabel,
    gameuserdata::GameUserData, gamevariable::*, menu::{
        config::{ ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuItem, BasicMenuItemAttribute, BasicMenuResult
    }
};
use crate::{utils::{get_nested_virtual_methods_mut, get_nested_nested_virtual_method_mut}, CONFIG};

pub struct IronmanMod;
impl ConfigBasicMenuItemSwitchMethods for IronmanMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().iron_man);
        if CONFIG.lock().unwrap().iron_man != result {
            CONFIG.lock().unwrap().iron_man = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().iron_man {  "Disables Draconic Time Crystal and bookmarks." }
            else {"Disables Ironman mode." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().iron_man {  "On" } else { "Off"}.into();
    }
}


pub fn ironman_code_edits(){
    if GameVariableManager::get_bool("G_Ironman") { unsafe { map_history_rewind_disable(None); } }
    else  {  unsafe { map_history_rewind_enable(None); } }
} 

pub fn map_save_proc_edit(map_sequence_human: &ProcInst) {
    let descs = map_sequence_human.descs.get();
    unsafe { 
    // Force MapSequenceHuman to Jump to Label 47 at Label 48 MapSequenceHumanLabel::SuspendMenu
        (*descs)[0xd0] = ProcDesc::jump(MapSequenceHumanLabel::SaveMenu as i32); 
    // Replace MapSequenceHuman$$SaveAndSuspendMenuBefore to remove Temporary Status
        (*descs)[0xcb] = ProcDesc::call(ProcVoidMethod::new(None, remove_temporary_game_status));  
    }
}

extern "C" fn remove_temporary_game_status(_proc: &mut ProcInst, _method_info: OptionalMethod) {
    let status = GameUserData::get_status();
    status.value &= !0x200;
}

pub fn map_save_menu_edits() {
    get_nested_virtual_methods_mut("App", "MapSystemMenu", "TemporarySaveItem", "GetName")
        .map(|m| m.method_ptr = map_system_temp_save_menu_name as _).unwrap();

    get_nested_virtual_methods_mut("App", "MapSystemMenu", "TemporarySaveItem", "GetHelpText")
        .map(|m| m.method_ptr = map_system_temp_save_get_help_text as _).unwrap();

    get_nested_virtual_methods_mut("App", "MapSystemMenu", "TemporarySaveItem", "GetMapAttribute")
        .map(|m| m.method_ptr = map_system_temp_save_build_attr as _).unwrap();

    get_nested_nested_virtual_method_mut("App", "MapSystemMenu", "SubSystemMenu", "RestartItem", "BuildAttribute")
        .map(|m| m.method_ptr = restart_menu_item_build_attr as _).unwrap();

    get_nested_nested_virtual_method_mut("App", "MapSystemMenu", "SubSystemMenu", "ResetItem", "BuildAttribute")
        .map(|m| m.method_ptr = reset_menu_item_build_attr as _).unwrap();
}

fn map_system_temp_save_menu_name(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { Mess::get("MID_SORTIE_SAVE") }

fn map_system_temp_save_get_help_text(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { Mess::get("MID_SORTIE_SAVE_HELP") }

fn map_system_temp_save_build_attr(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_bool("G_Ironman") { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

fn restart_menu_item_build_attr(restart_item: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_bool("G_Ironman") { BasicMenuItemAttribute::Hide }
    else {
        unsafe { original_restart_item_build_attr(restart_item, None) }
    }
}

fn reset_menu_item_build_attr(_reset_item: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_bool("G_Ironman") { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable } 
}

#[skyline::from_offset(0x01b72cb0)]
fn original_restart_item_build_attr(restart_item: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute;

#[unity::from_offset("App", "MapHistory", "RewindDisable")]
fn map_history_rewind_disable(method_info: OptionalMethod);

#[unity::from_offset("App", "MapHistory", "RewindEnable")]
fn map_history_rewind_enable(method_info: OptionalMethod);

