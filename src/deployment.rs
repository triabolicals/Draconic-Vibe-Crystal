use unity::{il2cpp::class::Il2CppRGCTXData, prelude::*};
use skyline::patching::Patch;
pub use engage::{
    random::Random,
    gamevariable::*, 
    gameuserdata::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    godpool::*,
    force::*,
    script::*,
    singleton::SingletonClass,
    gamedata::{*, GodData, unit::*},
};
use unity::system::List;
use unity::il2cpp::object::Array;
use super::CONFIG;
use crate::enums::*;

pub mod fulldeploy;
pub mod inspectors;

pub struct DeploymentMod;
impl ConfigBasicMenuItemSwitchMethods for DeploymentMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){ }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 4, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().deployment_type = result; }
            else {  GameVariableManager::set_number("G_DeploymentMode", result); } 
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let deploy_type =   
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        this.help_text = match deploy_type {
            1 => { "Lowest rating units will be deployed." },
            2 => { "Units will be deployed at random." }
            3 => { "No forced deployment restrictions."},
            4 => { "Full or more deployment slots if map allows it."}
            _ => { "Normal Deployment"},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let deploy_type =   
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number("G_DeploymentMode") };
        this.command_text = match deploy_type { 
            1 => { "Lowest Rating" },
            2 => { "Random" },
            3 => { "Free"},
            4 => { "Full / Expand"},
            _ => { "Default" },
        }.into();
    }
}

pub extern "C" fn vibe_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<DeploymentMod>("Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map2 as _);
    switch
} 

pub struct EmblemMod;
impl ConfigBasicMenuItemSwitchMethods for EmblemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment = result; }
            else { GameVariableManager::set_number("G_EmblemDeployMode", result) };
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        this.help_text = match emblem_deployment {
            1 => { "Emblems will be randomized onto deployed units." },
            2 => { "Emblems will not be equipped onto units." },
            _ => { "Emblems are freely selectable in battle preperations."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number("G_EmblemDeployMode") };
        this.command_text = match emblem_deployment { 
            1 => { "Random Emblems" },
            2 => { "No Emblems" },
            _ => { "Default"},
        }.into();
    }
}

pub extern "C" fn vibe_emblem_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<EmblemMod>("Emblem Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map2 as _);
    switch
} 

// Calculate the unit's displayed rating 
pub fn get_unit_rating(this: &Unit) -> i32 {
    let mut result: i32 = 0;
    for x in 1..9 { result += this.get_capability(x as i32, false);  }
    result
}

pub fn unit_status() {
    let emblem_lueur = GodPool::try_get_gid("GID_リュール", false);
    let has_emblem_lueur = emblem_lueur.is_some();
    let dead_force = Force::get(ForceType::Dead).unwrap();
    let mut force_iter = Force::iter( dead_force  );
    while let Some(unit) = force_iter.next() {
        if unit.person.pid.to_string() == "PID_リュール" && has_emblem_lueur {
            let god_lueur = GodData::get("GID_リュール").unwrap();
            if god_lueur.get_flag().value & -2147483648 != 0 {
                god_lueur.get_flag().value -= -2147483648;  //Alear is now an equippable ring due to death
            }
        }
    }
}
// Generating the list of equipable emblems
pub fn get_emblem_list() -> Vec<&'static str> {
    let mut result: Vec<&str> = Vec::new();
    for x in EMBLEM_GIDS {
        let god_unit = GodPool::try_get_gid(x, true);
        if god_unit.is_some() { if !god_unit.unwrap().get_escape() { result.push(x); } }
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

pub fn get_emblem_paralogue_level() {
    if !crate::utils::can_rand() { return; }
    if !GameVariableManager::get_bool("G_CustomEmblem") { return; }
    let cid = GameUserData::get_chapter().get_prefixless_cid().to_string();
    GameVariableManager::make_entry("G_Paralogue_Level", 0);

    let e_index = EMBELM_PARA.iter().position(|&x| x == cid);
    if e_index.is_none() { return; }
    let emblem_index = e_index.unwrap();
    let found = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[emblem_index as usize].to_string(), true);
    let new_emblem_index;
    if found != -1 { new_emblem_index = found;  }
    else { return; }
    let level_difference;
    if new_emblem_index >= 12 {
        let party_average = crate::autolevel::get_difficulty_adjusted_average_level();
        println!("Party Average: {}", party_average);
        level_difference = party_average - 2 - PARA_LEVEL[emblem_index as usize];
        if level_difference >= 0 { GameVariableManager::set_number("G_Paralogue_Level", 0); }
        else { GameVariableManager::set_number("G_Paralogue_Level", level_difference); }
    }
    else {
        level_difference = PARA_LEVEL[ new_emblem_index as usize ] - PARA_LEVEL[emblem_index as usize];
        GameVariableManager::set_number("G_Paralogue_Level", level_difference);
    }
    println!("Paralogue Level Difference: {} | {}", level_difference, GameVariableManager::get_number("G_Paralogue_Level"));
}


