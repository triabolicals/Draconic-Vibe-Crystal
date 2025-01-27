use unity::prelude::*;
use engage::{
    godpool::GodPool,
    force::*,
    gamedata::unit::Unit,
    dialog::yesno::*, force::ForceType, gamedata::{dispos::*, skill::SkillData, *}, gameuserdata::GameUserData, gamevariable::*, menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult}
};
use engage::dialog::yesno::TwoChoiceDialogMethods;
use skyline::patching::Patch;

use super::CONFIG;
use super::person::pid_to_index;
use crate::{enums::*, utils::*};
use std::sync::Mutex;

pub mod emblem_item;
pub mod emblem_structs;
pub mod engrave;
pub mod emblem_skill;
pub mod enemy;
//pub static mut RANDOMIZED_INDEX: [i32; 38] = [0; 38];
pub static mut CURRENT_SEED: i32 = -1;
pub static ENGRAVE_STATS: Mutex<[i8; 132]> = Mutex::new([0; 132]);
pub static mut EMBLEM_LIST: Vec<i32> = Vec::new();
pub static RECOMMENED_LVL: Mutex<[u8; 12]> = Mutex::new([0; 12]);
pub static CUSTOM_EMBLEMS: Mutex<[i32; 21]> = Mutex::new([-1; 21]);

pub fn get_god_from_index(index: i32, randomized: bool) -> Option<&'static GodData> {
    if index as usize >=  unsafe { EMBLEM_LIST.len() } { return None; }
    let hash = unsafe { EMBLEM_LIST[index as usize] };
    if GameVariableManager::get_number("G_Emblem_Mode") == 0 || !randomized { return GodData::try_get_hash(hash);  }
    else { 
        let key = format!("G_R_{}", GodData::try_get_hash(hash).unwrap().gid);
        return GodData::get(GameVariableManager::get_string(key));
    }
}

pub fn get_custom_emblems() {
    let emblem_list = unsafe { &mut EMBLEM_LIST };
    if emblem_list.len() > 0 { return; }
    let mut count = 0;

    EMBLEM_GIDS.iter()
        .for_each(|gid|{
            let hash = GodData::get(gid).unwrap().parent.hash;
            emblem_list.push(hash);
        }
    );

    if CUSTOM_EMBLEMS.lock().unwrap()[0] != -1 { return; }
    GodData::get_list().unwrap().iter()
        .filter(|god|
            {
                let gid = god.gid.to_string();
                !EMBLEM_ASSET.iter().any(|asset| gid.contains(asset)) && !gid.contains("M0") && !gid.contains("E00") && !gid.contains("GID_相手") && god.force_type == 0
            }
        ).for_each(|god|{
            if let Some(grow) = god.get_level_data() {
                if grow.len() >= 20 {
                    println!("Custom Emblem {}: Adding Emblem #{}: {}", count+1, god.parent.index, Mess::get(god.mid));
                    if count < 20 {
                        count += 1;
                        CUSTOM_EMBLEMS.lock().unwrap()[0] = count;
                        CUSTOM_EMBLEMS.lock().unwrap()[count as usize] = god.parent.index as i32; 
                        emblem_list.push(god.parent.hash);
                    }
                }
            }
        }
    );
}

