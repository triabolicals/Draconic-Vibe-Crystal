use unity::prelude::*;
pub use engage::{
    force::*,
    gamedata::{GodData, *},
    gameuserdata::*,
    gamevariable::*,
    god::*,
    random::Random,
    script::*,
    singleton::SingletonClass,
    unit::{Unit, UnitPool},
};
use engage::unit::Gender;
use super::{randomizer, DVCVariables};
use crate::enums::*;
use crate::config::DVCFlags;
use crate::randomizer::data::GameData;

pub mod fulldeploy;
pub mod sortie;

pub fn get_unit_rating(this: &Unit) -> i32 {
    let mut result: i32 = 0;
    for x in 1..9 { result += this.get_capability(x, false);  }
    result
}

pub fn get_emblem_list() -> Vec<String> {
    GameData::get_playable_god_list().iter().filter(|g| GodPool::try_get(g, false)
        .is_some_and(|g| !g.get_escape() && g.data.force_type == 0))
        .map(|g| g.main_data.gid.to_string()).collect()
}
pub fn unit_selection_menu_disable(enabled: bool) { GameVariableManager::set_bool("UnitDeploy", enabled); }
//Hook to function that creates the sortie deploy positions to do deployment stuff

pub fn get_emblem_paralogue_level() {
    if !DVCVariables::random_enabled() || DVCFlags::CustomEmblemsRecruit.get_value() { return; }
    let cid = GameUserData::get_chapter().prefixless_cid.to_string();
    GameVariableManager::make_entry(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0);
    GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0);
    if let Some(pos) = EMBELM_PARA.iter().position(|&x| x == cid) {
        let found = randomizer::person::pid_to_index(&EMBLEM_GIDS[pos].to_string(), true);
        let new_emblem_index;
        if found != -1 { new_emblem_index = found as usize;  } else { return; }
        let level_difference;
        if new_emblem_index >= 12 {
            let party_average = crate::autolevel::get_difficulty_adjusted_average_level();
            level_difference = party_average - 2 - PARA_LEVEL[pos];
            if level_difference >= 0 { GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, 0); }
            else { GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, level_difference); }
        }
        else {
            level_difference = PARA_LEVEL[ new_emblem_index] - PARA_LEVEL[pos];
            GameVariableManager::set_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL, level_difference);
        }
    }
}

pub fn deployment_modes(){
    if let Some(hero_unit) = UnitPool::get_hero(false) {
        let absent_force = Force::get(ForceType::Absent).unwrap();
        if DVCVariables::random_enabled() { randomizer::item::change_liberation_type(); }
        if !DVCVariables::is_main_chapter_complete(3) { return; }
        if DVCVariables::UnitDeployment.get_value() == 3 {
            if GameUserData::get_status().value & 64 != 0 { GameUserData::get_status().value &= !64; }  //Disables Continuous Flag
        }
        let player_force = Force::get(ForceType::Player).unwrap();
        let max_player = player_force.get_count();
        let mut player_count;
        let absent_count = absent_force.get_count();
        let rng = Random::get_game();
        let unit_deployment_mode = DVCVariables::UnitDeployment.get_value();
        if DVCVariables::EmblemDeployment.get_value() == 3 {
            let dlc_check = crate::utils::dlc_check();
            GameData::get_playable_god_list().iter().enumerate().for_each(|(i, x)| {
                if i < 12 || (i > 12 && i < 19 && dlc_check) {
                    GodPool::create(x);
                    if let Some(god_unit) = GodPool::try_get(x, false) { god_unit.set_escape(false); }
                }
            });
        } else if DVCVariables::EmblemDeployment.get_value() > 0 { crate::utils::remove_equip_emblems(); }
        // Free deployment
        if unit_deployment_mode == 3 && !GameUserData::is_encount_map() && !GameUserData::get_chapter().cid.str_contains("CID_M022") {
            if hero_unit.status.value & 20 != 0 { hero_unit.status.value &= !20; }
            return;
        } else if absent_count == 0 || GameUserData::is_evil_map() || unit_deployment_mode == 0 {
            unit_selection_menu_disable(false);
            return;
        }
        //Transfer Dead
        if unit_deployment_mode == 1 || unit_deployment_mode == 2 || unit_deployment_mode > 5 {
            Force::get(ForceType::Dead).unwrap().transfer(ForceType::Absent, true);
        }
        while let Some(unit) = hero_unit.next { unit.transfer(ForceType::Absent, true); }

        if !GameUserData::is_encount_map() && unit_deployment_mode != 3 { hero_unit.set_status(20); }
        player_count = player_force.get_count();
        match unit_deployment_mode {
            1 => {
                while player_count < max_player {
                    let mut pid: &Il2CppString = "PID_unit".into();
                    let mut capability_score = 99999;
                    let mut force_iter = Force::iter(absent_force);
                    while let Some(unit) = force_iter.next() {
                        let cap = get_unit_rating(unit);
                        if cap < capability_score {
                            capability_score = cap;
                            pid = unit.person.pid;
                        }
                    }
                    if let Some(unit) = UnitPool::get_from_pid(pid, false) {
                        unit.transfer(ForceType::Player, true);
                        unit.try_create_actor();
                    }
                    player_count = player_force.get_count();
                }
            }
            2 => {
                while player_count < max_player {
                    let rng_range = absent_force.get_count();
                    if rng_range == 0 { break; }
                    let mut index = 0;
                    let value = rng.get_value(rng_range);
                    let mut force_iter = Force::iter(absent_force);
                    while let Some(unit) = force_iter.next() {
                        if index == value {
                            unit.transfer(ForceType::Player, true);
                            unit.try_create_actor();
                            player_count = player_force.get_count();
                            break;
                        }
                        index += 1;
                    }
                }
            }
            5 => {
                while player_count < max_player {
                    let mut pid: &Il2CppString = "PID_unit".into();
                    let mut capability_score = 0;
                    let mut force_iter = Force::iter(absent_force);
                    while let Some(unit) = force_iter.next() {
                        let cap = get_unit_rating(unit);
                        if cap > capability_score {
                            capability_score = cap;
                            pid = unit.person.pid;
                        }
                    }
                    if let Some(unit) = UnitPool::get_from_pid(pid, false) {
                        unit.transfer(ForceType::Player, true);
                        unit.try_create_actor();
                    }
                    player_count = player_force.get_count();
                }
            }
            6 | 7 => {
                let gender = if unit_deployment_mode == 7 { Gender::Female } else { Gender::Male };
                let mut force_iter = Force::iter(absent_force);
                let mut count = player_force.get_count();
                while let Some(unit) = force_iter.next() {
                    if count >= max_player { break; }
                    if unit.get_gender() == gender {
                        unit.transfer(ForceType::Player, true);
                        unit.try_create_actor();
                        count += 1;
                    }
                }
            }
            _ => {}
        }
    }
}