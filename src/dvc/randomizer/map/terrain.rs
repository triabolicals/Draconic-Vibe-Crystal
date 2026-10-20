use engage::{
    app::{
        mapimage::*, disposdata::*, terraindata_2::*,
        IStructData_1Methods,
        IMapTerrain,
        MapOverlap,
        MapSetting,
        Random_2,
        Unit,
        IStructDataArray_1Methods,
        IRandom_2Methods,
        Force_Type,
        IMapImageCore_1Methods,
        ISingletonClass_1Methods,
    },
    List_1Ext,
};
use engage::app::IMapTerrainMethods;
use outfit_core::Randomizer;
use unity::{Cast, IlNull};
use crate::{DVCVariable, DVCVariables};
use crate::utils::to_opt;

const TERRAIN_PERSIST: [&str; 10] = [
    "TID_霧_永続", "TID_炎上_永続", "TID_アロマ_永続", "TID_ブロック_永続", "TID_水溜まり_永続",
    "TID_土柱_永続", "TID_氷床_永続", "TID_流砂", "TID_ツタ_永続", "TID_煙_G004",
];
pub fn adjust_miasma_tiles() {
    let count = DVCVariables::chapter_number_complete(false);
    ["TID_瘴気_永続", "TID_瘴気の領域", "TID_瘴気"]
        .iter()
        .map(|m|  TerrainData_2::get((*m).into()))
        .for_each(|miasma| {
            if DVCVariables::is_random_map() {
                let def = if count < 10 { 5 } else if count < 15 { 10 } else { 20 } as i8;
                miasma.set_player_defense(-def);
                miasma.set_enemy_defense(def);
            }
        })
}

pub fn randomized_emblem_power_spots() {
    let terrain_mode = DVCVariable::TerrainEffect.get_value();
    if terrain_mode == 0 { return; }
    let terrain = MapSetting::get_map_terrain();
    if terrain.is_null() { return; }
    let start_x = terrain.m_x();
    let end_x = terrain.m_width();
    let start_z = terrain.m_z();
    let end_z = terrain.m_height();
    let mut pos_list: Vec<(i32, i32)> = Vec::new();
    let energy = TerrainData_2::get("TID_紋章氣".into());
    let rng = Random_2::get_system();
    let map_image = MapImage::get_instance();
    let unit = map_image.m_unit().base();
    let mut deployment = vec![];
    for z in 0..32 {
        for x in 0..32 {
            let index = x | z << 5;
            if unit.get(index) != 0 || to_opt(terrain.get_tid(x, z))
                .and_then(|tid| to_opt(TerrainData_2::get(tid))).is_some_and(|t| t.is_treasure() || t.is_torch() || t.is_cannon() || t.is_door())
            {
                deployment.push((x, z));
            }
        }
    }
    if let Some(player) = to_opt(DisposData::get("Player1".into())).or_else(|| to_opt(DisposData::get("Player".into()))) {
        deployment.extend(player.base().base().iter().map(|x| (x.get_dispos_x() as i32, x.get_dispos_y() as i32)));
    }
    for z in start_z..end_z {
        for x in start_x..end_x {
            if !deployment.contains(&(x, z)) && MapOverlap::can_creation_3(Unit::null(), x, z, energy) { pos_list.push( (x, z)); }
        }
    }
    if terrain_mode & 1 != 0 {
        let mut n_add = 0;
        let terrain = DisposData::try_get("Terrain".into());
        if !terrain.is_null() {
            n_add = terrain.base().base().iter().filter(|v| to_opt(v.get_terrain()).is_some_and(|v| !v.is_not_engage_add())).count() as i32 / 2;
        }
        if n_add == 0 { n_add = 3; }
        for _ in 0..n_add {
            if let Some(tile) = pos_list.get_remove(rng) { MapOverlap::set_3(tile.0, tile.1, energy, -1, Force_Type::empty()); }
        }
    }
    if terrain_mode & 2 != 0 {
        let total = (pos_list.len() >> 2) as i32;
        let mut count = 0;
        while count < total && !pos_list.is_empty() {
            let idx = rng.get_value_2(13) as usize;
            if idx < 9 {
                if let Some(tile) = pos_list.get_remove(rng) {
                    MapOverlap::set_3(tile.0, tile.1, TerrainData_2::get(TERRAIN_PERSIST[idx].into()), -1, Force_Type::empty());
                }
            }
            count += 1;
        }
    }
}