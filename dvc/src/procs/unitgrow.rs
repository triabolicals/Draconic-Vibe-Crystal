use engage::{gameuserdata::GameUserData, sequence::unitgrowsequence::UnitGrowSequence};
use crate::{DVCVariables, randomizer::job::lockout::change_other_class_after_cc};
use unity::prelude::OptionalMethod;


pub extern "C" fn unit_grow_gain_exp(proc: &mut UnitGrowSequence, _method_info: OptionalMethod) {
    if DVCVariables::Continuous.get_value() > 0 {
        if proc.skill_point < proc.exp { proc.skill_point += proc.exp + 25 * (2 - GameUserData::get_difficulty(false)); }
    }
    proc.gain_exp();
}
pub extern "C" fn unit_grow_sequence_prepare(proc: &mut UnitGrowSequence, _: OptionalMethod) {
    if DVCVariables::ClassMode.get_value() >= 3 {
        if let Some(job) = proc.class_change_job.as_ref() { change_other_class_after_cc(proc.unit, job); }
    }
    proc.prepare();
}
