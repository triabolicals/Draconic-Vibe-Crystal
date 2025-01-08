use unity::prelude::*;
use engage::{
    gamedata::unit::*,
    random::*,
    force::*,
    dialog::yesno::*, force::ForceType, gamedata::*, gamevariable::*, menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuItemAttribute, BasicMenuResult}
};
pub static mut GROW_RANGE: [i32; 41] = [0; 41];
use concat_string::concat_string;

use crate::CONFIG;

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
        GameVariableManager::make_entry("G_PGMode", 0);
        GameVariableManager::make_entry("PGMode", GameVariableManager::get_number("G_PGMode") );
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let mode = GameVariableManager::get_number("PGMode");
        let result = ConfigBasicMenuItem::change_key_value_i(mode, 0, 2, 1);
        if mode != result {
            GameVariableManager::set_number("PGMode", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let current_mode = GameVariableManager::get_number("G_PGMode");
        let selection = GameVariableManager::get_number("PGMode");
        let description = match selection {
            1 => { "Growth rates are not restrictive by growth total."},
            2 => { "Growth rates are influenced by starting class."},
            _ => { "Growth rates are restrictive by growth total." },
        };
        this.help_text =
            if current_mode != selection { concat_string!(description, " (Press A to Change)") }
            else { description.to_string() }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =  match GameVariableManager::get_number("PGMode") {
            1 => { "Unrestrict" },
            2 => { "Adaptive" },
            _ => { "Balanced" },
        }.into();
    }
}

pub struct PersonalGrowConfirm;
impl TwoChoiceDialogMethods for PersonalGrowConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("G_PGMode", GameVariableManager::get_number("PGMode"));
        randomize_person_grow();
        unsafe { 
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent);
            let index = menu.select_index;
            PersonalGrowMode::set_help_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}


pub fn growth_mode_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let current_mode = GameVariableManager::get_number("G_PGMode");
    let selection = GameVariableManager::get_number("PGMode");
    if current_mode != selection {
        YesNoDialog::bind::<PersonalGrowConfirm>(this.menu, "Change Personal Growth Mode?", "Do it!", "Nah..");
    }
    return BasicMenuResult::new();
}

fn growth_mode_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameVariableManager::get_number("G_Random_Grow_Mode") & 1 == 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_pgmode() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<PersonalGrowMode>("Personal Growth Mode");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = growth_mode_setting_acall as _ );
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = growth_mode_build_attr as _ );
    switch
}


pub fn get_growth_min_max() {
    let grow_range = unsafe { &mut GROW_RANGE };
    if grow_range[40] != 0 { return; }  // already set
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
}

pub fn randomize_person_grow(){
    println!("Randomization Personal Growth Rates");
    let person_list = PersonData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();

    let mode = GameVariableManager::get_number("G_PGMode");
    let grow_range = unsafe { &GROW_RANGE };
    let limits: (i32, i32) =
        if mode == 0 { (150, 400) }
        else { (0, 2000) };

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
    if mode == 2 { player_pool_adaptive_growths(); }
}

pub fn adaptive_growths(unit: &Unit) {
    if GameVariableManager::get_number("G_PGMode") != 2 { return; }
    let seed = GameVariableManager::get_number("G_Random_Seed");
    let grow_range = unsafe { &GROW_RANGE };

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
    if GameVariableManager::get_number("G_PGMode") != 2 { return; }
    let force_types = [ForceType::Player, ForceType::Enemy, ForceType::Ally, ForceType::Absent, ForceType::Dead];
    for ff in force_types {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter { adaptive_growths(unit); }
    }
}

pub fn randomize_job_grow(){
    println!("Randomization Class Modifier Growth Rates");
    let grow_range = unsafe { &mut GROW_RANGE };
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
    match GameVariableManager::get_number("G_Random_Grow_Mode") {
        1 => { randomize_person_grow();  },
        2 => { randomize_job_grow(); }, 
        3 => { 
            randomize_person_grow();
            randomize_job_grow(); 
        },
        _ => {},
    }
}