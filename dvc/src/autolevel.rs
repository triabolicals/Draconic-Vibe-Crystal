use unity::prelude::*;
use engage::{
    unit::{UnitPool, Unit},
    force::*, gamedata::*,
    gameuserdata::*, 
    gamevariable::*, 
    map::situation::MapSituation,
    mess::*, 
    random::Random,
    util::get_instance
};
use crate::utils::*;
use crate::{randomizer, DVCConfig};
use crate::DVCVariables;
use crate::config::DVCFlags;
use crate::continuous::get_continious_total_map_complete_count;
use crate::randomizer::item::unit_items::adjust_missing_weapons;
use crate::randomizer::person::unit::fixed_unit_weapon_mask;

pub mod enemy;

pub fn emblem_paralogue_level_adjustment(unit: &Unit){
    if !DVCVariables::random_enabled() || DVCVariables::is_random_map() { return; }
    let level_difference = GameVariableManager::get_number(DVCVariables::EMBLEM_PARALOGUE_LEVEL);
    if level_difference == 0 { return; }
    let total_level = unit.level as i32 + if unit.job.is_high() { 20 } else { unit.internal_level as i32 };
    let new_level = total_level + level_difference;
    if GameUserData::get_chapter().cid.str_contains("S0") {
        if new_level <= 20 {
            if unit.job.rank == 0 { unit.auto_grow_capability(new_level, 19); }
            else { unit.auto_grow_capability(1, new_level); }
        }
        else {
            if unit.job.rank == 0 {
                if unit.job.max_level > 20 { unit.auto_grow_capability(new_level, new_level); }
                else { unit.auto_grow_capability(20, new_level); }
            }
            else { unit.auto_grow_capability(new_level-20, new_level); }
        }
    }
    else {
        unit.auto_grow_capability(new_level, new_level);
        fix_unit_level(unit, new_level);
    }
    unit.set_hp(unit.get_capability(0, true));
    randomizer::skill::learn::unit_update_learn_skill(unit);
}

pub fn auto_level_unit(unit: &mut Unit, leader: bool){
    if GameUserData::is_evil_map() && unit.person.flag.value & 512 != 0 {
        let level = unit.person.get_level() as i32;
        if level < 15  {
            if unit.job.is_high() { unit.auto_grow_capability(level, level + 20); }
            else if unit.job.is_low() && unit.job.max_level > 20 { unit.auto_grow_capability(20 + level, level + 20); }
            else { unit.auto_grow_capability(level, level + 20); }
        }
    }
    if (DVCVariables::is_random_map() || DVCVariables::Continuous.get_value() == 4) && DVCVariables::is_main_chapter_complete(4) {
        auto_level_unit_for_random_map(unit, leader);
        return;
    }
    if !DVCFlags::Autolevel.get_value() { return; }
    let mut avg_level = get_difficulty_adjusted_average_level();
    let recommended = GameUserData::get_chapter().recommended_level as i32;
    if avg_level < recommended { return; }
    if leader { avg_level += 3; }
    let unit_level = unit.internal_level as i32 + unit.level as i32;
    if avg_level < unit_level { return; }
    let current_job = &unit.job;
    let is_base = current_job.is_low() && current_job.has_high_jobs();
    let level_gains = avg_level - unit_level;
    let mut promoted = false;
    for x in 0..9 {
        let increase = unit.get_capability(x, true) * (1 + avg_level - unit_level) / 100;
        let new_stat = clamp_value(unit.base_capability[x as usize] as i32 + increase, 1, 120);
        unit.set_base_capability(x, new_stat);
    }
    if (unit.level as i32 + level_gains) > unit.job.max_level as i32 {
        let new_level = unit.level as i32 + level_gains - unit.job.max_level as i32;
        if is_base {
            let internal = unit.internal_level as i32 + unit.job.max_level as i32;
            let selected_mask = unit.selected_weapon_mask.value;
            unit.class_change(current_job.get_high_jobs()[0]);
            unit.selected_weapon_mask.value = selected_mask;
            fixed_unit_weapon_mask(unit);
            unit.set_level(new_level);
            unit.set_internal_level(internal);
            promoted = true;
        } else {
            unit.set_level(unit.job.max_level as i32);
            unit.set_internal_level(new_level);
        }
    }
    else { unit.set_level(unit.level as i32 + level_gains); }

    println!("Auto Level Unit: {} Gained {} levels [Promoted: {}]", unit.get_name(), level_gains, promoted);
    fix_unit_level(unit, avg_level);
    unit.set_hp(unit.get_capability(0, true));
    adjust_missing_weapons(unit);
    randomizer::skill::learn::unit_update_learn_skill(unit);
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
            if unit.person.get_internal_level() == 0 { if job.is_high() { job.internal_level as i32 } else { 0 } }
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
pub fn auto_level_unit_for_random_map(unit: &mut Unit, leader: bool){
    if !DVCVariables::is_main_chapter_complete(4) || GameUserData::is_evil_map() { return; }
    if DVCVariables::Continuous.get_value() != 4 && GameVariableManager::get_number(DVCVariables::CONTINUOUS) != 2 { return; }
    let rng = Random::get_game();
    let is_player = unit.person.get_asset_force() == 0;
    let diff = GameUserData::get_difficulty(false);
    let seq = GameUserData::get_sequence();
    let rng_level = crate::continuous::random::random_map_mode_level();
    let level =
    if GameUserData::get_chapter().cid.to_string().contains("S0") || get_continious_total_map_complete_count() < 10 {
        rng_level  + if leader { 3 } else { 0 } 
    }
    else {
        let map_sit_level = 
            if seq == 2 || seq == 3 { get_instance::<MapSituation>().average_level }
            else { get_difficulty_adjusted_average_level() };
        
        let l = if map_sit_level > rng_level  {  (map_sit_level + rng_level + 1) / 2  }
        else { rng_level } +  if is_player{ -1 } else { 0 };
        l
    };
    let current_job = &unit.job;
    let current_mask = unit.weapon_mask.value;
    if level <= current_job.max_level as i32 {
        if current_job.is_high() && !is_player {
            let lows = get_base_classes(current_job);
            let job_count = lows.len();
            if job_count > 0 {
                if job_count == 1 {
                    unit.class_change(lows[0]);
                }
                else if job_count > 1 {
                    let index = rng.get_value( job_count as i32) as usize;
                    unit.class_change(lows[index]);
                }
                unit.selected_weapon_mask.value = current_mask;
                unit.update_weapon_mask();
                randomizer::job::randomize_selected_weapon_mask(unit, None);
                randomizer::person::unit::adjust_unit_items(unit);
            }
        }
    }
    else if current_job.rank == 0 && current_job.has_high_jobs() {
        unit.class_change(current_job.get_high_jobs()[0]);
        unit.selected_weapon_mask.value = current_mask;
        unit.update_weapon_mask();
        randomizer::person::unit::adjust_unit_items(unit);
    }
    fix_unit_level(unit, level);
    if is_player {
        unit.set_sp( level * 150 + 500 );
        unit.set_weapon_mask_from_parson();
    }
    else {
        let offset = unit.person.get_offset_by_difficulty();
        unit.auto_grow_capability(unit.level as i32, level + diff );
        for x in 0..10 { unit.base_capability[x] -= offset[x]; }
    }
    unit.set_hp(unit.get_capability(0, true));
    randomizer::skill::learn::unit_update_learn_skill(unit);
}

//Get Average Level of Party
#[skyline::from_offset(0x02b4afa0)]
pub fn get_average_level(difficulty: i32, sortie_count: i32, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x024f2c10)]
pub fn get_sortie_unit_count(method_info: OptionalMethod) -> i32;

