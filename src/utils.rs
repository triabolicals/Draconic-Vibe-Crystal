use unity::prelude::*;
use engage::{
    random::*,
    force::*,
    mess::*,
    gamedata::{*, skill::*, god::*},
};
use crate::skill::STAT_BONUS;

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

pub fn str_contains(this: &Il2CppString, value: &str) -> bool { unsafe {string_contains(this, value.into(), None) } }

pub fn get_person_name(person: &PersonData) -> String {
    let name = person.get_name().unwrap();
    return mess_get(name);
}

pub fn get_skill_name(skill: &SkillData) -> String {
    if skill.name.is_some() { return format!("{} ({})", mess_get(skill.name.unwrap()), skill.sid.get_string().unwrap()); }
    else {  return format!(" --- ({})", skill.sid.get_string().unwrap()); }
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
    for x in 60..120 {
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
pub fn replace_str(this: &Il2CppString, old_value: &Il2CppString, new_value: &Il2CppString, method_info: OptionalMethod) -> &'static Il2CppString;

#[unity::from_offset("System", "String", "Contains")]
pub fn string_contains(this: &Il2CppString, value: &Il2CppString, method_info: OptionalMethod) -> bool;

