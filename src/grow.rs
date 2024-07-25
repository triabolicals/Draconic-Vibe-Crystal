use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::{unit::*, *},
    gameuserdata::GameUserData,
};
use super::CONFIG;
pub fn randomize_person_grow(){
    let mut max: [u8; 11] = [0; 11];
    let mut min: [u8; 11] = [100; 11];
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 0..person_list.len() {
        let grow = person_list[x].get_grow();
        if grow.is_zero() { continue; }
        for y in 0..11 {
            if grow[y as usize] > max[y as usize] { max[ y as usize ] = grow[y as usize]; }
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
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
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, max[y as usize] as i32) as u8;
            if person_list[x].get_asset_force() != 0 {
                grow[y as usize] = (v+2)*5;
            }
            else { grow[y as usize] = v*5; }
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
            if grow[y as usize] < min[y as usize] { min[ y as usize] = grow[y as usize]; }
        }
    }
    for y in 0..11 {
        max[y as usize] = max[y as usize] / 5 + 1;
        min[y as usize] = min[y as usize] / 5;
    }
    let rng = crate::utils::get_rng();
    for x in 0..job_list.len() {
        let grow = job_list[x].get_diff_grow();
        if grow.is_zero() { continue; } 
        for y in 0..11 {
            let v = rng.get_min_max(min[y as usize] as i32, (max[y as usize]) as i32) as i8;
            grow[y as usize] = (v+1)*5;
        }
    }
}
pub fn randomize_job_styles(){
    if !crate::utils::can_rand() { return; }
    if !GameVariableManager::get_bool("G_BattleStyles") { return; }
    let rng = crate::utils::get_rng();
    let job_list = JobData::get_list_mut().unwrap();
    let style_names = ["連携スタイル", "騎馬スタイル", "隠密スタイル", "重装スタイル",  "飛行スタイル", "魔法スタイル", "気功スタイル", "竜族スタイル"];
    for x in 1..job_list.len() {
        let style = style_names[ rng.get_value(8) as usize ];
        job_list[x].style_name = style.into();
        job_list[x].on_completed();
    }
}

pub fn random_grow(){
    if GameVariableManager::get_number("G_Random_Seed") == 0 { return; }
    match GameVariableManager::get_number("G_Random_Grow_Mode") {
        1 => { randomize_person_grow(); },
        2 => { randomize_job_grow(); }, 
        3 => { 
            randomize_person_grow();
            randomize_job_grow(); },
        _ => {},
    }
}

