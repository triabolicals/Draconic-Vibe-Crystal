use engage::god::GodPool;
use crate::{randomizer::{data::EmblemPool, Randomizer}};
use super::*;

pub fn try_equip_emblem(unit: &Unit) -> bool {
    if unit.person.gender == 0 || unit.person.gender == 3  { return false; }
    if let Some(god_data) = EmblemPool::available_opp_emblems().get_random_element(Random::get_game()) {
        if let Some(god_unit) = GodPool::try_get(god_data, false).or_else(|| GodPool::create(god_data)) {
            let valid = unit.try_connect_god_unit(god_unit).is_some();
            god_unit.set_escape(true);
            return valid;
        }
    }
    false
}
