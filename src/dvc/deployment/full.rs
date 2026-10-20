use std::collections::HashSet;
use super::*;
use engage::{
    app::{disposdata::*, terraindata_2::*, IBitField32, IMapTerrain, IStructDataArray_1Methods, MapSetting},
    system::collections::generic::IList_1Methods,
    List_1Ext,
};
use crate::utils::to_opt;

fn restrict_positions_map(unit_pos: &Vec<(i32, i32, i32)>, player: bool) -> HashSet<i32> {
    let mut hash_set = HashSet::new();
    unit_pos.iter().for_each(|(x, _y, f)| {
        let (rl, rh) = if player == (*f == 1) { (-4,5) } else { (0, 1) };
        for dx in rl..rh {
            let x1 = x + dx;
            if x1 <= 0 || x1 >= 32 { continue; }
            for dz in rl..rh {
                let z1 = dz + dz;
                if z1 <= 0 || z1 >= 32 { continue; }
                hash_set.insert(x1 + 32 * z1);
            }
        }
    });
    hash_set
}

pub fn encounter_map_dispos() {
    let absent_force = Force::get(Force_Type::absent());
    let dispos =  DisposData::get_list();
    let terrain = MapSetting::get_map_terrain();
    if absent_force.is_null() || dispos.is_null() || terrain.is_null() { return; }
    let absent_count = absent_force.get_count();
    if let Some(player) = to_opt(DisposData::get("Player1".into())).or_else(|| to_opt(DisposData::get("Player".into()))) {
        let mut player_count = player.get_count();
        if absent_count <= player_count { return; }

        let unit_pos: Vec<(i32, i32, i32)> =
            dispos.base().iter().flat_map(|group| group.base().base().iter())
                .filter(|d| !d.get_person().is_null() || d.get_flag().m_value() & 128 != 0)
                .map(|d| (d.get_dispos_x() as i32, d.get_dispos_y() as i32, d.get_force() as i32))
                .collect();
        
        let occ = restrict_positions_map(&unit_pos, true);
        let (end_x, end_z) = (terrain.m_width(),terrain.m_height());
        let title = terrain.m_terrains();
        let mut available_tiles: Vec<(i32, i32, i32)> =
            (1..end_x).into_iter()
                .flat_map(|x| (1..end_z).into_iter().map(move |y| (x, y)))
                .filter(|&(x, z)| x < 32 && z < 32 && !occ.contains(&(x + 32 * z)))
                .filter(|&(x, z)| to_opt(TerrainData_2::get(title.get((x + 32 * z) as usize ))).is_some_and(|t| t.get_prohibition().value == 0))
                .map(|(x, z)| {
                    let ally = unit_pos.iter().filter(|&(x1, z1, f)| {
                        let (dx, dz) = (x1 - x, z1 - x);
                        (dx * dx + dz * dz) < 9 && *f != 1
                    }).count() as i32;
                    let enemy = unit_pos.iter().filter(|&(x1, z1, f)| {
                        let (dx, dz) = (x1 - x, z1 - x);
                        (dx * dx + dz * dz) < 36 && *f == 1
                    }).count() as i32;
                    (x, z, ally - enemy)
                })
                .collect();
        available_tiles.sort_by(|(_, _, s1), (_, _, s2)| s2.cmp(s1));

        let rng = Random_2::get_system();
        available_tiles.iter().for_each(|&(x, z, _)| {
            if player_count < absent_count {
                let new_dispos = DisposData::new();
                new_dispos.set_flag(DisposData_FlagField::new(135));
                new_dispos.set_dispos_x(x as i8);
                new_dispos.set_dispos_y(z as i8);
                new_dispos.set_direction(DisposData_Directions{value: rng.get_value_2(8)});
                player.add(new_dispos);
                player_count += 1;
            }
        });
    }
}
// Full Deployment
pub fn load_extra_deployment_slots() {
    if !DVCVariables::is_main_chapter_complete(5) { return; }
    if DVCVariables::user_data().is_encount_map() {
        encounter_map_dispos();
        return;
    }
    let extra_slot = load_extras();
    if extra_slot.is_none() {
        encounter_map_dispos();
        return;
    }
    let slots = extra_slot.unwrap();
    let dispos = DisposData::get_list();
    let absent = Force::get(Force_Type::absent());
    let player = DisposData::get("Player".into());
    if absent.is_null() || dispos.is_null() || player.is_null() { return; }
    let unit_pos: Vec<(i32, i32, i32)> =
        dispos.base().iter().flat_map(|group| group.base().base().iter())
        .filter(|d| !d.get_person().is_null() || d.get_flag().m_value() & 128 != 0)
        .map(|d| (d.get_dispos_x() as i32, d.get_dispos_y() as i32, 0))
        .collect();

    for x in slots {
        if unit_pos.iter().find(|&dispos| dispos.0 == x.0 && dispos.1 == x.1 ).is_some() { continue; }
        let new_dispos = DisposData::new();
        new_dispos.set_flag(DisposData_FlagField::new(135));
        new_dispos.set_dispos_x(x.0 as i8);
        new_dispos.set_dispos_y(x.1 as i8);
        new_dispos.set_direction(DisposData_Directions{value: x.2});
        player.add(new_dispos);
    }
}
pub fn load_extras() -> Option<Vec<(i32, i32, i32)>> {
    let my_str = include_bytes!("../../data/deploy.dat");
    let chapter_index = DVCVariables::get_chapter_index();
    let idx = chapter_index + 32;
    if let Some(mut pos) = my_str.iter().position(|x| *x == idx as u8) {
        let count = my_str[pos+1];
        let mut slots = vec![];
        pos += 2;
        for _ in 0..count {
            let x = my_str[pos] as i32;
            let y = my_str[pos + 1] as i32;
            let r = my_str[pos + 2] as i32;
            slots.push((x, y, r));
            pos += 3;
        }
        return Some(slots);
    }
    None
}