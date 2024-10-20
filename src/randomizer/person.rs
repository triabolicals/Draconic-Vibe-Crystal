pub use unity::prelude::*;
pub use engage::{
    menu::{
        BasicMenuItemAttribute,
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::*, gameuserdata::*, hub::access::*, random::*, mess::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*, unit::*},
};
use skyline::patching::Patch;
use unity::il2cpp::object::Array;
use crate::{
    enums::*,
    utils::*, autolevel::*,
};
pub mod ai;
pub mod unit; 

use super::CONFIG;
pub static mut SET: i32 = 0;
use std::sync::Mutex;
pub static PLAYABLE: Mutex<Vec<i32>> = Mutex::new(Vec::new());

pub struct RandomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_recruitment, 0, 3, 1);
        if CONFIG.lock().unwrap().random_recruitment != result {
            CONFIG.lock().unwrap().random_recruitment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Characters will be recruited in a random order." }, 
            3 => { "Unit recruitment order is determined by list."},
            2 => { "Characters will be recruited in reversed order."}
            _ => { "Standard recruitment order." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Random"},
            3 => { "Custom"},
            2 => { "Reverse"},
            _ => { "Standard"},
        }.into();
    }
}

// Custom Person Mod
pub struct CustomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for CustomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().custom_units);
        if CONFIG.lock().unwrap().custom_units != result {
            CONFIG.lock().unwrap().custom_units = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().custom_units { "Custom units are include in random recruitment order." }
            else { "Custom units will excluded from random recruitment order." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().custom_units  { "Add" } 
            else { "Default"}.into();
    }
}
fn build_attribute_custom_units(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if PLAYABLE.lock().unwrap().len() == 41 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub extern "C" fn vibe_custom_units() -> &'static mut ConfigBasicMenuItem { 
    let item = ConfigBasicMenuItem::new_switch::<CustomPersonMod>("Custom Units (RR)");
    item.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_custom_units as _);
    item
}
pub fn get_playable_list() {
    if PLAYABLE.lock().unwrap().len() != 0 { return; }
    let mut list = PLAYABLE.lock().unwrap();
    // Add the 41 units first
    for x in PIDS {
        let index = PersonData::get(&x).unwrap().parent.index;
        list.push(index);
    }
    // Add all others that have non zero SP
    let person_list = PersonData::get_list().unwrap(); 
    let mut count = 0;
    for x in 0..person_list.len() { 
        let person = &person_list[x as usize];
        if person.get_sp() == 0 { continue; }
        count += 1;
    }
    if count > 150 { return; }
    for x in 1..person_list.len() { 
        let person = &person_list[x as usize];
        if str_contains(person.pid, "_竜化") { continue; }  //No Dragons
        if person.get_common_sids().is_none() { continue; }
        let index = person.parent.index; 
        if str_contains(person.pid, "PLAYABLE") || str_contains(person.pid, "layable") { person.set_asset_force(0); } 
        if person.get_sp() > 0 && person.get_asset_force() == 0 {
            if person.get_sp() < 300 { person.set_sp(300); }
            if list.iter().find(|r| **r == index).is_none() { 
                list.push(index);
                println!("Person #{}: {} was added", index, Mess::get_name(person.pid).get_string().unwrap());
            }
        }
    }
    println!("Total of {} Playable Units", list.len());
}

fn create_reverse() {
    for x in 0..41 {
        let key = format!("G_R_{}",PIDS[x as usize]);
        let pid = GameVariableManager::get_string(&key).get_string().unwrap();
        for y in 0..41 {
            if pid == PIDS[y as usize] {
                GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[y as usize]), PIDS[x as usize]);
            }
        }
    }
}

