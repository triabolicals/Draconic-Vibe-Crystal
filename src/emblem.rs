use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::GameUserData,
    gamedata::{*, skill::SkillData, dispos::*},
};
use super::CONFIG;
use crate::{utils::*, enums::*};
use std::sync::Mutex;

pub mod emblem_skill;
pub mod emblem_item;

//pub static mut RANDOMIZED_INDEX: [i32; 38] = [0; 38];
pub static mut CURRENT_SEED: i32 = -1;
pub static ENGRAVE_STATS: Mutex<[i8; 132]> = Mutex::new([0; 132]);

pub static RECOMMENED_LVL: Mutex<[u8; 12]> = Mutex::new([0; 12]);
pub fn emblem_gmap_spot_adjust(){
    if !crate::utils::can_rand() { return; }
    if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return; }
        let edelgard_obtain = GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み");
        for x in 0..19 {
            let e_index = crate::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
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
                println!("Paralogue CID_{}: {} is unlocked by G_Cleared_{}: {}", cid, GameVariableManager::get_number(&gmap_spot_flag), unlock_cid, GameVariableManager::get_bool(&unlock_flag) );
            }
        }
        for x in 0..12 {
            let cid_index = crate::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
            let chapter = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize]));
            if chapter.is_none() { continue; }  //Edelgard 
            if cid_index < 12 {
                chapter.unwrap().set_recommended_level( RECOMMENED_LVL.lock().unwrap()[x as usize]);
                println!("{} set to Level {}", EMBELM_PARA[cid_index as usize], EMBELM_PARA[x as usize] );
            }
            else {
                let chapter2 = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[x as usize]));
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if chapter2.is_none() { continue; }
                if average >= RECOMMENED_LVL.lock().unwrap()[x as usize] {
                    chapter2.unwrap().set_recommended_level(RECOMMENED_LVL.lock().unwrap()[x as usize]);
                    println!("{} set to Level {}", EMBELM_PARA[x as usize], RECOMMENED_LVL.lock().unwrap()[x as usize]);
                }
                else {
                    chapter2.unwrap().set_recommended_level(average);
                    println!("{} set to Level {}", EMBELM_PARA[x as usize], average);
                }
            }
        }
        for x in 12..19 {
            let cid_index = crate::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
            let chapter = ChapterData::get_mut(&format!("CID_{}", EMBELM_PARA[cid_index as usize]));
            if chapter.is_none() { continue; } 
            if cid_index < 12 {
                let average = crate::autolevel::get_difficulty_adjusted_average_level() as u8;
                if average >= RECOMMENED_LVL.lock().unwrap()[cid_index as usize] {
                    chapter.unwrap().set_recommended_level(  RECOMMENED_LVL.lock().unwrap()[cid_index as usize] );
                    println!("{} - {} set to Level {}", x, EMBELM_PARA[cid_index as usize],  RECOMMENED_LVL.lock().unwrap()[cid_index as usize]);
                }
                else {
                    chapter.unwrap().set_recommended_level(average);
                    println!("{} - {} set to Level {}", x, EMBELM_PARA[cid_index as usize], average);
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
    let random = unsafe { EMBLEM_RANDOM };
    if !random  { 
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
        let pid = GameVariableManager::get_string(&key).get_string().unwrap();
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
        let mut emblem_count: i32 = 0;
        let mut set_emblems: [bool; 20] = [false; 20];
        match GameVariableManager::get_number("G_Emblem_Mode") {
            1 => {
                while emblem_count < emblem_list_size {
                    let index = rng.get_value(emblem_list_size);  
                    if index >= emblem_list_size { continue; }
                    if !set_emblems[index as usize] {
                        let string = format!("G_R_{}",EMBLEM_GIDS[emblem_count as usize]);
                        GameVariableManager::set_string(&string, EMBLEM_GIDS[index as usize]);
                        GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[index  as usize]), EMBLEM_GIDS[emblem_count as usize]);
                        set_emblems[index as usize] = true;
                        emblem_count += 1;
                    }
                }
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
                        println!("Loop {}, Emblem {} -> {}", x, index, custom[index]);
                        set_emblems[index] = true;
                        index = custom[index] as usize;
                    }
                    if set_emblems[index] { continue; }
                    GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[index]), EMBLEM_GIDS[x as usize]);
                    GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[x as usize]), EMBLEM_GIDS[index]);
                    println!("Emblem {} -> {}", index, x);
                    set_emblems[index] = true;
                }
            },
            _ => {},
        }
    }
    set_m022_emblem_assets();
    set_emblem_paralogue_unlock();
    GameVariableManager::set_bool("G_Random_Emblem_Set", true);
}
fn set_emblem_paralogue_unlock() {
    for x in 0..19 {
        let index = crate::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
        let string2 = format!("CID_{}",EMBELM_PARA[index as usize]);
        let chapter = ChapterData::get(&string2);
        if chapter.is_some() {
            let emblem_chapter = chapter.unwrap();
            emblem_chapter.set_gmap_open_condition(UNLOCK_PARA[x as usize]);
            if UNLOCK_PARA[index as usize] == "" { emblem_chapter.set_gmap_open_condition("G001");  }
            println!("#{} - {} gmap open condition: {}", x, string2, emblem_chapter.get_gmap_open_condition().get_string().unwrap());
        }
    }
}
pub fn set_m022_emblem_assets() {
    for x in 1..12 {
        let pid = format!("PID_M022_紋章士_{}", EMBLEM_ASSET[x]);
        let person = PersonData::get_mut(&pid).unwrap();
        let replacement_gid = GameVariableManager::get_string(&format!("G_R_GID_{}", EMBLEM_ASSET[x])).get_string().unwrap();
        let mut index = x;
        for y in 0..19 {
            if EMBLEM_GIDS[y] == replacement_gid {
                index = y;
                break;
            } 
        }
        let jid = format!("JID_紋章士_{}", EMBLEM_ASSET[index]);
        let job = JobData::get(&jid).unwrap();
        let gender = if job.unit_icon_id_m.is_some() { 1 }  else { 2};
        person.gender = gender;
        person.name = Some( format!("MPID_{}", RINGS[index]).into());
        person.jid = Some( jid.into());
    }
}
pub fn get_engage_attack_type(skill: Option<&SkillData>) -> i32 {
    if skill.is_none() { return -1; }
    let engage_attack = skill.unwrap();
    let mut engage_type = -1;
    for y in 0..21 {
        let string = format!("SID_{}", EMBLEM_ASSET[y as usize]);
        if crate::utils::str_contains(engage_attack.sid,  &string){
            engage_type = y as i32;
            break;
        }
    }
    match engage_type {
        0|2|4|5|6|11|12|16|19|20|21 => { return 0; }, //AI_AT_EngageAttack
        1 => { return 1; }, //AI_AT_EngagePierce
        7 => { return 2; }, // AI_AT_EngageVision
        9 => { return 3; }, // AI_AT_EngageDance
        10 => { return 4; }, // AI_AT_EngageOverlap
        13 => { return 5; }, // AI_AT_EngageBless
        14 => { return 6; }, // AI_AT_EngageWaitGaze
        15 => { return 7; }, // AI_AT_EngageSummon
        17 => { return 8; }, // AI_AT_EngageCamilla
        _ => { return -1; },    // None
    }
}

