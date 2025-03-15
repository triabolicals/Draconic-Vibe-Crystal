use engage::unitpool::UnitPool;
use god::Dictionary;
pub use unity::prelude::*;
pub use engage::{
    menu::{
        BasicMenuItemAttribute,
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    mess::*,
    gamevariable::*, gameuserdata::*, hub::access::*, random::*,
    gamedata::{*, item::*, skill::SkillData, dispos::*, unit::*},
};
use std::sync::OnceLock;
use skyline::patching::Patch;
use unity::il2cpp::object::Array;
use crate::{
    enums::*,
    utils::*, autolevel::*,
};

use super::{DVCVariables, CONFIG, RANDOMIZER_STATUS};

pub mod ai;
pub mod unit; 
pub mod hub;
pub mod custom;

pub static mut SET: i32 = 0;
pub static PLAYABLE: OnceLock<Vec<i32>> = OnceLock::new();
pub static mut INDEX: i32 = -1;
pub static mut INDEX2: i32 = -1;
pub static mut SELECTION: i32 = 0;
pub static mut SELECTION2: i32 = 0;
pub static mut IS_EMBLEM: bool = false;
pub static ENEMY_PERSONS: OnceLock<Vec<(i32, i32)>> = OnceLock::new();

pub struct RandomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = custom::crecruitment_menu_a_call as _).unwrap();
    }
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
            3 => { "Unit recruitment order is determined by list. (Press A)"},
            2 => { "Characters will be recruited in reversed order."}
            _ => { "Standard recruitment order." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_recruitment {
            1 => { "Random"},
            3 => { "Custom Order (A)"},
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
        this.command_text = if CONFIG.lock().unwrap().custom_units  { "Include" } 
            else { "Default"}.into();
    }
}
fn build_attribute_custom_units(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if PLAYABLE.get().unwrap().len() == 41 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub struct CustomPersonRecruitDisable;
impl ConfigBasicMenuItemSwitchMethods for CustomPersonRecruitDisable {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if PLAYABLE.get().unwrap().len() > 94 {
            this.help_text = "Added recruitment slots are disabled. (Cannot Change)".into();
            this.command_text = "Disable".into();
            this.update_text();
            return BasicMenuResult::new();
        }
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().custom_unit_recruitment_disable);
        if CONFIG.lock().unwrap().custom_unit_recruitment_disable != result {
            CONFIG.lock().unwrap().custom_unit_recruitment_disable = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().custom_unit_recruitment_disable { "Added Somniel recruitment slots will be disabled." }
            else { "Added Somniel recruitment slots will be enabled." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().custom_unit_recruitment_disable  { "Disable" } 
            else { "Enable"}.into();
    }
}

pub extern "C" fn vibe_custom_units() -> &'static mut ConfigBasicMenuItem { 
    let item = ConfigBasicMenuItem::new_switch::<CustomPersonMod>("Custom Units");
    item.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_custom_units as _);
    item
}
pub extern "C" fn vibe_custom_slot_disable() -> &'static mut ConfigBasicMenuItem { 
    let item = ConfigBasicMenuItem::new_switch::<CustomPersonRecruitDisable>("Added Recruitment Slots");
    item.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_custom_units as _);
    item
}
pub fn get_playable_list() {
    // Add the 41 units first
    PLAYABLE.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        let mut hashes: Vec<i32> = Vec::new();
        PIDS.iter().for_each(|&pid| {
            let person = PersonData::get(pid).unwrap();
            list.push(person.parent.index);
            hashes.push(person.parent.hash);
        });
        // Add all others that have non zero SP
        let person_list = PersonData::get_list().unwrap(); 
        let mut count = 0;
    
        for x in 0..person_list.len() { 
            let person = &person_list[x as usize];
            if person.get_sp() == 0 { continue; }
            count += 1;
        }
        if count < 200 { 
            for x in 1..person_list.len() { 
                let person = &person_list[x as usize];
                if person.pid.to_string().contains("_竜化") { continue; }  //No Dragons
                if person.get_common_sids().is_none() { continue; }
                let index = person.parent.index; 
                if str_contains(person.pid, "PLAYABLE") || str_contains(person.pid, "layable") { person.set_asset_force(0); } 
                if person.get_sp() > 0 && person.get_asset_force() == 0 {
                    if person.get_sp() < 300 { person.set_sp(300); }
                    if hashes.iter().find(|r| **r == person.parent.hash).is_none() { 
                        list.push(index);
                        hashes.push(person.parent.hash);
                        println!("Person #{}: {} was added", index, Mess::get_name(person.pid).to_string());
                    }
                }
            }
        }
        if count < 95 { println!("Total of {} Playable Units", list.len()); }
        else { println!("Total of {} Possible Playable Units", list.len()); }

        list
    });
}
pub fn is_playable_person(person: &PersonData) -> bool { PLAYABLE.get().unwrap().iter().any(|&x| person.parent.index == x) }
pub fn check_playable_classes() {
    // Set valid classes to Sword Fighter or Swordmaster
    let list = PLAYABLE.get().unwrap();
    list.iter().for_each(|&index|{
        if let Some(person) = PersonData::try_index_get_mut(index) {
            if person.get_job().is_none() {
                if person.get_sp() >= 1000 { person.set_jid("JID_ソードマスター".into()); }
                else {  person.set_jid("JID_ソードファイター".into()); }
                person.on_completed();
            }
        }
    });
    super::names::give_names_to_generics();
}