fn get_custom_recruitment_list() -> [i32; 41] {
    let mut output: [i32; 41] = [-1; 41];
    let mut set: [bool; 41] = [false; 41];
    let length = crate::enums::SET_RECRUITMENT.lock().unwrap().len();
    for x in 0..length {
        let value = crate::enums::SET_RECRUITMENT.lock().unwrap()[x as usize];
        if value.2 { continue; } // emblem
        let index_1 = value.0;
        let index_2 = value.1;
        if output[index_1 as usize] == -1 && !set[index_2 as usize] { 
            output[index_1 as usize] = index_2; 
            set[index_2 as usize] = true;
        }
    }
    if unsafe { !UNIT_RANDOM } { 
        for x in 0..41 {
            if output[x as usize] != -1 {
                let position = output.iter().find(|y| **y == x);
                if position.is_none() {
                    let mut index = output[ x as usize ];
                    while output[ index as usize] != -1 { index = output[ index as usize]; }
                    output[ index as usize ] = x;
                }
            }
        }
     }
     else {
        println!("Adding randomization to custom unit recruitment");
        let person_pool = if dlc_check() { 41 } else { 36 };
        let rng = get_rng();
        for x in 0..person_pool {
            if output[ x as usize ] != -1 { continue; }
            let mut index;
            loop {
                index = rng.get_value(person_pool); 
                if !set[ index as usize] { break; }
            }
            set[index as usize] = true;
            output[x as usize] = index; 
        }
    }
    output
}

