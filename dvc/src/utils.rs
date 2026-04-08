use unity::prelude::*;
use unity::il2cpp::class::VirtualInvoke;
use engage::{
    random::*,
    force::*,
    mess::*,
    gameuserdata::GameUserData,
    gamedata::{*, terrain::TerrainData, skill::*},
};
use engage::gamevariable::GameVariableManager;
use engage::unit::UnitPool;
use skyline::patching::Patch;
use crate::config::DVCVariables;
use crate::enums::*;

/*
pub fn offset_to_addr<T: ?Sized>(offset: usize) -> &'static T {
    let s = unsafe { (skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as usize + offset) as *mut &T };
    unsafe { *s }
}
*/
pub fn get_nested_class(class: &Il2CppClass, class_name: &str) -> Option<&'static mut Il2CppClass> {
    class.get_nested_types().iter().find(|ty| ty.get_name().contains(class_name)).and_then(|ty| {
        Il2CppClass::from_il2cpptype(ty.get_type()).ok()
    })
}

pub fn get_base_classes(job: &JobData) -> Vec<&'static &'static mut JobData>{
   JobData::get_list().unwrap().iter()
        .filter(|x| x.rank == 0 && x.has_high_jobs() && x.get_high_jobs().iter().any(|j| j.parent.hash == job.parent.hash)).collect::<Vec<_>>()
}

pub fn get_total_unit_emblems(is_emblem: bool) -> i32 {
    match (dlc_check(), is_emblem) {
        (true, true) => { 19 }
        (true, false) => { 41 }
        (false, true) => { 12 }
        (false, false) => { 36 }
    }
}

pub fn get_random_and_remove<T: Clone>(vec: &mut Vec<T>, rng: &Random) -> Option<T> {
    if vec.len() == 0 { None }
    else {
        let index = rng.get_value(vec.len() as i32 ) as usize;
        let v = vec.get(index).cloned();
        if v.is_some() { vec.remove(index); }
        v
    }
}

pub fn get_rng() -> &'static Random {
    let rng = Random::instantiate().unwrap();
    rng.ctor(GameVariableManager::get_number(DVCVariables::SEED) as u32);
    rng
}


pub fn is_tile_good(tid: &Il2CppString) -> bool { TerrainData::get(&tid.to_string()).is_some_and(|f| f.prohibition == 0) }
pub fn tid_can_fly(tid: &Il2CppString) -> bool{ TerrainData::get(&tid.to_string()).is_some_and(|t|{ t.prohibition == 2 }) }
pub fn can_rand() -> bool { GameVariableManager::get_number(DVCVariables::SEED) != 0 }

pub fn create_rng(seed: i32, rng_mode: i32) -> &'static Random {
    let rng = Random::instantiate().unwrap();
    let r_seed = GameVariableManager::get_number(DVCVariables::SEED);
    let rng_seed = match rng_mode {
        1 => { ( seed >> 1 ) + ( r_seed >> 1 ) }
        2 => { ( seed >> 2) + ( r_seed >> 2)  }
        _ => { r_seed }
    };
    rng.ctor(rng_seed as u32);
    for _ in 0..rng_mode { rng.value(); }
    rng
}
pub fn lueur_on_map() -> bool {
    UnitPool::get_hero(false).filter(|f| f.force.is_some_and(|f| f.force_type < 3)).is_some()
}

pub fn is_player_unit(person: &PersonData) -> bool {
    let key = format!("G_R_{}", person.pid.to_string());
    if GameVariableManager::exist(&key) { return true; }
    let pid = person.pid.to_string();
    for x in PIDS { if *x == pid { return true; } }
    false
}

// Getting Player's name for file name
pub fn get_player_name() -> String {
    let f_type: [ForceType; 5] = [ForceType::Player, ForceType::Enemy, ForceType::Absent, ForceType::Dead, ForceType::Lost];
    for f in f_type {
        let force = Force::get(f).unwrap();
        let mut force_iter = Force::iter(force);
        while let Some(unit) = force_iter.next() {
            if unit.person.pid.to_string() == PIDS[0] {
                if unit.edit.name.is_some(){ return unit.edit.name.unwrap().to_string(); }
            }
        }
    }
    return "randomized".to_string();
}
pub fn get_lueur_name_gender(){
    GameVariableManager::make_entry(DVCVariables::LUEUR_GENDER, 0);
    GameVariableManager::make_entry(DVCVariables::LUEUR_NAME, 0);
    let f_type: [ForceType; 5] = [ForceType::Player, ForceType::Enemy, ForceType::Absent, ForceType::Dead, ForceType::Lost];
    for f in f_type {
        let force = Force::get(f).unwrap();
        let mut force_iter = Force::iter(force);
        while let Some(unit) = force_iter.next() {
            if unit.person.pid.to_string() == PIDS[0] {
                if unit.edit.name.is_some(){
                    if unit.edit.gender != 0 {
                        if unit.edit.gender > 2 { unit.edit.set_gender(1); }
                        GameVariableManager::set_number(DVCVariables::LUEUR_GENDER, unit.edit.gender);
                        GameVariableManager::set_string(DVCVariables::LUEUR_NAME, unit.edit.name.unwrap());
                        return;
                    }
                }
            }
        }
    }
}

