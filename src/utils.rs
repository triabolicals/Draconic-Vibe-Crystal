use unity::prelude::*;
use engage::{
    random::*,
    force::*,
    mess::*,
    gamedata::{*, unit::Unit, item::*, skill::*},
};
use engage::gamevariable::GameVariableManager;
use skyline::patching::Patch;
use crate::randomizer::emblem::emblem_skill::STAT_BONUS;
use crate::enums::*;

pub fn get_rng() -> &'static Random {
    let rng = Random::instantiate().unwrap();
    rng.ctor(GameVariableManager::get_number("G_Random_Seed") as u32);
    rng
}
pub fn can_rand() -> bool { return GameVariableManager::get_number("G_Random_Seed") != 0; }

pub fn class_count(jid: &str) -> i32 {
    let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
    let mut count = 0;
   for ff in force_type {
       let force_iter = Force::iter(Force::get(ff).unwrap());
       for unit in force_iter {
           if unit.job.jid.get_string().unwrap() == jid {  count += 1; }
       }
   }
   count
}

pub fn lueur_on_map() -> bool {
    let lueur_unit = unsafe { unit_pool_get_hero(true, None) };
    if lueur_unit.is_none() { return false;  }
    return lueur_unit.unwrap().force.unwrap().force_type < 3 ;
}

pub fn is_player_unit(person: &PersonData) -> bool {
    let key = format!("G_R_{}", person.pid.get_string().unwrap());
    if GameVariableManager::exist(&key) { return true; }
    let pid = person.pid.get_string().unwrap();
    for x in PIDS { if *x == pid { return true; } }
    return false;
}

// Getting Player's name for file name
pub fn get_player_name() -> String {
    let f_type: [ForceType; 5] = [ForceType::Player, ForceType::Enemy, ForceType::Absent, ForceType::Dead, ForceType::Lost];
    for f in f_type {
        let force = Force::get(f).unwrap();
        let mut force_iter = Force::iter(force);
        while let Some(unit) = force_iter.next() {
            if unit.person.pid.get_string().unwrap() == "PID_リュール" {
                if unit.edit.name.is_some(){ return unit.edit.name.unwrap().get_string().unwrap(); }
            }
        }
    }
    return "randomized".to_string();
}
pub fn get_lueur_name_gender(){
    GameVariableManager::make_entry("G_Lueur_Gender".into(), 0);
    GameVariableManager::make_entry("G_Lueur_Name".into(), 0);
    let f_type: [ForceType; 5] = [ForceType::Player, ForceType::Enemy, ForceType::Absent, ForceType::Dead, ForceType::Lost];
    for f in f_type {
        let force = Force::get(f).unwrap();
        let mut force_iter = Force::iter(force);
        while let Some(unit) = force_iter.next() {
            if unit.person.pid.get_string().unwrap() == "PID_リュール" {
                if unit.edit.name.is_some(){
                    if unit.edit.gender != 0 {
                        if unit.edit.gender > 2 { unit.edit.set_gender(1); }
                            GameVariableManager::set_number("G_Lueur_Gender".into(), unit.edit.gender);
                            GameVariableManager::set_string("G_Lueur_Name".into(), &unit.edit.name.unwrap().get_string().unwrap());
                            return;
                    }
                }
            }
        }
    }
}

pub fn str_contains(this: &Il2CppString, value: &str) -> bool { unsafe {string_contains(this, value.into(), None) } }
pub fn str_contains2<'a>(this: &Il2CppString, value: impl Into<&'a Il2CppString>) -> bool { unsafe {string_contains(this, value.into(), None) } }

pub fn get_person_name(person: &PersonData) -> String {
    let name = person.get_name().unwrap();
    return mess_get(name);
}

pub fn get_skill_name(skill: &SkillData) -> String {
    if skill.name.is_some() { return format!("{} ({})", mess_get(skill.name.unwrap()), skill.sid.get_string().unwrap()); }
    else {  return format!(" --- ({})", skill.sid.get_string().unwrap()); }
}
pub fn get_item_name(skill: &ItemData) -> String {
    unsafe {  
        if is_null_empty(skill.name, None) { 
            return format!(" --- ({})", skill.iid.get_string().unwrap()); 
        }
    }
    
    let item_name = Mess::get(skill.name ).get_string().unwrap();
    if item_name.len() != 0 { return format!("{} ({})", item_name, skill.iid.get_string().unwrap()); }
    else {
        return format!(" --- ({})", skill.iid.get_string().unwrap());
    }
}

pub fn sid_array_string(sids: &Array<&Il2CppString> ) -> String {
    let n_skills = sids.len();
    let mut n_print = 0;
    let mut out: String = "".to_string();
    for x in 0..n_skills {
        let skill = SkillData::get(&sids[x as usize].get_string().unwrap());
        if skill.is_none() { continue;  }
        if n_print == 0 { out = get_skill_name(skill.unwrap()); n_print += 1; }
        else { out = format!("{}, {}", out, get_skill_name(skill.unwrap())); n_print += 1; }
    }
    return out;
}
pub fn skill_array_string(skills: &SkillArray) -> String {
    let n_skills = skills.list.size;
    let mut n_print = 0;
    let mut out: String = "".to_string();
    let min_index = STAT_BONUS.lock().unwrap()[0]; //Lowest HP Index
    let max_index = STAT_BONUS.lock().unwrap()[65]; //Highest Move Index
    for x in 0..n_skills {
        let skill = skills.list.item[x as usize].get_skill().unwrap();
        let index = skill.parent.index;
        if min_index <= index && index <= max_index { continue; }
        if n_print == 0 { out = get_skill_name(skill); n_print += 1; }
        else { out = format!("{}, {}", out, get_skill_name(skill)); n_print += 1; }
    }
    return out;
}

