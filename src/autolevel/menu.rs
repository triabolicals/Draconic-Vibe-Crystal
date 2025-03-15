use super::*;

pub struct BenchAutoLevelOption;
impl ConfigBasicMenuItemSwitchMethods for BenchAutoLevelOption {
    fn init_content(_this: &mut ConfigBasicMenuItem)    {
        GameVariableManager::make_entry(DVCVariables::AUTOLEVEL_BENCH_KEY, 0); 
    } 
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle =  GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY);
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);
        if toggle != result {
            GameVariableManager::set_bool(DVCVariables::AUTOLEVEL_BENCH_KEY, result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new();  }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = 
            if !GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY) {"Undeployed will not be autoleveled at the end of the chapter." }
            else { "Undeployed units will autolevel to difficulty-adjusted average." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if !GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY) { "Disabled" } else { "Enabled" }.into();
    }
}
pub extern "C" fn autobench() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<BenchAutoLevelOption>("Post Chapter Autoleveling")   }

pub struct AutolevelMod;
impl ConfigBasicMenuItemSwitchMethods for AutolevelMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().autolevel = result; }
            else { GameVariableManager::set_bool(DVCVariables::DVC_AUTOLEVEL_KEY,result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        this.help_text = if value { "Units/enemies will be scaled to army's power." }
            else { "No changes to recruited/enemy unit's stats and levels." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        this.command_text = if value { "Autoscale" } else { "No Scaling" }.into();
    }
}

pub fn auto_level_build(this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCVariables::is_random_map() { return BasicMenuItemAttribute::Hide }
    else { crate::menus::buildattr::not_in_map_sortie_build_attr(this, None) }
}
pub extern "C" fn vibe_autolevel() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<AutolevelMod>("Level Scale Units");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = auto_level_build as _);
    switch
} 
