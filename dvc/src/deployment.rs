use unity::prelude::*;
pub use engage::{
    force::*, gamedata::{GodData, *},
    gameuserdata::*, gamevariable::*,
    god::*, random::Random, script::*,
    singleton::SingletonClass,
    unit::{Unit, UnitPool},
};
use engage::unit::Gender;
use super::{randomizer, DVCVariables};
use crate::{enums::*, config::DVCFlags, randomizer::data::GameData};

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
        println!("Unit Deployment Mode: {}", unit_deployment_mode);
        // Free deployment
        if unit_deployment_mode == 3 && !GameUserData::is_encount_map() && !GameUserData::get_chapter().cid.str_contains("CID_M022") {
            if hero_unit.status.value & 20 != 0 { hero_unit.status.value &= !20; }
            return;
        }
        else if absent_count == 0 || GameUserData::is_evil_map() || unit_deployment_mode == 0 { return; }
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
                    let mut hash = 0;
                    let mut capability_score = 99999;
                    let mut force_iter = Force::iter(absent_force);
                    while let Some(unit) = force_iter.next() {
                        let cap = get_unit_rating(unit);
                        if cap < capability_score {
                            capability_score = cap;
                            hash = unit.person.parent.hash;
                        }
                    }
                    if let Some(unit) = absent_force.iter().find(|f| f.person.parent.hash == hash) {
                        unit.transfer(ForceType::Player, true);
                        unit.try_create_actor();
                    }
                    player_count = player_force.get_count();
                    if absent_force.get_count() == 0 { break; }
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
                    let mut hash = 0;
                    let mut capability_score = 0;
                    let mut force_iter = Force::iter(absent_force);
                    while let Some(unit) = force_iter.next() {
                        let cap = get_unit_rating(unit);
                        if cap > capability_score {
                            capability_score = cap;
                            hash = unit.person.parent.hash;
                        }
                    }
                    if let Some(unit) = absent_force.iter().find(|f| f.person.parent.hash == hash) {
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