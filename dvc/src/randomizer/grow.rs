use engage::{
    unit::Unit,
    force::{ForceType, *}, 
    gamedata::*,
    gamevariable::*,
};
use crate::config::DVCFlags;
use crate::DVCVariables;
use crate::randomizer::get_data_read;

pub fn randomize_person_grow(){
    let person_list = PersonData::get_list_mut().unwrap();
    let data = get_data_read();
    let mode = DVCVariables::PersonalGrowthMode.get_value() & 255;
    let limits: (i32, i32) = 
        match mode {
            1|2 => { (50, 2000) },   // Unrestricted
            _ => { (175, 500) },    // Balance default
        };
    person_list.iter_mut().filter(|p| !p.get_grow().is_zero())
        .for_each(|p| {
            let rng = crate::utils::create_rng(p.parent.hash, 2);
            let grow = p.get_grow();
            let mut total = 0;
            while total < limits.0 || total > limits.1 { total = data.growth_data.get_personal(rng, grow); }
        });
    if DVCFlags::AdaptiveGrowths.get_value() { player_pool_adaptive_growths(); }
}

pub fn adaptive_growths(unit: &Unit, change_class: bool) {
    let key = format!("G_JG_{}", unit.person.pid);
    let current_job = unit.get_job();
    if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry(key.as_str(), current_job.parent.hash); }
    else if change_class || GameVariableManager::get_number(key.as_str()) != 0 { 
        GameVariableManager::set_number(key.as_str(), current_job.parent.hash);
    }
    if !DVCFlags::AdaptiveGrowths.get_value() { return; }
    if crate::randomizer::person::is_playable_person(unit.person) {
        if JobData::try_index_get(GameVariableManager::get_number(key.as_str())).is_none() {
            GameVariableManager::set_number(key.as_str(), current_job.parent.hash); 
        }
        let job = JobData::try_get_hash( GameVariableManager::get_number(key.as_str())).unwrap();
        let is_magic = super::job::is_magic_class(job);
        let grow = unit.person.get_grow();
        let str = grow[1];
        let mag = grow[6];

        if (is_magic && str > mag) || ( !is_magic && mag > str ) {
            grow[1] = mag;
            grow[6] = str;
        }
    }
}

pub fn player_pool_adaptive_growths() {
    if !DVCFlags::AdaptiveGrowths.get_value() { return; }
    let force_types = [ForceType::Player, ForceType::Enemy, ForceType::Ally, ForceType::Absent, ForceType::Dead];
    for ff in force_types {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter { adaptive_growths(unit, false); }
    }
}

pub fn randomize_job_grow() {
    let data = get_data_read();
    let job_list = JobData::get_list_mut().unwrap();
    job_list.iter_mut().for_each(|job|{
        let rng = crate::utils::create_rng(job.parent.hash, 2);
        let grow = job.get_diff_grow();
        data.growth_data.set_job_diff(rng, grow);
    });
}


pub fn random_grow(){
    let enabled = DVCVariables::random_enabled();
    if !enabled && !DVCFlags::Initialized.get_value() { return; }
    if GameVariableManager::find_starts_with("G_JG").len() == 0 {
        let force_types = [ForceType::Player, ForceType::Enemy, ForceType::Ally, ForceType::Absent, ForceType::Dead];
        for ff in force_types {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            for unit in force_iter.filter(|f| f.person.get_asset_force() == 0) {
                let key = format!("G_JG_{}", unit.person.pid);
                let current_job = unit.get_job();
                if !GameVariableManager::exist(key.as_str()) {
                    GameVariableManager::make_entry(key.as_str(), current_job.parent.hash);
                }
            }
        }
    }
    let data = get_data_read();
    if !enabled {
        data.growth_data.reset(3);
        return;
    }

    let growth_mode = DVCVariables::PersonalGrowthMode.get_value();
    let class_enable = DVCFlags::RandomClassGrowth.get_value();
    if growth_mode > 0 { randomize_person_grow(); } else { data.growth_data.reset(1); }
    if class_enable { randomize_job_grow();  } else { data.growth_data.reset(2); }


}