fn get_custom_recruitment_list() -> Vec<(i32, i32)> {   // person_x to person_y
    let mut output: Vec<(i32, i32)> = Vec::new();
    let table = custom::CUSTOM_RECRUITMENT_TABLE.lock().unwrap();
    let limit = if dlc_check() { 41 } else { 36 };
    let mut available: Vec<i32> = (0..limit).collect();
    let mut pool: Vec<i32> = Vec::new();
    for x in 0..limit {
        let value = table[x as usize];
        if table[x as usize] != 0 {
            output.push( (x, value - 1) );
            if let Some(pos) = available.iter().position(|&y| value - 1 == y) {
                available.remove(pos);
            }
        }
        else { pool.push(x); }
    }
    let rng = get_rng();
    [0, 4, 14, 17, 23, 27 ].iter().for_each(|&x_lord|{ 
        if let Some(pos) = pool.iter().position(|&xi| xi == x_lord){
            if available.len() > 0 {
                loop {  // Making sure lords do not get randomized with DLC Characters
                    let index = rng.get_value( available.len() as i32) as usize;
                    let xj = available[index];
                    if xj < 36 {
                        output.push( (x_lord, xj) );
                        available.remove(index);
                        break;
                    }
                }
                pool.remove(pos);
            }
        }
    });
    pool.iter().for_each(|&xi|{
        if available.len() > 0 {
            let index = rng.get_value( available.len() as i32) as usize;
            let xj = available[index];
            output.push( (xi, xj) );
            available.remove(index);
        }
    });
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
                if hub_locator.to_string() == locator[ x as usize] {
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
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 { 
            set_hub_facilities(); 
            hub::change_somniel_hub_dispos();
        }
        return; 
    }
    else {
        let rng = get_rng();
        match GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) {
            1 => {
                let playable_size = if CONFIG.lock().unwrap().custom_units && PLAYABLE.get().unwrap().len() > 41 { PLAYABLE.get().unwrap().len() }
                    else { 41 };
        
                let list = PLAYABLE.get().unwrap();
                let mut playable_list: Vec<usize> = (0..playable_size).collect();
                let mut to_replace_list: Vec<usize> = (0..playable_size).collect();
                if !dlc_check() || CONFIG.lock().unwrap().dlc & 2 != 0 { 
                    for x in 36..41 {   // Remove DLC characters in the pool
                        if let Some(index) = playable_list.iter().position(|&i| i == x) {  playable_list.remove(index);  }
                        if let Some(index) = to_replace_list.iter().position(|&i| i == x) {  to_replace_list.remove(index);  }
                    }
                }
                let person_list = PersonData::get_list().unwrap();
                let pids: Vec<String> = list.iter().map(|&x| person_list[x as usize].pid.to_string() ).collect();
                pids.iter().for_each(|pid|{
                    let key = format!("G_R_{}", pid.as_str());
                    GameVariableManager::make_entry_str(key.as_str(), pid.as_str());
                    let key2 = format!("G_R2_{}", pid.as_str());
                    GameVariableManager::make_entry_str(key2.as_str(), pid.as_str());
                });

                let set_recruitment = SET_RECRUITMENT.lock().unwrap();                
                set_recruitment.iter().filter(|x| x.1 == -1).map(|x| x.0 as usize).for_each(|index|{
                    if let Some(remove) = playable_list.iter().position(|&i| i == index) {  playable_list.remove(remove);  }
                    if let Some(remove) = to_replace_list.iter().position(|&i| i == index) {  to_replace_list.remove(remove);  }
                });

                println!("Playable Unit Size: {}", playable_list.len());
            // Alear and somniel royals must be switched with non-dlc units 
            //  x_i in to_replace, x_j in playable_list, royals are x_i
            //  x_j -> x_i, remove royal (x_i) from to_replace and remove x_j from playable_list
                let royals = [0, 23, 4, 17, 14, 27];
                for x_royal in royals {
                    if let Some(index_royal) = playable_list.iter().position(|&i| i == x_royal ){  // royal is no longer in the available pool, skip
                        loop {
                            let index_j =  rng.get_value(to_replace_list.len() as i32) as usize;
                            let x_j = to_replace_list[ index_j ];
                            if x_j > 35 || x_j == 30 { continue; }  // If DLC/Linden try again
                            DVCVariables::set_person_recruitment(x_j as i32, x_royal as i32);
                            to_replace_list.remove(index_j);  // Remove the unit who becomes the royal from the list of units to replace
                            playable_list.remove(index_royal); // Remove Royal from Pool from the playable list to replace to
                            println!("#{}: {} -> {}", x_j, Mess::get_name(PIDS[x_j as usize]),  Mess::get_name(PIDS[x_royal as usize]));
                            break;
                        }
                    }
                }
                to_replace_list.iter().for_each(|&x_i|{
                    let key_pid_x = format!("G_R_{}", pids[x_i].as_str());
                    let pool_size = playable_list.len() as i32;
                    if pool_size > 0 {
                        let index_j = rng.get_value(pool_size as i32) as usize;
                        let x_j = playable_list[ index_j ];
                        GameVariableManager::set_string(key_pid_x.as_str(), pids[x_j].as_str() );
                        let key_pid_j = format!("G_R2_{}", pids[x_j as usize]);
                        GameVariableManager::set_string(key_pid_j.as_str(), pids[x_i].as_str());
                        playable_list.remove(index_j);
                        println!("#{}: {} -> {}", x_i, Mess::get_name(pids[x_i as usize].as_str()),  Mess::get_name(pids[x_j as usize].as_str()));
                    }
                }); 
            },
            2 => {   //Reverse
                for x in 0..41 { DVCVariables::set_person_recruitment(x, RR_ORDER[x as usize] as i32); }
            },
            3 => { // Custom
                get_custom_recruitment_list().iter().for_each(|&x|{
                    DVCVariables::set_person_recruitment(x.0, x.1);
                    println!("{} -> {}", Mess::get_name(PIDS[x.0 as usize]), Mess::get_name(PIDS[x.1 as usize]));
                });
            },
            _ => {},
        }
    }
    GameVariableManager::set_bool("G_Random_Person_Set", true);
    set_hub_facilities(); 
    hub::change_somniel_hub_dispos();
}

pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<String>{
    if PIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).to_string()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).to_string()); }
    }
    if EMBLEM_GIDS.iter().position(|&x| x == *pid).is_some() {
        if reverse { return Some( GameVariableManager::get_string(&format!("G_R2_{}", pid)).to_string()); }   
        else { return Some( GameVariableManager::get_string(&format!("G_R_{}", pid)).to_string()); }
    }
    return None;
}

pub fn change_map_dispos() {
    let list = DisposData::get_list_mut();
    if list.is_none() || !can_rand() { return; }
    let t_list = list.unwrap();
// Framme and Clanne Replacement
    if !DVCVariables::is_main_chapter_complete(3) { GameVariableManager::make_entry("DDFanClub", 1); }
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_pid();
            if aid.is_none() { continue; }
            let pid = aid.unwrap().to_string();
            if pid == PIDS[0] { 
                t_list[x][y].set_pid(DVCVariables::get_dvc_person(0, true));   }
            else if ( t_list[x][y].get_force() == 0 || t_list[x][y].get_force() == 2 ) && GameVariableManager::get_bool("DDFanClub") && GameVariableManager::exist(&format!("G_R_{}", pid)) {
                t_list[x][y].set_pid(GameVariableManager::get_string(&format!("G_R_{}", pid)));
            }
        }
    }
}