pub fn stats_from_skill_array(skills: &SkillArray) -> String {
    let n_skills = skills.list.size;
    let mut n_print = 0;
    let mut out: String = "".to_string();
    let mut enhance_array: [i8; 11] = [0; 11];
    let min_index = STAT_BONUS.lock().unwrap()[0]; //Lowest HP Index
    let max_index = STAT_BONUS.lock().unwrap()[65]; //Highest Move Index
    for x in 0..n_skills {
        let skill = skills.list.item[x as usize].get_skill().unwrap();
        let index = skill.parent.index;
        if min_index <= index && index <= max_index  { 
            let enhanced = skill.get_enchance_value();
            for y in 0..11 {
                enhance_array[y as usize] += enhanced[ y as usize] as i8;
            }
        }
    }
    for y in 0..11 {
        let bonus = enhance_array[y as usize];
        if bonus > 0 {
            if n_print == 0 { out = format!("{} +{}", get_stat_label(y as usize), bonus); }
            else { out = format!("{}, {} +{}", out, get_stat_label(y as usize), bonus); }
            n_print += 1;            
        }
        else if bonus < 0 {
            if n_print == 0 { out = format!("{} {}", get_stat_label(y as usize), bonus); }
            else { out = format!("{}, {} {}", out, get_stat_label(y as usize), bonus); }
            n_print += 1;   
        }
    }
    return out;
}
pub fn get_emblem_name(key: &str) -> String {
    let god = GodData::get(key);
    if god.is_some() { return mess_get(god.unwrap().mid); }
    else { return "".to_string(); }
}

pub fn find_emblem_stat_bonus_index(stat: i32, priority: i32) -> i32 {
    let skill_list = SkillData::get_list().unwrap();
    let mut index = -1;
    for x in 60..140 {
        let skill = &skill_list[x as usize];
        if skill.get_flag() & 1 == 0 { continue; }  // must be hidden
        let enhance = skill.get_enchance_value();
        if enhance[stat as usize] != 0 && skill.get_priority() <= priority {
            index = x as i32;
        }
    }
    return index;
}

pub fn get_stats_for_emblem(rng: &Random) -> [i32; 4] {
    let mut out: [i32; 4] = [0; 4];
    let mut used: [bool; 11] = [false; 11];
    used[9] = true; //skip sight
    let mut value = rng.get_value(11);
    let mut index = 0;
    while index < 4 {
        while used[value as usize] { value = rng.get_value(11); }
        out[index as usize] = value;
        used[value as usize] = true;
        index += 1;
    }
    return out;
}

pub fn mess_get(value: &Il2CppString) -> String { return Mess::get(value).get_string().unwrap(); }

pub fn get_random_number_for_seed() -> u32 {
    //Convet frame count to a random seed
    unsafe {
        let seed = get_frame_count(None);
        let rng = Random::get_system();
        rng.initialize(seed as u32);
        let loop_n = 5 + rng.get_value(10);
        let mut count = 0;
        let mut result = rng.value() as u32;
        while count != loop_n {
            result = rng.value() as u32;
            count += 1;
        }
        return result;
    }
}

pub fn get_weapon_mask_str(value: i32) -> String {
    let mut out = format!("");
    if 2 & value != 0 { out = format!("{} Sword", out); }
    if 4 & value != 0 { out = format!("{} Lance", out); }
    if 8 & value != 0 { out = format!("{} Axe", out); }
    if 16 & value != 0 { out = format!("{} Bow", out); }
    if 32 & value != 0 { out = format!("{} Dagger", out); }
    if 64 & value != 0 { out = format!("{} Magic", out); }
    if 128 & value != 0 { out = format!("{} Rod", out); }
    if 512 & value != 0 { out = format!("{} Fist", out); }
    if 1024 & value != 0 { out = format!("{} Special", out); }

    return out;
}
pub fn get_stat_label(index: usize) -> String {
    match index {
        0 => { return Mess::get("MID_SYS_HP").get_string().unwrap();}
        1 => { return Mess::get("MID_SYS_Str").get_string().unwrap();}
        2 => { return Mess::get("MID_SYS_Tec").get_string().unwrap();}
        3 => { return Mess::get("MID_SYS_Spd").get_string().unwrap();}
        4 => { return Mess::get("MID_SYS_Lck").get_string().unwrap();}
        5 => { return Mess::get("MID_SYS_Def").get_string().unwrap();}
        6 => { return Mess::get("MID_SYS_Mag").get_string().unwrap();}
        7 => { return Mess::get("MID_SYS_Res").get_string().unwrap();}
        8 => { return Mess::get("MID_SYS_Phy").get_string().unwrap();}
        9 => { return Mess::get("MID_SYS_Vis").get_string().unwrap();}
        10 => { return Mess::get("MID_SYS_Mov").get_string().unwrap();}
        11 => { return Mess::get("MID_SYS_Avo").get_string().unwrap(); }
        12 => { return Mess::get("MID_SYS_Crit").get_string().unwrap();}
        13 => { return Mess::get("MID_SYS_Hit").get_string().unwrap();}
        14 => { return  Mess::get("MID_SYS_Mt").get_string().unwrap(); }
        15 => { return Mess::get("MID_SYS_Secure").get_string().unwrap(); }
        16 => { return Mess::get("MID_SYS_Weight").get_string().unwrap(); } 
        _ => { return "".to_string(); }
    }
}

