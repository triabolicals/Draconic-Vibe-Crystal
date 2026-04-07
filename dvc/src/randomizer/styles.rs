use engage::gamedata::{skill::SkillData, *};
use engage::gamedata::job::BattleStyles;
use engage::gamedata::skill::SkillDataCategorys;
use crate::config::DVCFlags;
use crate::randomizer::data::GameData;
use super::{get_data_read, DVCVariables, Randomizer};

pub fn randomize_job_styles() {
    if !DVCVariables::random_enabled() { return; }
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    match DVCVariables::BattleStyles.get_value() {
        1 => { job_list.iter_mut().for_each(|job| { job.style = rng.get_value(8) + 1; }); },
        2 => { job_list.iter_mut().for_each(|job| { job.style = 0; }); },
        _ => { get_data_read().job_style_attr.iter().zip(JobData::get_list_mut().unwrap().iter_mut()).for_each(|(x, j)| { j.style = x.0; }); },
    }
    job_list.iter_mut().for_each(|job| {
        job.mask_skills.clear();
        if let Some(array) = job.skills {
            array.iter().flat_map(|sid| SkillData::get(*sid)).for_each(|skill| { job.mask_skills.add_skill(skill, SkillDataCategorys::Job, 0); });
        }
        if let Some(array) = BattleStyles::get_skills2(job.style) {
            array.iter().flat_map(|sid| SkillData::get(*sid)).for_each(|skill| { job.mask_skills.add_skill(skill, SkillDataCategorys::Job, 0); });
        }
    });
}
pub fn randomize_job_attrs(){
    if !DVCVariables::random_enabled() { return; }
    let job_list = JobData::get_list_mut().unwrap();
    let rng = crate::utils::get_rng();
    if DVCFlags::RandomClassAttrs.get_value() {
        job_list.iter_mut().for_each(|j|{
            let mut v = vec![2, 4, 8, 16, 32, 64, 2, 4, 8, 16];
            j.attrs = 0;
            let mut rate = 50;
            for _ in 0..3 {
                if rng.get_value(100) < rate {
                     j.attrs |= v.get_remove(rng).unwrap_or(0);
                    rate = rate >> 1;
                }
                else { break; }
            }
        });
    }
    else { job_list.iter_mut().zip(GameData::get().job_style_attr.iter()).for_each(|(j, d)|{ j.attrs = d.1; }); }
}