pub fn change_lueur_for_recruitment(is_start: bool) {
    if !DVCVariables::random_enabled() || super::RANDOMIZER_STATUS.read().unwrap().alear_person_set { return; }
    if !GameVariableManager::exist("G_R_PID_リュール") ||GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    println!("Alear check");
    if DVCVariables::get_dvc_person(0, true).to_string() == PIDS[0] {
        let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.alear_person_set = true);
        return;
     }
    // remove hero status on alear and place it on the replacement and add alear skills on the replacement
    let person_lueur = PersonData::get(PIDS[0]).unwrap();
    let lueur_sids = person_lueur.get_common_sids().unwrap();
    if let Some(hero_sid) = lueur_sids.iter_mut().find(|x| x.to_string().contains("SID_主人公")) {
        *hero_sid =  "SID_無し".into();
    }
    person_lueur.on_complete();
    let new_hero = switch_person(person_lueur);
    if let Some(hero) = UnitPool::get_from_person_force_mask(&new_hero, -1) {
        hero.private_skill.add_sid("SID_主人公", 10, 0);
        hero.private_skill.add_sid("SID_王族", 10, 0);
        hero.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
        hero.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
    }
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
        if let Some(lueur_unit) = engage::unitpool::UnitPool::get_from_person_mut(PIDS[0].into(), false) {
            unit::change_unit_autolevel(lueur_unit, true);
            if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 {
                super::job::unit_change_to_random_class(lueur_unit);
                unit::fixed_unit_weapon_mask(lueur_unit);
                unit::adjust_unit_items(lueur_unit);
            }
            lueur_unit.transfer(5, false); 
            crate::utils::get_lueur_name_gender(); // grab gender and name
            GameVariableManager::make_entry(DVCVariables::LUEUR_GENDER, lueur_unit.edit.gender);
        }
        if let Some(unit) = unsafe { join_unit(new_hero, None) }{
            unit.edit.set_name( Mess::get( new_hero.get_name().unwrap()) );
            unit.edit.set_gender( new_hero.get_gender() );
            unit.private_skill.add_sid("SID_主人公", 10, 0);
            unit.private_skill.add_sid("SID_王族", 10, 0);
            unit.private_skill.add_sid("SID_リベラシオン装備可能", 10, 0);
            unit.private_skill.add_sid("SID_ヴィレグランツ装備可能", 10, 0);
            unit.transfer(3, false);
        }
    }
    Patch::in_text(0x02d524e0).nop().unwrap();
    Patch::in_text(0x02d524e4).nop().unwrap();

    // LueurW_God or Lueur_God in GetPath 
    if GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2 {  
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
    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() { 
        lock.alear_person_set = true; 
        lock.set_enable();
    }
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
    let pid = person.pid.to_string();
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return PersonData::get(&pid).unwrap(); }
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe { if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); } }
    if let Some(new_person) = PersonData::get(&new_pid.to_string()) { return new_person; }
    else { return PersonData::get(&pid).unwrap(); }
}
pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.to_string();
    let reverse = GameVariableManager::get_string(&format!("G_R2_{}", pid));
    return PersonData::get(reverse).unwrap();
}

