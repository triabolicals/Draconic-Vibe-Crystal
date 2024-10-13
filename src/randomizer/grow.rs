use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::*,
};
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

pub fn randomize_person_grow(){
    let mut max: [u8; 11] = [0; 11];
    let mut min: [u8; 11] = [100; 11];
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; }
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize ] = grow[y as usize]; }
        }
    }
    for y in 0..11 { 
        max[y as usize] = max[y as usize] / 5;
        min[y as usize] = min[y as usize] / 5;
    }
    let rng = crate::utils::get_rng();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; } 
        let mut total = 0;
        while total < 150 || total > 400 {
            total = 0;
            for y in 0..9 {
                let v = rng.get_min_max(min[y as usize] as i32 , max[y as usize] as i32) as u8;
                total += 5*(v as i32);
                grow[y as usize] = v*5;
            }
        }
    }
}

pub fn randomize_job_grow(){
    let mut max: [i8; 11] = [0; 11];
    let mut min: [i8; 11] = [100; 11];
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize ] = grow[y as usize]; }
        }
    }
    for y in 0..11 { 
        max[y as usize] = max[y as usize] / 5; 
        min[y as usize] = min[y as usize] / 5; 
    }
    let rng = crate::utils::get_rng();
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..9 {
            let v = rng.get_min_max(min[y as usize] as i32, (max[y as usize]) as i32) as i8;
            grow[y as usize] = v*5;
        }
    }
}

pub fn random_grow(){
    if !crate::utils::can_rand() { return; }
    match GameVariableManager::get_number("G_Random_Grow_Mode") {
        1 => { randomize_person_grow(); },
        2 => { randomize_job_grow(); }, 
        3 => { 
            randomize_person_grow();
            randomize_job_grow(); 
        },
        _ => {},
    }
}