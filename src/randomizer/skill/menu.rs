use engage::pad::Pad;
use engage::util::get_instance;
use super::*;

pub struct RandomSkillMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSkillMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() { DVCVariables::set_temp(DVCVariables::SKILL_KEY); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_skill();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_random_skill(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_skill();
        this.is_command_icon = DVCVariables::is_temp_change(DVCVariables::SKILL_KEY);
        this.command_text =
            match value {
                1 => "Personal",
                2 => "Class",
                3 => "Personal + Class",
                _ => "Default"
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_skill();
        this.help_text =
            match value {
                1 => "Personal skills are randomized",
                2 => "Class learn skills are randomized.",
                3 => "Personal/Learn skills are randomized.",
                _ => "No changes to personal and class skills.",
            }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::can_update_var(DVCVariables::SKILL_KEY){
            YesNoDialog::bind::<SkillChangeConfirm>(this.menu, "Change Skill Setting?\nRequires Save and Reload.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else if !DVCVariables::is_main_menu() { BasicMenuResult::se_miss() }
        else { BasicMenuResult::new() }
    }
}
crate::random_confirm!(SKILL_KEY, Skill);
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
            BasicMenuResult::se_cursor()
        } else {BasicMenuResult::new() }
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
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if GameUserData::get_sequence() == 0 { return BasicMenuResult::new(); }
        if GameVariableManager::get_number("RSkC") == GameVariableManager::get_number(DVCVariables::SP_KEY) { return BasicMenuResult::new();}
        YesNoDialog::bind::<SkillCostConfirm>(this.menu, "Change SP Setting?\nMust save and reload to take effect.", "Do it!", "Nah..");
        BasicMenuResult::new()
    }
}

pub struct SkillCostConfirm;
impl TwoChoiceDialogMethods for SkillCostConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::SP_KEY, GameVariableManager::get_number("RSkC"));
        crate::menus::utils::dialog_restore_text::<RandomSkillCost>(this, false);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}
pub fn spc_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if can_rand() { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_rand_spc() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<RandomSkillCost>("Skill Inheritance SP Cost");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = spc_build_attr as _ );
    switch
}

pub struct EnemySkillGauge;
impl ConfigBasicMenuItemGaugeMethods for EnemySkillGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_enemy_skill_rate as f32 / 100.0 }
            else {GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY) as f32 / 100.0 }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        let value = if is_main { CONFIG.lock().unwrap().random_enemy_skill_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 100, 10);
        if value != result {
            if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().random_enemy_skill_rate = result; } 
            else {  GameVariableManager::set_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY, result); }
            this.gauge_ratio = result as f32 * 0.01;
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ 
        let is_main = DVCVariables::is_main_menu();
        let gauge = if is_main {  CONFIG.lock().unwrap().random_enemy_skill_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_SKILL_GAUGE_KEY) };
        this.help_text =
            if gauge == 0 { "Enemy units will not gain a random skill.".into() }
            else if gauge == 10 { "Only bosses will gain a random skill".into() }
            else { format!("{}% chance of enemy units will gain a random skill.", gauge).into() };
    }
}
pub extern "C" fn vibe_skill_gauge() -> &'static mut ConfigBasicMenuItem {  
    let skill_gauge = ConfigBasicMenuItem::new_gauge::<EnemySkillGauge>("Random Enemy Skill Rate");
    skill_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = crate::menus::buildattr::skill_gauge_build_attr as _);
    skill_gauge
}

