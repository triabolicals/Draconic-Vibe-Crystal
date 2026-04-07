use engage::gameuserdata::GameUserData;
use engage::sequence::unitgrowsequence::UnitGrowSequence;
use unity::prelude::OptionalMethod;
use crate::DVCVariables;

pub extern "C" fn unit_grow_gain_exp(proc: &mut UnitGrowSequence, _method_info: OptionalMethod) {
    if DVCVariables::Continuous.get_value() > 0 {
        if proc.skill_point < proc.exp { proc.skill_point += proc.exp + 25 * (2 - GameUserData::get_difficulty(false)); }
    }
    proc.gain_exp();
}
