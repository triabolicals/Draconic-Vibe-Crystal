use engage::{
    app::{IJobDataMethods, IStructData_1Methods, IUnitMethods, JobData, SkillData, Unit},
};
use outfit_core::Randomizer;
use crate::{dvc::{unit::util::for_each_unit}, gamedata::find_first_non_hidden, utils::to_opt};
use super::*;

pub fn update_all_unit_learn_skills() {
    for_each_unit(31, |u|{ update_learn_skill(u) });
}
pub fn update_learn_skill(unit: Unit) {
    let job = unit.get_job();
    if !unit.m_learned_job_skill().is_null() || (get_job_learn_skill_level(unit, job) <= unit.get_level()) {
        if let Some(skill) = get_learn_job_skill(unit, unit.get_job()) { unit.set_learned_job_skill(skill); }
        else if job.get_learning_skill().is_null() { unit.set_learned_job_skill(SkillData::null()); }
    }
}
pub fn get_job_learn_skill_level(unit: Unit, job: JobData) -> i32 {
    let max_level = (job.get_max_level() as i32);
    let mode = DVCVariable::RandomLearnLevel.get_value();
    match DVCVariable::RandomLearnLevel.get_value() {
        1|2 => {
            let hash = if mode == 1 { unit.get_person().hash() } else { job.hash() };
            let rng = DVCVariables::create_hash_rng(hash, 2);
            rng.get_value_2((max_level * 3)/ 4) + 1
        }
        3 => {
            let hash = ( unit.get_person().hash() >> 2 ) + ( job.hash() >> 2 ) + ( DVCVariables::get_seed() >> 1 );
            let rng = Random_2::new_2(hash as u32);
            rng.get_value_2(max_level) + 1
        }
        _ => { job.get_learn_job_skill_level() }
    }
}
pub fn get_learn_job_skill(unit: Unit, job: JobData) -> Option<SkillData> {
    let job_learn_mode = DVCVariable::JobLearn.get_value();
    let learning_sid = job.get_learning_skill();
    if learning_sid.is_null()  { None }
    else {
        match job_learn_mode {
            1 => {
                let learn_skill_hash = GameVariableManager::get_number(format!("G_L_{}", job.get_jid()));
                to_opt(SkillData::try_get_from_hash(learn_skill_hash))
            }
            2 => { random_job_learn_skill(unit, job) }
            _ => { to_opt(SkillData::get(learning_sid)) }
        }
    }
}

pub fn random_job_learn_skill(unit: Unit, job: JobData) -> Option<SkillData> {
    let hash = ( unit.get_person().hash() >> 2 ) + ( job.hash() >> 2 ) + ( DVCVariables::get_seed() >> 1 );
    let rng = Random_2::new_2(hash as u32);
    let skill_pool = &GameData::get().skill_pool;
    let available = skill_pool.non_upgrades.clone();
    let mut count = 0;
    let personal = find_first_non_hidden(unit.get_person().get_common_skills()).map(|x| x.hash()).unwrap_or(0);
    let mut weapon_mask = 0;
    for x in 1..9 { if job.get_max_weapon_level(x).value > 0 { weapon_mask |= 1 << x; } }
    loop {
        if let Some(new_skill_hash) = available.get_filter(rng, |&x| x != personal){
            let skill = SkillData::try_get_from_hash(*new_skill_hash);
            if skill_pool.job_restrictions.is_valid_for_weapon_mask(skill, weapon_mask) || count > 100 {
                return Some(skill);
            }
            else { count += 1;}
        }
    }
}
