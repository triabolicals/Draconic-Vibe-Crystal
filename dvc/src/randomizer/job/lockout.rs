use std::collections::HashSet;
use engage::gamedata::{JobData, PersonData};
use engage::unit::{Unit, UnitPool};
use crate::randomizer::job::unit_change_to_random_class;

pub fn get_all_playable_unit_classes(person: &PersonData) -> HashSet<i32> {
    let sf = UnitPool::get_sf();
    sf.s_unit.iter().filter(|x| x.force.is_some_and(|f| f.force_type < 4))
        .filter(|x| !x.is_summon() && !x.is_vision() && x.person.asset_force == 0 && x.person.parent.hash != person.parent.hash)
        .map(|x| x.job.parent.hash)
        .collect()
}
pub fn change_other_class_after_cc(unit: &Unit, job: &JobData){
    let job_hash = job.parent.hash;
    if job_hash == 499211320 { return; }
    let person_hash = unit.person.parent.hash;

    let sf = UnitPool::get_sf();
    sf.s_unit.iter_mut().filter(|x| x.force.is_some_and(|f| f.force_type < 4))
        .filter(|x| !x.is_summon() && !x.is_vision() && x.person.asset_force == 0 && x.job.parent.hash == job_hash && x.person.parent.hash != person_hash)
        .for_each(|unit|{
            unit_change_to_random_class(unit, false);
            println!("[LockOut] {} Changed to {}", unit.get_name(), unit.job.get_name());
        });
}

pub fn lockout_classes() {
    let sf =  UnitPool::get_sf();
    let units: Vec<_> = sf.s_unit.iter_mut().filter(|x| x.force.is_some_and(|f| f.force_type < 4))
        .filter(|x| !x.is_summon() && !x.is_vision() && x.person.asset_force == 0)
        .collect();
    units.iter().for_each(|unit|{ change_other_class_after_cc(unit, unit.job); });
}