pub fn get_engrave_stats() {
    for x in 0..50 { if ENGRAVE_STATS.lock().unwrap()[x] != 0 { return; } }
    let mut max_engrave_stat: [i8; 6] = [0; 6];
    let mut min_engrave_stat: [i8; 6] = [100; 6];
    for x in 0..20 { 
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let index = x*6 as usize;
        ENGRAVE_STATS.lock().unwrap()[index] = god.get_engrave_avoid();
        ENGRAVE_STATS.lock().unwrap()[index + 1] = god.get_engrave_critical();
        ENGRAVE_STATS.lock().unwrap()[index + 2] = god.get_engrave_hit();
        ENGRAVE_STATS.lock().unwrap()[index + 3] = god.get_engrave_power();
        ENGRAVE_STATS.lock().unwrap()[index + 4] = god.get_engrave_secure();
        ENGRAVE_STATS.lock().unwrap()[index + 5] = god.get_engrave_weight();
        if max_engrave_stat[0] < god.get_engrave_avoid() { max_engrave_stat[0] = god.get_engrave_avoid(); }
        if max_engrave_stat[1] < god.get_engrave_critical() { max_engrave_stat[1] = god.get_engrave_critical();}
        if max_engrave_stat[2] < god.get_engrave_hit() { max_engrave_stat[2] = god.get_engrave_hit(); }
        if max_engrave_stat[3] < god.get_engrave_power() { max_engrave_stat[3] = god.get_engrave_power(); }
        if max_engrave_stat[4] < god.get_engrave_secure() { max_engrave_stat[4] = god.get_engrave_secure(); } 
        if max_engrave_stat[5] > god.get_engrave_weight() { max_engrave_stat[5] = god.get_engrave_weight(); }  
        if min_engrave_stat[0] > god.get_engrave_avoid() { min_engrave_stat[0] = god.get_engrave_avoid(); }
        if min_engrave_stat[1] > god.get_engrave_critical() { min_engrave_stat[1] = god.get_engrave_critical();}
        if min_engrave_stat[2] > god.get_engrave_hit() { min_engrave_stat[2] = god.get_engrave_hit(); }
        if min_engrave_stat[3] > god.get_engrave_power() { min_engrave_stat[3] = god.get_engrave_power(); }
        if min_engrave_stat[4] > god.get_engrave_secure() { min_engrave_stat[4] = god.get_engrave_secure(); }  
        if min_engrave_stat[5] > god.get_engrave_weight() { min_engrave_stat[5] = god.get_engrave_weight(); }  
    }
    for x in 0..5 { 
        if x == 3 { //Might
            ENGRAVE_STATS.lock().unwrap()[120+x] += 3;
            ENGRAVE_STATS.lock().unwrap()[126+x] -= 3;
        }
        else {
            ENGRAVE_STATS.lock().unwrap()[120+x] = ( max_engrave_stat[x] / 5) + 4;
            ENGRAVE_STATS.lock().unwrap()[126+x] = (min_engrave_stat[x] / 5 ) - 2;
        }
    }
    //Weight Limit from -5 to 5
    ENGRAVE_STATS.lock().unwrap()[125] = 5;
    ENGRAVE_STATS.lock().unwrap()[131] = -5;
}

