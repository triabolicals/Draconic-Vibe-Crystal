use unity::prelude::*;
use engage::{
    dialog::yesno::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::*,
};
use std::sync::OnceLock;
use super::DVCVariables;
pub static BATTLE_STYLES_DEFAULT: OnceLock<Vec<i32>> = OnceLock::new();

pub struct RandomBattleStyles;
impl ConfigBasicMenuItemSwitchMethods for RandomBattleStyles {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() { DVCVariables::set_temp(DVCVariables::STYLES_KEY); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_battle_styles();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            DVCVariables::set_random_battle_styles(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_battle_styles();
        this.is_command_icon = DVCVariables::is_temp_change(DVCVariables::STYLES_KEY);
        this.command_text = match value {
            1 => { "Random" },
            2 => { "No Types"},
            _ => { "Default"},
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_battle_styles();
        this.help_text = match value {
            1 => { "Class types will be randomized." },
            2 => { "Classes will have no special types." },
            _ => { "Classes will have their default type." },
        }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_main_menu() {return BasicMenuResult::new(); }
        if DVCVariables::is_temp_change(DVCVariables::STYLES_KEY) {
            let text =
                format!("Change Class Type Setting:\nFrom '{}' to '{}'?",
                        style_setting_text( GameVariableManager::get_number(DVCVariables::STYLES_KEY)),
                        style_setting_text( DVCVariables::get_temp_var(DVCVariables::STYLES_KEY))
                );
            YesNoDialog::bind::<BattleStyleConfirm>(this.menu, text, "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
}

fn style_setting_text(choice: i32) -> String {
    match choice {
        1 => {"Randomized"},
        2 => {"No Types"},
        _ => { "Default"},
    }.to_string()
}

pub struct BattleStyleConfirm;
impl TwoChoiceDialogMethods for BattleStyleConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        DVCVariables::update_var_from_temp(DVCVariables::STYLES_KEY);
        randomize_job_styles();
        crate::menus::utils::dialog_restore_text::<RandomBattleStyles>(this, false);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub extern "C" fn vibe_styles() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_switch::<RandomBattleStyles>("Random Class Types");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = crate::menus::buildattr::build_attribute_normal as _);
    item_gauge
}

pub fn randomize_job_styles(){
    if !DVCVariables::random_enabled() { return; }
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    match GameVariableManager::get_number(DVCVariables::STYLES_KEY) {
        1 => {  // Random
            job_list.iter_mut()
                .for_each(|job|{
                    if job.parent.index > 0 {
                        let style = crate::enums::STYLE_NAMES[ rng.get_value(8) as usize ];
                        job.style_name = style.into();
                        job.on_completed();
                    }
                }
            );
        },
        2 => {  // None
            job_list.iter_mut()
                .for_each(|job|{
                    if job.parent.index > 0 {
                        job.style_name = "スタイル無し".into();
                        job.on_completed();
                    }
                }
            );
        },
        0 => {  //Default
            let styles = BATTLE_STYLES_DEFAULT.get().unwrap();
            job_list.iter_mut()
                .for_each(|job|{
                    if job.parent.index > 0 {
                        let index = styles[job.parent.index as usize];
                        job.style_name = crate::enums::STYLE_NAMES[index as usize].into();
                        job.on_completed();
                    }
                }
            );
        },
        _ => {},
    }
}