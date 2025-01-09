use unity::prelude::*;
use engage::{
    dialog::yesno::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::*,
    gameuserdata::GameUserData,
};
use std::sync::Mutex;
use super::CONFIG;
pub static BATTLE_STYLES_DEFAULT: Mutex<[i32; 256]> = Mutex::new([0; 256]); 

pub struct RandomBattleStyles;
impl ConfigBasicMenuItemSwitchMethods for RandomBattleStyles {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("BattleStyles", GameVariableManager::get_number("G_BattleStyles") );
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_number("BattleStyles") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles = result; }
            else { GameVariableManager::set_number("BattleStyles", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().random_battle_styles } else { GameVariableManager::get_number("BattleStyles") };
        let string1 = match value {
            1 => { "Class types will be randomized." },
            2 => { "Classes will have no special types."},
            _ => { "Classes will have their default type."},
        }.to_string();
        if GameVariableManager::get_number("G_BattleStyles") != GameVariableManager::get_number("BattleStyles") {
            this.help_text = format!("{} (Press A to change)", string1).into();
        }
        else { this.help_text = string1.into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_number("BattleStyles") };
        this.command_text = match value {
            1 => { "Random" },
            2 => { "No Types"},
            _ => { "Default"},
        }.into();
    }
}


pub fn battle_style_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("G_BattleStyles") == GameVariableManager::get_number("BattleStyles") { return BasicMenuResult::new();}
    if GameVariableManager::get_number("BattleStyles") == 1 && !crate::utils::can_rand()  { return BasicMenuResult::new();}
    let text = format!("Change Class Type Setting:\nFrom '{}' to '{}'?",
        style_setting_text( GameVariableManager::get_number("G_BattleStyles")), 
        style_setting_text( GameVariableManager::get_number("BattleStyles")), 
    );
    YesNoDialog::bind::<BattleStyleConfirm>(this.menu, text, "Do it!", "Nah..");
    return BasicMenuResult::new();
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
        GameVariableManager::set_number("G_BattleStyles", GameVariableManager::get_number("BattleStyles"));
        randomize_job_styles();
        unsafe { 
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap());
            let index = menu.select_index;
            RandomBattleStyles::set_help_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub extern "C" fn vibe_styles() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_switch::<RandomBattleStyles>("Random Class Types");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_normal as _);
    item_gauge.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = battle_style_setting_acall as _ );
    item_gauge
}

pub fn randomize_job_styles(){
    if !crate::utils::can_rand() { return; }
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    match GameVariableManager::get_number("G_BattleStyles") {
        1 => {  // Random
            println!("Randomizing Battle Styles");
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
            println!("Battle Styles set to default");
            job_list.iter_mut()
                .for_each(|job|{
                    if job.parent.index > 0 {
                        let index = BATTLE_STYLES_DEFAULT.lock().unwrap()[job.parent.index as usize];
                        job.style_name = crate::enums::STYLE_NAMES[index as usize].into();
                        job.on_completed();
                    }
                }
            );
        },
        _ => {},
    }
}