pub fn emblem_gmap_spot_adjust(){
    if GameUserData::get_sequence() != 6 || !crate::utils::can_rand() { return; }
    if GameVariableManager::get_bool("G_CustomEmblem") { return; }  // Ignore if custom emblems
    let edelgard_obtain = GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み");
    if edelgard_obtain  {
        for x in 1..7 {
            let gmap_flag = format!("G_GmapSpot_G00{}", x);
            let flag_value = GameVariableManager::get_number(&gmap_flag);
            if flag_value == 1 || flag_value == 2 {  GameVariableManager::set_number(&gmap_flag, 3);  }
        }
    }
    if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return; }
    for x in 0..19 {
        let e_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        let cid = EMBELM_PARA[ e_index as usize ];
        let unlock_cid = UNLOCK_PARA[ x as usize]; 
        if cid == "G007" { continue; }  //There's no Edelgard paralogue to unlock
        if unlock_cid == "" {  // open tiki's divine paralogue if edelgard ring is obtained and unlock the emblem paralogue that replaces edelgard
            let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
            if edelgard_obtain {
                if GameVariableManager::get_number("G_GmapSpot_G001") == 1 || GameVariableManager::get_number("G_GmapSpot_G001") == 2  {  GameVariableManager::set_number("G_GmapSpot_G001", 3);  }
                if GameVariableManager::get_number(&gmap_spot_flag) == 1 {  GameVariableManager::set_number(&gmap_spot_flag, 3); }
            }
            else { 
                GameVariableManager::set_number("G_GmapSpot_G001", 1); 
                GameVariableManager::set_number(&gmap_spot_flag, 1); 
            }
            continue;
        }
        if cid.starts_with("G") {               // divine paralogue opened by edelgard ring
            if edelgard_obtain {
                let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
                if GameVariableManager::get_number(&gmap_spot_flag) != 3 {  GameVariableManager::set_number(&gmap_spot_flag, 3);  }
            }
        }
        else {
            let unlock_flag = format!("G_Cleared_{}", unlock_cid);
            let gmap_spot_flag = format!("G_GmapSpot_{}", cid);
            if GameVariableManager::get_bool(&unlock_flag) {
                if GameVariableManager::get_number(&gmap_spot_flag) <= 2 { GameVariableManager::set_number(&gmap_spot_flag, 3); }
            }
            else { GameVariableManager::set_number(&gmap_spot_flag, 1); }
            //println!("Paralogue CID_{}: {} is unlocked by G_Cleared_{}: {}", cid, GameVariableManager::get_number(&gmap_spot_flag), unlock_cid, GameVariableManager::get_bool(&unlock_flag) );
        }
    }
    //Calculating Recommended Level
    for x in 0..12 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 {
                chapter.set_recommended_level( RECOMMENED_LVL.lock().unwrap()[x as usize]);
                println!("{} set to Level {}", EMBELM_PARA[cid_index as usize], EMBELM_PARA[x as usize] );
            }
            else if let Some(chapter2) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[x as usize])){
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= RECOMMENED_LVL.lock().unwrap()[x as usize] {
                    chapter2.set_recommended_level(RECOMMENED_LVL.lock().unwrap()[x as usize]);
                    println!("{} set to Level {}", EMBELM_PARA[x as usize], RECOMMENED_LVL.lock().unwrap()[x as usize]);
                }
                else {
                    chapter2.set_recommended_level(average);
                    println!("{} set to Level {}", EMBELM_PARA[x as usize], average);
                }
            }
        }
    }
    for x in 12..19 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 {
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= RECOMMENED_LVL.lock().unwrap()[cid_index as usize] {
                    chapter.set_recommended_level(  RECOMMENED_LVL.lock().unwrap()[cid_index as usize] );
                    println!("{} - {} set to Level {}", x, EMBELM_PARA[cid_index as usize],  RECOMMENED_LVL.lock().unwrap()[cid_index as usize]);
                }
                else {
                    chapter.set_recommended_level(average);
                    println!("{} - {} set to Level {}", x, EMBELM_PARA[cid_index as usize], average);
                }
            }
        }

    }
}
pub fn get_recommended_paralogue_levels() {
    for x in 0..12 {
        let cid = format!("CID_{}", EMBELM_PARA[x]);
        let level = ChapterData::get(&cid).unwrap().get_recommended_level();
        RECOMMENED_LVL.lock().unwrap()[x] = level;
    }
}

