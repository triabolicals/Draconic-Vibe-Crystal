use unity::prelude::*;
use skyline::patching::Patch;
use engage::random::Random;
use engage::{
    gamevariable::*, 
    gameuserdata::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    godpool::*,
    force::*,
    gamedata::unit::*,
};
use super::CONFIG;
use crate::person;
use crate::item;
pub const EMBLEM_GIDS: &[&str] = &["GID_マルス", "GID_シグルド", "GID_セリカ", "GID_ミカヤ", "GID_ロイ", "GID_リーフ", "GID_ルキナ", "GID_リン", "GID_アイク", "GID_ベレト", "GID_カムイ", "GID_エイリーク", "GID_エーデルガルト", "GID_チキ", "GID_ヘクトル", "GID_ヴェロニカ", "GID_セネリオ", "GID_カミラ", "GID_クロム"];


// Calculate the unit's displayed rating 
pub fn get_unit_rating(this: &Unit) -> i32 {
    let mut result: i32 = 0;
    for x in 1..8 { result += this.get_capability(x as i32, false);  }
    result
}

// Generating the list of equipable emblems
pub fn get_emblem_list() -> Vec<&'static str> {
    let mut result: Vec<&str> = Vec::new();
    for x in EMBLEM_GIDS {
        let god_unit = GodPool::try_get_gid(x, true);
        if god_unit.is_some() {
            if !god_unit.unwrap().get_escape() { result.push(x); }
        }
    }
    result
}
pub fn unit_selection_menu_enable(enabled: bool) {
    if enabled { Patch::in_text(0x01d78578).bytes(&[0x20, 0x00, 0x80, 0x52]).unwrap(); }
    else { Patch::in_text(0x01d78578).bytes(&[0x80, 0x00, 0x80, 0x52]).unwrap(); }
}
pub fn emblem_selection_menu_enable(enabled: bool) {
    if enabled { Patch::in_text(0x01d76fb8).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap(); }
    else { Patch::in_text(0x01d76fb8).bytes(&[0x88, 0x00, 0x80, 0x52]).unwrap(); }
}
//Hook to function that creates the sortie deploy positions to do deployment stuff
#[unity::hook("App", "MapDispos", "CreatePlayerTeam")]
pub fn create_player_team(group: &Il2CppString, method_info: OptionalMethod){
    println!("Deploy changed start");
    if GameVariableManager::get_bool("G_Random_Recruitment"){
        person::change_map_dispos();
    }
    let absent_force = Force::get(ForceType::Absent).unwrap();
    if GameVariableManager::get_bool("G_Random_Job") && !GameVariableManager::get_bool("G_Lueur_Random") {
        let hero_unit = absent_force.get_hero_unit();
        item::unit_change_to_random_class(hero_unit);
        GameVariableManager::set_bool("G_Lueur_Random", true);
        person::adjust_unit_items(hero_unit);
    }

    call_original!(group, method_info);
    println!("Deploy changed start 2");
    if !GameVariableManager::get_bool("G_Cleared_M003") {return; }
    let player_force = Force::get(ForceType::Player).unwrap();

    let max_player = player_force.get_count();
    let mut player_count;
    let absent_count = absent_force.get_count();
    let rng = Random::get_game();
    let config = CONFIG.lock().unwrap();
    config.save();

    unsafe {
        if absent_count == 0 || GameUserData::is_evil_map() { 
            unit_selection_menu_enable(true);
            emblem_selection_menu_enable(true);
            if GameUserData::is_evil_map() { return; }
        }
        if config.emblem_deployment != 0 && config.deployment_type == 0 {
            unit_selection_menu_enable(true);
            emblem_selection_menu_enable(false);
            remove_all_rings(0, None);
            if config.emblem_deployment == 2 {
                return;
            }
            let emblem_list = get_emblem_list();
            let mut emblem_count = emblem_list.len();
            let mut set_emblems: [bool; 20] = [false; 20];
            if emblem_count > max_player as usize {
                emblem_count = max_player as usize;
            }
            let mut current_emblem_count = 0;
            let mut force_iter = Force::iter(player_force);
            while let Some(unit) = force_iter.next() {
                let mut value = rng.get_value(emblem_list.len() as i32) as usize;
                while set_emblems[value] == true {
                    value = rng.get_value(emblem_list.len() as i32) as usize;
                }
                let god_unit = GodPool::try_get_gid(emblem_list[value], true).unwrap();
                unit.set_god_unit(god_unit);
                current_emblem_count += 1;
                set_emblems[value] = true;
                if current_emblem_count == emblem_count {  
                    break;
                } 
            }
            return;
        }
        //Normal Deployment
        if config.deployment_type == 0 || absent_count == 0 {
            unit_selection_menu_enable(true);
            return;
        } 
        // Move currently deployed units to absent and then move back hero unit (Alear or Veyle)
        player_force.transfer(3, true);

        //Transfer Dead
        if config.deployment_type != 0 { Force::get(ForceType::Dead).unwrap().transfer(3, true); }

        let hero_unit = absent_force.get_hero_unit();
        hero_unit.transfer(0, true);
        hero_unit.try_create_actor();
        if !GameUserData::is_encount_map() { hero_unit.set_status(20); }
        player_count = player_force.get_count();
        //unit_update_actor(hero_unit,None);

        // Lowest Rating Deployment
        if config.deployment_type == 1 {
            unit_selection_menu_enable(false);
            emblem_selection_menu_enable(true);
            while player_count < max_player {
                let mut pid: &Il2CppString = "PID_unit".into();
                let mut mpid: &Il2CppString = "MPID_unit".into();
                let mut capability_score = 99999;
                let mut force_iter = Force::iter(absent_force);
                while let Some(unit) = force_iter.next() {
                    let cap = get_unit_rating(unit);
                    if cap < capability_score {
                        capability_score = cap;
                        pid = unit.person.pid;
                        mpid = unit.person.get_name().unwrap();
                    }
                }
                println!("{} is deployed with rating of {}", mpid.get_string().unwrap(), capability_score);
                let move_unit = force_get_unit_from_pid(pid, false, None);
                if move_unit.is_some() {
                    let unit = move_unit.unwrap();
                    unit.transfer(0, true);
                    unit.try_create_actor();
                   // unit_update_actor(unit, None);
                }
                player_count = player_force.get_count();
            }
        }
        // Random Deployment
        else if config.deployment_type == 2  {
            unit_selection_menu_enable(false);
            emblem_selection_menu_enable(true);

            while player_count < max_player {
                let rng_range = absent_force.get_count();
                let mut index = 0;
                let value = rng.get_value(rng_range);
                let mut force_iter = Force::iter(absent_force);
                while let Some(unit) = force_iter.next() {
                    if index == value {
                        unit.transfer(0, true);
                        unit.try_create_actor();
                   //     unit_update_actor(unit, None);
                        player_count = player_force.get_count();
                        break;
                    }
                    index += 1;
                }
            }
        }
        // Random Emblems
        if config.emblem_deployment != 0  {
            emblem_selection_menu_enable(false);
            remove_all_rings(0, None);
            if config.emblem_deployment == 2 {
                return;
            }
            let emblem_list = get_emblem_list();
            let mut emblem_count = emblem_list.len();
            let mut set_emblems: [bool; 20] = [false; 20];
            if emblem_count > max_player as usize {
                emblem_count = max_player as usize;
            }
            let mut current_emblem_count = 0;
            let mut force_iter = Force::iter(player_force);
            while let Some(unit) = force_iter.next() {
                let mut value = rng.get_value(emblem_list.len() as i32) as usize;
                while set_emblems[value] == true {
                    value = rng.get_value(emblem_list.len() as i32) as usize;
                }
                let god_unit = GodPool::try_get_gid(emblem_list[value], true).unwrap();
                unit.set_god_unit(god_unit);
                current_emblem_count += 1;
                set_emblems[value] = true;
                if current_emblem_count == emblem_count { break; } 
            }
        }
        else { let _ = Patch::in_text(0x01d77028).bytes(&[0xc0, 0x00, 0x00, 0x36]);}
    }
}
// Global Menu Stuff
pub struct DeploymentMod;
impl ConfigBasicMenuItemSwitchMethods for DeploymentMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){ }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().deployment_type, 0, 2, 1);
        if CONFIG.lock().unwrap().deployment_type != result {
            CONFIG.lock().unwrap().deployment_type = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().deployment_type {
            1 => { this.help_text ="Lowest rating units will be deployed.".into(); },
            2 => { this.help_text = "Units will be deployed at random.".into(); }
            _ => { this.help_text = "Normal Deployment".into(); },
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().deployment_type {
            1 => { this.command_text = "Lowest Rating".into(); },
            2 => { this.command_text = "Random".into(); },
            _ => { this.command_text = "Default".into(); },
        }
    }
}