fn set_hub_facilities() {
    let aid = ["AID_蚤の市", "AID_筋肉体操", "AID_ドラゴンライド", "AID_釣り", "AID_占い小屋"];
    let locator = ["LocatorSell01", "LocatorTraining01", "LocatorDragon01", "LocatorFish01", "LocatorFortune01"];
    let index = [ 23, 4, 17, 14, 27];
    let hub_dispos = HubDisposData::get_array_mut().unwrap();
    for x in 0..aid.len() {
        let data = HubFacilityData::get_mut(aid[x as usize]);
        let pid = PIDS[index[x] as usize];
        let a_index = pid_to_index(&pid.to_string(), true) as usize;
        println!("Hub Person: {} -> {}", index[x], a_index);
        if data.is_some() {
            let facility = data.unwrap();
            facility.condition_cid = format!("CID_{}", RECRUIT_CID[ a_index as usize] ).into() ;
            for y in 0..hub_dispos[1].len() {
                let hub_locator = hub_dispos[1][y as usize].get_locator();
                if hub_locator.get_string().unwrap() == locator[ x as usize] {
                    hub_dispos[1][y as usize].set_chapter(RECRUIT_CID[ a_index as usize].into() );
                    break;
                }
            }
        }
    }
}
pub fn randomize_person() {
    if !can_rand() { return; }
    if !GameVariableManager::exist("G_Random_Person_Set") {  GameVariableManager::make_entry("G_Random_Person_Set", 0);  }

    if GameVariableManager::get_bool("G_Random_Person_Set") { 
        if GameVariableManager::get_number("G_Random_Recruitment") != 0 { 
            set_hub_facilities(); 
            change_hub_dispos(false);
        }
        return; 
    }
    else if GameVariableManager::exist("G_R_PID_リュール") && !GameVariableManager::exist("G_R2_PID_リュール") { create_reverse();}
    else {
        for i in 0..41 { 
            GameVariableManager::make_entry_str(&format!("G_R_{}",PIDS[i as usize]), PIDS[i as usize]);
            GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[i as usize]), PIDS[i as usize]);
        }
        let rng = get_rng();
        let mut set_emblems: [bool; 41] = [false; 41];
        match GameVariableManager::get_number("G_Random_Recruitment") {
            1 => {
                let playable_size = if CONFIG.lock().unwrap().custom_units && PLAYABLE.lock().unwrap().len() > 41 { 
                    PLAYABLE.lock().unwrap().len() }
                    else { 41 };
        
                let list = PLAYABLE.lock().unwrap();
                let mut playable_list: Vec<usize> = (0..playable_size).collect();
                let mut to_replace_list: Vec<usize> = (0..playable_size).collect();
                if !dlc_check() { 
                    for x in 36..41 {   // Remove DLC characters in the pool
                        if let Some(index) = playable_list.iter().position(|&i| i == x) {  playable_list.remove(index);  }
                        if let Some(index) = to_replace_list.iter().position(|&i| i == x) {  to_replace_list.remove(index);  }
                    }
                }
                let person_list = PersonData::get_list().unwrap();
                let pids: Vec<String> = list.iter().map(|&x| person_list[x as usize].pid.get_string().unwrap() ).collect();

            // Alear and somniel royals must be switched with non-dlc units
                let royals = [0, 23, 4, 17, 14, 27];
                for x_i in royals {
                    if !playable_list.iter().any(|&i| i == x_i) { continue; }
                    loop {
                        let x_j = to_replace_list[ rng.get_value(to_replace_list.len() as i32) as usize ];
                        if x_j > 35 || x_j == 30 { continue; }
                        GameVariableManager::set_string(&format!("G_R_{}",PIDS[x_j as usize]), PIDS[x_i as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}", PIDS[x_i as usize]), PIDS[x_j as usize]);
                        if let Some(index1) = to_replace_list.iter().position(|&i| i == x_j) { to_replace_list.remove(index1); }
                        if let Some(index2) = playable_list.iter().position(|&i| i == x_i) {  playable_list.remove(index2);  }
                        println!("#{}: {} -> {}", x_j, Mess::get_name(PIDS[x_j as usize]).get_string().unwrap(),  Mess::get_name(PIDS[x_i as usize]).get_string().unwrap());
                        break;
                    }
                }
                for x_i in 0..playable_size {
                    if !to_replace_list.iter().any(|&x| x_i == x) { continue; }
                    let key_pid_x = format!("G_R_{}", pids[x_i as usize]);
                    if playable_list.len() == 0 {
                        println!("out of playables :(");
                        GameVariableManager::make_entry_str(&key_pid_x, &pids[x_i as usize]);
                        GameVariableManager::set_string(&key_pid_x, &pids[x_i as usize]);
                        GameVariableManager::make_entry_str(&format!("G_R2_{}", pids[x_i as usize]), &pids[x_i as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}", pids[x_i as usize]), &pids[x_i as usize]);
                        continue; 
                    }
                    let x_j = playable_list[rng.get_value(playable_list.len() as i32) as usize ];
                    GameVariableManager::make_entry_str(&key_pid_x, &pids[x_j as usize]);
                    GameVariableManager::set_string(&key_pid_x, &pids[x_j as usize]);
                    let key_pid_j = format!("G_R2_{}", pids[x_j as usize]);
                    GameVariableManager::make_entry_str(&key_pid_j, &pids[x_i as usize]);
                    GameVariableManager::set_string(&key_pid_j, &pids[x_i as usize]);
                    if let Some(index) = playable_list.iter().position(|&i| i == x_j) {  playable_list.remove(index);  }
                    println!("#{}: {} -> {}", x_i, Mess::get_name(pids[x_i as usize].clone()).get_string().unwrap(),  Mess::get_name(pids[x_j as usize].clone()).get_string().unwrap());
                }
            },
            2 => {   //Reverse
                for x in 0..41 {
                    let index = RR_ORDER[x as usize] as usize;
                    GameVariableManager::set_string(&format!("G_R_{}",PIDS[x as usize]), PIDS[index]);
                    GameVariableManager::set_string(&format!("G_R2_{}",PIDS[index]), PIDS[x as usize]);
                    println!("{} -> {}", pid_to_mpid(&PIDS[x as usize].to_string()), pid_to_mpid(&PIDS[index].to_string()));
                }
            },
            3 => { // Custom
                let custom = get_custom_recruitment_list();
                println!("Custom Recruitment List");
                for x in 0..41 {
                    let mut index = x as usize; 
                    if set_emblems[index] { continue; }
                    while custom[index] != -1 {
                        if set_emblems[index] { break; }
                        GameVariableManager::set_string(&format!("G_R_{}",PIDS[index]), PIDS[custom[index] as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}",PIDS[custom[index] as usize]), PIDS[index]);
                        set_emblems[index] = true;
                        println!("Loop {}, {} -> {}", x, pid_to_mpid(&PIDS[index].to_string()), pid_to_mpid(&PIDS[custom[index] as usize].to_string()));
                        index = custom[index] as usize;
                    }
                    if set_emblems[index] { continue; }
                    GameVariableManager::set_string(&format!("G_R_{}",PIDS[index]), PIDS[x as usize]);
                    GameVariableManager::set_string(&format!("G_R2_{}",PIDS[x as usize]), PIDS[index]);
                    println!("{} -> {}", pid_to_mpid(&PIDS[index].to_string()), pid_to_mpid(&PIDS[x as usize].to_string()));
                    set_emblems[index] = true;
                }
            },
            _ => {},
        }
    }
    GameVariableManager::set_bool("G_Random_Person_Set", true);
    set_hub_facilities(); 
    change_hub_dispos(false);
}

pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<String>{
    if PIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).get_string().unwrap()); }
    }
    else if EMBLEM_GIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).get_string().unwrap()); }
    }
    return None;
}

