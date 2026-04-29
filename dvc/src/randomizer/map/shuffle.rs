use std::collections::{HashMap, HashSet};
use engage::{
    map::terrain::MapTerrain, random::Random,
    gamedata::{GamedataArray, dispos::DisposData},
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
};
use crate::{config::DVCFlags, randomizer::Randomizer, utils};

pub struct MapRect {
    pub x1: i8,
    pub z1: i8,
    pub x2: i8,
    pub z2: i8,
}


pub struct MapTileData {
    pub exclude_tile: Vec<(i8, i8)>,
    pub areas: Vec<MapRect>,
}

impl MapTileData {
    pub fn new() -> Self {
        Self {
            exclude_tile: vec![],
            areas: vec![],
        }
    }
    pub fn add(&mut self, line: &str) -> bool {
        let spilt: Vec<_> = line.split_whitespace().collect();
        if spilt.len() == 2 {
            if let Some((x, z)) = spilt[0].parse::<i8>().ok().zip(spilt[1].parse::<i8>().ok()) {
                self.exclude_tile.push((x, z));
                true
            }
            else { false }
        }
        else if spilt.len() == 4 {
            let x1 = spilt[0].parse::<i8>().ok().unwrap_or(0);
            let z1 = spilt[1].parse::<i8>().unwrap_or(0);
            let x2 = spilt[2].parse::<i8>().unwrap_or(0);
            let z2 = spilt[0].parse::<i8>().unwrap_or(0);
            if x1 != 0 && z1 != 0 && x2 != 0 && z2 != 0 {
                self.areas.push( MapRect { x1, x2, z1, z2 });
                true
            }
            else { false }
        }
        else { false }
    }
    pub fn get_area(&self, pos_x: i8, pos_z: i8) -> i8 {
        if self.areas.is_empty() { return 0; }
        if let Some(pos) = self.areas.iter().position(|x| pos_x > x.x1 && pos_x < x.x2 && pos_z > x.z1 && pos_z < x.z2) {
            pos as i8 + 1
        }
        else { 0 }
    }
    pub fn exclude(&self, pos_x: i8, pos_y: i8) -> bool {
        self.exclude_tile.contains(&(pos_x, pos_y))
    }
}

