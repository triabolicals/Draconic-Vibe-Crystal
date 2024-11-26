use super::*;
#[unity::class("App", "MapTerrain")]
 pub struct MapTerrain {
     _super: u64,
     pub x: i32,
     pub z: i32,
     pub width: i32,
     pub height: i32,
     layers: u64,
     overlaps: u64,
     pub terrains: &'static Array<&'static Il2CppString>, 
 }
 #[unity::from_offset("App", "MapSetting", "get_MapTerrain")]
 pub fn get_map_terrain(method_info: OptionalMethod) -> Option<&'static MapTerrain>;

 #[unity::class("App", "TerrainData")]
 pub struct TerrainData {
    pub parent: StructBaseFields,
    pub tid: &'static Il2CppString,
    pub name: &'static Il2CppString,
    pub cost_name: &'static Il2CppString,
    pub cost_type: i32,
    pub layer: i32,
    pub prohibition: i32,
 }
 impl Gamedata for TerrainData  {}

 pub fn is_tile_good(tid: &Il2CppString) -> bool{
    let terrain = TerrainData::get(&tid.to_string());
    if terrain.is_none() { false   }
    else { 
        let t = terrain.unwrap();
        t.prohibition == 0 
    }
 }

 use engage::gamedata::dispos::DisposData;
 use engage::gamedata::dispos::DisposDataFlag;

 pub fn encounter_map_dispos() {
    if Force::get(ForceType::Absent).is_none() || DisposData::get_list_mut().is_none() { return; }
    let absent_count = Force::get(ForceType::Absent).unwrap().get_count(); 
    let dispos = DisposData::get_list_mut().unwrap();
    let is_encounter =  GameUserData::is_encount_map();
    let mut unit_pos: Vec<(i32, i32, i32)> = Vec::new();
    let mut player_count = 0;
    let mut player_array = if is_encounter { 0 } else { -1 };

    for x in 0..dispos.len() {
        if dispos[x].array_name.to_string() == "Terrain" { continue; }
        if dispos[x].array_name.contains("Player") { player_array = x as i32; }
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
    let terrain =    unsafe { get_map_terrain(None) };
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
            if !is_tile_good(tid) { continue; }
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
                    println!("Add Point {} {}", x, z);
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
    // Add tiles in which player/ally position are within 7/8 tiles 
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
    let dispos_flag = dispos[0][0].get_flag().get_class();
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
        let dispos_flags = Il2CppObject::<DisposDataFlag>::from_class( dispos_flag ).unwrap();
        dispos_flags.value = 135;
        unsafe { dispos_data_ctor(new_dispos, None); }
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
    if !GameVariableManager::get_bool("G_Cleared_M005") { return; }
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
    let dispos_flag = dispos[0][0].get_flag().get_class();
    if player_array == -1 { return; }
    for x in slots {
        if unit_pos.iter().find(|&dispos| dispos.0 == x.0 && dispos.1 == x.1 ).is_some() { continue; }
        let new_dispos = DisposData::instantiate().unwrap();
        let dispos_flags = Il2CppObject::<DisposDataFlag>::from_class( dispos_flag ).unwrap();
        dispos_flags.value = 135;
        unsafe { dispos_data_ctor(new_dispos, None); }
        new_dispos.set_flag(dispos_flags);
        new_dispos.dispos_x = x.0 as i8;
        new_dispos.dispos_y = x.1 as i8;
        new_dispos.direction = x.2 as i32;
        dispos[ player_array as usize ].add(new_dispos);
    }
}
pub fn load_extras() -> Option<Vec<(i32, i32, i32)>> {
    let my_str = include_str!("DVC.fdp");
    let cid = GameUserData::get_chapter().cid.to_string();
    println!("Current Chapter: {}", cid);
    let mut read_slots = false;
    let mut out: Vec<(i32, i32, i32)> = Vec::new();
    let lines = my_str.lines();
    for line in lines {
        let spilt: Vec<_> = line.split_whitespace().collect();
        if spilt.len() == 1 {
            if spilt[0] == cid { read_slots = true;}
            if read_slots && spilt[0] != cid {
                println!("Found {} slots", out.len());
                return Some(out);
            }
        }
        if spilt.len() != 3 || !read_slots { continue; }
        let x = spilt[0].parse::<i32>();
        let y = spilt[1].parse::<i32>();
        let d = spilt[2].parse::<i32>();
        if x.is_ok() && y.is_ok() && d.is_ok() {
            out.push((x.unwrap(), y.unwrap(), d.unwrap() ) );
        }
    }
    None
}
#[skyline::from_offset(0x01cfa220)]
fn dispos_data_ctor(this: &DisposData, method_info: OptionalMethod);