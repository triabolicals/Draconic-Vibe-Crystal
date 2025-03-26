use unity::prelude::*;
use engage::{
    force::*, gamedata::{dispos::*, unit::*, *}, 
    gameuserdata::*, 
    gamevariable::*, 
    map::situation::MapSituation, 
    menu::{config::*, BasicMenuItem, BasicMenuItemAttribute, BasicMenuResult}, 
    mess::*, 
    random::Random,
    util::get_instance
};
use super::CONFIG;
use crate::{randomizer::{self, emblem::ENEMY_EMBLEM_LIST}, utils::*, DVCVariables};

pub mod enemy;
pub mod revival;
pub mod menu;

pub fn calculate_player_cap() -> i32 {
    let mut caps: Vec<_> = Force::get(ForceType::Player).unwrap().iter()
        .chain( Force::get(ForceType::Absent).unwrap().iter() )
        .chain( Force::get(ForceType::Dead).unwrap().iter() )
        .map(|unit| unit_cap_total(unit, true, false)).collect();
    if caps.len() == 0 { 
        
        return 0;
    
    }
    caps.sort_by(|a, b| b.cmp(a));
    let mut total: i32 = 0;

    let diff = GameUserData::get_difficulty(false);
    let mut count = min(caps.len() as i32, 14 - 2 * diff);
    if count == 0 { count = 1; }
    for i in 0..count{ total += caps[i as usize];  }
    let average = total / ( count as i32 );
    println!("{} unit Average is {}", count, average);
    if average > GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP) {
         GameVariableManager::set_number(DVCVariables::PLAYER_AVERAGE_CAP, average);
    }
    average
}

pub fn unit_cap_total(this: &Unit, with_hp: bool, enhanced: bool) -> i32 {
    let mut total = 0;
    if with_hp {
        [0, 2, 3, 4, 10].into_iter().for_each(|x| total += this.get_capability(x, enhanced));
        total += 2* this.get_capability(8, enhanced);
        total += 2* max( this.get_capability(5, enhanced), this.get_capability(7, enhanced) );
        total += 2* max( this.get_capability(1, enhanced), this.get_capability(6, enhanced) );
    }
    else { for x in 1..8 { total += this.get_capability(x, false); } }
    total
}

pub fn get_chapters_completed() -> i32 {
    ChapterData::get_list_mut().unwrap().iter().filter(|c| GameUserData::is_chapter_completed(c) && !c.is_evil()).count() as i32
}

pub fn emblem_paralogue_level_adjustment(unit: &Unit){
    if !DVCVariables::random_enabled() || DVCVariables::is_random_map() { return; }
    let level_difference = GameVariableManager::get_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL);
    if level_difference == 0 { return; }
    let total_level = unit.level as i32 + unit.internal_level as i32; 
    let new_level = total_level + level_difference;
    unit.auto_grow_capability(new_level, new_level);
    fix_unit_level(unit, new_level);
    unit.set_hp(unit.get_capability(0, true));
    crate::randomizer::skill::learn::unit_update_learn_skill(unit);
}

pub fn auto_level_unit(unit: &mut Unit, leader: bool){
    if GameUserData::is_evil_map() { return; }

    if ( DVCVariables::is_random_map() || GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 5 ) && DVCVariables::is_main_chapter_complete(4) {
        auto_level_unit_for_random_map(unit, leader);
        return;
    }
    if !GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) || !DVCVariables::is_main_chapter_complete(6) { 
        if GameVariableManager::get_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL) != 0 {
            emblem_paralogue_level_adjustment(unit);
        }
        return;
    }
    let diff = GameUserData::get_difficulty(false);
    if !GameVariableManager::exist("AvgLvl") || GameUserData::get_sequence() == 3  {
        GameVariableManager::make_entry("AvgLvl", calculate_average_level(10 - diff ));
    }
    if !GameVariableManager::exist("ChpCnt") {
        GameVariableManager::make_entry("ChpCnt", get_chapters_completed());
    }
    let avg_level = if leader { 3 } else { 0 } + GameVariableManager::get_number("AvgLvl");
    let chapter_count = GameVariableManager::get_number("ChpCnt");

    let current_job = &unit.job;
    if avg_level <= current_job.max_level as i32 {
        if current_job.is_high() && current_job.get_low_jobs().len() > 0 {
            unit.class_change(current_job.get_low_jobs()[0]);
        }
    }
    else if current_job.is_low() && current_job.has_high() { unit.class_change(current_job.get_high_jobs()[0]);  }
    fix_unit_level(unit, avg_level);
    let avg_player_cap = GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP);
    let mut enemy_cap = unit_cap_total(unit, true, false);
    let max_cap = min(1, diff*3*(chapter_count - 10)) + avg_player_cap + 10;
    let mut counter = 0;
    let rng = Random::get_game();

    while counter < 40 && enemy_cap < max_cap {
        for x in 0..9 {
            if unit.get_capability_grow(x, false) > rng.get_value(100) {
                unit.base_capability.capability[x as usize] += 1;
                enemy_cap += 1;
            }
        }
        counter += 1;
    }
    unit.set_hp(unit.get_capability(0, true));
    crate::randomizer::skill::learn::unit_update_learn_skill(unit);
}