// Handle the case of Chapter 11 ends with not escape
pub fn m011_ivy_recruitment_check(){
    if !DVCVariables::random_enabled() || GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    if GameUserData::get_chapter().cid.to_string() == "CID_M011" && crate::utils::lueur_on_map() {
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
    }
}
pub fn lueur_recruitment_check() {
    if let Some(lueur) = UnitPool::get_from_person_force_mask(PersonData::get(PIDS[0]).unwrap(), 6){
        if lueur.force.is_some_and(|f| ( GameUserData::get_chapter().cid.to_string().contains("M018") && f.force_type == 1 ) || f.force_type == 2) {
            if GameUserData::get_sequence() == 3 { lueur.transfer(0, true); }
            else if GameUserData::get_sequence() == 5 { lueur.transfer(3, true); }
        }
    }
}

#[skyline::from_offset(0x01c73960)]
fn join_unit(person: &PersonData, method_info: OptionalMethod) -> Option<&'static mut Unit>;

#[skyline::hook(offset=0x02d51d80)]
pub fn get_thumb_face(this: &Unit, _method_info: OptionalMethod) -> &Il2CppString {
    let name = this.person.get_name().unwrap().to_string();
    if let Some(pos) = MPIDS.iter().position(|&x| name == x ) {
        if pos == 0 { return get_gender_lueur_ascii(false).into(); }
        let new_name = &MPIDS[pos][5..];
        return new_name.into();
    }
    let il2 = this.person.get_name().unwrap();
    if let Some(pos) = RINGS.iter().position(|&x| str_contains( il2, x)) {
        if pos > 11 && pos < 21{ return format!("{}_DLC", RINGS[pos]).into();  }
        else { return RINGS[pos].into();  }
    }
    return this.person.get_ascii_name().unwrap();
}
#[skyline::hook(offset=0x02d52340)]
pub fn get_god_thumb_face(this: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    let name = this.mid;
    if this.gid.to_string() == "GID_リュール" {
        if this.mid.to_string().contains("Lueur") { return get_gender_lueur_ascii(true).into(); }
    }
    if let Some(pos) = MPIDS.iter().position(|&x| str_contains(name, x)) {
        if pos == 0 { return get_gender_lueur_ascii(false).into(); }
        let new_name = &MPIDS[pos][5..];
        return new_name.into();
    }
    call_original!(this, method_info)
}
#[skyline::hook(offset=0x021e1250)]
pub fn get_bond_face(this: &Unit, _method_info: OptionalMethod) -> &Il2CppString {
    let name = this.person.get_name().unwrap().to_string();
    let result = call_original!(this, None);
    if let Some(old) = MPIDS.iter().position(|&x| x == name) { 
        if old == 0 { return format!("Telop/LevelUp/FaceThumb/{}", get_gender_lueur_ascii(false)).into(); }
        let new_name = &MPIDS[old][5..]; 
        return format!("Telop/LevelUp/FaceThumb/{}", new_name).into();
    }
    else if let Some(pos) = RINGS.iter().find(|&x| str_contains(this.person.get_name().unwrap(), x)) {
        format!("Telop/LevelUp/FaceThumb/{}", pos).into()
    } 
    else if unsafe { get_sprite(result, None) } { return result;  }
    else {
        let size = if dlc_check() { 42 } else { 37 };
        if this.person.parent.index == unsafe { INDEX } && unsafe { SELECTION != -1 } {
            let sel = unsafe { SELECTION};
            let new_name = if sel == size - 1 { "LueurW" }
            else  { &MPIDS[sel as usize][5..] };
            let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
            return path.into();
        }
        unsafe { INDEX = this.person.parent.index };
        let rng = Random::get_system();
        let sel = rng.get_value(size);
        let new_name = if sel == size - 1 { "LueurW" }
            else  { &MPIDS[sel as usize][5..] };
        let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
        unsafe { SELECTION = sel };
        return path.into();
    }
}

