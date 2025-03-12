use super::*;
use engage::{
    map::{sight::MapSight, situation::MapSituation},
    util::get_instance,
};

pub fn change_map_sight() {
    if GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3 {
        let sight = get_instance::<MapSight>();
        let inital = sight.usable;
        let value = match GameVariableManager::get_number(DVCVariables::FOW) {
            1 => { true }
            2 => { false }
            3 => {
                let rng = Random::get_system();
                rng.get_value(2) == 1
            }
            4 => { GameVariableManager::get_bool("MapFog") }
            _ => { GameUserData::get_chapter().get_flag() & 4 != 0 }
        };
        sight.usable = value;
        if GameVariableManager::get_number(DVCVariables::FOW) != 3 && inital != value {
            sight.update_all();
        }
    }

}
pub fn rando_fow() {
    let sit = get_instance::<MapSituation>();
    if GameVariableManager::get_number(DVCVariables::FOW) == 3 && sit.current_force == 2 {
        let rng = Random::get_system();
        let sight = get_instance::<MapSight>();
        let initial = sight.usable;
        sight.usable = rng.get_value(10) % 2 == 0;
        if sight.usable != initial { println!("Player Turn FOW Change to: {}", sight.usable); }
    }
}
pub fn resume_fow() {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 {
        println!("Resume FOW");
        let sight = get_instance::<MapSight>();
        let initial = sight.usable;
        match GameVariableManager::get_number(DVCVariables::FOW) {
            1 => { sight.usable = true; }
            2 => { sight.usable = false; }
            3 => {
                let rng = Random::get_system();
                sight.usable = rng.get_value(10) % 2 == 0;
            }
            4 => { sight.usable = GameVariableManager::get_bool("MapFog"); }
            _ => {}
        }
        if initial != sight.usable { sight.update_all(); }
    }
}
pub fn map_start_fow() {
    let sight = get_instance::<MapSight>();
    let initial = sight.usable;
    let rng = Random::get_system();
    let value = match GameVariableManager::get_number(DVCVariables::FOW) {
        1 => { true }
        2 => { false }
        3 => { rng.get_value(10) % 2 == 0  }
        _ => { GameUserData::get_chapter().get_flag() & 4 != 0 }
    };
    sight.usable = GameVariableManager::make_entry_norewind("MapFog", (rng.get_value(10) % 2 == 0) as i32);
    sight.usable = value;
    if sight.usable != initial {   sight.update_all(); }
}