pub fn fix_unit_level(unit: &Unit, total_level: i32) {
    let job = &unit.job;
    let max_level = job.max_level as i32;
    if job.is_high() {
        if total_level < job.internal_level as i32 {
            unit.set_level(total_level);
            unit.set_internal_level(0);
        }
        else {
            let current_internal_level = 
            if unit.person.get_internal_level() == 0 {
                if job.is_high() { job.internal_level as i32 } else { 0 }
            }
            else { unit.person.get_internal_level() as i32 };
            let level = clamp_value(total_level - current_internal_level, 1, job.max_level as i32);
            unit.set_level(level);
            unit.set_internal_level( total_level - level);
        }
    }
    else if total_level > max_level {
        unit.set_level(max_level);
        let internal = if max_level > 20 { total_level - max_level } else { 0 };
        unit.set_internal_level(internal);
    }
    else {
        unit.set_level(total_level);
        unit.set_internal_level(0);
    }
}
// Autolevel based on Map Completed
pub fn auto_level_unit_for_random_map(unit: &mut Unit, leader: bool){
    if !DVCVariables::is_main_chapter_complete(4) || GameUserData::is_evil_map() || !DVCVariables::is_random_map() { return; } 
    let rng = Random::get_game();
    let is_player = unit.person.get_asset_force() == 0;
    let diff = GameUserData::get_difficulty(false);
    let mut level =  
        crate::continuous::random::random_map_mode_level() +
        if leader { 3 } else { 0 } + 
        if !is_player { rng.get_value(2+diff) - 1 } else { -1 };

    let map_sit_level = get_instance::<MapSituation>().average_level;
    if map_sit_level > level { level = (map_sit_level + level + 1) / 2; }
    let current_job = &unit.job;
    // let is_special = current_job.max_level > 20 && current_job.is_low();

    if level <= current_job.max_level as i32 {
        let job_len = current_job.get_low_jobs().len();
        if current_job.is_high() && current_job.get_low_jobs().len() > 0 && !is_player {
            if job_len == 1 { unit.class_change(current_job.get_low_jobs()[0]);  }
            else {
                let index = rng.get_value(job_len as i32) as usize;
                unit.class_change(current_job.get_low_jobs()[index]);
            }
            randomizer::job::randomize_selected_weapon_mask(unit);
            randomizer::person::unit::adjust_unit_items(unit);
        }

    }
    else if current_job.is_low() && current_job.has_high() { 
        unit.class_change(current_job.get_high_jobs()[0]); 
        randomizer::job::randomize_selected_weapon_mask(unit);
        randomizer::person::unit::adjust_unit_items(unit);
    }
    println!("Autolevel for Random Map: {} for {}", level, Mess::get_name(unit.person.pid));
    fix_unit_level(unit, level);

    if is_player {
        unit.set_sp( level * 100 + 300 );
        unit.set_weapon_mask_from_person();
    }
    else {
        let offset = unit.person.get_offset_by_difficulty();
        let completed_maps = crate::continuous::get_continious_total_map_complete_count();
        unit.auto_grow_capability(unit.level as i32, level + diff + completed_maps / 4 );
        for x in 0..10 { unit.base_capability.capability[x] -= offset[x]; }
    }

    unit.set_hp(unit.get_capability(0, true));
    crate::randomizer::skill::learn::unit_update_learn_skill(unit);
}

pub fn calculate_average_level(sortie_count: i32) -> i32 {
    let vander_replace = DVCVariables::get_dvc_person(1, true);

    let count = if sortie_count == 0 { 10 } else { sortie_count };
    let mut collection: Vec<i32> = Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Absent).unwrap().iter())
        .filter(|unit| unit.person.pid != vander_replace)
        .map(|unit| unit.level as i32 + unit.internal_level as i32).collect();

    collection.sort_by(|a, b| b.cmp(a));

    let length = collection.len() as i32;
    let sum_count = if length < count { length } else { count };
    let mut sum = 0;
    for x in 0..sum_count { sum += collection[x as usize]; }
    let average = sum / count;
    println!("Player Average Level: {}", average);
    average
}


//Get Average Level of Party
#[skyline::from_offset(0x02b4afa0)]
pub fn get_average_level(difficulty: i32, sortie_count: i32, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x024f2c10)]
pub fn get_sortie_unit_count(method_info: OptionalMethod) -> i32;

pub fn get_difficulty_adjusted_average_level() -> i32 {
    unsafe {
        if get_sortie_unit_count(None) == 0 { return get_average_level(0, 10, None); }
        else { return get_average_level(0, get_sortie_unit_count(None), None); }
    }
}

pub fn autolevel_party(){
    if !GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY) { return; }
    let player_average = get_instance::<MapSituation>().average_level - 1 - 2*GameUserData::get_difficulty(false);
    println!("Autoleveling Bench to average of {}", player_average);
    Force::get(ForceType::Absent).unwrap().iter()
        .for_each(|unit|{
            let total_level: i32 = unit.level as i32 + unit.internal_level as i32;
            let number_of_levelups = player_average - total_level;
            if number_of_levelups < 1 { 
                let job_max_level = unit.get_job().get_max_level();
                for _x in 0..number_of_levelups { 
                    if job_max_level <= unit.level { unit.set_level( (job_max_level - 1) as i32 ); }
                    unit.level_up(3);
                    unit.add_sp(100);
                }
            }
            unit.set_hp(unit.get_capability(0, true));
            println!("{} gained {} levels", Mess::get_name(unit.person.pid), number_of_levelups);
        }
    );
}