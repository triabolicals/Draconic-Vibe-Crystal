use super::*;
use engage::app::mapsight::*;

pub fn init_fog() { 
    GameVariableManager::make_entry_norewind("MapFog", 0);
    update_fog();
}
pub fn update_fog() {
    let sight = MapSight::get_instance();
    let init = sight.get_is_usable();
    let new = 
        match DVCVariable::FogOfWar.get_value() {
            1 => { true }
            2 => { false }
            3 => { 
                let v = Random_2::get_system().get_value_2(2) == 0;
                GameVariableManager::set_bool("MapFog", v);
                v
            }
            _ => { GameUserData::get_instance().get_chapter().get_flag().value & 4 != 0 }
        };
    if init != new {
        sight.set_is_usable(new);
        sight.update_all();
    }
}