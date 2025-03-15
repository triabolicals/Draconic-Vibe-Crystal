use super::*;

pub struct RandomEnable;
impl ConfigBasicMenuItemSwitchMethods for RandomEnable {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().randomized;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().randomized = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().randomized { "Enables randomization settings on a new save." } 
            else {"Disables randomization settings on a new save." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ this.command_text = if CONFIG.lock().unwrap().randomized { "Enable" } else { "Disable" }.into(); }
}

pub struct RandoSave;
impl ConfigBasicMenuItemSwitchMethods for RandoSave {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().apply_rando_post_new_game;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().apply_rando_post_new_game = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().apply_rando_post_new_game { "Apply disabled randomization settings to saves." } 
            else { "No actions done to previous save files." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().apply_rando_post_new_game { "Enable" } else { "Disable" }.into();
    }
}

pub struct MaxStatCaps;
impl ConfigBasicMenuItemSwitchMethods for MaxStatCaps {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().max_stat_caps;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().max_stat_caps = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().max_stat_caps { "Class stat caps are set to max during the game." } 
            else { "Default class stat caps." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().max_stat_caps { "Enable" } else { "Disable" }.into();
    }
}

pub struct DLCSetting;
impl ConfigBasicMenuItemSwitchMethods for DLCSetting {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().dlc;
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            CONFIG.lock().unwrap().dlc = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().dlc  {
            1 => { "DLC emblems will be excluded from the randomization pool." }
            2 => { "DLC units will be excluded from the randomization pool." }
            3 => { "DLC emblems/units will be excluded from the randomization pool." }
            _ => { "DLC emblems/units will be included from the randomization pool." }
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().dlc {
            1 => { "Exclude Emblems"},
            2 => { "Exclude Units"},
            3 => { "Exclude All"},
            _ => { "Include All"}
        }.into();
    }
}

pub extern "C" fn vibe_enable() -> &'static mut ConfigBasicMenuItem {  ConfigBasicMenuItem::new_switch::<RandomEnable>("DVC Randomization") }
pub extern "C" fn vibe_post_save() -> &'static mut ConfigBasicMenuItem {  ConfigBasicMenuItem::new_switch::<RandoSave>("Randomize Save Files") }
pub extern "C" fn vibe_stats() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<MaxStatCaps>("Max Stat Caps") }
pub extern "C" fn vibe_dlc() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<DLCSetting>("DLC Emblems / Units");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = buildattr::dlc_build_attr as _);
    switch
}
