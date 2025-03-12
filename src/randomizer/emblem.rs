use unity::prelude::*;
use super::*;
use engage::{
    force::{ForceType, *},
    gamedata::{skill::SkillData, unit::Unit}, 
    gameuserdata::GameUserData, 
    godpool::GodPool, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult},
};
use engage::dialog::yesno::TwoChoiceDialogMethods;
use skyline::patching::Patch;
use super::CONFIG;
use super::person::pid_to_index;
use crate::utils::*;
use std::sync::Mutex;

pub mod emblem_item;
pub mod emblem_structs;
pub mod engrave;
pub mod emblem_skill;
pub mod enemy;
pub mod custom;
pub mod menuitem;

pub static EMBLEM_LIST: OnceLock<Vec<i32>> = OnceLock::new();
pub static ENEMY_EMBLEM_LIST: OnceLock<Vec<i32>> = OnceLock::new();
pub static RECOMMENED_LVL: OnceLock<Vec<u8>> = OnceLock::new();
pub static CUSTOM_EMBLEMS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

pub fn init_emblem_list() -> Vec<i32> {
    let mut list: Vec<i32> = Vec::new();
    EMBLEM_GIDS.iter().for_each(|gid| list.push(GodData::get(gid).unwrap().parent.hash));
    let mut custom_emblem = CUSTOM_EMBLEMS.lock().unwrap();
    custom_emblem.clear();
    custom_emblem.push(0);
    let mut ggids: Vec<String> = Vec::new();
    GodData::get_list().unwrap().iter()
        .filter(|god|
        {
            let gid = god.gid.to_string();
            !EMBLEM_ASSET.iter().any(|asset| gid.contains(asset)) && !gid.contains("M0") && !gid.contains("E00") && !gid.contains("GID_相手") && god.force_type == 0
        }
    ).for_each(|god|{
        if let Some(grow) = god.get_level_data() {
            let ggid = god.grow_table.unwrap().to_string();
            if grow.len() >= 20 && ggids.iter().find(|&c_ggid| *c_ggid == ggid).is_none() {
                custom_emblem[0] += 1;
                custom_emblem.push(god.parent.index);
                ggids.push(ggid);
                println!("{} Is added as custom emblem #{}", Mess::get(god.mid), custom_emblem[0]);
                list.push(god.parent.hash);
            }
        }
    });
    list
}

pub fn initialize_emblem_list() {
    EMBLEM_LIST.get_or_init(||init_emblem_list());
    RECOMMENED_LVL.get_or_init(||{
        let mut list: Vec<u8> = Vec::new();
        for x in 0..12 {
            let cid = format!("CID_{}", EMBELM_PARA[x]);
            list.push(ChapterData::get(&cid).unwrap().get_recommended_level());
        }
        list
    });
    ENEMY_EMBLEM_LIST.get_or_init(||{
        let mut list: Vec<i32> = Vec::new();
        for x in 0..20 {
            GodData::get(format!("GID_相手{}", EMBLEM_ASSET[x])).map(|god| list.push(god.parent.index));
        }
        GodData::get_list().unwrap().iter()
        .filter(|god|
        {
            let gid = god.gid.to_string();
            !EMBLEM_ASSET.iter().any(|asset| gid.contains(asset)) && gid.contains("GID_相手")
        }
        ).for_each(|god| list.push(god.parent.index));
        list
    });
    println!("Number of Enemy Emblems: {}", ENEMY_EMBLEM_LIST.get().unwrap().len());
}

pub fn emblem_gmap_spot_adjust(){
    if GameUserData::get_sequence() != 6 || !crate::utils::can_rand() { return; }
    let edelgard_obtain = GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み");
    if edelgard_obtain  {
        for x in 1..7 {
            let gmap_flag = format!("G_GmapSpot_G00{}", x);
            let flag_value = GameVariableManager::get_number(gmap_flag.as_str());
            if flag_value == 1 || flag_value == 2 {  GameVariableManager::set_number(gmap_flag.as_str(), 3);  }
        }
    }
    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 || GameVariableManager::get_bool("G_CustomEmblem") { return; }
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
    let rec_level = RECOMMENED_LVL.get().unwrap();
    for x in 0..12 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 { chapter.set_recommended_level( rec_level[x as usize]); }
            else if let Some(chapter2) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[x as usize])){
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;

                if average >= rec_level[x as usize] { 
                    chapter2.set_recommended_level(rec_level[x as usize]); 
                }
                else { 
                    chapter2.set_recommended_level(average);  
                }
            }
        }
    }
    for x in 12..19 {
        let cid_index = pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        if let Some(chapter) = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize])) {
            if cid_index < 12 {
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= rec_level[cid_index as usize] {
                    chapter.set_recommended_level(  rec_level[cid_index as usize] );
                }
                else {
                    chapter.set_recommended_level(average);
                }
            }
        }
    }
}