fn get_custom_recruitment_list() -> [i32; 19] {
    let mut output: [i32; 19] = [-1; 19];
    let mut set: [bool; 19] = [false; 19];
    let length = crate::enums::SET_RECRUITMENT.lock().unwrap().len();
    for x in 0..length {
        let value = crate::enums::SET_RECRUITMENT.lock().unwrap()[x as usize];
        if !value.2 { continue; } // emblem
        let index_1 = value.0;
        let index_2 = value.1;
        if output[index_1 as usize] == -1 && !set[index_2 as usize] { 
            output[index_1 as usize] = index_2; 
            set[index_2 as usize] = true;
        }
    }
    if unsafe { !EMBLEM_RANDOM }  { 
        for x in 0..19 {
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
        let person_pool = if dlc_check() { 19 } else { 12 };
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
fn create_reverse_emblem() {
    for x in 0..19 {
        let key = format!("G_R_{}",EMBLEM_GIDS[x as usize]);
        let pid = GameVariableManager::get_string(&key).to_string();
        for y in 0..19 {
            if pid == EMBLEM_GIDS[y as usize] {
                GameVariableManager::make_entry_str(&format!("G_R2_{}",EMBLEM_GIDS[y as usize]), EMBLEM_GIDS[x as usize]);
                GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[y as usize]), EMBLEM_GIDS[x as usize]);
            }
        }
    }
}
pub fn randomize_emblems() {
    if !crate::utils::can_rand() { return; }
    GameVariableManager::make_entry("G_CustomEmblem", 0);
    if !GameVariableManager::exist("G_Random_Emblem_Set") { GameVariableManager::make_entry("G_Random_Emblem_Set", 0); }
    if GameVariableManager::get_bool("G_Random_Emblem_Set") {
        set_emblem_paralogue_unlock();
        set_m022_emblem_assets();
        return; 
    }
    if GameVariableManager::exist(&format!("G_R_{}",EMBLEM_GIDS[0])){
        if !GameVariableManager::exist(&format!("G_R2_{}",EMBLEM_GIDS[0])) { create_reverse_emblem(); }
    }
    else {
        for i in 0..19 { 
            GameVariableManager::make_entry_str(&format!("G_R_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[i as usize]);
            GameVariableManager::make_entry_str(&format!("G_R2_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[i as usize]);
        }
        let rng = crate::utils::get_rng();
        let emblem_list_size = if dlc_check() { 19 } else { 12 };
        let mut set_emblems: [bool; 20] = [false; 20];
        match GameVariableManager::get_number("G_Emblem_Mode") {
            1 => {
                let mut emblem_list: Vec<usize> = (0..emblem_list_size as usize).collect();
                EMBLEM_GIDS.iter().for_each(|&gid_i|{
                    if emblem_list.len() !=  0 {
                        let x_j = emblem_list[ rng.get_value( emblem_list.len() as i32 ) as usize];
                        GameVariableManager::set_string(&format!("G_R_{}", gid_i), EMBLEM_GIDS[x_j]);
                        GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[x_j]), gid_i.into());
                        if let Some(index) = emblem_list.iter().position(|&i| i == x_j) {  emblem_list.remove(index);  }
                    }
                    else {
                        GameVariableManager::set_string(&format!("G_R_{}", gid_i), gid_i);
                        GameVariableManager::set_string(&format!("G_R2_{}", gid_i), gid_i);
                    }
                });
            },
            2 => {  // Reverse
                for i in 0..12 {
                    GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[ (11 - i ) as usize]);
                    GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[(11 - i ) as usize]), EMBLEM_GIDS[ i as usize]);
                }
            }, 
            3 => {  // Custom
                let custom = get_custom_recruitment_list();
                for x in 0..19 {
                    let mut index = x as usize; 
                    if set_emblems[index] { continue; }
                    while custom[index] != 0 {
                        if set_emblems[index] { break; }
                        GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[index]), EMBLEM_GIDS[custom[index] as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[custom[index] as usize]), EMBLEM_GIDS[index]);
                        set_emblems[index] = true;
                        index = custom[index] as usize;
                    }
                    if set_emblems[index] { continue; }
                    GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[index]), EMBLEM_GIDS[x as usize]);
                    GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[x as usize]), EMBLEM_GIDS[index]);
                    set_emblems[index] = true;
                }
            },
            4 => {
                let mut list = unsafe { EMBLEM_LIST.clone() };
                let mut available = list.clone();
                if !dlc_check() {
                    available.drain(12..19);
                    list.drain(12..19);
                }
                list.iter()
                    .for_each(|&hash|{
                        let gid = GodData::try_get_hash(hash).unwrap().gid;
                        let key = format!("G_R_{}", gid);
                        if available.len() != 0 {
                            let index = rng.get_value(available.len() as i32) as usize;
                            let gid2 = GodData::try_get_hash( available[ index ] ).unwrap().gid;
                            if GameVariableManager::exist(key.as_str()) { GameVariableManager::set_string(key.as_str(), gid2.to_string().as_str()); }
                            else { GameVariableManager::make_entry_str(key.as_str(), gid2.to_string().as_str()); }
                            available.remove(index);
                        }
                    }
                );
                GameVariableManager::set_bool("G_CustomEmblem", true);
            },
            _ => {},
        }
    }
    set_m022_emblem_assets();
    set_emblem_paralogue_unlock();
    GameVariableManager::set_bool("G_Random_Emblem_Set", true);
}
fn set_emblem_paralogue_unlock() {
    if GameVariableManager::get_bool("G_CustomEmblem") { return; } 
    for x in 0..19 {
        let index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        let string2 = format!("CID_{}",EMBELM_PARA[index as usize]);
        if let Some(emblem_chapter) = ChapterData::get(&string2){
            emblem_chapter.set_gmap_open_condition(UNLOCK_PARA[x as usize]);
            if UNLOCK_PARA[index as usize] == "" { emblem_chapter.set_gmap_open_condition("G001");  }
            // println!("#{} - {} gmap open condition: {}", x, string2, emblem_chapter.get_gmap_open_condition().to_string());
        }
    }
}
pub fn set_m022_emblem_assets() {
    // if GameVariableManager::get_bool("G_CustomEmblem") { return; } 
    for x in 1..12 {
        if let Some(person) = PersonData::get_mut(format!("PID_M022_紋章士_{}", EMBLEM_ASSET[x])) {
            let replacement_gid = GameVariableManager::get_string(&format!("G_R_GID_{}", EMBLEM_ASSET[x])).to_string();
            if let Some(index) = EMBLEM_GIDS.iter().position(|&gid| gid == replacement_gid) {
                let jid = format!("JID_紋章士_{}", EMBLEM_ASSET[index]);
                let job = JobData::get(&jid).unwrap();
                let gender = if job.unit_icon_id_m.is_some() { 1 }  else { 2 };
                person.gender = gender;
                person.name = Some( format!("MPID_{}", RINGS[index]).into());
                person.jid = Some( jid.into());
            }
            else {
                if let Some(god) = GodData::get(replacement_gid) {
                    person.name = Some(god.mid.clone());
                    person.gender = if god.female == 2 { 2 } else { 1 };
                }
            }
        }
    }
}
pub fn get_engage_attack_type(skill: Option<&SkillData>) -> i32 {
    if let Some(engage_attack) = skill {
        if let Some(engage_type) = EMBLEM_ASSET.iter().position(|sid| engage_attack.sid.contains(sid)) {
            match engage_type {
                0|2|4|5|6|11|12|16|18|19|20|21 => { return 0; }, //AI_AT_EngageAttack
                1 => { return 1; }, //AI_AT_EngagePierce
                3 => { return 9; }, //AI_AT_Versus
                7 => { return 2; }, // AI_AT_EngageVision
                8 => { return 10; }, // AI_AT_EngageWait
                9 => { return 3; }, // AI_AT_EngageDance
                10 => { return 4; }, // AI_AT_EngageOverlap
                13 => { return 5; }, // AI_AT_EngageBless
                14 => { return 6; }, // AI_AT_EngageWaitGaze
                15 => { return 7; }, // AI_AT_EngageSummon
                17 => { return 8; }, // AI_AT_EngageCamilla
                _ => { return -1; },    // None
            }
        }
    }
    -1
}