pub fn get_difficulty_adjusted_average_level() -> i32 {
    let diff = GameUserData::get_difficulty(false);
    if GameUserData::get_sequence() != 3 {
        unsafe {
            let count = get_sortie_unit_count(None);
            if count == 0 { get_average_level(0, 10, None) }
            else { get_average_level(diff, get_sortie_unit_count(None), None) }
        }
    }
    else { MapSituation::get_instance().average_level + diff }
}

pub fn autolevel_party() -> Option<(i32, i32)> {
    fixed_level();
    if !DVCFlags::PostChapterAutolevel.get_value() { None }
    else {
        let player_average =
            if DVCConfig::get().debug { GameUserData::get_chapter().recommended_level as i32 + 3 }
            else { get_difficulty_adjusted_average_level() - 2 * GameUserData::get_difficulty(false) };

        println!("Autoleveling Bench to average of {}", player_average);
        let mut count = 0;
        Force::get(ForceType::Absent).unwrap().iter().for_each(|unit| { level_up_unit(unit, player_average); });
        if DVCConfig::get().debug { Force::get(ForceType::Player).unwrap().iter().for_each(|unit| { level_up_unit(unit, player_average); }); }
        fixed_level();
        if count > 0 { Some((player_average, count)) }
        else { None }
    }
}
fn level_up_unit(unit: &Unit, target_level: i32) -> bool {
    let total_level: i32 = unit.level as i32 + unit.internal_level as i32;
    let number_of_level_ups = target_level - total_level;
    let job_max_level = unit.get_job().get_max_level();
    if number_of_level_ups >= 1 {
        for _ in 0..number_of_level_ups {
            if unit.level >= job_max_level {
                unit.set_level( (job_max_level - 1) as i32 );
                unit.set_internal_level( unit.internal_level as i32 + 1);
            }
            unit.level_up(3);
            unit.add_sp(100);
        }
        println!("{} gained {} level [{}/{}]", Mess::get_name(unit.person.pid), number_of_level_ups, unit.level, unit.internal_level);
        unit.set_hp(unit.get_capability(0, true));
        true
    }
    else { false }

}
pub fn fixed_level() {
    for x in 1..250 {
        if let Some(unit) = UnitPool::get(x).filter(|u| u.force.is_some_and(|f| (1 << f.force_type) & 57 != 0)){
            let job_max_level = unit.get_job().get_max_level();
            if unit.level >= job_max_level {
                let diff = unit.level as i32 - job_max_level as i32;
                unit.level = job_max_level;
                unit.internal_level += diff as i8;
            }
        }
    }
}