use super::*;

pub struct BustGauge;
impl ConfigBasicMenuItemGaugeMethods  for BustGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = CONFIG.lock().unwrap().misc_option_1 / 5.0;
        GameVariableManager::make_entry("BustSettingChange", 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = CONFIG.lock().unwrap().misc_option_1 / 5.0;
        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.05);
        let value = (result * 100.0).trunc() / 100.0; 
        if gauge != value {
            CONFIG.lock().unwrap().misc_option_1 = 5.0*value;
            this.gauge_ratio = result;
            GameVariableManager::set_bool("BustSettingChange", true);
            Self::set_help_text(this, None);
            this.update_text();
            CONFIG.lock().unwrap().save();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.is_command_icon = GameVariableManager::get_bool("BustSettingChange");
        this.help_text = 
            if this.gauge_ratio <= 0.09 {  "Current Volume Value: Default".into() }
            else if this.gauge_ratio >= 0.95 { "Current Volume Value: Randomized".into() }
            else { format!("Current Volume Value: {:2}.", this.gauge_ratio*5.0).into() };
    }
}

pub struct BustConfirm;
impl TwoChoiceDialogMethods for BustConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_bool("BustSettingChange", false);
        SEARCH_LIST.get().unwrap().bust.apply_bust_changes();
        let menu =
            unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        menu.menu_item_list[index as usize].is_command_icon = false;
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub fn bust_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if !GameVariableManager::get_bool("BustSettingChange") { return BasicMenuResult::new();}
    YesNoDialog::bind::<BustConfirm>(this.menu, "Change value?", "Do it!", "Nah..");
    BasicMenuResult::new()
}

pub extern "C" fn vibe_bust() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_gauge::<BustGauge>("Unit Bust Volume Slider");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = bust_setting_acall as _ );
    switch
}