pub fn randomize_engage_links(reset: bool) {
    if reset || !crate::utils::can_rand() {
        let dummy_god = GodData::get("GID_マルス").unwrap();
        // To remove links from the dictionary, by on_release
        for x in 1..PIDS.len() {
            let person = PersonData::get(PIDS[x]).unwrap();
            if GodData::try_get_link(person).is_some() {
                dummy_god.set_link(PIDS[x].into());
                dummy_god.on_release();
            }
        }
        dummy_god.set_link("".into());
    }
    if !GameVariableManager::get_bool("G_EngagePlus") { return; }
    let mut pid_set: [bool; 41] = [false; 41];
    pid_set[0] = true;
    let rng = get_rng();
    let dic = GodData::get_link_dictionary();
    Patch::in_text(0x01dc9f8c).bytes(&[0x20, 0x00, 0x80, 0x52]).unwrap();     // God Exp bypass check
    let emblem_count;
    let person_count;
    if unsafe { has_content(0, None) } {
        emblem_count = 19;
        person_count = 41;
    }
    else {
        emblem_count = 12;
        person_count = 36;
    }

    for x in 0..emblem_count {
        if x == 13 { continue; }    // skip tiki, causes crashes :(
        let gid = format!("GID_{}", EMBLEM_ASSET[x as usize]);
        let god = GodData::get(&gid).unwrap();
        let mut index: usize = rng.get_value(person_count as i32) as usize;
        let mut pid = PIDS[index];

        while pid_set[index] || GodData::try_get_link(&PersonData::get(pid).unwrap()).is_some() {
            index = rng.get_value(person_count as i32) as usize;
            pid = PIDS[index];
        }
        unsafe { super::LINKED[ x as usize ] = index as i32; }
        let person = PersonData::get(&pid).unwrap();
        dic.add(PIDS[index].into(), god);
        person.on_complete();
        pid_set[index] = true;
    }
}
use engage::mess::*;