pub fn change_hub_dispos(revert: bool) {
    let t_list = HubDisposData::get_array_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_aid();
            if aid.is_some() { 
                if str_contains(aid.unwrap(), "GID_") && str_contains(t_list[x][y].parent.array_name, "Fld_S0") { continue; }
                let pid = aid.unwrap().get_string().unwrap();
                let new_pid = find_pid_replacement(&pid, revert);
                if new_pid.is_some() { 
                    let n_pid = new_pid.unwrap();
                    t_list[x][y].set_aid(n_pid.into());
                 }
            }
        }
    }
    if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == "PID_リュール" { return;  }
    let replacement = GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap();
    let hublist = super::item::shop::HubRandomSet::get_list_mut().unwrap();
    for x in 0..hublist.len() {
        let list = &mut hublist[x]; 
        for y in 0..list.len() {
            if list.parent.list[y].iid.get_string().unwrap() == replacement {
                list.parent.list[y].iid = "PID_リュール".into();
            }
        }
    }
}

pub fn change_map_dispos() {
    let list = DisposData::get_list_mut();
    if list.is_none() || !can_rand() { return; }
    let t_list = list.unwrap();
    let cid = GameUserData::get_chapter().cid.get_string().unwrap();
// Framme and Clanne Replacement
    if cid == "CID_M002" ||  cid == "CID_M001" || cid == "CID_M003" { GameVariableManager::make_entry("DDFanClub", 1); }
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_pid();
            if aid.is_none() { continue; }
            let pid = aid.unwrap().get_string().unwrap();
            if pid == "PID_リュール" { 
                let new_pif = GameVariableManager::get_string("G_R_PID_リュール");
                t_list[x][y].set_pid(new_pif); 
            }
            else if t_list[x][y].get_force() == 0  && GameVariableManager::get_bool("DDFanClub") && GameVariableManager::exist(&format!("G_R_{}", pid)) {
                let new_pid = GameVariableManager::get_string(&format!("G_R_{}", pid));
                t_list[x][y].set_pid(new_pid);
            }
            else if t_list[x][y].get_force() == 2 && GameVariableManager::get_bool("DDFanClub") && GameVariableManager::exist(&format!("G_R_{}", pid)) {
                let new_pid = GameVariableManager::get_string(&format!("G_R_{}", pid));
                t_list[x][y].set_pid(new_pid);
            }
        }
    }
}

