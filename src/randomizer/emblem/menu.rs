use unity::prelude::*;
use super::*;
use engage::menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult};
use engage::dialog::yesno::TwoChoiceDialogMethods;

pub struct RandomEmblemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemMod {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = custom::cemblem_recruitment_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let max = if EMBLEM_LIST.get().unwrap().len() > 20 { 4 } else { 3 };
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_mode, 0, max, 1);
        if CONFIG.lock().unwrap().emblem_mode != result {
            CONFIG.lock().unwrap().emblem_mode = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().emblem_mode {
            1 => { "Emblem recruitment will be randomized." },
            2 => { "Emblem recruitment will be in reversed order" },
            3 => { "Emblem recruitment will determined by list. (Press A)"},
            4 => { "Random recruitment with custom emblems."},
            _ => { "Default recruitment order for emblems." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_mode  {
            1 => { "Random" },
            2 => { "Reverse" },
            3 => { "Custom Order (A)" },
            4 => { "Custom Emblems"},
            _ => { "Standard"},
        }.into();
    }
}

pub struct RandomEmblemLinkMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemLinkMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("EngagePlus", GameVariableManager::get_number(DVCVariables::ENGAGE_P_KEY));
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engage_link = result; }
            else { GameVariableManager::set_bool("EngagePlus", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        this.help_text = if value { "Units are linked to emblems for Engage+." }
            else { "Units will not be linked to emblems." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        this.command_text = if value { "Random Links" } else { "No Links" }.into();
    }
}

pub struct RandomGodMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGodMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_mode != result {
            CONFIG.lock().unwrap().random_god_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_god_mode {
            1 => { "Inheritiable skills will be randomized."},
            2 => { "Engage Attacks and Linked Engage Attacks will be randomized." },
            3 => { "Inheritiable skills and Engage Attacks will be randomized." },
            _ => { "No Randomization to emblem data."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_god_mode {
            1 => { "Skill Inheritance" },
            2 => { "Engage Atks" },
            3 => { "Inherits/Engage Atks"},
            _ => { "None" },
        }.into();
    }
}
pub struct RandomSynchoMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSynchoMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_sync_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_sync_mode != result {
            CONFIG.lock().unwrap().random_god_sync_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_god_sync_mode {
            1 => { "Emblem stat bonuses are randomized." },
            2 => { "Emblem sync and engage skills are randomized." },
            3 => { "Emblem stats, sync, and engage skills are randomized." },
            _ => { "No changes to sync/engage emblem data."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_god_sync_mode {
            1 => { "Stat Bonuses" },
            2 => { "Sync/Engage Skills" },
            3 => { "All Sync"},
            _ => { "None"},
        }.into();
    }
}
pub struct RandomEngageWepMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEngageWepMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_engage_weapon);
        if CONFIG.lock().unwrap().random_engage_weapon != result {
            CONFIG.lock().unwrap().random_engage_weapon  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_engage_weapon {  "Engage Items/Weapons are randomized"  }
            else { "No changes to Engage items/weapons." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_engage_weapon {  "Randomize Weapons" }
            else { "Default Item/Weapons" }.into();
    }
}

pub struct EngageLinkConfirm;
impl TwoChoiceDialogMethods for EngageLinkConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomize_engage_links(true);
        GameVariableManager::set_number(DVCVariables::ENGAGE_P_KEY, GameVariableManager::get_number("EngagePlus"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        RandomEmblemLinkMod::set_help_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn engage_link_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::is_main_menu() {return BasicMenuResult::new(); }
    if GameVariableManager::get_number(DVCVariables::ENGAGE_P_KEY) == GameVariableManager::get_number("EngagePlus") { return BasicMenuResult::new();}
    YesNoDialog::bind::<EngageLinkConfirm>(this.menu, "Change Engage Link Settings?", "Do it!", "Nah..");
    return BasicMenuResult::new();
}

pub extern "C" fn vibe_engage_links() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<RandomEmblemLinkMod>("Unit-Emblem Links");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = engage_link_acall as _ );
    switch
}