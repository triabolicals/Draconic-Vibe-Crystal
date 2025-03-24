use engage::{
    random::Random,
    gamedata::{Gamedata, assettable::*},
};
use crate::CONFIG;
pub struct BustData {
    pub entries: Vec<(i32, f32)>,
}

impl BustData {
    pub fn new() -> Self {
        Self {
            entries: 
                AssetTableStaticFields::get().search_lists[2].iter()
                    .filter(|entry| entry.scale_stuff[11] > 0.6)
                    .map(|entry| (entry.parent.index, entry.scale_stuff[11]))
                    .collect()
        }
    }
    pub fn apply_bust_changes(&self) {
        let value = CONFIG.lock().unwrap().misc_option_1;
        if value  <= 0.4 { self.reset_busts(); }
        else if value >= 4.75 { self.randomized_busts(); }
        else { self.set_busts(); }
    }
    pub fn reset_busts(&self) {
        self.entries.iter().for_each(|bentry|{ AssetTable::try_index_get_mut(bentry.0).map(|entry| entry.scale_stuff[11] = bentry.1).unwrap(); });
    }
    pub fn set_busts(&self) {
        let rng = Random::get_game();
        let value = CONFIG.lock().unwrap().misc_option_1;
        let x0 = value * 2.5 / 5.0;
        let var = ( value - x0 ) * 0.1;
        self.entries.iter().for_each(|bentry|{ AssetTable::try_index_get_mut(bentry.0).map(|entry| entry.scale_stuff[11] == x0 + rng.get_value(10) as f32 * var).unwrap(); } );
    }
    pub fn randomized_busts(&self) {
        let rng = Random::get_game();
        self.entries.iter().for_each(|bentry|{ AssetTable::try_index_get_mut(bentry.0).map(|entry| entry.scale_stuff[11] = 1.0 + rng.get_value(50) as f32 * 0.02).unwrap(); });
    }

}