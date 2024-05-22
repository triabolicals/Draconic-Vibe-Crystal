use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    gamedata::{*, dispos::*},
};
use super::CONFIG;
use crate::{utils::*, enums::*};

pub mod emblem_skill;
pub mod emblem_item;

pub static mut RANDOMIZED_INDEX: [i32; 38] = [0; 38];
pub static mut CURRENT_SEED: i32 = -1;

pub fn emblem_gmap_spot_adjust(){
    if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return; }
    unsafe {
        let edelgard_obtain = GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み");
        for x in 0..19 {
            let cid = EMBELM_PARA[ RANDOMIZED_INDEX [ x as usize] as usize ];
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
    }
}
pub fn randomize_emblems() {
    unsafe { 
        for i in 0..19 { 
            RANDOMIZED_INDEX[i as usize] = i; 
            RANDOMIZED_INDEX[ (i + 19) as usize] = i; 
            let string = format!("G_R_{}",EMBLEM_GIDS[i as usize]);
            GameVariableManager::make_entry_str(&string, EMBLEM_GIDS[i as usize]);
        }
        for x in 0..12 {
            let string = format!("CID_{}",EMBELM_PARA[x as usize]);
            let chapter = ChapterData::get(&string);
            if chapter.is_some() {
                let emblem_chapter = chapter.unwrap();
                println!("{} gmap open condition: {}", string, emblem_chapter.get_gmap_open_condition().get_string().unwrap());
                emblem_chapter.set_gmap_open_condition(UNLOCK_PARA[x as usize]);
            }
        }
        ChapterData::get("CID_G001").unwrap().set_gmap_open_condition("");
        for x in 13..19 {
            let string = format!("CID_{}",EMBELM_PARA[x as usize]);
            let chapter = ChapterData::get(&string);
            if chapter.is_some() {
                let emblem_chapter = chapter.unwrap();
                emblem_chapter.set_gmap_open_condition("G001");
            }
        }
    }
    if GameVariableManager::get_number("G_Emblem_Mode") == 0 {         println!("Emblem mode: 0"); return; }
    else if GameVariableManager::get_number("G_Emblem_Mode") == 1 {
        unsafe {
            println!("Emblem mode: 1");
            let mut emblem_list_size: i32 = 12;
            if has_content(0, None) { emblem_list_size = 19; }
           // let god_list = GodData::get_list_mut().unwrap();
            let rng = Random::instantiate().unwrap();
            let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
            rng.ctor(seed);
            let mut emblem_count: i32 = 0;
            let mut set_emblems: [bool; 20] = [false; 20];
            while emblem_count < emblem_list_size {
                let index = rng.get_value(emblem_list_size);
                if index >= emblem_list_size { continue; }
                if !set_emblems[index as usize] {
                    let string = format!("G_R_{}",EMBLEM_GIDS[emblem_count as usize]);
                    GameVariableManager::set_string(&string, EMBLEM_GIDS[index as usize]);
                    RANDOMIZED_INDEX[ emblem_count as usize ] = index;
                    RANDOMIZED_INDEX[ (index + 19) as usize ] = emblem_count;
                    set_emblems[ index as usize ] = true;
                    let string2 = format!("CID_{}",EMBELM_PARA[index as usize]);
                    let chapter = ChapterData::get(&string2);
                    if chapter.is_some() {
                        let emblem_chapter = chapter.unwrap();
                        emblem_chapter.set_gmap_open_condition(UNLOCK_PARA[emblem_count  as usize]);
                        if UNLOCK_PARA[index as usize] == "" {
                            emblem_chapter.set_gmap_open_condition("G001");
                        }
                        println!("{} gmap open condition: {}", string2, emblem_chapter.get_gmap_open_condition().get_string().unwrap());
                    }
                    emblem_count += 1;
                }
            }
        }
    }
    else if GameVariableManager::get_number("G_Emblem_Mode") == 2 {
        unsafe { 
            for i in 0..12 { 
                RANDOMIZED_INDEX[i as usize] = 11 - i; 
                RANDOMIZED_INDEX[ (30 - i) as usize] = i;
                let string = format!("G_R_{}",EMBLEM_GIDS[i as usize]);
                let string2 = format!("CID_{}",EMBELM_PARA[i as usize]);
                GameVariableManager::set_string(&string, EMBLEM_GIDS[ (11 - i ) as usize]);
                let chapter = ChapterData::get(&string2);
                if chapter.is_some() {
                    let emblem_chapter = chapter.unwrap();
                    emblem_chapter.set_gmap_open_condition(UNLOCK_PARA[(11 - i) as usize]);
                }
            }
        }
    }
}

pub struct RandomEmblemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_mode, 0, 2, 1);
        if CONFIG.lock().unwrap().emblem_mode != result {
            CONFIG.lock().unwrap().emblem_mode = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().emblem_mode {
            1 => { this.help_text = "Emblem recruitment will be randomized.".into(); },
            2 => { this.help_text = "Emblem recruitment will be in reversed order".into(); },
            _ => { this.help_text = "Defaut recruitment order for emblems.".into(); },
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().emblem_mode  {
            1 => { this.command_text = "Random".into(); },
            2 => { this.command_text = "Reverse".into(); },
            _ => { this.command_text = "Standard".into(); },
        }
    }
}

pub struct RandomEmblemLinkMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemLinkMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().engage_link);
        if CONFIG.lock().unwrap().engage_link != result {
            CONFIG.lock().unwrap().engage_link = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().engage_link {  this.help_text = "Playable characters will be linked to emblems for Engage+. (Togglable)".into(); }
        else { this.help_text = "Playable characters will not be linked to emblems.".into();  }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().engage_link { this.command_text = "Randomize Links".into(); }
        else { this.command_text = "No Random Links".into(); }
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
        if CONFIG.lock().unwrap().random_god_mode == 1 {  this.help_text = "Engraves and inheritiable skills are randomized".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 2 { this.help_text = "Emblem link and engage attacks are randomized.".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 3 { this.help_text = "Engrave, inheritiable skills, and Engage attacks are all randomized.".into(); }
        else { this.help_text = "No changes to emblem data.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_mode == 1 { this.command_text = "Engraves and Inherits".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 2 { this.command_text = "Link Engage Attacks".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 3 { this.command_text = "Engraves, Inherits, Engage Atk".into(); }
        else { this.command_text = "No Randomization".into(); }
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
        if CONFIG.lock().unwrap().random_god_sync_mode== 1 {  this.help_text = "Emblem stat bonuses are randomized".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 2 { this.help_text = "Emblem sync and engage skills are randomized.".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 3 { this.help_text = "Emblem stats, sync, and engage skills are randomized.".into(); }
        else { this.help_text = "No changes to sync/engage emblem data.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_sync_mode == 1 { this.command_text = "Stat Bonuses Only".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 2 { this.command_text = "Sync/Engage Skills".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 3 { this.command_text = "All Sync Data".into(); }
        else { this.command_text = "No Randomization".into(); }
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
        if CONFIG.lock().unwrap().random_engage_weapon {  this.help_text = "Engage Items/Weapons are randomized".into(); }
        else { this.help_text = "No changes to Engage items/weapons.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_engage_weapon { this.command_text = "Randomized".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}