use std::collections::HashSet;
use engage::{gamedata::{JobData, PersonData}, unit::{Unit, UnitPool}};
use crate::randomizer::job::reclass::{unit_reclass, ReclassType};
use crate::utils::for_each_unit;

pub fn get_all_playable_unit_classes(person: &PersonData) -> HashSet<i32> {
    let sf = UnitPool::get_sf();
    sf.s_unit.iter().filter(|x| x.force.is_some_and(|f| f.force_type < 4))
        .filter(|x| !x.is_summon() && !x.is_vision() && x.person.asset_force == 0 && x.person.parent.hash != person.parent.hash)
        .map(|x| x.job.parent.hash)
        .collect()
}
pub fn change_other_class_after_cc(unit: &Unit, job: &JobData) {
    let job_hash = job.parent.hash;
    if job_hash == 499211320 { return; }
    let person_hash = unit.person.parent.hash;
    for_each_unit(11, |unit|{
        if playable_reclassable_unit(unit) && unit.job.parent.hash == job_hash && unit.person.parent.hash != person_hash {
            unit_reclass(unit, ReclassType::PlayerLockout(false, true));
        }
    });
}
pub fn lockout_classes() {
    for_each_unit(11, |unit| {
        if playable_reclassable_unit(unit) { change_other_class_after_cc(unit, unit.job); }
    });
}
fn playable_reclassable_unit(unit: &Unit) -> bool {
    !unit.is_summon() && !unit.is_summon() && unit.person.asset_force == 0
}