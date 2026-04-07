use engage::gamedata::god::GodGrowthData;
use engage::gamedata::{Gamedata, GodData};
use engage::random::Random;
use crate::config::DVCVariables;
use crate::randomizer::{get_data_read, Randomizer};
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
           let mut x: Vec<u8> = (0..8).collect();
            for z in 0..8 { apt[z] = x.get_remove(rng).unwrap(); }
        });
    }
    pub fn commit(&self, gdata: &GameData) {
        let mode = DVCVariables::EmblemWepProf.get_value();
        gdata.emblem_pool.emblem_data.iter().for_each(|e| {e.reset_weapon_prof(); });
        if mode == 1 {
            gdata.emblem_pool.emblem_data.iter().zip(self.apts.iter()).for_each(|(g, apt)| {
                if let Some(grow) = GodGrowthData::try_get_from_god_data(g.get_god()) {
                    grow.iter_mut().filter(|l| l.aptitude.value != 0 ).for_each(|l|{
                        let mut new_mask = 0;
                        for x in 0..8 { if l.aptitude.value & (1 << (x+ 1)) != 0 { new_mask |= 1 << apt[x]; } }
                        l.aptitude.value = new_mask;
                    });
                }
                if let Some(level_data) = g.get_god().get_level_data() {
                    level_data.iter_mut().filter(|x| x.aptitude.value != 0).for_each(|lvl|{
                        let mut new_mask = 0;
                        for x in 0..8 { if lvl.aptitude.value & (1 << (x+ 1)) != 0 { new_mask |= 1 << apt[x]; } }
                        lvl.aptitude.value = new_mask;
                    });
                }
            })
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

pub fn randomized_emblem_apts(on_save_load: bool) {
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_apt_randomized && on_save_load { return; }
    let mode = DVCVariables::EmblemWepProf.get_value();
    let data = get_data_read();
    if mode == 0 {
        if !on_save_load {
            data.emblem_pool.emblem_data.iter().for_each(|emblem| {
                if let Some(god) = GodData::try_get_hash(emblem.hash){
                    if let Some(level) = god.get_level_data() {
                        level.iter_mut().zip(emblem.level_data.iter()).for_each(|(level_data, entry)| {
                            level_data.aptitude.value = entry.apt;
                        });
                    }
                    if let Some(grow) = GodGrowthData::try_get_from_god_data(god) {
                        grow.iter_mut().zip(emblem.growth_apt.iter()).for_each(|(level_data, apt)| {
                            level_data.aptitude.value = *apt;
                        });
                    }
                }
            });
        }
        return;
    }
    let rng = get_rng();
    GameData::get_playable_god_list().iter().for_each(|god|{ randomize_god_apts(god, mode, rng); });
    let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.emblem_apt_randomized = true);
}

fn randomize_god_apts(god: &GodData, mode: i32, rng: &Random) {
    if let Some((level_data, grow_data)) = god.grow_table
        .and_then(|ggid|  GodGrowthData::get_level_data(ggid.to_string()).zip(GodGrowthData::try_get_from_god_data(god)))
    {
        if mode == 1 {  // Randomized
            let mut weapons_set: [bool; 8] = [false; 8];
            let mut apt: [i32; 25] = [0; 25];
            let mut current_apt = 0;
            let max = crate::utils::min(level_data.len() as i32, 24) as usize;
            let gmax =  crate::utils::min(grow_data.len() as i32, 24) as usize;
            for y in 1..max { apt[y] = level_data[y].aptitude.value; }
            
            let mut count = 0;
            let mut kind: usize;

            for y in 2..max {
                if apt[y] == apt[y-1] {
                    level_data[y].aptitude.value = current_apt;
                    continue;
                }
                loop {
                    kind = rng.get_value(8) as usize;
                    if !weapons_set[kind] { break; }
                }
                current_apt |= 1 << ( kind + 1);
                level_data[y].aptitude.value = current_apt;
                weapons_set[kind] = true;
                count += 1;
                if y < gmax { grow_data[y-1].aptitude.value = 1 << ( kind + 1); }
            }
            if count < 3 {
                loop {
                    kind = rng.get_value(8) as usize;
                    if !weapons_set[kind] { break; }
                }
                current_apt |= 1 << ( kind + 1);
                level_data[max-1].aptitude.value = current_apt;
                grow_data[gmax-1].aptitude.value = 1 << ( kind + 1);
            }
            level_data[0].aptitude.value = current_apt;
        }
        else {  // None
            level_data[0].aptitude.value = 0;
            grow_data.iter_mut().for_each(|level|level.aptitude.value = 0);
            level_data.iter_mut().for_each(|level| level.aptitude.value = 0);
        }
    }
}


