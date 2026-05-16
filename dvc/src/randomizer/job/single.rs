use engage::{gamedata::{Gamedata, JobData}};
use crate::{config::{DVCFlags, DVCVariables}, utils::dlc_check};
use crate::randomizer::job::{LUEUR_CLASS, MONSTER_CLASS};

fn valid_single_class(job: &JobData) -> bool {
    let hash = job.parent.hash;
    if hash == super::ENCHANTER || hash == super::MAGE_CANNON || hash == MONSTER_CLASS[2] || hash == MONSTER_CLASS[4] { dlc_check() }
    else if hash == MONSTER_CLASS[0] || hash == LUEUR_CLASS[1] { true }
    else {
        job.get_flag().value & 23 == 3 && (job.is_high() || (job.is_low() && !job.has_high_jobs()))
    }
}

pub fn get_next_class(current_hash_job: i32, increase: bool) -> i32 {
    let job_count = JobData::get_count();
    let dlc = dlc_check();
    let hash = current_hash_job;
    let mut current_index =
        if current_hash_job == 1000 && dlc { 1000 }
        else if (hash == super::ENCHANTER || hash == super::MAGE_CANNON) && !dlc_check() { 0 }
        else {
            JobData::try_get_hash(hash)
                .filter(|x| valid_single_class(x))
                .map(|j| j.parent.index)
                .unwrap_or(0)
        };

    loop {
        if increase {
            if current_index == 1000 { current_index = 0 }
            else if (current_index + 1) == job_count {
                if dlc_check() { return 1000; } else { current_index = 0; }
            }
            else { current_index += 1; }
        }
        else {
            if current_index == 1000 { current_index = job_count - 1; }
            else if current_index == 0 {
                if dlc_check() { return 1000; } else { current_index = job_count - 1; }
            }
            else { current_index -= 1; }
        }
        if let Some(job) = JobData::try_index_get(current_index)
            .filter(|x| valid_single_class(x))
        {
            return job.parent.hash;
        }
    }
}

pub fn single_class_exists() {
    let v = DVCVariables::SingleJob.get_value();
    let enable = DVCFlags::SingleJobEnabled.get_value();
    let class_mode = DVCVariables::ClassMode.get_value();
    if enable {
        let exist = (v == 1000 && !dlc_check()) || JobData::try_get_hash(v).is_some();
        if !exist {
            DVCVariables::SingleJob.set_value(0);
            if class_mode == 2 { DVCVariables::ClassMode.set_value(0); }
        }
    }
    else {
        DVCVariables::SingleJob.set_value(0);
        if class_mode == 2 { DVCVariables::ClassMode.set_value(0); }
    }
}