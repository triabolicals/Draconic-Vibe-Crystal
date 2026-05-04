use engage::god::GodPool;
use crate::randomizer::data::GameData;
use super::*;

pub fn try_equip_emblem(unit: &Unit, emblem: usize) -> bool {
    if let Some(g_unit) = GameData::get_playable_god_list().get(emblem)
        .and_then(|god_data| GodPool::try_get(god_data, true))
    {
        if !GameVariableManager::exist("EnemyEmblemSet") { GameVariableManager::make_entry_norewind("EnemyEmblemSet", 0); }
        let mut emblem_set_flag = GameVariableManager::get_number("EnemyEmblemSet");
        if unit.person.gender == 0 || unit.person.gender == 3  { return false; }
        if emblem < 31 { if emblem_set_flag & (1 << emblem) != 0 { return false; }  }
        let gid = g_unit.data.main_data.gid.to_string();
        
        
        if let Some(god_unit) = GodData::get(gid.replace("GID_", "GID_相手")).and_then(|data| GodPool::create(data)){
            let valid = unit.try_connect_god_unit(god_unit).is_some();
            god_unit.set_escape(true);
            if valid && emblem < 31 {
                emblem_set_flag |= 1 << emblem;
                GameVariableManager::set_number("EnemyEmblemSet", emblem_set_flag);
            }
            return valid;
        }
    }
    false
}
