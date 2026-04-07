use super::*;
use engage::{
    gamedata::dispos::*,
    map::terrain::MapTerrain,
};

pub fn encounter_map_dispos() {
    if Force::get(ForceType::Absent).is_none() || DisposData::get_list_mut().is_none() { return; }
    let absent_count = Force::get(ForceType::Absent).unwrap().get_count(); 
    let dispos = DisposData::get_list_mut().unwrap();
    let is_encounter =  GameUserData::is_encount_map();
    let mut unit_pos: Vec<(i32, i32, i32)> = Vec::new();
    let mut player_count = 0;
    let mut player_array = if is_encounter { 0 } else { -1 };

    for x in 0..dispos.len() {
        let array_name = dispos[x].array_name.to_string();
        if array_name == "Terrain" { continue; }
        if array_name.contains("Player") { player_array = x as i32; }
        let is_enemy = dispos[x].array_name.to_string() == "Enemy" || is_encounter;
        for y in 0..dispos[x].len() {
            if dispos[x][y].get_force() == 0 {
                unit_pos.push( (dispos[x][y].dispos_x as i32, dispos[x][y].dispos_y as i32, 0) );
                player_count += 1;
            }
            if dispos[x][y].get_force() == 2 {
                unit_pos.push( (dispos[x][y].dispos_x as i32, dispos[x][y].dispos_y as i32, 2) );
            }
            if dispos[x][y].get_force() == 1 && is_enemy {
                unit_pos.push( (dispos[x][y].dispos_x as i32, dispos[x][y].dispos_y as i32, 1) );
            }
        }
    }
    if absent_count <= player_count || player_array == -1 { 
        println!("Can't find player array or player count is less than absent count: {} < {}", absent_count, player_count);
        return;
     }
    let terrain = MapTerrain::get_instance();
    if terrain.is_none() { return; }
    let map_terrain = terrain.unwrap();
    let start_x = map_terrain.x;
    let end_x = map_terrain.width;
    let start_z = map_terrain.z;
    let end_z = map_terrain.height;
    let mut available_tiles: Vec<(i32, i32)> = Vec::new();
    // gathering all valid tiles to place units
    for z in start_z+1..end_z-1 {
        for x in start_x+1..end_x-1 {
            let index: usize = ( x + 32 * z ) as usize;
            let tid = map_terrain.terrains[ index ];
            if !crate::utils::is_tile_good(tid) { continue; }
            if unit_pos.iter().find(|&unit_pos| unit_pos.0 == x && unit_pos.1 == z).is_none() {
                available_tiles.push( (x, z) );
                let mut overlap: bool = false;
                for dx in -5..6 {
                    if overlap { break; }
                    for dz in -5..6 {
                        if dx*dx + dz*dz >= 36 { continue; }
                        if let Some(_index) = unit_pos.iter().position(|&value| 
                            ( value.0 == x + dx && value.1 == z + dz ) && value.2 == 1 )
                        { 
                            overlap = true;
                            break;
                        }
                    }
                }
                if !overlap {
                    unit_pos.push( (x, z, 3) );
                }
            }
        }
    }
    // Remove out all deployment slots within 3/4 tiles of an enemy
    let threshold = if is_encounter { 9 } else { 16 };
    for x in &unit_pos {
        if x.2 == 1 {
            for dx in 0..5 {
                for dz in 0..5 {
                    if dx*dx + dz*dz >= threshold { continue; }
                    if let Some(index) = available_tiles.iter().position(|&value| 
                        ( value.0 == x.0 + dx && value.1 == x.1 + dz) || 
                        ( value.0 == x.0 - dx && value.1 == x.1 - dz) || 
                        ( value.0 == x.0 - dx && value.1 == x.1 + dz) || 
                        ( value.0 == x.0 + dx && value.1 == x.1 - dz) )
                    { 
                        available_tiles.swap_remove(index);
                    }
                }
            }
        }
    }
    let mut valid_tiles = Vec::new();
    let threshold2 = if is_encounter { 36 } else { 16 };
    // Add tiles in which player / ally position are within 7/8 tiles
    for x in &unit_pos {
        if x.2 != 1 {
            for dx in 0..4 {
                for dz in 0..4 {
                    if dx*dx + dz*dz >= threshold2 { continue; }
                    if let Some(index) = available_tiles.iter().find(|&value| 
                        ( value.0 == x.0 + dx && value.1 == x.1 + dz) || 
                        ( value.0 == x.0 - dx && value.1 == x.1 - dz) || 
                        ( value.0 == x.0 - dx && value.1 == x.1 + dz) || 
                        ( value.0 == x.0 + dx && value.1 == x.1 - dz) )
                    { 
                        valid_tiles.push( index.clone() );
                    }
                }
            }
        }
    }
    let size = valid_tiles.len() as i32;
    let rng = Random::get_system();
    if size == 0 { return; }
    for _x in 0..absent_count-player_count {
        let mut index;
        let mut count = 0;
        loop {
            index = rng.get_value( size );
            let t = valid_tiles[ index as usize];
            if unit_pos.iter().find(|&x| x.0 == t.0 && x.1 == t.1 && x.2 != 3).is_none() {
                break;
            }
            count += 1;
            if count > 300 { return; }  // failed
        }
        let new_dispos = DisposData::instantiate().unwrap();
        new_dispos.ctor();
        let dispos_flags = DisposDataFlagField::instantiate().unwrap();
        dispos_flags.value = 135;
        new_dispos.set_flag(dispos_flags);
        let tile = valid_tiles[ index as usize];
        new_dispos.dispos_x = tile.0 as i8;
        new_dispos.dispos_y = tile.1 as i8;
        unit_pos.push( (tile.0, tile.1, 0));
        new_dispos.direction = rng.get_value(8);
        dispos[ player_array as usize ].add(new_dispos);
    }

}
// Full Deployment
pub fn load_extra_deployment_slots() {
    if !DVCVariables::is_main_chapter_complete(5) { return; }
    if GameUserData::is_encount_map() {  
        encounter_map_dispos(); 
        return;
    }
    let extra_slot = load_extras();
    if extra_slot.is_none() { 
        encounter_map_dispos(); 
        return;
    }
    let slots = extra_slot.unwrap();
    if Force::get(ForceType::Absent).is_none() || DisposData::get_list_mut().is_none() { return; }
    let dispos = DisposData::get_list_mut().unwrap();
    let mut unit_pos: Vec<(i32, i32, i32)> = Vec::new();
    let mut player_array = -1;
    for x in 0..dispos.len() {
        if dispos[x].array_name.to_string() == "Player" { player_array = x as i32; }
        if dispos[x].array_name.to_string() == "Terrain" { continue; }
        for y in 0..dispos[x].len() {
            let force = dispos[x][y].get_force();
            if  force == 0 || force == 1 {
                unit_pos.push( (dispos[x][y].dispos_x as i32, dispos[x][y].dispos_y as i32, 0) );
            }
        }
    }
    if player_array == -1 { return; }
    for x in slots {
        if unit_pos.iter().find(|&dispos| dispos.0 == x.0 && dispos.1 == x.1 ).is_some() { continue; }
        let new_dispos = DisposData::instantiate().unwrap();
        new_dispos.ctor();
        let dispos_flags = DisposDataFlagField::instantiate().unwrap();
        dispos_flags.value = 135;
        new_dispos.set_flag(dispos_flags);
        new_dispos.dispos_x = x.0 as i8;
        new_dispos.dispos_y = x.1 as i8;
        new_dispos.direction = x.2 as i32;
        dispos[ player_array as usize ].add(new_dispos);
    }
}
pub fn load_extras() -> Option<Vec<(i32, i32, i32)>> {
    let my_str = include_bytes!("../../data/deploy.dat");
    let cid = GameUserData::get_chapter().cid.to_string();
    let l = ["CID_M0", "CID_S0", "CID_G00", "CID_E00"];
    let l2 = [0, 30, 50, 60];
    if let Some(p) = l.iter().position(|x| cid.contains(*x)) {
        if let Ok(n) = cid.replace(l[p], "").parse::<i32>(){
            let idx = n + 32 + l2[p];
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
        }
    }
    None
}

