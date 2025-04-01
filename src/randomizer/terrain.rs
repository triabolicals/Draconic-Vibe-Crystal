use super::*;
use engage::{
    gamedata::{dispos::DisposData, terrain::TerrainData},
    map::{overlap::MapOverlap, terrain::MapTerrain},
};
use utils::get_random_and_remove;
pub mod fow;
pub mod menu;

const TERRAIN: [&str; 9] = ["TID_瘴気の領域", "TID_瘴気", "TID_氷の領域", "TID_霧", "TID_流砂", "TID_煙", "TID_アロマ", "TID_水溜まり", "TID_土柱", ];


pub fn adjust_miasma_tiles() {
    ["TID_瘴気_永続", "TID_瘴気の領域", "TID_瘴気"]
        .iter()
        .flat_map(|m|  TerrainData::get_mut(m))
        .for_each(|miasma|{
            if DVCVariables::is_random_map() {
                if crate::continuous::get_story_chapters_completed() < 10 {
                    miasma.player_defense = -5;
                    miasma.enemy_defense = 5;
                }
                else if crate::continuous::get_story_chapters_completed() < 15 {
                    miasma.player_defense = -10;
                    miasma.enemy_defense = 10;
                }
                else {
                    miasma.player_defense = -20;
                    miasma.enemy_defense = 20;
                }
            }
            else {
                miasma.player_defense = -20;
                miasma.enemy_defense = 20;
            }
        }
    );
}

pub fn randomized_emblem_power_spots() {
    if !GameVariableManager::get_bool(DVCVariables::TERRAIN) || GameUserData::is_evil_map() { return; }
    println!("Randomizing Location for Emblem Energy");
    if let Some(terrain) = MapTerrain::get_instance() {
        let start_x = terrain.x;
        let end_x = terrain.width;
        let start_z = terrain.z;
        let end_z = terrain.height;
        let mut pos_list: Vec<(i32, i32)> = Vec::new();
        let energy = TerrainData::get("TID_紋章氣").unwrap();
        let rng = Random::get_system();
        for z in start_z..end_z {
            for x in start_x..end_x {
                if !crate::utils::is_tile_good(terrain.terrains[ ( x + 32 * z ) as usize ]) { continue; }
                if MapOverlap::can_create(None, x, z, energy) {
                    pos_list.push( (x, z));
                }
            }
        }
        let mut n_add = 0;
        if let Some(terrain) = DisposData::try_get_mut("Terrain") {
            terrain.iter_mut().filter(|data| data.pid.to_string().contains("紋章氣"))
                .for_each(|data|{
                    if rng.get_value(2) < 1 { n_add += 1; }
                    if let Some(tile)  = get_random_and_remove(&mut pos_list, rng) {
                        if MapOverlap::set(tile.0, tile.1, energy.tid, -1, 7) {
                            data.get_flag().value = 0;
                            data.set_pid("".into());
                        }
                    }
                }
            );
        }
        else { n_add = 5; }
        while n_add > 0 {
            if let Some(tile) = get_random_and_remove(&mut pos_list, rng) {
                if MapOverlap::set(tile.0, tile.1, energy.tid, -1, 7)  {  n_add -= 1; }
            }
        }
    }
}

pub fn terrain_spots() {
    let value = GameVariableManager::get_number(DVCVariables::TERRAIN);
    if value == 0 { return; }
    let rng = Random::get_system();
    let v = rng.get_value(100);
    let pool = &Il2CppClass::from_name("App", "UnitPool").unwrap().get_static_fields_mut::<crate::randomizer::job::UnitPoolStaticFieldsMut>().s_unit;
    let mut pos_list: Vec<(i32, i32)> = pool.iter().filter(|unit| unit.force.is_some_and(|force| force.force_type < 3)).map(|unit| ( unit.get_x(), unit.get_z())).collect();
    let count = pos_list.len();
    if count < 5 { return; }
    let selection_count_max = rng.get_value(( count / 3 ) as i32 ) ;
    if v < 15 && value > 0 {
        if let Some(tile) = get_random_and_remove(&mut pos_list, rng) { MapOverlap::set(tile.0, tile.1, "TID_紋章氣".into(), -1, 7); }
    }
    else if v < 50 && value == 2 {
        if let Some(tile) = get_random_and_remove(&mut pos_list, rng){
            MapOverlap::set(tile.0, tile.1, TERRAIN[rng.get_value(9) as usize].into(), -1, 7);
        }
    }
    else if v < 80 {
        let m_terrain = MapTerrain::get_instance();
        if m_terrain.is_none() { return; }
        let map_terrain = m_terrain.unwrap();
        let start_x = map_terrain.x;
        let end_x = map_terrain.width;
        let start_z = map_terrain.z;
        let end_z = map_terrain.height;
        let mut chance = 100;

        if let Some(tile) = get_random_and_remove(&mut pos_list, rng){ 
            let terrain = match rng.get_value(5) {
                0 => { TerrainData::get("TID_炎上") }
                1 => { TerrainData::get("TID_アロマ") }
                2 => { TerrainData::get("TID_霧") }
                _ => { TerrainData::get("TID_ブロック") }
            }.unwrap();

            for dx in -2..3 {
                let total_x = tile.0 + dx;
                if total_x < start_x || total_x >= end_x { continue; }
                for dz in -2..3 {
                    let total_z = tile.1 + dz;
                    if total_z < start_z || total_z >= end_z { continue; }
                    if rng.get_value(100) < chance {
                        if MapOverlap::can_create(None, total_x, total_z, terrain) {
                            if MapOverlap::set(total_x, total_z, terrain.tid, -1, 7) { chance *= 2 / 3; }
                        }
                    }
                }
            }
        }
    }
}

#[skyline::from_offset(0x01cfa220)]
fn dispos_data_ctor(this: &DisposData, method_info: OptionalMethod);