use super::*;
use engage::{
    gamedata::{dispos::DisposData, terrain::TerrainData},
    map::{overlap::MapOverlap, terrain::MapTerrain},
};
pub mod fow;
pub mod menu;

const TERRAIN: [&str; 9] = ["TID_瘴気の領域", "TID_瘴気", "TID_氷の領域", "TID_霧", "TID_流砂", "TID_煙", "TID_アロマ", "TID_水溜まり", "TID_土柱", ];


pub fn adjust_miasma_tiles() {
    ["TID_瘴気_永続", "TID_瘴気の領域", "TID_瘴気"].iter()
        .for_each(|m|{
            if let Some(miasma) = TerrainData::get_mut(m){
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
        let mut terrain_array = -1;
        let mut pos_list: Vec<(i32, i32)> = Vec::new();
        let energy = TerrainData::get("TID_紋章氣").unwrap();
        let dispos = DisposData::get_list_mut().unwrap();
        let rng = Random::get_system();
        for z in start_z..end_z {
            for x in start_x..end_x {
                if !crate::utils::is_tile_good(terrain.terrains[ ( x + 32 * z ) as usize ]) { continue; }
                if MapOverlap::can_create(None, x, z, energy) {
                    pos_list.push( (x, z));
                }
            }
        }
        for x in 0..dispos.len() {
            if dispos[x].array_name.to_string() == "Terrain" { terrain_array = x as i32; break; }
        }
        if terrain_array == -1 {
            println!("No Terrain Array in Dispos");
            let mut count = 0;
            while count < 5 && pos_list.len() > 0 {
                let index = rng.get_value( pos_list.len() as i32 ) as usize;
                let tile = pos_list[index];
                if MapOverlap::set(tile.0, tile.1, energy.tid, -1, 7)  {
                    count += 1;
                    println!("Energy added at {}, {}", tile.0, tile.1);
                    pos_list.remove(index);
                }
            }
        }
        else {
            let size = dispos[terrain_array as usize].len();
            let mut count = size;
            let dispos_flag = dispos[0][0].get_flag().get_class();
            for x in 0..size {
                if dispos[terrain_array as usize][x].pid.to_string().contains("紋章氣") && pos_list.len() > 2 {
                    let index = rng.get_value( pos_list.len() as i32 ) as usize;
                    let tile = pos_list[index];
                    dispos[terrain_array as usize][x].dispos_x = tile.0 as i8;
                    dispos[terrain_array as usize][x].dispos_y = tile.1 as i8;
                    pos_list.remove(index as usize);
                    if rng.get_value(2) == 1 || count < 3 {
                        let new_dispos = DisposData::instantiate().unwrap();
                        let dispos_flags = Il2CppObject::<DisposDataFlag>::from_class( dispos_flag ).unwrap();
                        dispos_flags.value = 7;
                        unsafe { dispos_data_ctor(new_dispos, None); }
                        new_dispos.set_flag(dispos_flags);
                        new_dispos.pid = "PID_紋章氣".into();
                        new_dispos.tid = "TID_紋章氣".into();
                        let index2 = rng.get_value( pos_list.len() as i32 ) as usize;
                        let tile2 = pos_list[index2];
                        new_dispos.dispos_x = tile2.0 as i8;
                        new_dispos.dispos_y = tile2.1 as i8;
                        println!("Energy added at {}, {}", tile.0, tile.1);
                        pos_list.remove(index2);
                        dispos[ terrain_array as usize ].add(new_dispos);
                        count += 1;
                    }
                }
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
    let mut iter = pool.iter().filter(|unit| unit.force.filter(|f| f.force_type < 3 ).is_some());
    let count = pool.iter().filter(|unit| unit.force.filter(|f| f.force_type < 3 ).is_some()).count();
    if count < 5 { return; }
    let selection_count_max = rng.get_value(( count / 3 ) as i32 ) ;
    if v < 15 && value > 0 {
        if let Some(tile) = iter.nth(rng.get_value( count as i32 ) as usize) {
            println!("Energy Tile added at {}, {}", tile.x , tile.z );
            MapOverlap::set(tile.x as i32, tile.z as i32, "TID_紋章氣".into(), -1, 7);
        }

    }
    else if v < 50 && value == 2 {
        if let Some(tile) = iter.nth(rng.get_value( count as i32 ) as usize) {
            println!("Other Tile added at {}, {}", tile.x , tile.z );
            MapOverlap::set(tile.x as i32, tile.z as i32, TERRAIN[rng.get_value(9) as usize].into(), -1, 7);
        }
    }
    else if v < 80 {
        let m_terrain = MapTerrain::get_instance();
        if m_terrain.is_none() { return; }
        let mut selection = 0;
        iter
            .for_each(|unit|{
                if selection < selection_count_max && rng.get_value(10) < 1 {
                    let terrain = match rng.get_value(5) {
                        0 => { TerrainData::get("TID_炎上") }
                        1 => { TerrainData::get("TID_アロマ") }
                        2 => { TerrainData::get("TID_霧") }
                        _ => { TerrainData::get("TID_ブロック") }
                    }.unwrap();
                    let map_terrain = m_terrain.unwrap();
                    let start_x = map_terrain.x;
                    let end_x = map_terrain.width;
                    let start_z = map_terrain.z;
                    let end_z = map_terrain.height;
                    let mut chance = 100;
                    let mut pos: Vec<(i32, i32)> = Vec::new();
                    for dx in -2..3 {
                        let total_x = unit.x as i32 + dx;
                        if total_x < start_x || total_x >= end_x { continue; }
                        for dz in -2..3 {
                            let total_z = unit.z as i32 + dz;
                            if total_z < start_z || total_z >= end_z { continue; }
                            pos.push((total_x, total_z));
                        }
                    }
                    if pos.len() > 0 {
                        while pos.len() > 1 {
                            let index = rng.get_value(pos.len() as i32) as usize;
                            if MapOverlap::can_create(Some(unit), pos[index].0, pos[index].1, terrain) {
                                if rng.get_value(100) < chance {
                                    chance = chance / 3;
                                    MapOverlap::set(pos[index].0, pos[index].1, terrain.tid, -1, 7);
                                }
                            }
                            pos.remove(index);
                        }
                    }
                    selection += 1;
                }
           }
        );
    }
}

#[skyline::from_offset(0x01cfa220)]
fn dispos_data_ctor(this: &DisposData, method_info: OptionalMethod);