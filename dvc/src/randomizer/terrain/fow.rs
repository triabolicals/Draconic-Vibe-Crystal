use engage::{map::{sight::MapSight, situation::MapSituation}, util::get_instance};
use super::*;
pub fn rando_fow() {
    let sit = get_instance::<MapSituation>();
    if DVCVariables::FogOfWar.get_value() == 3 && sit.current_force == 2 {
        let rng = Random::get_system();
        let sight = get_instance::<MapSight>();
        sight.usable = rng.get_value(10) % 2 == 0;
    }
}
pub fn resume_fow() {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 {
        let sight = get_instance::<MapSight>();
        let initial = sight.usable;
        match DVCVariables::FogOfWar.get_value() {
            1 => { sight.usable = true; }   // Always On
            2 => { sight.usable = false; }  // Always Off
            3 => {  // Turn By Turn
                let rng = Random::get_system();
                sight.usable = rng.get_value(10) % 2 == 0;
            }
            4 => { sight.usable = GameVariableManager::get_bool("MapFog"); }    // MapDefined
            _ => { sight.usable = GameUserData::get_chapter().flag & 4 != 0  }  // Default
        }
        if initial != sight.usable { sight.update_all(); }
    }
}
pub fn map_start_fow() {
    let sight = get_instance::<MapSight>();
    let initial = sight.usable;
    let rng = Random::get_system();
    let value = match DVCVariables::FogOfWar.get_value()  {
        1 => { true }
        2 => { false }
        3 => { rng.get_value(10) % 2 == 0  }
        _ => { GameUserData::get_chapter().flag & 4 != 0 }
    };
    sight.usable = GameVariableManager::make_entry_norewind("MapFog", (rng.get_value(10) % 2 == 0) as i32);
    sight.usable = value;
    if sight.usable != initial {   sight.update_all(); }
}