fn get_custom_recruitment_list() -> Vec<(i32, i32)> {   // person_x to person_y
    let mut output: Vec<(i32, i32)> = Vec::new();
    let table = custom::CUSTOM_EMBLEM_TABLE.lock().unwrap();
    let limit = if dlc_check() { 19 } else { 12 };
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

pub fn randomize_emblems() {
    if !crate::utils::can_rand() { return; }
    GameVariableManager::make_entry("G_CustomEmblem", 0);
    if !GameVariableManager::exist("G_Random_Emblem_Set") { GameVariableManager::make_entry("G_Random_Emblem_Set", 0); }
    if GameVariableManager::get_bool("G_Random_Emblem_Set") {
        set_emblem_paralogue_unlock();
        set_m022_emblem_assets();
        return; 
    }
    else {
        let rng = crate::utils::get_rng();
        let emblem_list_size = if dlc_check() { if CONFIG.lock().unwrap().dlc & 1 != 0 { 12 } else {19 } } else { 12 };
        match GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) {
            1 => {
                let mut emblem_list: Vec<usize> = (0..emblem_list_size as usize).collect();
                let emblem_i: Vec<usize> = (0..emblem_list_size as usize).collect();
                emblem_i.iter().for_each(|&x_i|{
                    if emblem_list.len() >  0 {
                        let x_j = emblem_list[ rng.get_value( emblem_list.len() as i32 ) as usize];
                        if let Some(index) = emblem_list.iter().position(|&i| i == x_j) {
                            DVCVariables::set_emblem_recruitment(x_i as i32, x_j as i32);
                            emblem_list.remove(index); 
                        }
                    }
                    else { DVCVariables::set_emblem_recruitment(x_i as i32, x_i as i32); }
                });
            },
            2 => {  // Reverse
                for i in 0..12 { DVCVariables::set_emblem_recruitment(i, 11 - i); }
            }, 
            3 => {  // Custom
                get_custom_recruitment_list().iter().for_each(|&x|{
                    DVCVariables::set_emblem_recruitment(x.0, x.1);
                    println!("Custom Emblem Order: {} -> {}", x.0, x.1);
                });
            },
            4 => {
                let mut list = EMBLEM_LIST.get().unwrap().clone();
                list.remove(19);    // Emblem Alear Removed
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
    for x in 1..12 {
        if let Some(person) = PersonData::get_mut(format!("PID_M022_紋章士_{}", EMBLEM_ASSET[x])) {
            let replacement_gid = GameVariableManager::get_string(&format!("G_R_GID_{}", EMBLEM_ASSET[x])).to_string();
            if let Some(index) = EMBLEM_GIDS.iter().position(|&gid| gid == replacement_gid) {
                let jid = format!("JID_紋章士_{}", EMBLEM_ASSET[index]);
                if let Some(job) = JobData::get(&jid){
                    let gender = if job.unit_icon_id_m.is_some() { 1 }  else { 2 };
                    person.gender = gender;
                    person.name = Some( format!("MPID_{}", RINGS[index]).into());
                    person.jid = Some( jid.into());
                }
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
        let engage_str = engage_attack.sid.to_string();
        if let Some(engage_type) = EMBLEM_ASSET.iter().position(|sid|  engage_str.contains(sid)) {
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
        let dic = GodData::get_link_dictionary();
        for x in 1..PIDS.len() {
            let person = PersonData::get(PIDS[x]).unwrap();
            if GodData::try_get_link(person).is_some() {
                dic.remove(person.pid);
                println!("Removed {} Engage+ Link", Mess::get_name(PIDS[x]));
                person.set_link_god(None);
            }
        }
    }
    if !GameVariableManager::get_bool(DVCVariables::ENGAGE_P_KEY) { return; }
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
        person.set_link_god(Some(god));
        pid_set[index] = true;
    }
}

pub fn pre_map_emblem_adjustment() {
    if !GameVariableManager::get_bool(DVCVariables::ENGAGE_P_KEY) { return; }
    for x in EMBLEM_GIDS {
        let god = GodData::get(x).unwrap();
        if let Some(god_unit) = GodPool::try_get(god, false) {
            let key = format!("E_{}", god_unit.data.gid);
            if let Some(parent) = god_unit.parent_unit {
                GameVariableManager::make_entry(key.as_str(), parent.person.parent.hash);
            }
        }
    }
}


pub fn post_map_emblem_adjustment() {
    if !GameVariableManager::get_bool(DVCVariables::ENGAGE_P_KEY) { return; }
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
            if let Some(god_unit) = GodPool::try_get(god_data, false) {
                if let Some(parent) = god_unit.parent_unit {
                    if parent.person.parent.hash == person.parent.hash { 
                        // unsafe { unit_set_engage(parent, false, None, None);}
                        god_unit_pair.push( (parent.person.parent.hash, god_data.parent.hash));
                     }
                    else if let Some(unit) = engage::unitpool::UnitPool::get_from_person_mut(person.pid, false) {
                        god_unit_pair.push( (unit.person.parent.hash, god_data.parent.hash));
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
                if let Some(unit2) = engage::unitpool::UnitPool::get_from_person_mut(unit.person.pid, false) {
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
                if let Some(unit) = engage::unitpool::UnitPool::get_from_person_mut(person.pid, false) {
                    if let Some(force) = unit.force {
                        if force.force_type == 0 || force.force_type == 3 {
                            unit.try_connect_god(god_unit);
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
                    if let Some(unit2) = engage::unitpool::UnitPool::get_from_person_mut(unit.person.pid, false) {
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
    if !GameVariableManager::get_bool(DVCVariables::EMBLEM_ITEM_KEY) { 
        call_original!(this, unit, god, bond_level, method_info);
    }
    else {
        if let Some(item) = super::job::get_weapon_for_asset_table(unit.job) {
            unit.put_off_all_item();
            unit.add_item(item);
            unit.item_list.add_item_no_duplicate(item);
            unsafe { unit_equip(unit, None); }
            unit.auto_equip();
        }
    }
}

#[skyline::from_offset(0x01a19ba0)]
fn unit_set_engage(this: &Unit, enable: bool, link: Option<&Unit>, method_info: OptionalMethod);

#[skyline::from_offset(0x01a21530)]
fn unit_equip(this: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x01dbb6c0)]
fn play_map_effect(effect: &Il2CppString, unit: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x02334b50)]
fn build_god_unit(this: &GodUnit, data: &GodData, method_info: OptionalMethod) -> &'static GodUnit;

#[unity::hook("App", "GodPool", "OnDeserialize")]
pub fn on_deserialize(this: &GodPool, stream: u64, version: i32, method_info: OptionalMethod) {
    println!("GodPool::OnDeserialize");
    call_original!(this, stream, version, method_info);
    GodData::get_list().unwrap().iter().filter(|god| god.force_type == 0)
        .for_each(|god|{
            if let Some(g_unit) = GodPool::try_get(god, true) {
                if g_unit.bonds == 0 {
                    println!("GodUnit: {} has broken bonds!", Mess::get(god.mid));
                    let escape = g_unit.get_escape();
                    let gunit2 = unsafe { build_god_unit(g_unit, god, None) };
                    let level = if DVCVariables::is_main_chapter_complete(22) {
                        if let Some(lock) = god.unlock_level_cap_flag {
                            if GameVariableManager::get_bool(lock) { 20 }
                            else { 10 }
                        }
                        else { 20 }
                    }
                    else if DVCVariables::is_main_chapter_complete(20) {
                        if let Some(lock) = god.unlock_level_cap_flag {
                            if GameVariableManager::get_bool(lock) { 15}
                            else { 10 }
                        }
                        else { 15 }
                    }
                    else if DVCVariables::is_main_chapter_complete(5) { 10 } else { 5 };
                    gunit2.set_escape(escape);
                    Force::get(ForceType::Absent).unwrap().iter().chain(  Force::get(ForceType::Player).unwrap().iter() )
                        .for_each(|unit|{
                            if let Some(g_bond) = g_unit.get_bond(unit){
                                for _x in 0..level-1 { g_bond.level_up(); }
                                unit.inherit_apt(g_unit);  
                            }
                        }  
                    );
                }
            }
        }
    );
}