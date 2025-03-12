use super::*;
pub struct RandomSkillMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSkillMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_skill);
        if CONFIG.lock().unwrap().random_skill != result {
            CONFIG.lock().unwrap().random_skill  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_skill {  "Personals and class skills are randomized." }
            else { "No changes to personal and class skills." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_skill { "Randomize" }  else { "Default" }.into();
    }
}
pub struct RandomSkillCost;
impl ConfigBasicMenuItemSwitchMethods for RandomSkillCost {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry_norewind("RSkC",  GameVariableManager::get_number(DVCVariables::SP_KEY));
            GameVariableManager::set_number("RSkC",  GameVariableManager::get_number(DVCVariables::SP_KEY));
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_skill_cost }
            else { GameVariableManager::get_number("RSkC") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_skill_cost = result; }
            else { GameVariableManager::set_number("RSkC", result) };
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = 
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_skill_cost }
            else { GameVariableManager::get_number("RSkC") };
        let changed = DVCVariables::changed_setting_text("RSkC", DVCVariables::SP_KEY);
        this.help_text = format!("{}{}", match value  {
            1 => { "SP cost for skills will be randomized." },
            2 => { "All possible skills can be inherited with random cost."}
            _ => { "Default SP cost for inheritance." }
        }, changed).into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_skill_cost }
            else { GameVariableManager::get_number("RSkC") };

        let changed = if DVCVariables::is_main_menu() { "" }
            else if GameVariableManager::get_number("RSkC") != GameVariableManager::get_number(DVCVariables::SP_KEY) { "*"}
            else { "" };

        this.command_text = format!("{}{}", changed, match value  {
            1 => { "Random Cost"},
            2 => { "Chaos" },
            _ => { "Default" }
        }).into();
    }
}

pub struct SkillCostConfirm;
impl TwoChoiceDialogMethods for SkillCostConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::SP_KEY, GameVariableManager::get_number("RSkC"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        RandomSkillCost::set_help_text(menu.menu_item_list[index as usize], None);
        RandomSkillCost::set_command_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn spc_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_number("RSkC") == GameVariableManager::get_number(DVCVariables::SP_KEY) { return BasicMenuResult::new();}
    YesNoDialog::bind::<SkillCostConfirm>(this.menu, "Change Randomization Setting?\nMust save and reload to take effect.", "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub fn spc_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if can_rand() { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_rand_spc() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<RandomSkillCost>("Skill Inheritance SP Cost");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = spc_acall as _ );
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = spc_build_attr as _ );
    switch
}

pub struct EnemySkillGauge;
impl ConfigBasicMenuItemGaugeMethods for EnemySkillGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_enemy_skill_rate as f32 / 100.0 }
            else {GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GUAGE_KEY) as f32 / 100.0 }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        if is_main && !CONFIG.lock().unwrap().random_skill {
            this.help_text = "Enable skill randomization to enable this setting.".into();
            this.update_text();
            return BasicMenuResult::new();
        }
        let value = if DVCVariables::is_main_menu()  { CONFIG.lock().unwrap().random_enemy_skill_rate } 
            else { GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GUAGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 100, 10);
        if value != result {
            if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().random_enemy_skill_rate = result; } 
            else {  GameVariableManager::set_number(DVCVariables::ENEMY_SKILL_GUAGE_KEY, result); }
            this.gauge_ratio = result as f32 * 0.01;
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ 
        let is_main = DVCVariables::is_main_menu();
        if is_main && !CONFIG.lock().unwrap().random_skill {
            this.help_text = "Enable skill randomization to enable this setting.".into();
            return;
        }
        let gauge = if is_main {  CONFIG.lock().unwrap().random_enemy_skill_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GUAGE_KEY) };

        if gauge == 10 { this.help_text = "Only bosses will gain a random skill".into(); }
        else {this.help_text = format!("{}% chance of enemy units will gain a random skill.", gauge).into(); }
    }
}
pub extern "C" fn vibe_skill_gauge() -> &'static mut ConfigBasicMenuItem {  
    let skill_gauge = ConfigBasicMenuItem::new_gauge::<EnemySkillGauge>("Random Enemy Skill Rate");
    skill_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_skill_gauge as _);
    skill_gauge
}