#[unity::hook("App", "MapDispos", "CreatePlayerTeam")]
pub fn create_player_team(group: &Il2CppString, method_info: OptionalMethod){
    if GameVariableManager::get_number("G_DeploymentMode") == 4 { fulldeploy::load_extra_deployment_slots();  }
    fulldeploy::randomized_emblem_power_spots();
    if GameUserData::get_chapter().cid.to_string().contains("CID_S0") && GameVariableManager::get_number("G_Emblem_Mode") != 0 { get_emblem_paralogue_level(); }

    let absent_force = Force::get(ForceType::Absent).unwrap();
    let hero_unit = absent_force.get_hero_unit();
    println!("Create Player Team Hook: Hero Unit: {}", hero_unit.person.get_name().unwrap().to_string());
    if GameVariableManager::get_number("G_DeploymentMode") == 3 {
        if GameUserData::get_status().value & 64 != 0 { GameUserData::get_status().value -= 64;  }  //Disables Continuous Flag 
    }
    if crate::utils::can_rand() {  
        let random_job =  GameVariableManager::get_number("G_Random_Job") == 1 || GameVariableManager::get_number("G_Random_Job") == 3;
        if random_job && !GameVariableManager::get_bool("G_Lueur_Random") {
            crate::randomizer::job::unit_change_to_random_class(hero_unit);
            GameVariableManager::set_bool("G_Lueur_Random", true);
            crate::randomizer::person::unit::adjust_unit_items(hero_unit);
        }
        // Liberation Weapon Change
        crate::randomizer::item::change_liberation_type();
    }
    
    call_original!(group, method_info);
    if !GameVariableManager::get_bool("G_Cleared_M003") {return; }
    let player_force = Force::get(ForceType::Player).unwrap();
    let max_player = player_force.get_count();
    let mut player_count;
    let absent_count = absent_force.get_count();
    let rng = Random::get_game();

    if GameVariableManager::get_number("G_DeploymentMode") == 3 && !GameUserData::is_encount_map() && GameUserData::get_chapter().cid.to_string() != "CID_M022" { 
        if hero_unit.status.value & 20 != 0 { hero_unit.status.value -= 20; }
        if GameVariableManager::get_number("G_EmblemDeployMode")!= 0 {
            emblem_selection_menu_enable(false);
            unsafe {remove_all_rings(0, None); }
        }
        return;
    }
    if absent_count == 0 || GameUserData::is_evil_map() { 
        unit_selection_menu_enable(true);
        emblem_selection_menu_enable(true);
        if GameUserData::is_evil_map() { return; }
    }
    if GameVariableManager::get_number("G_EmblemDeployMode")!= 0 && GameVariableManager::get_number("G_DeploymentMode")  == 0 {
        unit_selection_menu_enable(true);
        emblem_selection_menu_enable(false);
        unsafe { remove_all_rings(0, None) };
        return;
    }
        //Normal Deployment
    if GameVariableManager::get_number("G_DeploymentMode")  == 0 || absent_count == 0 {
        unit_selection_menu_enable(true);
        return;
    } 
    // Move currently deployed units to absent and then move back hero unit (Alear or Veyle)
    player_force.transfer(3, true);

    //Transfer Dead
    if GameVariableManager::get_number("G_DeploymentMode")  == 1 || GameVariableManager::get_number("G_DeploymentMode")  == 2 { Force::get(ForceType::Dead).unwrap().transfer(3, true); }

    let hero_unit = absent_force.get_hero_unit();
    hero_unit.transfer(0, true);
    hero_unit.try_create_actor();
    if !GameUserData::is_encount_map() && GameVariableManager::get_number("G_DeploymentMode") != 3 { hero_unit.set_status(20); }   
    player_count = player_force.get_count();
    // Lowest Rating Deployment
    if GameVariableManager::get_number("G_DeploymentMode") == 1 {
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
            println!("{} is deployed with rating of {}", mpid.to_string(), capability_score);
            let move_unit = unsafe { force_get_unit_from_pid(pid, false, None) };
            if move_unit.is_some() {
                let unit = move_unit.unwrap();
                unit.transfer(0, true);
                unit.try_create_actor();
            }
            player_count = player_force.get_count();
        }
    }
        // Random Deployment
    else if GameVariableManager::get_number("G_DeploymentMode") == 2  {
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
                    player_count = player_force.get_count();
                    break;
                }
                 index += 1;
            }
        }
    }
}

#[skyline::from_offset(0x01c616f0)]
pub fn remove_all_rings(this: u64, method_info: OptionalMethod);

#[unity::from_offset("App","MapDispos", "GetSortieLimit")]
pub fn get_sortie_limit(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c54fa0)]
pub fn force_get_unit_from_pid(pid: &Il2CppString, relay: bool, method_info: OptionalMethod) -> Option<&'static mut Unit>;

// Global Menu Stuff