pub fn change_lueur_for_recruitment(is_start: bool) {
    if !crate::utils::can_rand() { return; }
    if GameVariableManager::get_string("G_R_PID_リュール").get_string().unwrap() == "PID_リュール" { return;  }
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return; }
        // remove hero status on alear and place it on the replacement and add alear skills on the replacement
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    let lueur_sids = person_lueur.get_common_sids().unwrap();
    for x in 0..lueur_sids.len() {
       if lueur_sids[x].get_string().unwrap() == "SID_主人公" { lueur_sids[x] = "SID_無し".into(); }
    }
    person_lueur.on_complete();
    let new_hero = switch_person(person_lueur);
    let sids = new_hero.get_common_sids().unwrap();
    let new_sids = Array::<&Il2CppString>::new_specific( sids.get_class(), sids.len()+4).unwrap();
    for x in 0..sids.len() { new_sids[x] = sids[x]; }
    new_sids[sids.len() ] = "SID_主人公".into();
    new_sids[sids.len() + 1 ] = "SID_リベラシオン装備可能".into();
    new_sids[sids.len() + 2 ] = "SID_ヴィレグランツ装備可能".into();
    new_sids[sids.len() + 3 ] = "SID_王族".into();
    new_hero.set_common_sids(new_sids);
    new_hero.on_complete();
    if is_start {   // Move alear to force 5
        let lueur = unsafe { crate::deployment::force_get_unit_from_pid(PIDS[0].into(), true, None) };
        if lueur.is_some() { 
            let lueur_unit = lueur.unwrap();
            unit::change_unit_autolevel(lueur_unit, true);
            if GameVariableManager::get_number("G_Random_Job") == 1 ||  GameVariableManager::get_number("G_Random_Job") == 3  {
                super::job::unit_change_to_random_class(lueur_unit);
                unit::fixed_unit_weapon_mask(lueur_unit);
                unit::adjust_unit_items(lueur_unit);
            }
            lueur_unit.transfer(5, false); 
            println!("Lueur transfer to force 5");
            crate::utils::get_lueur_name_gender(); // grab gender and name
            GameVariableManager::make_entry("G_Lueur_Gender2", lueur_unit.edit.gender);
        }
        let new_unit =  unsafe { join_unit(new_hero, None) };
        if new_unit.is_some() {
            let unit = new_unit.unwrap();
            unit.edit.set_name( Mess::get( new_hero.get_name().unwrap()) );
            unit.edit.set_gender( new_hero.get_gender() );
            println!("{} unit edit set", new_hero.get_name().unwrap().get_string().unwrap());
            unit.transfer(3, false);
        }
    }
    Patch::in_text(0x02d524e0).nop().unwrap();
    Patch::in_text(0x02d524e4).nop().unwrap();

    // LueurW_God or Lueur_God in GetPath 
    if GameVariableManager::get_number("G_Lueur_Gender2") == 2 {  
        Patch::in_text(0x02d524e8).bytes(&[0x48, 0x00, 0x80, 0x52]).unwrap(); 
       person_lueur.set_gender(2);
    }
    else { 
        Patch::in_text(0x02d524e8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();
        person_lueur.set_gender(1);
    }

    Patch::in_text(0x0233f104).bytes(&[0x01,0x10, 0x80, 0x52]).unwrap(); // GodUnit$$GetName ignore hero flag on Emblem Alear
    let lueur_god_offsets = [0x02d51dec, 0x021e12ac, 0x02915844, 0x02915844, 0x02915694, 0x01c666ac,0x02081edc, 0x01c69d60, 0x01c66588];
    for x in lueur_god_offsets { mov_x0_0(x); }

    // For Hub-Related Activites
    let offsets = [0x02ae8d28, 0x02ae9000, 0x02a5d0f4, 0x01cfd4c4, 0x01d03184, 0x01e5fe00, 0x01e5ff4c, 0x027049c8];
    let new_hero_gender = if new_hero.get_gender() == 2 || (new_hero.get_gender() == 1 && new_hero.get_flag().value & 32 != 0 ) { 2 } else { 1 };
    for x in offsets {
        if new_hero_gender == 1 {  crate::utils::mov_x0_0(x); }
        else { crate::utils::mov_1(x); }
    }
    unsafe { LUEUR_CHANGE = true; }
}


pub fn pid_to_index(pid: &String, reverse: bool) -> i32 {
    if let Some(replacement) = find_pid_replacement(pid, reverse) {
        if let Some(found_pid) = PIDS.iter().position(|&x| x == replacement) { return found_pid as i32; }
        if let Some(found_gid) = EMBLEM_GIDS.iter().position(|&x| x == replacement) { return found_gid as i32;  }
    }
    return -1;  // to cause crashes
}

pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.get_aptitude().value;
    for x in 0..3 {
        if apt & (1 << (x+1) ) != 0 { return x; }
    }
    let apt2 = this.get_sub_aptitude().value;
    for x in 0..3 {
        if apt2 & (1 << (x+1) ) != 0 { return x; }
    }
    return 0;
}

pub fn switch_person(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return PersonData::get(&pid).unwrap(); }
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe { if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); } }
    let new_person = PersonData::get(&new_pid.get_string().unwrap());
    if new_person.is_some() { return new_person.unwrap(); }
    else { return PersonData::get(&pid).unwrap(); }
}
pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    let reverse = GameVariableManager::get_string(&format!("G_R2_{}", pid)).get_string().unwrap();
    return PersonData::get(&reverse).unwrap();
}

// Handle the case of Chapter 11 ends with not escape
pub fn m011_ivy_recruitment_check(){
    if !crate::utils::can_rand() { return; }
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return; }
    if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M011" && crate::utils::lueur_on_map() {
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
    }
}


#[skyline::from_offset(0x01c73960)]
fn join_unit(person: &PersonData, method_info: OptionalMethod) -> Option<&'static mut Unit>;