/*
pub struct DeploySlots {
    pub map_index: i32,
    pub slots: Vec<(i32, i32, i32)>,
}
pub fn load_extras2() {
    let my_str = include_str!("DVC.fdp");;
    let lines = my_str.lines();
    let l = ["CID_M0", "CID_S0", "CID_G00", "CID_E00"];
    let l2 = [0, 30, 50, 60];
    let mut slots = Vec::new();
    for line in lines {
        let spilt: Vec<_> = line.split_whitespace().collect();
        if spilt.len() == 1 {
            if let Some(p) = l.iter().position(|x| spilt[0].contains(*x)) {
                let s = spilt[0].replace(l[p], "").parse::<i32>().unwrap() + l2[p] + 32;
                slots.push(DeploySlots { map_index: s, slots: Vec::new(), });
            }
        }
        else if let Some(((x, y), z)) = spilt[0].parse::<i32>().ok()
            .zip(spilt[1].parse::<i32>().ok())
            .zip(spilt[2].parse::<i32>().ok())
        {
            if let Some(s) = slots.last_mut() {
                s.slots.push((x, y, z));
            }
        }
    }
    if let Ok(mut file) = fs::File::create("sd:/deploy.dat") {
        slots.iter().for_each(|s|{
            let buf = [s.map_index as u8, s.slots.len() as u8];
            file.write(&buf).unwrap();
            s.slots.iter().for_each(|s|{
                file.write(&[s.0 as u8, s.1 as u8, s.2 as u8]).unwrap();
            });
        });
    }
}
*/
pub fn create_new_dispos(flag: i32) -> &'static mut DisposData {
    let data = DisposData::instantiate().unwrap();
    data.ctor();
    let flag_field = DisposDataFlagField::instantiate().unwrap();
    flag_field.value |= flag;
    data.set_flag(flag_field);
    data
}
