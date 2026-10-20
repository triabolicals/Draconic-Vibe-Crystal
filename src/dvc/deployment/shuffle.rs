use std::collections::{HashMap, HashSet};
use engage::{
    app::{
        disposdata::*, chapterdata::*, terraindata_2::*, 
        IStructData_1Methods, IBitField32, IGameUserDataMethods, IMapTerrainMethods, 
        IRandom_2Methods, IStructDataArray_1Methods, MapSetting, Random_2, 
        IPersonDataMethods, IMapTerrain, IJobDataMethods
    },
    GameVariableManager,
    List_1Ext
};
use outfit_core::Randomizer;
use unity::Cast;
use crate::{config::DVCFlags, DVCVariables};
use crate::utils::to_opt;
#[allow(unused)]
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
    /*
    pub fn get_area(&self, pos_x: i8, pos_z: i8) -> i8 {
        if self.areas.is_empty() { return 0; }
        if let Some(pos) = self.areas.iter().position(|x| pos_x > x.x1 && pos_x < x.x2 && pos_z > x.z1 && pos_z < x.z2) {
            pos as i8 + 1
        }
        else { 0 }
    }
     */
    pub fn exclude(&self, pos_x: i8, pos_y: i8) -> bool { self.exclude_tile.contains(&(pos_x, pos_y)) }
}

pub fn get_tile_restrictions() ->  MapTileData  {
    let my_str = include_str!("../../data/shuffle");
    let binding = DVCVariables::user_data().get_chapter().get_cid().to_rust_string();
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
    let terrain = MapSetting::get_map_terrain();
    let dispos = DisposData::get_list();
    if DVCVariables::user_data().is_encount_map() { return; }
    if !DVCFlags::RandomDeploySpot.get_value() || dispos.is_null() || terrain.is_null()  { return; }
    if GameVariableManager::get_number("ShuffleDeployRNG") == 0 {
        let seed = Random_2::get_system().get_value();
        GameVariableManager::make_entry_norewind("ShuffleDeployRNG", seed);
        GameVariableManager::set_number("ShuffleDeployRNG", seed);
    }
    let seed = GameVariableManager::get_number("ShuffleDeployRNG");
    let rng = Random_2::new_2( seed as u32);
    let tile_restrictions = get_tile_restrictions();
    let terrain = MapSetting::get_map_terrain();
    let start_x = terrain.m_x() as i8 + 1;
    let end_x = terrain.m_width() as i8 - 1;
    let start_z = terrain.m_z() as i8 + 1;
    let end_z = terrain.m_height() as i8 - 1;
    let is_m015 = DVCVariables::get_chapter_index() == 15;
    let mut player_deployment = vec![];
    let mut occupied = HashSet::new();
    // Player Position are ignored
    if let Some(player) = to_opt(DisposData::get("Player".into())).or_else(|| to_opt(DisposData::get("Player1".into()))){
        player_deployment.extend(player.base().base().iter().map(|d|(d.get_dispos_x() as i32, d.get_dispos_y() as i32)));
        player.base().base().iter().for_each(|v|{
            for dx in -3..4 {
                let x = dx + v.get_dispos_x();
                if x <= 0 || x >= 32 { continue; }
                for dz in -3..4 {
                    let z = dz + v.get_dispos_y();
                    if z <= 0 || z >= 32 { continue; }
                    occupied.insert((x as i32, z as i32));
                }
            }
        });
    }
    // Ally Position are ignored
    dispos.base().iter().flat_map(|group| group.base().base().iter())
        .filter(|x| x.get_force() == 2 && !x.get_person().is_null())
        .for_each(|v|{
            for dx in -3..3 {
                let x = dx + v.get_dispos_x();
                if x <= 0 || x >= 32 { continue; }
                for dz in -3..3 {
                    let z = dz + v.get_dispos_y();
                    if z <= 0 || z >= 32 { continue; }
                    occupied.insert((x as i32, z as i32));
                }
            }
        });
        dispos.base().iter().flat_map(|group| group.base().base().iter())
            .filter(|x| x.get_force() == 1 && !to_opt(x.get_person()).is_some_and(|x| x.get_bmap_size() == 1))
            .for_each(|data|{
                let pos_x = data.get_dispos_x() as i32;
                let pos_z = data.get_dispos_y() as i32;
                let bmap_size =  data.get_person().get_bmap_size() as i32;
                if bmap_size > 1 {
                    for dx in -2..bmap_size+1 {
                        if (pos_x + dx) <= 0 { continue; }
                        for dz in -2..bmap_size+1 {
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
                let tile = TerrainData_2::get(terrain.get_tid(x, z));
                let flag = if is_m015 && original_m015_area(x, z) { 4 } else { 0 };
                if tile.is_fly_enable() { positions.insert((x, z), 2); }
                else if tile.get_prohibition().value == 0 { positions.insert((x, z), flag); }
            }
        }
    }
    let mut boss_positions = vec![];
    let mut boss_distance = 25;
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
    let difficulty_flag = 1 << DVCVariables::user_data().get_difficulty(false).value;
    dispos.base().iter().flat_map(|group| group.base().base().iter())
        .filter(|x| x.get_force() == 1 && x.get_flag().m_value() & difficulty_flag != 0 && x.get_flag().m_value() & 16 != 0)
        .for_each(|data| {
            if let Some(pos) = boss_positions.get_remove(rng){
                data.set_dispos_x(pos.0);
                data.set_dispos_y(pos.1);
                data.set_direction(DisposData_Directions{value: rng.get_value_2(8)});
            }
            let (x, y) = (data.get_dispos_x(), data.get_dispos_y());
            positions.remove(&(x, y));
        });
    dispos.base().iter().flat_map(|group| group.base().base().iter())
        .filter(|x| x.get_force() == 1 && to_opt(x.get_person()).is_some_and(|x| x.get_bmap_size() == 1) && x.get_flag().m_value() & difficulty_flag != 0)
        .for_each(|data| {
            if let Some(pos) =
                if is_m015 && original_m015_area(data.get_dispos_x(), data.get_dispos_y()) {
                    let count = positions.iter().filter(|p| *p.1 & 4 != 0).count();
                    positions.iter().filter(|p| *p.1 & 4 != 0).nth(rng.get_value_2(count as i32) as usize)
                }
                else {
                    if to_opt(data.get_person()).is_some_and(|p| p.get_job().get_move_type().value == 3){
                        let count = positions.iter().filter(|p| *p.1 & 4 == 0).count();
                        positions.iter().filter(|p| *p.1 & 4 == 0).nth(rng.get_value_2(count as i32) as usize)
                    }
                    else{
                        let count = positions.iter().filter(|p| *p.1 & 6 == 0).count();
                        positions.iter().filter(|p| *p.1 & 6 == 0).nth(rng.get_value_2(count as i32) as usize)
                    }
                }
            {
                data.set_dispos_x(pos.0.0);
                data.set_dispos_y(pos.0.1);
                data.set_direction(DisposData_Directions{value: rng.get_value_2(8)});
            }
            let x = data.get_dispos_x();
            let y = data.get_dispos_y();
            positions.remove(&(x, y));
        });
}

