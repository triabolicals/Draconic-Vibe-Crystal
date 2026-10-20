use std::collections::HashSet;
use engage::{app::{IBitField32, IJobDataMethods, IPersonDataMethods, IStructBase, IStructData_1Methods, IUnit, IUnitMethods, ItemData, JobData, PersonData, Random_2, Unit, UnitPool}, GameVariableManager, List_1Ext, Mess};
use outfit_core::Randomizer;
use unity::{Cast, IlNull};
use crate::{dvc::{
    unit::{
        assign_selected_weapon_mask_by_apt,
        randomize_selected_weapon_mask,
        learn_skill::update_learn_skill,
        weapon_mask
    }
}, DVCVariables, gamedata::{ClassTier, ReclassType, JobDataBase, PersonPool, GameData}, utils::{clamp_value, to_opt}, DVCFlags, DVCVariable, gamedata};
use crate::dvc::DVC;
use crate::dvc::unit::grow::get_scaling_level;
use crate::gamedata::GrowthData;

// Get New Class, does not adjust level to scaling level.
pub fn unit_reclass(unit: Unit, kind: ReclassType) -> bool {
    if !DVCFlags::RandomEnable.get_value() {
        if unit.get_person().get_asset_force().value != 0 { return false }
        match kind {
            ReclassType::Recruitment(_)|ReclassType::Assigned => {}
            _ => { return false; }
        }
    }
    let current_job = unit.get_job();
    if current_job.get_jid().to_rust_string().starts_with("JID_紋章士_") { return false; }
    let old_class = current_job.hash();
    let person = unit.get_person();
    let gender = Unit::get_gender_2(unit.m_edit(), unit.get_person());
    let female = if person.get_flag().m_value() & 32 != 0 { gender.value == 1 } else { gender.value == 2};
    let mut tier = ClassTier::from_job(current_job);
    let mut level = unit.get_level();
    let mut il = unit.get_internal_level();
    let mut initial_apt = false;
    match kind {
        ReclassType::Assigned => {
            let mut random = false;
            if let Some(idx) = PersonPool::get_person_index(unit.get_person().hash()).filter(|&i| i < 41){
                GrowthData::set_temp_key(unit.get_person(), DVC::get().unit_job[idx as usize]);
                if let Some(job) = GrowthData::get_starting_class(unit){
                    if let Some(level) = get_scaling_level() { tier = if level < 16 { ClassTier::Base } else { ClassTier::Promoted } }
                    else if let Some(old_person) = PersonPool::get_replacement_2(unit.get_person(), true){
                        let old_job = old_person.get_job();
                        tier = ClassTier::from_job(old_job);
                        level = old_person.get_level() as i32;
                        il = old_person.get_internal_level() as i32;
                        if old_job.is_high() && il == 0 { il = 20; }
                    }
                    let cc = JobDataBase::get().get_reclass_job(unit, job, tier).unwrap_or(job);
                    let changed = cc.hash() != old_class;
                    if changed {
                        unit.class_change(cc, ItemData::null());
                        reclass_level_adjustment(unit, level, il, tier);
                        assign_selected_weapon_mask_by_apt(unit, None);
                        weapon_mask::update_weapon_apt(unit, initial_apt);
                        unit.set_hp(unit.get_capability(0, true));
                        update_learn_skill(unit);
                    }
                    GrowthData::set_starting_class(unit);
                    GrowthData::adaptive_grow(unit.get_person());
                    return changed;
                }
                else {
                    random = GameVariableManager::get_number(GrowthData::get_starting_class_key(unit.get_person())) == -1;
                }
            }
            return unit_reclass(unit, ReclassType::Recruitment(random));
        }
        ReclassType::Enemy => {
            let leader = unit.has_private_skill("SID_リーダー");
            let slvl = get_scaling_level().map(|v| if leader { v + 3} else { v });
            let high_job = (current_job.is_high() && (leader || slvl.is_none_or(|v| v > 20))) || (current_job.get_max_level() > 20 && (slvl.unwrap_or(level) > 20));
            let pool: Vec<_> = JobData::get_list().iter()
                .filter(|j| JobDataBase::can_unit_reclass(*j, female, high_job, false, false))
                .collect();

            if let Some(j) = pool.get_random_element(Random_2::get_game()) {
                unit.class_change(*j, ItemData::null());
                reclass_level_adjustment(unit, level, il, tier);
            }
        }
        ReclassType::Recruitment(random) => {
            if let Some(old_person) = PersonPool::get_replacement_2(unit.get_person(), true){
                level = old_person.get_level() as i32;
                let old_job = old_person.get_job();
                let tier =  ClassTier::from_job(old_job);
                let mut class_job = unit.get_person().get_job();
                if random {
                    let high_job = get_scaling_level().is_none_or(|v| v > 15) && (tier == ClassTier::Promoted || level > 20);
                    let pool: Vec<_> = JobData::get_list().iter()
                        .filter(|j| JobDataBase::can_unit_reclass(*j, female, high_job, true, false))
                        .collect();
                    if let Some(job) = pool.get_random_element(Random_2::get_game()) { class_job = *job; }
                }
                recruitment_job_level_adjustment(unit, old_person, class_job, random);
            }
            GrowthData::set_starting_class(unit);
            GrowthData::adaptive_grow(unit.get_person());
            initial_apt = true;
        }
        ReclassType::PlayerSingle(recruitment) => {
            if recruitment {
                if let Some(old_person) = PersonPool::get_replacement_2(unit.get_person(), true){
                    level = old_person.get_level() as i32;
                    il = old_person.get_internal_level() as i32;
                    let old_job = old_person.get_job();
                    if old_job.is_high() && il == 0 { il = 20; }
                    tier = ClassTier::from_job(old_job);
                }
            }
            let is_base = tier == ClassTier::Base || level > 20 || (get_scaling_level().is_some_and(|v| v < 16) && recruitment);
            if let Some(job) = DVCVariables::get_single_class(is_base, female){
                unit.class_change(job, ItemData::null());
                reclass_level_adjustment(unit, level, il, tier);
            }
            if recruitment {
                GrowthData::set_starting_class(unit);
                GrowthData::adaptive_grow(unit.get_person());
                initial_apt = true;
            }
        }
        ReclassType::GenderSingle(recruitment) => {
            let gender = unit.get_dress_gender();
            let is_base = tier == ClassTier::Base || level > 20 || (get_scaling_level().is_some_and(|v| v < 16) && recruitment);
            if let Some(job) = DVCVariables::get_single_class_gender(is_base, gender){
                unit.class_change(job, ItemData::null());
                reclass_level_adjustment(unit, level, il, tier);
                GrowthData::set_starting_class(unit);
                if recruitment {
                    GrowthData::adaptive_grow(unit.get_person());
                    initial_apt = true;
                }
            }
            else if recruitment { return unit_reclass(unit, ReclassType::Recruitment(recruitment)); }
        }
        ReclassType::PlayerLockout(recruitment, random) => {
            let playable = get_current_player_unit_classes();
            let mut selected_job = 0;
            let scaled_base_class = get_scaling_level().is_some_and(|v| v < 16) && recruitment;
            if recruitment {
                if let Some(old_person) = PersonPool::get_replacement_2(unit.get_person(), true){
                    level = old_person.get_level() as i32;
                    il = old_person.get_internal_level() as i32;
                    let old_job = old_person.get_job();
                    if old_job.is_high() && il == 0 { il = 20; }
                    if scaled_base_class { tier = ClassTier::Base } else { tier = ClassTier::from_job(old_job); }
                    if !random {
                        if let Some(job) = GameData::get().job_db.get_reclass_job(unit, unit.get_person().get_job(), tier) {
                            if !playable.contains(&job.hash()) { selected_job = job.hash(); }
                        }
                    }
                }
                initial_apt = true;
            }
            if let Some(job) = to_opt(JobData::try_get_from_hash(selected_job)){
                unit.class_change(job, ItemData::null());
                reclass_level_adjustment(unit, level, il, tier);
            }
            else {
                let is_high = !scaled_base_class && ( tier == ClassTier::Promoted || (level > 20));
                let pool: Vec<_> = JobData::get_list().iter()
                    .filter(|j| JobDataBase::can_unit_reclass(*j, female, is_high, true, false) && !playable.contains(&j.hash()))
                    .collect();

                if let Some(j) = pool.get_random_element(Random_2::get_game()) {
                    unit.class_change(*j, ItemData::null());
                    reclass_level_adjustment(unit, level, il, tier)
                }
                else {
                    unit.class_change(JobData::try_get_from_hash(gamedata::VILLAGER), ItemData::null());
                    reclass_level_adjustment(unit, level, il, tier)
                }
            }
            if recruitment {
                initial_apt = true;
                GrowthData::set_starting_class(unit);
                GrowthData::adaptive_grow(unit.get_person());
            }
        }
    }
    assign_selected_weapon_mask_by_apt(unit, None);
    weapon_mask::update_weapon_apt(unit, initial_apt);
    unit.set_hp(unit.get_capability(0, true));
    update_learn_skill(unit);
    unit.get_job().hash() != old_class
}
pub fn get_lookout_unit_reclass() -> Vec<i32> {
    let mut job_hashes: HashSet<i32> = HashSet::new();
    let mut person_list: Vec<i32> = vec![];
    UnitPool::s_units().iter()
        .filter(|u| !u.get_person().is_null() && !u.get_force().is_null() && !u.get_job().is_null())
        .filter(|u| PersonPool::is_playable(u.get_person()))
        .for_each(|u|{
            let hash = u.get_job().hash();
            if !job_hashes.contains(&hash) { job_hashes.insert(hash); }
            else { person_list.push(u.get_person().hash()); }
        });
    person_list
}

