use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    gamedata::*,
};
use super::CONFIG;

pub fn randomize_person_grow(){
    let mut max: [u8; 11] = [0; 11];
    let mut min: [u8; 11] = [100; 11];
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
        }
    }
    for y in 0..11 {
        max[y as usize] = max[y as usize] / 5;
        min[y as usize] = min[y as usize] / 5;
        println!("Person Stat {}: min {}, max {}", y, min[y as usize], max[y as usize]);
    }
    let rng = Random::instantiate().unwrap();
    let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, max[y as usize] as i32) as u8;
            grow[y as usize] = v*5;
        }
    }
}

pub fn randomize_job_grow(){
    let mut max: [i8; 11] = [0; 11];
    let mut min: [i8; 11] = [100; 11];
    let job_list = JobData::get_list_mut().unwrap();

    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
        }
    }
    for y in 0..11 {
        max[y as usize] = max[y as usize] / 5;
        min[y as usize] = min[y as usize] / 5;
    }
    let rng = Random::instantiate().unwrap();
    let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, (max[y as usize]) as i32) as i8;
            grow[y as usize] = v*5;
        }
    }
}

pub fn random_grow(){
    if GameVariableManager::get_number("G_Random_Seed") == 0 { return; }
    let grow_mode = GameVariableManager::get_number("G_Random_Grow_Mode");
    if grow_mode == 3 {
        randomize_person_grow();
        randomize_job_grow();
    }
    else if grow_mode == 2 { randomize_job_grow(); }
    else if grow_mode == 1 { randomize_person_grow(); }
    else {
        return;
    }
}

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
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().random_grow {
            1 => { this.help_text = "Personal growths rate will be randomized.".into(); },
            2 => { this.help_text = "Class growth rates modifiers will be randomized.".into(); },
            3 => { this.help_text = "Personal and class growth rates modifiers will be randomized.".into(); },
            _ => { this.help_text = "No changes made to growth rates.".into(); },
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().random_grow {
            1 => { this.command_text = "Personal".into(); },
            2 => { this.command_text = "Class".into(); },
            3 => { this.command_text = "Personal + Class".into(); },
            _ => { this.command_text = "No Randomization".into(); },
        }
    }
}

#[no_mangle]
extern "C" fn grow_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomGrowMod>("Random Growth Mode") } 
pub fn install_rng_grow() { cobapi::install_global_game_setting(*&grow_rnd ); }
