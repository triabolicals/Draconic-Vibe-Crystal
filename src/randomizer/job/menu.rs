use super::*;

pub struct RandomJobMod;
impl ConfigBasicMenuItemSwitchMethods for RandomJobMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_job, 0, 3, 1);
        if CONFIG.lock().unwrap().random_job != result {
            CONFIG.lock().unwrap().random_job  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_job {
            1 => { "Playable units will be in random classes." },
            2 => { "Enemy/NPC units will be in random classes."},
            3 => { "All units will be in random classes."},
            _ => { "Units will be in their assigned class"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match  CONFIG.lock().unwrap().random_job {
            1 => {  "Player" },
            2 => { "Enemy/NPC" },
            3 => { "All Units" },
            _ => { "None"},
        }.into();
    }
}
pub struct CustomJobs;
impl ConfigBasicMenuItemSwitchMethods for CustomJobs {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().get_custom_jobs();
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().set_custom_jobs(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().get_custom_jobs() { "Allows customs classes in the randomization pool." }
        else { "Only vanilla classes in the randomization pool" }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().get_custom_jobs() { "Include"} else { "Exclude"}.into();
    }
}

fn custom_job_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if JobData::get_count() <= 111 { BasicMenuItemAttribute::Hide }
    else if DVCVariables::is_main_menu() { BasicMenuItemAttribute::Enable }
    else if GameVariableManager::get_number(DVCVariables::JOB_KEY) > 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_custom_job() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<CustomJobs>("Custom Classes"); 
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = custom_job_build_attr as _);
    switch
} 

pub struct EnemyJobGauge;
impl ConfigBasicMenuItemGaugeMethods for EnemyJobGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_enemy_job_rate as f32 / 100.0 }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_JOB_GAUGE_KEY) as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        if is_main && CONFIG.lock().unwrap().random_job & 2 == 0 {
            this.help_text = "Enable enemy class randomization to enable this setting.".into();
            this.update_text();
            return BasicMenuResult::new();
        }
        let value = if is_main { CONFIG.lock().unwrap().random_enemy_job_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_JOB_GAUGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 100, 10);
        if value != result {
            if is_main { CONFIG.lock().unwrap().random_enemy_job_rate = result; }
            else { GameVariableManager::set_number(DVCVariables::ENEMY_JOB_GAUGE_KEY, result); }
            this.gauge_ratio = result as f32 * 0.01;
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let gauge = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_enemy_job_rate } else { GameVariableManager::get_number(DVCVariables::ENEMY_JOB_GAUGE_KEY) };
        if gauge == 0 {  this.help_text = "Enemy units will not be in a random class.".into();}
        else if gauge == 10 { this.help_text = "Only bosses will be in a random class (if possible).".into(); }
        else { this.help_text = format!("{}% chance of enemy units will be in a random class (if possible).", gauge).into(); }
    }
}

pub extern "C" fn vibe_job_gauge() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_gauge::<EnemyJobGauge>("Random Enemy Class Rate"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::job_gauge_build_attr as _);
    class_gauge
}

pub struct RandomCC;
impl ConfigBasicMenuItemSwitchMethods for RandomCC {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let previous = CONFIG.lock().unwrap().get_random_cc();
        let result = ConfigBasicMenuItem::change_key_value_b(previous);
        if previous != result {
            CONFIG.lock().unwrap().set_random_cc(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().get_random_cc() { "When reclassing, the new class will be determined randomly." }
            else { "Default reclassing behavior."}.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().get_random_cc() { "Random" } else { "Default" }.into();
    }
}

pub struct RerandomizeJobs;
impl ConfigBasicMenuItemCommandMethods for  RerandomizeJobs {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.a() {
            YesNoDialog::bind::<RerandomizeJobsConfirm>(this.menu, "Randomize Classes for Ally Units?\nItems will be replaced.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "Randomize".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = if !DVCVariables::is_main_chapter_complete(3) { "Re-randomize all player units' classes." }
            else { "Re-randomize allies/unrecruited player units' classes."}.into(); 
    }
}

pub struct RerandomizeJobsConfirm;
impl TwoChoiceDialogMethods for RerandomizeJobsConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        rerandomize_jobs();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn re_randomize_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 == 0 { return  BasicMenuItemAttribute::Hide; }
    if DVCVariables::is_main_chapter_complete(3) {
        let count = if Force::get(ForceType::Ally).is_some() { Force::get(ForceType::Ally).unwrap().get_count() } else { 0 };
        if GameUserData::get_chapter().cid.contains("M018") && GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Enable }
        else if GameUserData::get_sequence() == 2 && count > 0 { BasicMenuItemAttribute::Enable }
        else {  BasicMenuItemAttribute::Hide  }
    }
    else if GameUserData::get_sequence() == 3 && GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 { BasicMenuItemAttribute::Enable }
    else {  BasicMenuItemAttribute::Hide  }
}

pub extern "C" fn vibe_job_rerand() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_command::<RerandomizeJobs>("Re-Randomize Classes"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = re_randomize_build_attr as _);
    class_gauge
}