pub struct EmblemMod;
impl ConfigBasicMenuItemSwitchMethods for EmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_deployment, 0, 2, 1);
        if CONFIG.lock().unwrap().emblem_deployment != result {
            CONFIG.lock().unwrap().emblem_deployment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().emblem_deployment {
            1 => { this.help_text = "Emblems will be randomized onto deployed units.".into();  }
            2 => { this.help_text = "Emblems will not be equipped onto units.".into(); }
            _ => { this.help_text = "Emblems are freely selectable in battle preperations.".into(); }
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        match CONFIG.lock().unwrap().emblem_deployment {
            1 => { this.command_text = "Random".into();  }
            2 => { this.command_text = "None".into(); }
            _ => { this.command_text = "Default".into(); }
        }
    }
}

#[no_mangle]
extern "C" fn deploy_create() -> &'static mut ConfigBasicMenuItem { 
    ConfigBasicMenuItem::new_switch::<DeploymentMod>("Deployment Mode")
 } 
 #[no_mangle]
extern "C" fn emblem_create() -> &'static mut ConfigBasicMenuItem {  ConfigBasicMenuItem::new_switch::<EmblemMod>("Emblem Deployment Mode") } 

 pub fn install_deployment() {
    cobapi::install_global_game_setting(deploy_create);
    cobapi::install_global_game_setting(emblem_create);
 }

#[skyline::from_offset(0x01c616f0)]
pub fn remove_all_rings(this: u64, method_info: OptionalMethod);

#[unity::from_offset("App","MapDispos", "GetSortieLimit")]
pub fn get_sortie_limit(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c54fa0)]
pub fn force_get_unit_from_pid(pid: &Il2CppString, relay: bool, method_info: OptionalMethod) -> Option<&'static Unit>;

#[skyline::from_offset(0x01a220b0)]
pub fn unit_update_actor(this: &Unit, method_info: OptionalMethod);