use unity::prelude::*;
use engage::{
    dialog::yesno::*, 
    force::{ForceType, *}, 
    gamedata::{unit::*, *}, 
    gamevariable::*, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuItemAttribute, BasicMenuResult}, 
    random::*
};
use std::sync::OnceLock;
use concat_string::concat_string;
pub static GROW_RANGE: OnceLock<[i32; 41]> = OnceLock::new();


use crate::{DVCVariables, CONFIG};

pub struct RandomGrowMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGrowMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_grow, 0, 3, 1);
        if CONFIG.lock().unwrap().random_grow != result {
            CONFIG.lock().unwrap().random_grow = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            CONFIG.lock().unwrap().save();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_grow {
            1 => { "Personal growths rate will be randomized."},
            2 => { "Class growth rates modifiers will be randomized."},
            3 => { "Personal and class growth rates modifiers will be randomized." },
            _ => { "No changes made to growth rates." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =  match CONFIG.lock().unwrap().random_grow {
            1 => { "Personal Growths" },
            2 => { "Class Growths" },
            3 => { "Personal / Class" },
            _ => { "Disable" },
        }.into();
    }
}

pub struct PersonalGrowMode;
impl ConfigBasicMenuItemSwitchMethods for PersonalGrowMode {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry("PGMode", GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY) );
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_main_menu() {
            if CONFIG.lock().unwrap().random_grow & 1 == 0 { 
                this.help_text = "Enable personal growth randomization to enable this setting.".into();
                this.update_text();
                return BasicMenuResult::new(); 
            }
            let mode = CONFIG.lock().unwrap().player_growth;
            let result = ConfigBasicMenuItem::change_key_value_i(mode, 0, 3, 1);
            if mode != result {
                CONFIG.lock().unwrap().player_growth = result;
                Self::set_command_text(this, None);
                Self::set_help_text(this, None);
                this.update_text();
                return BasicMenuResult::se_cursor();
            } else {return BasicMenuResult::new(); }
        }
        else {
            let mode = GameVariableManager::get_number("PGMode");
            let result = ConfigBasicMenuItem::change_key_value_i(mode, 0, 3, 1);
            if mode != result {
                GameVariableManager::set_number("PGMode", result);
                Self::set_command_text(this, None);
                Self::set_help_text(this, None);
                this.update_text();
                return BasicMenuResult::se_cursor();
            } else {return BasicMenuResult::new(); }
        }

    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if DVCVariables::is_main_menu() {
            this.help_text = match CONFIG.lock().unwrap().player_growth {
                1 => { "Growth rates are not restrictive by growth total."},
                2 => { "Growth rates are influenced by starting class and unrestrictive."},
                3 => { "Growth rates are influenced by starting class and restrictive."},
                _ => { "Growth rates are restrictive by growth total." },
            }.into();
        }
        else {
            let current_mode = GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY);
            let selection = GameVariableManager::get_number("PGMode");
            let description = match selection {
                1 => { "Growth rates are not restrictive by growth total."},
                2 => { "Growth rates are influenced by starting class and unrestrictive."},
                3 => { "Growth rates are influenced by starting class and restrictive."},
                _ => { "Growth rates are restrictive by growth total." },
            };
            this.help_text =
                if current_mode != selection { concat_string!(description, " (Press A to Change)") }
                else { description.to_string() }.into();
        }

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().player_growth } else {GameVariableManager::get_number("PGMode")  };
        this.command_text =  match value {
            1 => { "Unrestrict" },
            2 => { "U-Adaptive" },
            3 => { "R-Adaptive" },
            _ => { "Balanced" },
        }.into();
    }
}

pub struct PersonalGrowConfirm;
impl TwoChoiceDialogMethods for PersonalGrowConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::PERSONAL_GROWTH_KEY, GameVariableManager::get_number("PGMode"));
        randomize_person_grow();
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        PersonalGrowMode::set_help_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}


pub fn growth_mode_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::is_main_menu() {  return BasicMenuResult::new();  }
    let current_mode = GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY);
    let selection = GameVariableManager::get_number("PGMode");
    if current_mode != selection { YesNoDialog::bind::<PersonalGrowConfirm>(this.menu, "Change Personal Growth Mode?", "Do it!", "Nah..");  }
    return BasicMenuResult::new();
}

fn growth_mode_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameVariableManager::get_number(DVCVariables::GROWTH_KEY) & 1 == 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_pgmode() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<PersonalGrowMode>("Personal Growth Mode");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = growth_mode_setting_acall as _ );
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = growth_mode_build_attr as _ );
    switch
}


