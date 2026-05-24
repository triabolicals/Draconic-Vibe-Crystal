use super::*;
use engage::{
    force::ForceType,
    gamedata::{dispos::DisposData, terrain::TerrainData},
    map::{overlap::MapOverlap, terrain::MapTerrain},
    map::situation::MapSituation,
    force::Force
};
use crate::utils::for_each_unit;

pub mod fow;

const TERRAIN_PERSIST: [&str; 10] = [
        "TID_霧_永続", "TID_炎上_永続", "TID_アロマ_永続", "TID_ブロック_永続", "TID_水溜まり_永続",
        "TID_土柱_永続", "TID_氷床_永続", "TID_流砂", "TID_ツタ_永続", "TID_煙_G004",
    ];
pub fn adjust_miasma_tiles() {
    let count = DVCVariables::chapter_number_complete(false);
    ["TID_瘴気_永続", "TID_瘴気の領域", "TID_瘴気"]
        .iter()
        .flat_map(|m|  TerrainData::get_mut(m))
        .for_each(|miasma|{
            if DVCVariables::is_random_map() {
                if count  < 10 {
                    miasma.player_defense = -5;
                    miasma.enemy_defense = 5;
                }
                else if count < 15 {
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
    let terrain_mode = DVCVariables::TerrainEffect.get_value();
    if terrain_mode == 0 || GameUserData::is_evil_map() { return; }
    if let Some(terrain) = MapTerrain::get_instance() {
        let start_x = terrain.x;
        let end_x = terrain.width;
        let start_z = terrain.z;
        let end_z = terrain.height;
        let mut pos_list: Vec<(i32, i32)> = Vec::new();
        let energy = TerrainData::get("TID_紋章氣").unwrap();
        let rng = Random::get_system();
        let deployment: Vec<_> =
            DisposData::get_list().unwrap().iter()
                .flat_map(|v| v.iter()).filter(|v| v.get_terrain().is_none() )
                .map(|d| (d.dispos_x as i32, d.dispos_y as i32)).collect();

        for z in start_z..end_z {
            for x in start_x..end_x {
                if !deployment.contains(&(x, z)) && !utils::is_tile_good(terrain.terrains[ ( x + 32 * z ) as usize ]) { continue; }
                if MapOverlap::can_create(None, x, z, energy) { pos_list.push( (x, z)); }
            }
        }
        if terrain_mode & 1 != 0 {
            let mut n_add = 0;
            if let Some(terrain) = DisposData::try_get_mut("Terrain") {
                terrain.iter_mut().filter(|data| data.pid.str_contains("紋章氣"))
                    .for_each(|data|{
                        if rng.get_value(2) < 1 { n_add += 1; }
                        if let Some(tile)  = pos_list.get_remove(rng){
                            if MapOverlap::set_by_terrain(tile.0, tile.1, energy, -1, ForceType::Empty) {
                                data.flag.value = 0;
                                data.set_pid(None);
                            }
                        }
                    });
            }
            else { n_add = 5; }
            while n_add > 0 {
                if let Some(tile) = pos_list.get_remove(rng){
                    if MapOverlap::set_by_terrain(tile.0, tile.1, energy, -1, ForceType::Empty)  {  n_add -= 1; }
                }
            }
        }
        if terrain_mode & 2 != 0 {
            let unit_pos =
                Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Enemy).unwrap().iter())
                    .map(|f| (f.x as i32, f.z as i32)).collect::<Vec<_>>();
            let total = (pos_list.len() >> 3) as i32;
            let mut count = 0;
            while count < total && !pos_list.is_empty() {
                let idx = rng.get_value(15) as usize;
                if idx > 9 { count += 1; }
                else if let Some(terrain) = TerrainData::get(TERRAIN_PERSIST[idx]) {
                    let pos =
                        if terrain.prohibition != 0 { pos_list.get_remove_filter(rng, |(x, z)| !unit_pos.contains(&(*x, *z))) }
                        else { pos_list.get_remove(rng) };
                    if let Some(tile) = pos {
                        if MapOverlap::set_by_terrain(tile.0, tile.1, terrain, -1, ForceType::Empty) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
}

pub fn terrain_spots() {
    let value = DVCVariables::TerrainEffect.get_value();
    if value & 1 == 0 { return; }
    let rng = Random::get_system();
    let rate = MapSituation::get_instance().turn * 2;
    for_each_unit(7, |unit|{
        if rng.get_value(100) < rate {
            MapOverlap::set(unit.x as i32, unit.z as i32, "TID_紋章氣".into(), -1, ForceType::Empty);
        }
    });
}