pub fn randomized_engraves2(lower: i8, upper: i8) {
    if lower == upper || !crate::utils::can_rand() {
        for x in 0..20 { 
            let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
            for i in 0..6 {
                let index = ( x*6 + i ) as usize;
                let value = ENGRAVE_STATS.lock().unwrap()[index];
                god.set_engrave(i, value);
            }
        }
        println!("Engraves stats are reset");
        return;
    }
    let mut max_engrave_stat: [i8; 6] = [0; 6];
    let mut min_engrave_stat: [i8; 6] = [0; 6];
    for x in 0..6 {
        max_engrave_stat[x as usize] = ENGRAVE_STATS.lock().unwrap()[120+x];
        min_engrave_stat[x as usize] = ENGRAVE_STATS.lock().unwrap()[126+x];
    }
    println!("Engraving Score Limits: Upper: {}, Lower: {}", upper, lower);
    let rng = crate::utils::get_rng();
    for x in 0..20 { 
        let mut values: [i8; 6] = [0;6];
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let mut total = 127;
        while total < lower || total  > upper {
            for i in 0..6 {
                values[i as usize] =  if i == 3 || i == 5 { 1 } else { 5 }*rng.get_min_max( min_engrave_stat[i as usize] as i32, max_engrave_stat[i as usize ] as i32) as i8; 
            }
            total = values[0] + values[1] + values[2] + 10*values[3] + values[4] - 5*values[5];
        }
        for i in 0..6 { god.set_engrave(i,values[i as usize]); }
    }
}