pub fn change_unit_autolevel(unit: &mut Unit, reverse: bool) {
    let person = if reverse { crate::person::switch_person_reverse(unit.person) } else { &unit.person };
    let new_person = if reverse { &unit.person } else { crate::person::switch_person(unit.person) }; 
    println!("{} -> {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap());
    let is_low = person.get_job().unwrap().is_low();
    let is_new_low = new_person.get_job().unwrap().is_low();
    let current_level = person.get_level() as i32;
    let mut current_internal_level = person.get_internal_level() as i32;
    if current_internal_level == 0 && !is_low { current_internal_level = 20; }
    let mut original_growth_rates: [u8; 11] = [0; 11];  // storing growth rates of the original person
    let original_gr = person.get_grow();    // growth rate of the original person
    let new_gr = new_person.get_grow(); // growth rate of the new person
        // Switch Growths rates to calculate stats, store the previous person's growths to restore it at the end
    for x in 0..11 { 
        original_growth_rates[x as usize] = original_gr[x as usize];
        original_gr[x as usize] = new_gr[x as usize];  
    }
    if is_low {
       if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
            if is_new_low && new_person.get_job().unwrap().has_high() {    // new unpromoted unit can promoted 
                let level = current_level - 20;
                let new_job = &new_person.get_job().unwrap().get_high_jobs()[0];
                unit.auto_grow_capability( level, current_level);
                unit.class_change(new_job);
                unit.set_level( level );
                unit.set_internal_level( 20 );
            }
            else if is_new_low && !new_person.get_job().unwrap().has_high() {   // special -> special
                unit.class_change(new_person.get_job().unwrap());
                unit.auto_grow_capability( current_level, current_level);
                unit.set_level( current_level );
                unit.set_internal_level( 0 );
            }
            else {  // special -> high
                unit.class_change(new_person.get_job().unwrap());
                unit.auto_grow_capability( current_level-20, current_level);
                unit.set_level( current_level - 20 );
                unit.set_internal_level( 20 );
            }
        }
        if is_new_low { // base or special class lvl < 20 -> base class
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability( current_level, current_level);
            unit.set_level( current_level );
            unit.set_internal_level( 0 );
        }
        else {
            let new_job_list = new_person.get_job().unwrap().get_low_jobs();
            unit.auto_grow_capability(current_level, current_level);
            if new_job_list.len() == 3 {
                let index = crate::person::get_low_class_index(new_person);
                unit.class_change(&new_job_list[index as usize]);
            }
            else if new_job_list.len() == 0 { unit.class_change(JobData::get("JID_ソードファイター").unwrap()); }    // if promoted class doesn't have a low class, change to sword fighter
            else {  unit.class_change(&new_job_list[0]); }
            unit.set_level(current_level);
            unit.set_internal_level(0);
        }
    }
    else {  // Promoted
        if is_new_low { // new unit has a base class
            if new_person.get_job().unwrap().has_high() {   // base class -> 1st promoted class
                let new_high_jobs = new_person.get_job().unwrap().get_high_jobs();
                if new_high_jobs.len() == 0 { unit.class_change(JobData::get("JID_ソードマスター").unwrap());  } // if no high class, change to Swordmaster
                else { unit.class_change(&new_high_jobs[0]); }
                unit.auto_grow_capability(current_level, current_level + 20);
                unit.set_level(current_level);
                unit.set_internal_level(current_internal_level);
                println!("Promoted Unit -> Base Unit");
            }
            else { // Promoted -> Special
                if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  { 
                    unit.auto_grow_capability(current_level, current_level + 20);
                    unit.set_level(current_level);
                    unit.set_internal_level( current_internal_level );
                    println!("Promoted Unit -> Special Unit but promoted");
                } 
                else {
                    let total_level = current_internal_level + current_level;
                    unit.class_change(new_person.get_job().unwrap());
                    unit.auto_grow_capability(total_level, 20+current_level);
                    unit.set_level(total_level);
                    unit.set_internal_level(0);
                    unit.set_level( ( person.get_level() + person.get_internal_level() as u8 ).into() );
                    println!("Promoted Unit -> Special Unit");
                }
            }
        }
        else {  // Promoted -> Promoted
            unit.class_change(new_person.get_job().unwrap());
            unit.auto_grow_capability(current_level, current_level + 20);
            unit.set_level(current_level);
            unit.set_internal_level( current_internal_level );
        }
    }
    for x in 0..11 { original_gr[x as usize] = original_growth_rates[x as usize]; } // Change back to original growth rate
    unit.set_person(new_person);    // change person
    crate::person::fixed_unit_weapon_mask(unit);   // fixed weapon mask due to class changes
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
pub struct RandomBattleStyles;
impl ConfigBasicMenuItemSwitchMethods for RandomBattleStyles {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_bool("G_BattleStyles") };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles = result; }
            else { GameVariableManager::set_bool("G_BattleStyles", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_bool("G_BattleStyles") };
        this.help_text = if value { "Class types will be randomized. (Reload to take in effect)" }
                         else { "Classes will have their default type. (Reload to take in effect)"  }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_battle_styles }
                    else { GameVariableManager::get_bool("G_BattleStyles") };
        this.command_text = if value { "Random Types" } else { "Default Types" }.into();
    }
}