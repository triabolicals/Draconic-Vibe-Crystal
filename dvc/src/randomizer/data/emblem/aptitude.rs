use engage::gamedata::god::GodGrowthData;
use engage::gamedata::{GodData};
use engage::random::Random;
use crate::config::DVCVariables;
use crate::randomizer::{Randomizer};
use crate::randomizer::data::GameData;
use crate::utils::get_rng;


pub struct EmblemAptitudeRandomizer {
    pub apts: Vec<[u8; 8]>,
}
impl EmblemAptitudeRandomizer {
    pub fn new(n_emblems: usize) -> Self { Self { apts: vec![[0; 8]; n_emblems], } }
    pub fn randomize(&mut self) {
        let rng = get_rng();
        self.apts.iter_mut().for_each(|apt| {
            let mut x: Vec<u8> = (1..9).collect();
            for z in 0..8 { apt[z] = x.get_remove(rng).unwrap(); }
        });
    }
    pub fn commit(&self, gdata: &GameData) {
        let mode = DVCVariables::EmblemWepProf.get_value();
        gdata.emblem_pool.emblem_data.iter().for_each(|e| {e.reset_weapon_prof(); });
        if mode == 1 {
            gdata.emblem_pool.emblem_data.iter().zip(self.apts.iter()).for_each(|(g, apt)| {
                if let Some(grow) = GodGrowthData::try_get_from_god_data(g.get_god()) {
                    grow.iter_mut().for_each(|l| {
                        let old_apt = l.aptitude.value;
                        if old_apt != 0 {
                            let mut new_mask = 0;
                            for x in 0..8 {
                                if old_apt & (2 << x) != 0 { new_mask |= 1 << apt[x]; }
                            }
                            l.aptitude.value = new_mask;
                        }
                    });
                }
                if let Some(level_data) = g.get_god().get_level_data() {
                    level_data.iter_mut().for_each(|l| {
                        let old_apt = l.aptitude.value;
                        if old_apt != 0 {
                            let mut new_mask = 0;
                            for x in 0..8 {
                                if old_apt & (2 << x) != 0 { new_mask |= 1 << apt[x]; }
                            }
                            l.aptitude.value = new_mask;
                        }
                    });
                }
            });
        }
        else if mode == 2 {
            gdata.emblem_pool.emblem_data.iter().map(|d| d.get_god()).for_each(|god|{
                if let Some(level) = god.get_level_data() {
                    level.iter_mut().for_each(|l| { l.aptitude.value = 0; });
                }
                if let Some(growth) = GodGrowthData::try_get_from_god_data(god) { 
                    growth.iter_mut().for_each(|g| g.aptitude.value = 0);
                }
            });
        }
    }
}