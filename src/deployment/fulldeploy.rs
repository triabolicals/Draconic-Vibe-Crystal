use super::*;
use engage::gamedata::dispos::DisposData;
use engage::gamedata::dispos::DisposDataFlag;
const TERRAIN: [&str; 5] = ["TID_氷の床", "TID_火炎砲台_対戦", "TID_煙", "TID_アロマ", "TID_水溜まり"];

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
#[skyline::from_offset(0x01dfe300)]
fn can_create(attacker: Option<&Unit>, x: i32, y: i32, terrain: &TerrainData, method_info: OptionalMethod) -> bool;
#[skyline::from_offset(0x01decd40)]
fn mapoverlap_set(x: i32, z: i32, tid: &Il2CppString, turn: i32, phase: i32,  method_info: OptionalMethod) -> bool;

 #[unity::class("App", "TerrainData")]
 pub struct TerrainData {
    pub parent: StructBaseFields,
    pub tid: &'static Il2CppString,
    pub name: &'static Il2CppString,
    pub cost_name: &'static Il2CppString,
    pub cost_type: i32,
    pub layer: i32,
    pub prohibition: i32,
    pub command: i32,
    pub sight: u8,
    pub destroyer: i32,
    pub hp: [i32; 3],
    pub defense: i8,
    pub avoid: i8,
    pub player_defense: i8,
    pub enemy_defense: i8,
 }
 impl Gamedata for TerrainData  {}

 pub fn is_tile_good(tid: &Il2CppString) -> bool{
    if let Some(terrain) = TerrainData::get(&tid.to_string()) {
        terrain.prohibition == 0 
    }
    else { false }
 }

pub fn adjust_miasma_tiles() {
    ["TID_瘴気_永続", "TID_瘴気の領域", "TID_瘴気"].iter().for_each(|m|{
        if let Some(miasma) = TerrainData::get_mut(m){
            if GameVariableManager::get_number("G_Continuous")  == 3 {
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
    });
}

 pub struct RandomEmblemEnergy;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemEnergy {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry("G_RandomEnergy", 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(GameVariableManager::get_number("G_RandomEnergy"), 0, 2, 1);
        if GameVariableManager::get_number("G_RandomEnergy") != result {
            GameVariableManager::set_number("G_RandomEnergy", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match GameVariableManager::get_number("G_RandomEnergy") {
            1 => { "Emblem energy will be randomized. Chance for spots at the start of player phase." },
            2 => { "Chance at the start of player phase for random terrain/energy tile." }
            _ => { "No changes to emblem energy / terrain on the map." }
        }.into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match GameVariableManager::get_number("G_RandomEnergy") {
            1 => { "Energy" },
            2 => { "Terrain" },
            _ => { "Default" },
        }.into();
    }
}

pub extern "C" fn vibe_energy() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<RandomEmblemEnergy>("Terrain Effects");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map as _);
    switch
}

pub struct UnitPoolStaticFields {
    pub s_unit: &'static mut Array<&'static mut Unit>,
    pub forces: &'static mut Array<&'static Force>,
}

pub fn power_spot() {
    let value = GameVariableManager::get_number("G_RandomEnergy");
    if value == 0 { return; }
    let rng = Random::get_system();
    let v = rng.get_value(100);
    let pool = &Il2CppClass::from_name("App", "UnitPool").unwrap().get_static_fields_mut::<UnitPoolStaticFields>().s_unit;
    let mut iter = pool.iter().filter(|unit| unit.force.filter(|f| f.force_type < 3 ).is_some());
    let count = pool.iter().filter(|unit| unit.force.filter(|f| f.force_type < 3 ).is_some()).count();
    if count < 5 { return; }
    if v < 10 && value > 0 {
        if let Some(tile) = iter.nth(rng.get_value( count as i32 ) as usize) {
            println!("Energy Tile added at {}, {}", tile.x , tile.z );
            unsafe { mapoverlap_set(tile.x as i32 , tile.z as i32, "TID_紋章氣".into(), -1, 7, None) };
        }

    }
    else if v < 60 && value == 2 {
        if let Some(tile) = iter.nth(rng.get_value( count as i32 ) as usize) {
            println!("Other Tile added at {}, {}", tile.x , tile.z );
            unsafe { mapoverlap_set(tile.x as i32 , tile.z as i32, TERRAIN[rng.get_value(5) as usize].into(), -1, 7, None) };
        }
    }
}

pub fn randomized_emblem_power_spots() {
    if !GameVariableManager::get_bool("G_RandomEnergy") { return; }
    println!("Randomizing Location for Emblem Energy");
    if let Some(terrain) =    unsafe { get_map_terrain(None) } {
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
                if !is_tile_good(terrain.terrains[ ( x + 32 * z ) as usize ]) { continue; }
                if unsafe { can_create(None, x, z, energy, None) } {
                    pos_list.push( (x, z));
                }
            }
        }
        println!("Number of Placement Locations: {}", pos_list.len());
        for x in 0..dispos.len() {
            if dispos[x].array_name.to_string() == "Terrain" { terrain_array = x as i32; break; }
        }
        if terrain_array == -1 {
            println!("No Terrain Array in Dispos");
            let mut count = 0;
            while count < 5 && pos_list.len() > 0 {
                let index = rng.get_value( pos_list.len() as i32 ) as usize;
                let tile = pos_list[index];
                if unsafe { mapoverlap_set(tile.0, tile.1, energy.tid, -1, 7, None) } {
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
                if dispos[terrain_array as usize][x].pid.contains("紋章氣") && pos_list.len() > 2 {
                    let index = rng.get_value( pos_list.len() as i32 ) as usize;
                    let tile = pos_list[index];
                    dispos[terrain_array as usize][x].dispos_x = tile.0 as i8;
                    dispos[terrain_array as usize][x].dispos_y = tile.1 as i8;
                    println!("Energy at {}, {}", tile.0, tile.1);
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