pub fn get_growth_min_max() {
    GROW_RANGE.get_or_init(||{
        let mut grow_range: [i32; 41] = [0; 41];
        for y in 0..9 {
            grow_range[y] = 100; // personal    Min
            grow_range[10+y] = 0;   // Personal Max
            grow_range[20+y] = 100; //job Min
            grow_range[30+y] = 0;   // job Max
        }
        let person_list = PersonData::get_list_mut().unwrap();
        person_list.iter()
            .filter(|person| !person.get_grow().is_zero())
            .for_each(|person|{
                let grow = person.get_grow();
                for y in 0..9 {
                    let v: i32 = grow[y as usize] as i32;
                    if v > grow_range[10 + y as usize] { grow_range[10 + y as usize] = v; }
                    if v < grow_range[y as usize] { grow_range[y as usize] = v; }
                }
            }
        );
        let job_list = JobData::get_list_mut().unwrap();
        job_list.iter()
            .for_each(|job|{
                let grow = job.get_diff_grow();
                if !grow.is_zero() {
                    for y in 0..9 {
                        let v: i32 = grow[y as usize] as i32;
                        if v > grow_range[30 + y as usize] { grow_range[30 + y as usize] = v; }
                        if v < grow_range[20 + y as usize] { grow_range[20 + y as usize ] = v; }
                    }
                }
            }
        );
        for x in 0..40 { grow_range[x] = grow_range[x] / 5; }
        grow_range[40] = 1;
        grow_range
    });
}

pub fn randomize_person_grow(){

    let person_list = PersonData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();

    let mode = GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY);
    println!("Randomization Personal Growth Rates: {}", mode);
    let grow_range = GROW_RANGE.get().unwrap();
    let limits: (i32, i32) = 
        match mode {
            0|3 => { (150, 400) },  // Balanced
            1|2 => { (100, 2000) },   // Unrestricted
            _ => { (150, 400) },
        };

    person_list.iter_mut()
        .for_each(|person|{
            let grow = person.get_grow();
            if !grow.is_zero() {
                let mut total = 0;
                while total < limits.0 || total > limits.1 {
                    total = 0;
                    for y in 0..9 {
                        let mut v = rng.get_min_max(grow_range[y as usize], grow_range[10 + y as usize]) as u8;
                        if v > 24 { v = 24 };
                        total += 5*(v as i32);
                        grow[y as usize] = v*5;
                    }
                }
            }
        }
    );
    if mode & 2 != 0 { player_pool_adaptive_growths(); }
}

pub fn adaptive_growths(unit: &Unit) {
    if GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY) & 2 == 0 { return; }
    let seed = DVCVariables::get_seed();
    let grow_range = GROW_RANGE.get().unwrap();

    if crate::randomizer::person::is_playable_person(unit.person) {
        let key = concat_string!("G_JG_", unit.person.pid.to_string());
        let current_job = unit.get_job();
        if !GameVariableManager::exist(key.as_str()) { 
            GameVariableManager::make_entry(key.as_str(), current_job.parent.hash); 
        }
        else if GameVariableManager::get_number(key.as_str()) == 0 {
            GameVariableManager::set_number(key.as_str(), current_job.parent.hash); 
        }
        if JobData::try_index_get(GameVariableManager::get_number(key.as_str())).is_none() {
            GameVariableManager::set_number(key.as_str(), current_job.parent.hash); 
        }
        let job = JobData::try_get_hash( GameVariableManager::get_number(key.as_str())).unwrap();
        let is_magic = super::job::is_magic_class(job);
        let grow_seed = ( job.parent.hash >> 2 ) + ( unit.person.parent.hash >> 2 ) + ( seed >> 1 );
        let rng = Random::instantiate().unwrap();
        rng.ctor(grow_seed as u32);
        let grow = unit.person.get_grow();
        for y in 0..9 {
            let mut v = rng.get_min_max(grow_range[y as usize], grow_range[10 + y as usize]) as u8;
            if v > 24 { v = 24 };
            grow[y as usize] = v*5;
        }
        let str = grow[1];
        let mag = grow[6];

        if (is_magic && str > mag) || ( !is_magic && mag > str ) {
            grow[1] = mag;
            grow[6] = str;
        }
    }
}

pub fn player_pool_adaptive_growths() {
    if GameVariableManager::get_number(DVCVariables::PERSONAL_GROWTH_KEY) & 2 == 0 { return; }
    let force_types = [ForceType::Player, ForceType::Enemy, ForceType::Ally, ForceType::Absent, ForceType::Dead];
    for ff in force_types {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter { adaptive_growths(unit); }
    }
}

pub fn randomize_job_grow(){
    let grow_range = GROW_RANGE.get().unwrap();
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    job_list.iter_mut()
        .for_each(|job|{
            let grow = job.get_diff_grow();
            if !grow.is_zero() {
                for y in 0..9 {
                    let mut v = rng.get_min_max(grow_range[20 + y as usize] , grow_range[30 + y as usize]) as i8;
                    if v < -10 { v = -10;}
                    else if v > 10 { v = 10; } 
                    grow[y as usize] = v*5;
                }
            }
        }
    );
    job_list.iter_mut()
    .for_each(|job|{
        let grow = job.get_diff_grow();
        if grow.is_zero() {
            for y in 0..9 {
                let mut v = rng.get_min_max(grow_range[20 + y as usize] , grow_range[30 + y as usize]) as i8;
                if v < -10 { v = -10;}
                else if v > 10 { v = 10; } 
                grow[y as usize] = v*5;
            }
        }
    }
);
}


pub fn random_grow(){
    if !crate::utils::can_rand() { return; }
    let growth_mode = GameVariableManager::get_number(DVCVariables::GROWTH_KEY);
    println!("Growth Randomization: {}", growth_mode);
    if growth_mode & 1 != 0 { randomize_person_grow(); }
    if growth_mode & 2 != 0 { randomize_job_grow();  }
}