pub struct RandomEmblemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_mode, 0, 3, 1);
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
            _ => { "Default recruitment order for emblems." },
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_mode  {
            1 => { "Random" },
            2 => { "Reverse" },
            3 => { "Custom" },
            _ => { "Standard"},
        }.into();
    }
}

pub struct RandomEmblemLinkMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemLinkMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
                    else { GameVariableManager::get_bool("G_EngagePlus") };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link = result; }
            else { GameVariableManager::set_bool("G_EngagePlus", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
                    else { GameVariableManager::get_bool("G_EngagePlus") };
        this.help_text = if value { "Units are linked to emblems for Engage+. (Need to Save/Reload)" }
                         else { "Units will not be linked to emblems.  (Need to Save/Reload)" }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engage_link }
                    else { GameVariableManager::get_bool("G_EngagePlus") };
        this.command_text = if value { "Randomize Links" } else { "No Random Links" }.into();
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
                            else { "Default Weapons" }.into();
    }
}
pub struct EngraveSettings;
impl ConfigBasicMenuItemSwitchMethods for EngraveSettings {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engrave_settings } else { GameVariableManager::get_number("G_EngraveSetting2") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 5, 1);
            if value != result {
                if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().engrave_settings = result;  }
                else { GameVariableManager::set_number("G_EngraveSetting2", result); }
                Self::set_command_text(this, None);
                Self::set_help_text(this, None);
                this.update_text();
                return BasicMenuResult::se_cursor();
            } else {return BasicMenuResult::new(); }
        } 
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if GameUserData::get_sequence() == 0 { this.help_text = "Sets the level of randomness for engraves".into(); }
        else {
            let current_setting = GameVariableManager::get_number("G_EngraveSetting");
            let current_str = match current_setting {
                1 => { "Low"},
                2 => { "Medium"},
                3 => { "High"},
                4 => { "Chaotic"},
                5 => { "Custom"},
                _ => { "None"},
            };
            this.help_text = format!("Engrave Random Level (Current: {}, A to confirm change)", current_str).into();
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engrave_settings }
                    else { GameVariableManager::get_number("G_EngraveSetting2") }.into();
        this.command_text = match value {
            1 => { "Low"},
            2 => { "Medium"},
            3 => { "High"},
            4 => { "Chaotic"},
            5 => { "Custom"},
            _ => { "No Randomization"},
        }.into();
    }
}
pub fn random_engrave_by_setting(setting: i32) {
    match setting {
        1 => { randomized_engraves2(0, 30); },
        2 => { randomized_engraves2(-25, 25); }, 
        3 => { randomized_engraves2(-50, 50); },
        4 => { randomized_engraves2(-100,100); },
        5 => {
            let engrave_limits = CONFIG.lock().unwrap().get_engrave_limits();
            randomized_engraves2(engrave_limits.0 as i8, engrave_limits.1 as i8);
        },
        _ => { randomized_engraves2(0, 0); },
    }
}

pub fn engrave_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("G_EngraveSetting") == GameVariableManager::get_number("G_EngraveSetting2") { return BasicMenuResult::new();}
    random_engrave_by_setting( GameVariableManager::get_number("G_EngraveSetting2") );
    GameVariableManager::set_number("G_EngraveSetting", GameVariableManager::get_number("G_EngraveSetting2"));
    let current_setting = GameVariableManager::get_number("G_EngraveSetting");
    let current_str = match current_setting {
        1 => { "Low"},
        2 => { "Medium"},
        3 => { "High"},
        4 => { "Chaotic"},
        5 => { "Custom"},
        _ => { "None"},
    };
    this.help_text = format!("Engrave Random Level (Current: {}, A to confirm change)", current_str).into();
    this.update_text();
    return BasicMenuResult::new();
}