pub fn get_person_growth_line(person: &PersonData) -> String {
    let pid = person.pid.get_string().unwrap();
    let mut name = " -- ".to_string();
    if person.get_name().is_some() { name = get_person_name(person); }
    let grow = person.get_grow();
    return format!("{} ({})\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |", name, pid, 
    Mess::get("MID_SYS_HP").get_string().unwrap(), grow[0], Mess::get("MID_SYS_Str").get_string().unwrap(), grow[1], Mess::get("MID_SYS_Mag").get_string().unwrap(), grow[6], 
    Mess::get("MID_SYS_Tec").get_string().unwrap(), grow[2], Mess::get("MID_SYS_Spd").get_string().unwrap(), grow[3], Mess::get("MID_SYS_Lck").get_string().unwrap(), grow[4],
    Mess::get("MID_SYS_Def").get_string().unwrap(), grow[5], Mess::get("MID_SYS_Res").get_string().unwrap(), grow[7], Mess::get("MID_SYS_Phy").get_string().unwrap(), grow[8]);
}

pub fn mov_1(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap();
}

pub fn mov_x0_0(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x00,0x00, 0x80, 0x52]).unwrap();
}

pub fn return_true(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
 }
 pub fn return_4(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x80,0x00, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
 }

pub fn dlc_check() -> bool {
    unsafe {
        if has_content(0, None) {
            //mov_1(0x0253d7c0);
            //mov_1(0x0253d8b0);
        return true;
        }
        return false;
    }
}

pub fn is_valid_skill_index(index: i32 ) -> bool {
    if let Some(skill) = SkillData::try_index_get(index) {
        if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x ==  skill.parent.index).is_some() { return false; }
        if skill.help.is_none() { return false; }
        else if  Mess::get( skill.name.unwrap() ).get_string().unwrap().len() == 0 { return false; }
        if skill.name.is_none() { return false; }
        else if Mess::get( skill.help.unwrap() ).get_string().unwrap().len() == 0 { return false; }
        if skill.is_style_skill() { return false; }
        return  skill.get_flag() & 511 == 0;
    }
    return false;
}
pub fn pid_to_mpid(pid: &String) -> String { return PersonData::get(&pid).unwrap().get_name().unwrap().get_string().unwrap(); }

pub fn clamp_value(v: i32, min: i32, max: i32) -> i32 {
    unsafe { clamp(v, min, max, None)  }
}

pub fn replace_strs(this: &Il2CppString, str1: &str, str2: &str) -> &'static Il2CppString {
    unsafe {
        replace_str(this, str1.into(), str2.into(), None)
    }
}

pub fn replace_strs_il2str<'a>(this: &Il2CppString, str1: impl Into<&'a Il2CppString>, str2: impl Into<&'a Il2CppString>) -> &'static mut Il2CppString {
    unsafe {
        replace_str(this, str1.into(), str2.into(), None)
    }
}
pub fn il2_str_substring(this: &Il2CppString, start: i32) -> &'static Il2CppString {
    unsafe { sub_string(this, start, None)}
}

#[skyline::from_offset(0x032dfb20)]
pub fn clamp(value: i32, min: i32, max: i32, method_info: OptionalMethod) -> i32;

//
// Unity Functions from Engage
//DLC Check 
#[unity::from_offset("App", "DLCManager", "HasContent")]
pub fn has_content(content: i32, method_info: OptionalMethod) -> bool;

// Frame Count
#[skyline::from_offset(0x0250c6a0)]
pub fn get_frame_count(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x3784700)]
pub fn string_start_with(this: &Il2CppString, value: &Il2CppString, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x037815b0)]
pub fn sub_string(this: &Il2CppString, start: i32, method_info: OptionalMethod) -> &'static Il2CppString;

#[skyline::from_offset(0x3780700)]
pub fn is_null_empty(this: &Il2CppString, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x03773720)]
pub fn replace_str(this: &Il2CppString, old_value: &Il2CppString, new_value: &Il2CppString, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[unity::from_offset("System", "String", "Contains")]
pub fn string_contains(this: &Il2CppString, value: &Il2CppString, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "UnitPool", "GetHero")]
pub fn unit_pool_get_hero(replay :bool, method_info: OptionalMethod) -> Option<&'static Unit>;