#[skyline::hook(offset=0x01a22eb0)]
pub fn get_unit_ascii_name(unit: &Unit, method_info: OptionalMethod) -> &'static Il2CppString {
    let pid = unit.person.pid.to_string();
    if let Some(pos) = PIDS.iter().position(|&x| x == pid) {  il2_str_substring(MPIDS[pos].into(), 5) }
    else { call_original!(unit, method_info)  }
}

pub fn get_gender_lueur_ascii(god: bool) -> String {
    let is_female = 
        if GameVariableManager::exist(DVCVariables::LUEUR_GENDER) {  GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2  }
        else if let Some(lueur_unit) = engage::unitpool::UnitPool::get_from_person_mut(PIDS[0].into(), false) {
            if lueur_unit.edit.is_enabled() { lueur_unit.edit.gender == 2  }
            else { false }
        }
        else { false };
    match (god, is_female) {
        (true, true) => { "LueurW_God"}
        (false, true) =>  { "LueurW"}
        (true, false) => { "Lueur_God"}
        (false, false) => {"Lueur"}
    }.to_string()
}


#[skyline::hook(offset=0x021e16f0)]
pub fn get_god_face(this: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    let mid = this.mid;
    let result = call_original!(this, method_info);
    if mid.to_string().contains("Lueur") && this.gid.to_string().contains("リュール") {
        return format!("Telop/LevelUp/FaceThumb/God{}", get_gender_lueur_ascii(false)).into();
    }
    if let Some(pos) = MPIDS.iter().position(|&x| mid.contains(x)) {
        let new_name = &MPIDS[pos][5..];
        let path = format!("Telop/LevelUp/FaceThumb/{}", new_name);
        return path.into();
    }

    if RINGS.iter().any(|&x| str_contains(mid, x)) {  return result;  }
    else if unsafe { get_sprite(result, None) } {
        println!("God Result: {}", result);
        return result; 
    }
    else {
        let dlc = dlc_check();
        let emblem_range = if dlc { 23 } else { 12 };
        let person_limit = if dlc { 40 } else { 35 };

        if this.parent.index == unsafe { INDEX2 } && unsafe { SELECTION2 != -1 } {
            let sel = unsafe { SELECTION2} as usize;
            if unsafe {IS_EMBLEM} {
                return format!("Telop/LevelUp/FaceThumb/{}", RINGS[sel]).into();
            }
            else {
                let new_name = &MPIDS[sel][5..];
                return format!("Telop/LevelUp/FaceThumb/{}", new_name).into();
            }
        }
        unsafe { INDEX2 = this.parent.index; }
        let rng = Random::get_system();
        if rng.get_value(4) < 2 {
            unsafe { IS_EMBLEM = true }
            let index = rng.get_value(emblem_range) as usize;
            unsafe { SELECTION2 = index as i32 };
            return format!("Telop/LevelUp/FaceThumb/{}", RINGS[index]).into();
        }
        else {
            unsafe {IS_EMBLEM = false; }
            let index = rng.get_value(person_limit)  as usize + 1;
            unsafe { SELECTION2 = index as i32};
            let new_name = &MPIDS[index][5..];
            return format!("Telop/LevelUp/FaceThumb/{}", new_name).into();
        }
    }
}

