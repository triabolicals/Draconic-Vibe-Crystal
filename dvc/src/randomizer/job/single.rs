use engage::{gamedata::{Gamedata, JobData}, gamevariable::GameVariableManager, };
use crate::{config::{DVCFlags, DVCVariables}, utils::dlc_check};

pub fn get_next_class(current_hash_job: i32, increase: bool) -> i32 {
    let job_count = JobData::get_count();
    let hash = current_hash_job;
    let mut current_index = if current_hash_job == 1 && dlc_check() { 1 }
    else {
        JobData::try_get_hash(hash)
            .filter(|x| x.flag.value & 23 == 3 && (x.is_high() || x.max_level >= 40))
            .map(|j| j.parent.index).unwrap_or(0)
    };

    loop {
        if increase {
            if current_index < 0 { current_index = 1; }
            else if current_index < job_count { current_index += 1; }
            else if current_index >= job_count && dlc_check() { return 1; }
            else { current_index = 0; }
        }
        else {
            if current_index >= job_count && dlc_check() { return 1; }
            else if current_index > 1 { current_index -= 1 }
            else if current_index == 0 { current_index = job_count - 1;}
            else { current_index = 0; }
        }
        if let Some(job) = JobData::try_index_get(current_index).filter(|x| x.flag.value & 23 == 3 && (x.is_high() || x.max_level >= 40)) {
            let jid = job.jid.to_string();
            if (jid == "JID_マージカノン" || jid == "JID_エンチャント") && !dlc_check() { continue; }
            return job.parent.hash;
        }
    }
}

pub fn single_class_exists() {
    if GameVariableManager::get_number(DVCVariables::SINGLE_CLASS) != 0 {
        let hash = GameVariableManager::get_number(DVCVariables::SINGLE_CLASS);
        if hash != 1 && !JobData::try_get_hash(hash).is_none() {
            GameVariableManager::set_number(DVCVariables::SINGLE_CLASS, 0);
            DVCFlags::SingleJobEnabled.set_value(false);
            DVCVariables::ClassMode.set_value(0);
        }
    }
}