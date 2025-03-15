use unity::prelude::*;
use super::*;
use engage::{
    gamevariable::GameVariableManager,
    gameuserdata::GameUserData, 
    menu::{config::ConfigBasicMenuItem, BasicMenuResult}, 
};
pub struct RandomEmblemEnergy;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemEnergy {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry(DVCVariables::TERRAIN, 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().terrain } else { GameVariableManager::get_number(DVCVariables::TERRAIN) };
        let result =  ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().terrain = result; }
            else { GameVariableManager::set_number(DVCVariables::TERRAIN, result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().terrain } else { GameVariableManager::get_number(DVCVariables::TERRAIN) };
        this.help_text = match value {
            1 => { "Randomizes emblem energy spots." },
            2 => { "Chance for terrain/energy tile each turn." }
            _ => { "No changes to emblem energy / terrain on the map." }
        }.into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().terrain } else { GameVariableManager::get_number(DVCVariables::TERRAIN) };
        this.command_text = match value {
            1 => { "Energy" },
            2 => { "Terrain" },
            _ => { "Default" },
        }.into();
    }
}

pub extern "C" fn vibe_energy() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<RandomEmblemEnergy>("Terrain Effects");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    switch
}

pub struct FOWSetting;
impl ConfigBasicMenuItemSwitchMethods for FOWSetting {
    fn init_content(_this: &mut ConfigBasicMenuItem){ 
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry(DVCVariables::FOW, 0);
            GameVariableManager::make_entry("FOW2", GameVariableManager::get_number(DVCVariables::FOW));
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().fow } else { GameVariableManager::get_number("FOW2") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 4, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().fow = result; }
            else { GameVariableManager::set_number("FOW2", result); };
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().fow } else { GameVariableManager::get_number("FOW2") };
        this.help_text = match value {
            1 => { "Enables fog of war on maps." },
            2 => { "Disables fog of war on maps." }
            3 => { "Fog will be randomly enabled (Turn by Turn)."}
            4 => { "Fog will be randomly enabled (Map by Map)."}
            _ => { "Default behavior for fog of war." }
        }.into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().fow } else { GameVariableManager::get_number("FOW2") };
        let changed = if GameUserData::get_sequence() != 3 && GameUserData::get_sequence() != 2 { "".to_string() }
            else { DVCVariables::changed_setting_text("FOW2", DVCVariables::FOW) };
        this.command_text = format!("{}{}", changed, match value {
            1 => { "On" },
            2 => { "Off" },
            3 => { "Turn Random"},
            4 => { "Map Random"},
            _ => { "Default" },
        }).into();
    }
}


pub struct FOWConfirm;
impl TwoChoiceDialogMethods for FOWConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::FOW, GameVariableManager::get_number("FOW2"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;

        FOWSetting::set_help_text(menu.menu_item_list[index as usize], None);
        FOWSetting::set_command_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        super::fow::change_map_sight();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn fow_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::is_main_menu() { return BasicMenuResult::new(); }
    if GameVariableManager::get_number("FOW2") == GameVariableManager::get_number(DVCVariables::FOW) { return BasicMenuResult::new();}
    if GameUserData::get_sequence() != 2 && GameUserData::get_sequence() != 3 {
        GameVariableManager::set_number(DVCVariables::FOW, GameVariableManager::get_number("FOW2"));
        return BasicMenuResult::new();
    }
    YesNoDialog::bind::<FOWConfirm>(this.menu, "Change Fog of War Setting?", "Do it!", "Nah..");
    return BasicMenuResult::new();
}

pub extern "C" fn vibe_fow() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<FOWSetting>("Fog of War Setting");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = fow_acall as _ );
    switch
}