pub fn get_all_enemy_persons() {
    ENEMY_PERSONS.get_or_init(||{
        let mut enemy_list: Vec<(i32, i32)> = Vec::new();
        let index = 26;
        enemy_list.push((index, PersonData::get_index("PID_M007_オルテンシア"))); 
        enemy_list.push((index, PersonData::get_index("PID_M010_オルテンシア"))); 
        enemy_list.push((index, PersonData::get_index("PID_M014_オルテンシア"))); 
        enemy_list.push((index, PersonData::get_index("PID_E005_Hide1"))); 
        enemy_list.push((index, PersonData::get_index("PID_E006_Hide6"))); 
        if let Some(pos) = PIDS.iter().position(|p| p.contains("ロサード")){
            let index = pos as i32;
            enemy_list.push((index, PersonData::get_index("PID_M007_ロサード"))); 
            enemy_list.push((index, PersonData::get_index("PID_M010_ロサード"))); 
        }
        if let Some(pos) = PIDS.iter().position(|p| p.contains("ゴルドマリー")){
            let index = pos as i32;
            enemy_list.push((index, PersonData::get_index("PID_M007_ゴルドマリー"))); 
            enemy_list.push((index, PersonData::get_index("PID_M010_ゴルドマリー"))); 
        }
        if let Some(pos) = PIDS.iter().position(|p| p.contains("アイビー")){
            let index = pos as i32;
            enemy_list.push((index, PersonData::get_index("PID_M008_アイビー"))); 
            enemy_list.push((index, PersonData::get_index("PID_M009_アイビー"))); 
            enemy_list.push((index, PersonData::get_index("PID_E004_Boss"))); 
            enemy_list.push((index, PersonData::get_index("PID_E006_Hide5"))); 
        }
        enemy_list.push((33, PersonData::get_index("PID_M011_モーヴ"))); 
        enemy_list.push((33, PersonData::get_index("PID_M014_モーヴ"))); 
        enemy_list.push((33, PersonData::get_index("PID_M016_モーヴ"))); 
        enemy_list.push((33, PersonData::get_index("PID_M017_モーヴ"))); 
        enemy_list.push((33, PersonData::get_index("PID_M019_モーヴ"))); 
        ["カゲツ", "ゼルコバ"].iter().for_each(|p|{
            if let Some(pos) = PIDS.iter().position(|pid| pid.contains(p)) {
                let index = pos as i32;
                enemy_list.push( (index, PersonData::get_index(format!("PID_M008_{}", p))));
                enemy_list.push( (index, PersonData::get_index(format!("PID_M009_{}", p))));
            }
        });
        for x in 36..41 {
            if x == 37 { continue; }
            let sub = &PIDS[x][4..];
            for y in 1..7 {
                let pid = format!("PID_E00{}_{}", y, sub);
                if let Some(dlc) = PersonData::get(pid) { enemy_list.push( (x as i32, dlc.parent.index ) ); }
            }
        }
        enemy_list.push((20, PersonData::get_index("PID_E001_Boss")));  
        enemy_list.push((20, PersonData::get_index("PID_E005_Hide2")));   
        enemy_list.push((20, PersonData::get_index("PID_E006_Hide8"))); 
        enemy_list.push((4, PersonData::get_index("PID_E002_Boss")));   
        enemy_list.push((4, PersonData::get_index("PID_E006_Hide1"))); 
        enemy_list.push((7, PersonData::get_index("PID_E002_Hide")));   
        enemy_list.push((7, PersonData::get_index("PID_E006_Hide2"))); 
        enemy_list.push((14, PersonData::get_index("PID_E003_Boss")));  
        enemy_list.push((14, PersonData::get_index("PID_E006_Hide3"))); 
        enemy_list.push((11, PersonData::get_index("PID_E003_Hide"))); 
        enemy_list.push((11, PersonData::get_index("PID_E006_Hide4"))); 
        enemy_list.push((23, PersonData::get_index("PID_E004_Hide")));  
        enemy_list.push((23, PersonData::get_index("PID_E006_Hide7"))); 
        enemy_list
    });
}

#[skyline::from_offset(0x020169f0)]
fn get_sprite(path: &Il2CppString, method_info: OptionalMethod) -> bool;
#[skyline::from_offset(0x02014930)]
fn get_resource_globals(method_info: OptionalMethod) -> &'static Dictionary<&'static Il2CppString, u64>;