#[skyline::hook(offset=0x02d51d80)]
pub fn get_thumb_face(this: &Unit, method_info: OptionalMethod) -> &Il2CppString {
    let pid = this.person.pid.get_string().unwrap();
    if pid == "PID_リュール" {
        if this.person.get_name().unwrap().get_string().unwrap() == "MPID_Lueur" {
            if GameVariableManager::exist("G_Lueur_Gender2") { 
                if GameVariableManager::get_number("G_Lueur_Gender2") == 2 { return "LueurW".into(); }
                else { return "Lueur".into(); }
            }
        }
    }
    let name = this.person.get_name().unwrap();
    if let Some(pos) = RINGS.iter().position(|&x| str_contains(name, x)) {
        if pos > 11 { return format!("{}_DLC", RINGS[pos]).into();  }
        else { return RINGS[pos].into();  }
    }
    return this.person.get_ascii_name().unwrap();
}
#[skyline::hook(offset=0x02d52340)]
pub fn get_god_thumb_face(this: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    let name = this.mid;
    if this.gid.get_string().unwrap() == "GID_リュール" {
        if name.get_string().unwrap() == "MPID_Lueur" {
            if GameVariableManager::exist("G_Lueur_Gender2") { 
                if GameVariableManager::get_number("G_Lueur_Gender2") == 2 { return "LueurW".into(); }
                else { return "Lueur".into(); }
            }
        }
    }
    if let Some(pos) = MPIDS.iter().position(|&x| str_contains(name, x)) {
        let new_name = &MPIDS[pos][5..];
        return new_name.into();
    }
    return call_original!(this, method_info);
}
#[skyline::hook(offset=0x021e1250)]
pub fn get_bond_face(this: &Unit, method_info: OptionalMethod) -> &Il2CppString {
    let name = this.person.get_name().unwrap().get_string().unwrap();
    if this.person.pid.get_string().unwrap() == "PID_リュール" {
        if this.person.get_name().unwrap().get_string().unwrap() == "MPID_Lueur" {
            if GameVariableManager::exist("G_Lueur_Gender2") { 
                if GameVariableManager::get_number("G_Lueur_Gender2") == 2 { return "Telop/LevelUp/FaceThumb/LueurW".into(); }
                else { return "Telop/LevelUp/FaceThumb/Lueur".into(); }
            }
        }
    }
    if MPIDS.iter().position(|&x| x == name).is_some() { call_original!(this, method_info) }
    else {
        let rng = Random::get_system();
        let size = if dlc_check() { 41 } else { 36 };
        let new_name = &MPIDS[rng.get_value(size) as usize][5..];
        let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
        println!("{}", path);
        return path.into();
    }
}

#[skyline::hook(offset=0x01a22eb0)]
pub fn get_unit_ascii_name(unit: &Unit, method_info: OptionalMethod) -> &'static Il2CppString {
    let pid = unit.person.pid.get_string().unwrap();
    if let Some(pos) = PIDS.iter().position(|&x| x == pid) {  il2_str_substring(MPIDS[pos].into(), 5) }
    else { call_original!(unit, method_info)  }
}

#[skyline::hook(offset=0x021e16f0)]
pub fn get_god_face(this: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    let mid = this.mid;
    let result = call_original!(this, method_info);
    println!("Result: {}", result.get_string().unwrap());
    if str_contains(mid, "Lueur") {
        if GameVariableManager::exist("G_Lueur_Gender2") { 
            if GameVariableManager::get_number("G_Lueur_Gender2") == 2 { return "Telop/LevelUp/FaceThumb/God/LueurW".into(); }
            else { return "Telop/LevelUp/FaceThumb/God/Lueur".into(); }
        }
    }
    if let Some(pos) = MPIDS.iter().position(|&x| str_contains(mid, x)) {
        let new_name = &MPIDS[pos][5..];
        let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
        println!("{}", path);
        return path.into();
    }
    if RINGS.iter().any(|&x| str_contains(mid, x)) {  return result;  }
    else {
        let rng = Random::get_system();
        let dlc = dlc_check();
        let size = if dlc { 61 } else { 47 };
        let emblem_range = if dlc { 22 } else { 12 };
        let person_limit = if dlc { 41 } else { 36 };
        let index = rng.get_value(size) as usize + 1;
        let new_name = 
            if index < person_limit { &MPIDS[rng.get_value(size) as usize][5..] }
            else { RINGS[ rng.get_value(emblem_range) as usize ] };
        let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
        println!("{}", path);
        return path.into();
    }
}