pub fn remove_equip_emblems() {
    Force::get(ForceType::Player).unwrap().iter()
        .chain( Force::get(ForceType::Absent).unwrap().iter() )
        .chain( Force::get(ForceType::Dead).unwrap().iter() )
        .for_each(|unit| unit.clear_god_unit());
}
/*
pub fn get_nested_virtual_methods_mut(namespace: &str, class_name: &str, nested_class: &str, method_name: &str) -> Option<&'static mut VirtualInvoke> {
    if let Some(cc) = Il2CppClass::from_name(namespace, class_name).unwrap().get_nested_types().iter()
        .find(|x| x.get_name() == nested_class) {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut(method_name)
    }
    else { None }
}
pub fn get_nested_nested_virtual_method_mut(namespace: &str, class_name: &str, nested_class: &str, nested_class2: &str, method_name: &str) -> Option<&'static mut VirtualInvoke> {
    if let Some(cc) = Il2CppClass::from_name(namespace, class_name).unwrap().get_nested_types().iter()
        .find(|x| x.get_name() == nested_class).unwrap().get_nested_types().iter()
        .find(|x| x.get_name() == nested_class2) {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut(method_name)
    }
    else { None }
}
*/
pub fn get_random_number_for_seed() -> u32 {
    //Convert frame count to a random seed
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
/*
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
*/
pub fn get_stat_label(index: usize) -> String {
    match index {
        0 => { return Mess::get("MID_SYS_HP").to_string();}
        1 => { return Mess::get("MID_SYS_Str").to_string();}
        2 => { return Mess::get("MID_SYS_Tec").to_string();}
        3 => { return Mess::get("MID_SYS_Spd").to_string();}
        4 => { return Mess::get("MID_SYS_Lck").to_string();}
        5 => { return Mess::get("MID_SYS_Def").to_string();}
        6 => { return Mess::get("MID_SYS_Mag").to_string();}
        7 => { return Mess::get("MID_SYS_Res").to_string();}
        8 => { return Mess::get("MID_SYS_Phy").to_string();}
        9 => { return Mess::get("MID_SYS_Vis").to_string();}
        10 => { return Mess::get("MID_SYS_Mov").to_string();}
        11 => { return Mess::get("MID_SYS_Avo").to_string(); }
        12 => { return Mess::get("MID_SYS_Crit").to_string();}
        13 => { return Mess::get("MID_SYS_Hit").to_string();}
        14 => { return  Mess::get("MID_SYS_Mt").to_string(); }
        15 => { return Mess::get("MID_SYS_Secure").to_string(); }
        16 => { return Mess::get("MID_SYS_Weight").to_string(); } 
        _ => { return "".to_string(); }
    }
}
/*
pub fn get_person_growth_line(person: &PersonData) -> String {
    let grow = person.get_grow();
    return format!("{} ({})\n\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |",  Mess::get_name(person.pid), person.pid, 
    Mess::get("MID_SYS_HP").to_string(), grow[0], Mess::get("MID_SYS_Str").to_string(), grow[1], Mess::get("MID_SYS_Mag").to_string(), grow[6], 
    Mess::get("MID_SYS_Tec").to_string(), grow[2], Mess::get("MID_SYS_Spd").to_string(), grow[3], Mess::get("MID_SYS_Lck").to_string(), grow[4],
    Mess::get("MID_SYS_Def").to_string(), grow[5], Mess::get("MID_SYS_Res").to_string(), grow[7], Mess::get("MID_SYS_Phy").to_string(), grow[8]);
}
pub fn help_stat(person: &PersonData, second: bool) -> String {
    let grow = person.get_grow();
    if second {
        format!("{}: {}: {}% {}: {}% {}: {}% {}: {}%,", 
                Mess::get("MID_GAMESTART_GROWMODE_SELECT_TITLE"),
                Mess::get("MID_SYS_Spd"), grow[3],
                Mess::get("MID_SYS_Lck"), grow[4],
                Mess::get("MID_SYS_Def"), grow[5],
                Mess::get("MID_SYS_Res"), grow[7],
        )
    }
    else {
        format!("{} {}: {}% {}: {}% {}: {}% {}: {}%,", 
                Mess::get("MID_GAMESTART_GROWMODE_SELECT_TITLE"), 
                Mess::get("MID_SYS_HP"), grow[0], 
                Mess::get("MID_SYS_Str"), grow[1], 
                Mess::get("MID_SYS_Mag"), grow[6], 
                Mess::get("MID_SYS_Tec"), grow[2],
        )
    }
}
 */
pub fn mov_1(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap();
}

pub fn mov_x0_0(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x00,0x00, 0x80, 0x52]).unwrap();
}

pub fn mov_x0_xzr(address: usize){
    let _ = Patch::in_text(address).bytes(&[0xe0, 0x03, 0x1f, 0xaa]).unwrap();
}

pub fn return_true(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
 }
 pub fn return_4(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x80,0x00, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
 }
pub fn return_n(address: usize, value: u8){
    let div = value / 8;
    let n = (value % 8) * 32;
    let _ = Patch::in_text(address).bytes(&[n, div, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
}

pub fn dlc_check() -> bool { unsafe { has_content(0, None) }  }

pub fn clamp_value(v: i32, min: i32, max: i32) -> i32 { unsafe { clamp(v, min, max, None)  } }

pub fn replace_strs(this: &Il2CppString, str1: &str, str2: &str) -> &'static Il2CppString {
    unsafe { replace_str(this, str1.into(), str2.into(), None) }
}

pub fn replace_string(this: &Il2CppString, str1: &Il2CppString, str2: &Il2CppString) -> &'static mut Il2CppString {
    unsafe { replace_str(this, str1, str2, None) }
}
pub fn il2_str_substring(this: &Il2CppString, start: i32) -> &'static Il2CppString {
    unsafe { sub_string(this, start, None)}
}

#[skyline::from_offset(0x032dfb20)]
pub fn clamp(value: i32, min: i32, max: i32, method_info: OptionalMethod) -> i32;

pub fn max(v1: i32, v2: i32) -> i32 { if v1 > v2 { v1 } else { v2 } }
pub fn min(v1: i32, v2: i32) -> i32 { if v1 > v2 { v2 } else { v1 } }

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