pub fn get_current_player_unit_classes() -> Vec<i32> {
    UnitPool::s_units().iter()
        .filter(|u| !u.get_person().is_null() && !u.get_force().is_null() && !u.get_job().is_null())
        .filter(|u| PersonPool::is_playable(u.get_person()))
        .map(|u| u.get_job().hash())
    .collect()
}

pub fn recruitment_job_level_adjustment(unit: Unit, old_person: PersonData, target: JobData, random: bool) {
    let old_level = old_person.get_level() as i32;
    let old_il = old_person.get_internal_level() as i32;
    let old_tier = ClassTier::from_job(old_person.get_job());
    let new_tier = ClassTier::from_job(target);
    match (old_tier, new_tier) {
        (ClassTier::Base, ClassTier::Promoted)|(ClassTier::Promoted, ClassTier::Base) => {
            if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, old_tier) {
                unit.class_change(job,  ItemData::null());
                unit.set_hp(unit.get_capability(0, true));
            }
        }
        (ClassTier::Special, ClassTier::Base) => {
            if old_level > 20 {
                if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, ClassTier::Promoted) {
                    unit.class_change(job,  ItemData::null());
                }
            }
            else { unit.class_change(target,  ItemData::null()); }
        }
        (ClassTier::Special, ClassTier::Promoted) => {
            if old_level <= 20 {
                if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, ClassTier::Base) { unit.class_change(job,  ItemData::null()); }
            }
            else { unit.class_change(target, ItemData::null()); }
        }
        _ => { unit.class_change(target,  ItemData::null()); }
    }
    if random { randomize_selected_weapon_mask(unit, None); }
    else { assign_selected_weapon_mask_by_apt(unit, None); }
    reclass_level_adjustment(unit, old_level, old_il, old_tier);
}
fn reclass_level_adjustment(unit: Unit, target_level: i32, target_il: i32, from: ClassTier) {
    let total_level = target_level + target_il;
    let job_max_level = unit.get_job().get_max_level() as i32;
    let to = ClassTier::from_job(unit.get_job());
    let mut set_level = target_level;
    if to != from {
        match (from, to) {
            (ClassTier::Special, ClassTier::Base) => {
                if total_level > job_max_level { set_level = clamp_value(total_level - job_max_level, 1, job_max_level); }
            }
            (ClassTier::Special, ClassTier::Promoted) => {
                set_level =
                    if total_level > (job_max_level + 5) { clamp_value(total_level - job_max_level, 1, job_max_level) } 
                    else if total_level > (job_max_level - 5) { clamp_value(total_level - job_max_level - 5, 1, job_max_level) } 
                    else if total_level > 20 { clamp_value(total_level - 20, 1, job_max_level) } else { total_level }
            }
            (ClassTier::Promoted|ClassTier::Base, ClassTier::Special) => { set_level = clamp_value(total_level, 1, job_max_level); }
            _ => {}
        }
    }
    unit.set_level(set_level);
    let max_il = 50;
    unit.set_internal_level(clamp_value(total_level-set_level, 0, max_il));
}