pub fn pre_map_emblem_adjustment() {
    if !GameVariableManager::get_bool("G_EngagePlus") { return; }
    for x in EMBLEM_GIDS {
        let god = GodData::get(x).unwrap();
        if let Some(god_unit) =GodPool::try_get(god, false) {
            let key = format!("E_{}", god_unit.data.gid);
            if let Some(parent) = god_unit.parent_unit {
                println!("{}'s parent is {}",  Mess::get(god.mid), Mess::get_name(parent.person.pid));
                GameVariableManager::make_entry(key.as_str(), parent.person.parent.hash);
            }
        }
    }
}


pub fn post_map_emblem_adjustment() {
    if !GameVariableManager::get_bool("G_EngagePlus") { return; }
    let mut god_unit_pair: Vec<(i32, i32)> = Vec::new();

    let variables = GameVariableManager::find_starts_with("E_GID");
    let god_hashes: Vec<_> = variables.iter().map(|key| {
        let gid_key = key.to_string();
        let gid = &gid_key.as_str()[2..];
        GodData::get(gid).unwrap().parent.hash
    }).collect();
    // Adding any emblems obtained during the map and the current unit that has it
    EMBLEM_GIDS.iter()
        .for_each(|gid|{
            let god = GodData::get(gid).unwrap();
            if let Some(god_unit) = GodPool::try_get(god, false) {
                if let Some(parent) = god_unit.parent_unit {
                    if !god_hashes.iter().any(|&hash| hash == god.parent.hash) {
                        god_unit_pair.push( (parent.person.parent.hash, god.parent.hash) );
                        println!("Adding 1 {} to {}", Mess::get_name(parent.person.pid), Mess::get(god.mid));
                    }
                }
            }
        }
    );
    // Adding any emblem set from the start of the map
    variables.iter()
        .for_each(|key|{
            let gid_key = key.to_string();
            let person = PersonData::try_get_hash(GameVariableManager::get_number(gid_key.as_str())).unwrap();
            let gid = &gid_key.as_str()[2..];
            let god_data = GodData::get(gid).unwrap();
            println!("Variable: {} and {}", Mess::get_name(person.pid), Mess::get(god_data.mid));
            if let Some(god_unit) = GodPool::try_get(god_data, false) {
                if let Some(parent) = god_unit.parent_unit {
                    if parent.person.parent.hash == person.parent.hash { 
                        println!("Adding 2 {} to {}", Mess::get_name(parent.person.pid), Mess::get(god_data.mid));
                        // unsafe { unit_set_engage(parent, false, None, None);}
                        god_unit_pair.push( (parent.person.parent.hash, god_data.parent.hash));
                     }
                    else if let Some(unit) = unsafe { crate::deployment::force_get_unit_from_pid(person.pid, false, None) } {
                        god_unit_pair.push( (unit.person.parent.hash, god_data.parent.hash));
                        println!("Adding 3 {} to {}", Mess::get_name(unit.person.pid), Mess::get(god_data.mid));
                    }
                }
            }
        }
    );
    let force_type = [ForceType::Player, ForceType::Ally, ForceType::Dead];
    // Removing all Emblems from all units
    for f in force_type {
        if let Some(force) = Force::get(f){
            let iter = Force::iter(force);
            for unit in iter {
                if let Some(unit2) = unsafe { crate::deployment::force_get_unit_from_pid(unit.person.pid, false, None) } {
                    unit2.god_link = None;
                    unit2.god_unit = None;
                    if unit2.status.value & 0x800000 != 0 { unit2.status.value -= 0x800000; }
                    unit.clear_parent();
                    unit.update();
                }
            }
        }
    }
    god_unit_pair.iter()
        .for_each(|pair|{
            let person = PersonData::try_get_hash(pair.0).unwrap();
            let god = GodData::try_get_hash(pair.1).unwrap();
            if let Some(god_unit) =  GodPool::try_get(god, false) {
                god_unit.set_parent(None, 0);
                god_unit.set_child(None);
                if let Some(unit) = unsafe { crate::deployment::force_get_unit_from_pid(person.pid, false, None) } {
                    if let Some(force) = unit.force {
                        if force.force_type == 0 || force.force_type == 3 {
                            unit.try_connect_god(god_unit);
                            println!("Connecting {} to {}", Mess::get_name(unit.person.pid), Mess::get(god.mid));
                        }
                    }
                }
            }
        }
    );
}
pub fn player_emblem_check() {
    if let Some(force) = Force::get(ForceType::Player){
        for unit in Force::iter(force) {
            // if unit.status.value & 0x800000 != 0 { println!("Person {} is engaged", Mess::get_name(unit.person.pid)); }
            if let Some(god_unit) = unit.god_unit {
               // println!("{} equipped with Emblem {}", Mess::get_name(unit.person.pid), emblem);
              //  if let Some(parent) = god_unit.parent_unit { println!("{}'s Parent Unit: {}", emblem, Mess::get_name(parent.person.pid)); }
                if god_unit.child.is_none() && god_unit.parent_unit.is_none() {
                    god_unit.set_parent(Some(unit), 0);
                    // println!("Set {}'s Parent to {}", emblem, Mess::get_name(unit.person.pid));
                }
            }
            if let Some(god_link) = unit.god_link {
                if god_link.child.is_none() {
                    // println!("{} Linked with Emblem {} but no child", Mess::get_name(unit.person.pid), emblem);
                    if let Some(unit2) = unsafe { crate::deployment::force_get_unit_from_pid(unit.person.pid, false, None) } {
                        unit2.god_link = None;
                        unit2.status.value &= !0x800000;
                        unsafe { play_map_effect("エンゲージOff".into(), unit2, None); }
                        unit2.reload_actor();
                    }
                }
            }
        }
    }
}
#[unity::hook("App", "ArenaOrderSequence", "SetEmblemWeapon")]
pub fn arena_emblem_weapon(this: u64, unit: &mut Unit, god: &engage::gamedata::unit::GodUnit, bond_level: i32, method_info: OptionalMethod) {
    if !GameVariableManager::get_bool("G_Random_Engage_Weps") { 
        call_original!(this, unit, god, bond_level, method_info);
        return;
    }
    println!("Emblem Job: {}", Mess::get(unit.job.name));
    if let Some(item) = super::job::get_weapon_for_asset_table(unit.job) {
        println!("Item: {}", Mess::get(item.name));
        unit.put_off_all_item();
        unit.add_item(item);
        unit.item_list.add_item_no_duplicate(item);
        unsafe { unit_equip(unit, None); }
        unit.auto_equip();
    }
}
#[skyline::from_offset(0x01a19ba0)]
fn unit_set_engage(this: &Unit, enable: bool, link: Option<&Unit>, method_info: OptionalMethod);
#[skyline::from_offset(0x01a21530)]
fn unit_equip(this: &Unit, method_info: OptionalMethod);
#[skyline::from_offset(0x01dbb6c0)]
fn play_map_effect(effect: &Il2CppString, unit: &Unit, method_info: OptionalMethod);
// Menu Items
pub struct RandomEmblemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let max = if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 4 } else { 3 };
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_mode, 0, max, 1);
        if CONFIG.lock().unwrap().emblem_mode != result {
            CONFIG.lock().unwrap().emblem_mode = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().emblem_mode {
            1 => { "Emblem recruitment will be randomized." },
            2 => { "Emblem recruitment will be in reversed order" },
            3 => { "Emblem recruitment will determined by list."},
            4 => { "Random recruitment with custom emblems."},
            _ => { "Default recruitment order for emblems." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_mode  {
            1 => { "Random" },
            2 => { "Reverse" },
            3 => { "Custom" },
            4 => { "Custom Emblems"},
            _ => { "Standard"},
        }.into();
    }
}

pub struct RandomEmblemLinkMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemLinkMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("EngagePlus", GameVariableManager::get_number("G_EngagePlus"));
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link = result; }
            else { GameVariableManager::set_bool("EngagePlus", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        this.help_text = if value { "Units are linked to emblems for Engage+." }
            else { "Units will not be linked to emblems." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
            else { GameVariableManager::get_bool("EngagePlus") };

        this.command_text = if value { "Random Links" } else { "No Links" }.into();
    }
}

pub struct RandomGodMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGodMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_mode != result {
            CONFIG.lock().unwrap().random_god_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_god_mode {
            1 => { "Inheritiable skills will be randomized."},
            2 => { "Engage Attacks and Linked Engage Attacks will be randomized." },
            3 => { "Inheritiable skills and Engage Attacks will be randomized." },
            _ => { "No Randomization to emblem data."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_god_mode {
            1 => { "Skill Inheritance" },
            2 => { "Engage Atks" },
            3 => { "Inherits/Engage Atks"},
            _ => { "None" },
        }.into();
    }
}
pub struct RandomSynchoMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSynchoMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_sync_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_sync_mode != result {
            CONFIG.lock().unwrap().random_god_sync_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().random_god_sync_mode {
            1 => { "Emblem stat bonuses are randomized." },
            2 => { "Emblem sync and engage skills are randomized." },
            3 => { "Emblem stats, sync, and engage skills are randomized." },
            _ => { "No changes to sync/engage emblem data."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().random_god_sync_mode {
            1 => { "Stat Bonuses" },
            2 => { "Sync/Engage Skills" },
            3 => { "All Sync"},
            _ => { "None"},
        }.into();
    }
}
pub struct RandomEngageWepMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEngageWepMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_engage_weapon);
        if CONFIG.lock().unwrap().random_engage_weapon != result {
            CONFIG.lock().unwrap().random_engage_weapon  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_engage_weapon {  "Engage Items/Weapons are randomized"  }
            else { "No changes to Engage items/weapons." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_engage_weapon {  "Randomize Weapons" }
            else { "Default Item/Weapons" }.into();
    }
}

pub struct EngageLinkConfirm;
impl TwoChoiceDialogMethods for EngageLinkConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomize_engage_links(true);
        GameVariableManager::set_number("G_EngagePlus", GameVariableManager::get_number("EngagePlus"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        RandomEmblemLinkMod::set_help_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn engage_link_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("G_EngagePlus") == GameVariableManager::get_number("EngagePlus") { return BasicMenuResult::new();}
    YesNoDialog::bind::<EngageLinkConfirm>(this.menu, "Change Engage Link Settings?", "Do it!", "Nah..");
    return BasicMenuResult::new();
}

pub extern "C" fn vibe_engage_links() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<RandomEmblemLinkMod>("Unit-Emblem Links");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = engage_link_acall as _ );
    switch
}