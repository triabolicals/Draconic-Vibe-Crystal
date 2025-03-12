use unity::prelude::*;
pub use engage::{
    random::Random,
    gamevariable::*, 
    gameuserdata::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    godpool::*,
    unitpool::UnitPool,
    force::*,
    script::*,
    singleton::SingletonClass,
    gamedata::{*, GodData, unit::*},
};
use super::{DVCVariables, CONFIG};
use crate::{enums::*, randomizer::emblem::EMBLEM_LIST};

pub mod fulldeploy;
pub mod sortie;

pub struct DeploymentMod;
impl ConfigBasicMenuItemSwitchMethods for DeploymentMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){ }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        let value = 
            if is_main { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 4, 1);
        if value != result {
            if is_main {  CONFIG.lock().unwrap().deployment_type = result; }
            else {  GameVariableManager::set_number(DVCVariables::DEPLOYMENT_KEY, result); } 
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let deploy_type =   
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) };
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
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().deployment_type }
            else { GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) };
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
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_deployment = result; }
            else { GameVariableManager::set_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY, result) };
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) };
        this.help_text = match emblem_deployment {
            1 => { "Emblems will be randomized onto deployed units." },
            2 => { "Emblems will not be equipped onto units." },
            _ => { "Emblems are freely selectable in battle preperations."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let emblem_deployment = 
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_deployment }
            else { GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) };
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

pub fn lueur_status_check() {
    if GodPool::try_get_gid(EMBLEM_GIDS[19], false).is_some() {
        let lueur_god = GodData::get_mut(EMBLEM_GIDS[19]).unwrap();
        if UnitPool::get_from_person_force_mask(PersonData::get(PIDS[0]).unwrap(), 9).is_none() {
            if lueur_god.get_flag().value & -2147483648 != 0 {
                lueur_god.get_flag().value -= -2147483648;  //Alear is now an equippable ring due to death or not recruited
            }
        }
        else { lueur_god.get_flag().value |= -2147483648; }
    }
}
// Generating the list of equipable emblems
pub fn get_emblem_list() -> Vec<String> {
    let mut result: Vec<String> = Vec::new(); 
    EMBLEM_LIST.get().unwrap().iter()
        .flat_map(|&hash| GodData::try_get_hash(hash))
        .for_each(|god|{
            if let Some(g_unit) = GodPool::try_get(god, true) {
                if  god.parent.index != 13 && !g_unit.get_escape() && god.force_type == 0 {
                    result.push(g_unit.data.gid.to_string());
                }
            }
        }
    );
    result
}
pub fn unit_selection_menu_disable(enabled: bool) { GameVariableManager::set_bool("UnitDeploy", enabled); 
}
//Hook to function that creates the sortie deploy positions to do deployment stuff

pub fn get_emblem_paralogue_level() {
    if !crate::utils::can_rand() || GameVariableManager::get_bool("G_CustomEmblem") { return; }
    let cid = GameUserData::get_chapter().get_prefixless_cid().to_string();
    GameVariableManager::make_entry(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0);
    GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0);
    let e_index = EMBELM_PARA.iter().position(|&x| x == cid);

    if e_index.is_none() {  return;  }
    let emblem_index = e_index.unwrap();
    let found = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[emblem_index as usize].to_string(), true);
    let new_emblem_index;
    if found != -1 { new_emblem_index = found;  }
    else { return; }
    let level_difference;
    if new_emblem_index >= 12 {
        let party_average = crate::autolevel::get_difficulty_adjusted_average_level();
        level_difference = party_average - 2 - PARA_LEVEL[emblem_index as usize];
        if level_difference >= 0 { GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0); }
        else { GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, level_difference); }
    }
    else {
        level_difference = PARA_LEVEL[ new_emblem_index as usize ] - PARA_LEVEL[emblem_index as usize];
        GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, level_difference);
    }
    println!("Paralogue Level Difference: {} | {}", level_difference, GameVariableManager::get_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL));
}


#[unity::hook("App", "MapDispos", "CreatePlayerTeam")]
pub fn create_player_team(group: &Il2CppString, method_info: OptionalMethod){
    if GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 4 { fulldeploy::load_extra_deployment_slots();  }
    crate::randomizer::terrain::randomized_emblem_power_spots();
    if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 { get_emblem_paralogue_level(); }
    crate::randomizer::terrain::fow::map_start_fow();
    let absent_force = Force::get(ForceType::Absent).unwrap();
    let hero_unit = absent_force.get_hero_unit();
    GameVariableManager::make_entry("UnitDeploy", 0);
    GameVariableManager::set_bool("UnitDeploy", false);
    if GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3 {
        if GameUserData::get_status().value & 64 != 0 { GameUserData::get_status().value &= !64;  }  //Disables Continuous Flag 
    }
    if crate::utils::can_rand() {  
        if GameVariableManager::get_number(DVCVariables::JOB_KEY) & 1 != 0 && !GameVariableManager::get_bool(DVCVariables::LUEUR_RANDOM_JOB_KEY) {
            crate::randomizer::job::unit_change_to_random_class(hero_unit);
            GameVariableManager::set_bool(DVCVariables::LUEUR_RANDOM_JOB_KEY, true);
            crate::randomizer::person::unit::adjust_unit_items(hero_unit);
        }
        crate::randomizer::item::change_liberation_type();
    }

    call_original!(group, method_info);
    if !DVCVariables::is_main_chapter_complete(3) {return; }
    let player_force = Force::get(ForceType::Player).unwrap();
    let max_player = player_force.get_count();
    let mut player_count;
    let absent_count = absent_force.get_count();
    let rng = Random::get_game();
    let unit_deployment_mode = GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY);

    // Remove Emblems
    if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) > 0 { crate::utils::remove_equip_emblems(); }

    // Free deployment
    if unit_deployment_mode == 3 && !GameUserData::is_encount_map() && GameUserData::get_chapter().cid.to_string() != "CID_M022" { 
        if hero_unit.status.value & 20 != 0 { hero_unit.status.value &= !20; }
        return;
    }
    else if absent_count == 0 || GameUserData::is_evil_map() || unit_deployment_mode == 0 { 
        unit_selection_menu_disable(false);
        return;
    }

    // Move currently deployed units to absent and then move back hero unit (Alear or Veyle)
    player_force.transfer(3, true);

    //Transfer Dead
    if unit_deployment_mode == 1 || unit_deployment_mode == 2  { Force::get(ForceType::Dead).unwrap().transfer(3, true); }

    let hero_unit = absent_force.get_hero_unit();
    hero_unit.transfer(0, true);
    hero_unit.try_create_actor();
    if !GameUserData::is_encount_map() &&  unit_deployment_mode != 3 { hero_unit.set_status(20); }   
    player_count = player_force.get_count();

    // Lowest Rating Deployment
    if unit_deployment_mode == 1 {
        unit_selection_menu_disable(true);
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
            if let Some(unit) = engage::unitpool::UnitPool::get_from_person_mut(pid, false) {
                unit.transfer(0, true);
                unit.try_create_actor();
            }
            player_count = player_force.get_count();
        }
    }
        // Random Deployment
    else if unit_deployment_mode == 2  {
        println!("Random Deployment");
        unit_selection_menu_disable(true);
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