pub fn get_tile_restrictions() ->  MapTileData  {
    let my_str = include_str!("../../../data/shuffle");
    let binding = GameUserData::get_chapter().cid.to_string();
    let cid = binding.trim_start_matches("CID_");
    let mut read_slots = false;
    let mut out: MapTileData = MapTileData::new();
    let lines = my_str.lines();
    for line in lines {
        if read_slots { if !out.add(line) { return out; } }
        else if line.contains(cid) { read_slots = true; }
    }
    out
}
fn original_m015_area(x: i8, z: i8) -> bool { (z < 7 && x < 7) || (x >= 7 && x < 16  && z < 14) }
pub fn shuffle_deployment() {
    if DisposData::get_list_mut().is_none() || !DVCFlags::RandomDeploySpot.get_value()  { return; }
    if GameVariableManager::get_number("ShuffleDeployRNG") == 0 {
        let seed = Random::get_system().value();
        GameVariableManager::make_entry_norewind("ShuffleDeployRNG", seed);
    }
    let seed = GameVariableManager::get_number("ShuffleDeployRNG");
    let rng = Random::new( seed as u32);
    let tile_restrictions = get_tile_restrictions();
    if let Some((terrain, dispos_data)) = MapTerrain::get_instance().zip(DisposData::get_list_mut()) {
        let start_x = terrain.x as i8 + 1;
        let end_x = terrain.width as i8 - 1;
        let start_z = terrain.z as i8 + 1;
        let end_z = terrain.height as i8 - 1;
        let is_m015 = GameUserData::get_chapter().cid.str_contains("CID_M015");
        let mut player_deployment = vec![];
        let mut occupied = HashSet::new();
            dispos_data.iter().flat_map(|d| d.iter())
                .filter(|x| x.get_person().is_some() && (x.flag.value & 896 != 0 || x.force == 0 || x.force == 2 || x.get_person().is_some_and(|x| x.get_bmap_size() > 1)))
                .for_each(|data| {
                    let pos_x = data.dispos_x as i32;
                    let pos_z = data.dispos_y as i32;
                    let size = if (1 << data.force) & 5 != 0 { 4 } else { data.get_person().unwrap().bmap_size + 1 } as i32;
                    if data.force == 0 { player_deployment.push((pos_x, pos_z)); }
                    occupied.insert((pos_x, pos_z));
                    if size > 1 {
                        for dx in -size..size+1 { 
                            if (pos_x + dx) <= 0 { continue; }
                            for dz in -size..size+1 { 
                                if (pos_z + dz) <= 0 { continue; }
                                occupied.insert((pos_x+dx, pos_z+dz)); 
                            } 
                        }
                    }
                });
        let mut positions: HashMap<(i8, i8), u8> = HashMap::new();
        for z in start_z..end_z {
            for x in start_x..end_x {
                if !tile_restrictions.exclude(x, z) && !occupied.contains(&(x as i32, z as i32)){
                    let index: usize = x as usize + 32 * z as usize;
                    let tid = terrain.terrains[index];
                    let flag = if is_m015 && original_m015_area(x, z) { 4 } else { 0 };
                    if utils::tid_can_fly(tid) { positions.insert((x, z), 2); }
                    else if utils::is_tile_good(tid){ positions.insert((x, z), flag); }
                }
            }
        }
        let mut boss_positions = vec![];
        let mut boss_distance = 30;
        while boss_distance > 15 {
            let distance_2 = boss_distance * boss_distance;
            positions.iter()
                .filter(|pos|
                    *pos.1 & 6 == 0 &&
                    player_deployment.iter().all(|x|{
                        let dx = x.0 - pos.0.0 as i32;
                        let dz = x.1 - pos.0.1 as i32;
                        let total = dx*dx + dz*dz;
                        total > distance_2
                    })
                )
                .for_each(|p|{ boss_positions.push(( p.0.0, p.0.1, *p.1)); });
            boss_distance -= 2;
        }
        let difficulty_flag = 1 << GameUserData::get_difficulty(false);
        dispos_data.iter_mut()
            .flat_map(|d| d.iter_mut())
            .filter(|x| x.force == 1 && x.get_person().is_some_and(|x| x.get_bmap_size() == 1) && x.flag.value & 16 != 0 && x.flag.value & difficulty_flag != 0)
            .for_each(|data| {
                if let Some(pos) = boss_positions.get_remove(rng){
                    data.dispos_x = pos.0;
                    data.dispos_y = pos.1;
                    data.direction = rng.get_value(8);
                    if let Some(name) = data.get_person().map(|p| p.get_name()) { println!("Boss Unit: {} moved to {}, {}", name, pos.0, pos.1); }
                }
                positions.remove(&(data.dispos_x, data.dispos_y));
                for x in -2..2 {
                    for y in -2..2 {
                        if rng.get_value(3) == 0 { positions.remove(&(data.dispos_x+x, data.dispos_y+y)); }
                    }
                }
            });
        dispos_data.iter_mut()
            .flat_map(|d| d.iter_mut())
            .filter(|x| x.force == 1 && x.get_person().is_some_and(|x| x.get_bmap_size() == 1) && x.flag.value & 16 == 0 && x.flag.value & difficulty_flag != 0)
            .for_each(|data| {
                if let Some(pos) =
                    if is_m015 && original_m015_area(data.dispos_x, data.dispos_y) {
                        let count = positions.iter().filter(|p| *p.1 & 4 != 0).count();
                        positions.iter().filter(|p| *p.1 & 4 != 0).nth(rng.get_value(count as i32) as usize)
                    }
                    else {
                        if data.get_person().is_some_and(|p| p.get_job().is_some_and(|j| j.move_type == 3)) {
                            let count = positions.iter().filter(|p| *p.1 & 4 == 0).count();
                            positions.iter().filter(|p| *p.1 & 4 == 0).nth(rng.get_value(count as i32) as usize)
                        }
                        else{
                            let count = positions.iter().filter(|p| *p.1 & 6 == 0).count();
                            positions.iter().filter(|p| *p.1 & 6 == 0).nth(rng.get_value(count as i32) as usize)
                        }
                    }
                {
                    data.dispos_x = pos.0.0;
                    data.dispos_y = pos.0.1;
                    data.direction = rng.get_value(8);
                }
                positions.remove(&(data.dispos_x, data.dispos_y));
                for x in -2..2 {
                    for y in -2..2 {
                        if rng.get_value(3) == 0 { positions.remove(&(data.dispos_x+x, data.dispos_y+y)); }
                    }
                }
            });
    }
}

