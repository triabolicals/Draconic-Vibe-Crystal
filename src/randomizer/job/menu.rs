use super::*;
pub struct RandomJobMod;
impl ConfigBasicMenuItemSwitchMethods for RandomJobMod {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_job();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_random_job(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match DVCVariables::get_random_job() {
            1 => {  "Player" },
            2 => { "Enemy/NPC" },
            3 => { "All Units" },
            _ => { "None"},
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match DVCVariables::get_random_job() {
            1 => { "Playable units will be in random classes." },
            2 => { "Enemy/NPC units will be in random classes."},
            3 => { "All units will be in random classes."},
            _ => { "Units will be in their assigned class"},
        }.into();
    }
}
pub struct CustomJobs;
impl ConfigBasicMenuItemSwitchMethods for CustomJobs {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_custom_jobs(false);
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_custom_jobs(result, false);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if DVCVariables::get_custom_jobs(false) { "Include"} else { "Exclude"}.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if DVCVariables::get_custom_jobs(false) { "Allows customs classes in the reclass/randomization pool." }
        else { "No custom classes in the reclass/randomization pool" }.into();
    }
    extern "C" fn build_attributes(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        if JobData::get_count() <= 111 { BasicMenuItemAttribute::Hide }
        else if DVCVariables::is_main_menu() { BasicMenuItemAttribute::Enable }
        else if GameVariableManager::get_number(DVCVariables::JOB_KEY) > 0 { BasicMenuItemAttribute::Enable }
        else { BasicMenuItemAttribute::Hide }
    }
}

pub extern "C" fn vibe_custom_job() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<CustomJobs>("Custom Classes") }

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
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let gauge = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().random_enemy_job_rate } else { GameVariableManager::get_number(DVCVariables::ENEMY_JOB_GAUGE_KEY) };
        if gauge == 0 {  this.help_text = "Enemy units will not be in a random class.".into();}
        else if gauge == 10 { this.help_text = "Only bosses will be in a random class (if possible).".into(); }
        else { this.help_text = format!("{}% chance of enemy units will be in a random class (if possible).", gauge).into(); }
    }
   extern "C" fn build_attributes(this: &mut ConfigBasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute {
        crate::menus::buildattr::job_gauge_build_attr(this, method_info)
   }
}

pub extern "C" fn vibe_job_gauge() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_gauge::<EnemyJobGauge>("Random Enemy Class Rate") }

pub struct RandomCC;
impl ConfigBasicMenuItemSwitchMethods for RandomCC {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let previous = DVCVariables::get_random_reclass();
        let result = ConfigBasicMenuItem::change_key_value_i(previous, 0, 2, 1);
        if previous != result {
            DVCVariables::set_random_reclass(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =
            match DVCVariables::get_random_reclass() {
                1 => "Random",
                2 => "None",
                _ => "Default"
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match DVCVariables::get_random_reclass() {
            1 => "When reclassing, the new class will be determined randomly.",
            2 => "Units can only change class within their current class lines.",
            _ =>  "Default reclassing behavior.",
        }.into();
    }
}

pub struct RerandomizeJobs;
impl ConfigBasicMenuItemCommandMethods for  RerandomizeJobs {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        BasicMenuResult::new()
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "Randomize".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = if !DVCVariables::is_main_chapter_complete(3) { "Re-randomize all player units' classes." }
            else { "Re-randomize enemy / ally units' classes."}.into(); 
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        YesNoDialog::bind::<RerandomizeJobsConfirm>(this.menu, "Randomize Classes?\nItems will be replaced.", "Do it!", "Nah..");
        BasicMenuResult::se_cursor()
    }
    extern "C" fn build_attributes(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 == 0 || DVCVariables::get_single_class(false).is_some() { return  BasicMenuItemAttribute::Hide; }
        if DVCVariables::is_main_chapter_complete(3) && GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Enable }
        else if GameUserData::get_sequence() == 3 && GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 { BasicMenuItemAttribute::Enable }
        else {  BasicMenuItemAttribute::Hide  }
    }
}

pub struct RerandomizeJobsConfirm;
impl TwoChoiceDialogMethods for RerandomizeJobsConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        rerandomize_jobs();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub extern "C" fn vibe_job_rerand() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_command::<RerandomizeJobs>("Re-Randomize Classes") }