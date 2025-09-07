use std::collections::HashMap;
use engage::force::{Force, ForceType};
use engage::gamedata::dispos::DisposData;
use engage::gamedata::{Gamedata, GamedataArray};
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::map::terrain::MapTerrain;
use engage::random::Random;
use crate::config::DVCVariables;
use crate::utils;


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
    let my_str = include_str!("shuffle");
    let binding = GameUserData::get_chapter().cid.to_string();
    let cid = binding.trim_start_matches("CID_");
    let mut read_slots = false;
    let mut out: MapTileData = MapTileData::new();
    let lines = my_str.lines();
    for line in lines {
        if read_slots {
            if !out.add(line) { return out; }
        }
        else if line.contains(cid) { read_slots = true; }
    }
    out
}
fn original_m015_area(x: i8, z: i8) -> bool { (z < 7 && x < 7) || (x >= 7 && x < 16  && z < 14) }
pub fn shuffle_deployment() {
    if DisposData::get_list_mut().is_none() { return; }
    let tile_restrictions = get_tile_restrictions();
    if let Some(player) = DisposData::get_list_mut().unwrap().iter_mut().find(|array| array.array_name.str_contains("Player")) {
        if let Some((terrain, dispos_data)) = MapTerrain::get_instance().zip(DisposData::get_list_mut()) {
            let ally_playables = dispos_data.iter()
                .flat_map(|d| d.iter())
                .filter(|x| x.force == 2 && x.get_person().is_some_and(|s| s.get_asset_force() == 0)).count();

            let full_deploy;
            if GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 4 {
                full_deploy = true;
                let absent_count = Force::get(ForceType::Absent).map(|f| f.get_count()).unwrap_or(0);
                let player_count = Force::get(ForceType::Player).map(|f| f.get_count()).unwrap_or(0);
                let total_count = if (absent_count + player_count) > 45 { 45 } else { absent_count + player_count };
                let count = player.iter().count() as i32;
                if count < total_count {
                    for _ in 0..total_count - count { player.add(crate::deployment::fulldeploy::create_new_dispos(135)); }
                }
            } else { full_deploy = false; }
            let player_count = player.iter().count() + ally_playables;
            let rng = Random::get_system();
            let start_x = terrain.x as i8 + 1;
            let end_x = terrain.width as i8 - 1;
            let start_z = terrain.z as i8 + 1;
            let end_z = terrain.height as i8 - 1;
            let is_m015 = GameUserData::get_chapter().cid.str_contains("CID_M015");
            let is_s010 = GameUserData::get_chapter().cid.str_contains("CID_S010");
            let mut positions: HashMap<(i8, i8), u8> = HashMap::new();
            for z in start_z..end_z {
                for x in start_x..end_x {
                    if !tile_restrictions.exclude(x, z) {
                        let index: usize = x as usize + 32 * z as usize;
                        let tid = terrain.terrains[index];
                        let flag =
                        if is_s010  && x < 20 { 4 } else if is_m015 && original_m015_area(x, z) { 4 } else { 0 };
                        if utils::tid_can_fly(tid) { positions.insert((x, z), 2); }
                        else if utils::is_tile_good(tid){ positions.insert((x, z), flag); }
                    }
                }
            }
            let monster_range = if full_deploy { 2 } else { 4 };
            dispos_data.iter().flat_map(|d| d.iter())
                .filter(|x| x.get_person().is_some_and(|p| p.get_bmap_size() > 1) || (x.force == 1 && x.flag.value & 16 != 0))
                .for_each(|data| {
                    let size = data.get_person().map(|p| p.get_bmap_size()).unwrap_or(1) as i8;
                    let pos_x = data.dispos_x;
                    let pos_z = data.dispos_y;
                    for x in 0..size {
                        for z in 0..size {
                            positions.remove(&(pos_x + x, pos_z + z));
                            positions.remove(&(data.appear_x + x, data.appear_y + z));
                        } // Remove All Big Units Positions
                    }
                    let range = if data.flag.value & 16 != 0 { 10 } else { monster_range };
                    for r1 in 0..range {
                        for r2 in 0..range {
                            for s in 0..size {
                                if let Some(position) = positions.get_mut(&(pos_x - r1, pos_z + s + r2)) { *position |= 1; } // LeftUp
                                if let Some(position) = positions.get_mut(&(pos_x + s + r1, pos_z + s + r2)) { *position |= 1; } // LeftUp
                                if let Some(position) = positions.get_mut(&(pos_x - r1, pos_z - r2)) { *position |= 1; } //RightDown
                                if let Some(position) = positions.get_mut(&(pos_x + s + r1, pos_z - r2)) { *position |= 1; } //RightDown
                            }
                        }
                    }
                });
            let min_count =
                if player_count < 8 { player_count }
                else if full_deploy || player_count < 9 { 8 }
                else if player_count < 12 { player_count }
                else { player_count / 2 };
            let mut player_deploy_area: Vec<Vec<(i8, i8)>> = vec![];
            dispos_data.iter().flat_map(|d| d.iter())
                .filter(|x| x.get_person().is_some_and(|p| p.get_bmap_size() == 1) )
                .for_each(|data| {
                    let mut set = Vec::new();
                    let pos = get_player_deployment_pattern();
                    let area = tile_restrictions.get_area(data.dispos_x, data.dispos_y);
                    pos.iter().for_each(|p| {
                        let pos_xz = (data.dispos_x + p.0, data.dispos_y + p.1);
                        let area2 =  tile_restrictions.get_area(data.dispos_x + p.0, data.dispos_y + p.1);
                        if set.len() < player_count && positions.get_mut(&pos_xz).is_some_and(|x| ((*x|4) == 4) && area == area2) {
                            set.push(pos_xz);
                        }
                    });
                    if set.len() >= min_count {
                        set.iter().for_each(|x|{
                            if let Some(pos) = positions.get_mut(x) { *pos |= 1; }
                        });
                        player_deploy_area.push(set);
                    }
                });
            // Player/Ally Area
            let mut player_list = player.iter_mut().chain(
                DisposData::get_list_mut().unwrap().iter_mut().flat_map(|d| d.iter_mut().filter(|x| x.force == 2))
            ).peekable();
            let mut is_player = true;
            while player_list.peek().is_some() && player_deploy_area.len() > 0 {
                let selection = rng.get_value(player_deploy_area.len() as i32) as usize;
                let mut v = player_deploy_area[selection].clone();
                player_deploy_area.remove(selection);
                while let Some(pos) = utils::get_random_and_remove(&mut v, rng) {
                    if is_player && player_list.peek().is_some_and(|p| p.force == 2 && p.get_person().is_some_and(|pd| pd.get_asset_force() > 0)) {
                        is_player = false;
                        break;
                    }
                    if let Some(data) = player_list.next() {
                        let range = if data.force == 0 { monster_range + 1 } else { monster_range };
                        if let Some(person) = data.get_person() {
                            GameVariableManager::make_entry(format!("RD_{}", person.pid).as_str(), 100 * pos.1 as i32 + pos.0 as i32);
                        }
                        data.dispos_x = pos.0;
                        data.dispos_y = pos.1;
                        data.appear_x = pos.0;
                        data.appear_y = pos.1;
                        data.direction = rng.get_value(8);
                        let area = tile_restrictions.get_area(pos.0, pos.1);
                        for x in -range..range + 1 {
                            for z in -range..range +1  {
                                if tile_restrictions.get_area(pos.0+x, pos.1+z) == area {
                                    positions.remove(&(pos.0+x, pos.1+z));
                                }
                            }
                        }
                    }
                }
            }
            dispos_data.iter_mut()
                .flat_map(|d| d.iter_mut())
                .filter(|x| x.force == 1 && x.get_person().is_some_and(|x| x.get_bmap_size() == 1) && x.flag.value & 16 == 0)
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
                });
        }
    }
}

fn get_player_deployment_pattern() -> Vec<(i8, i8)> {
    let mut rel_pos = vec![(0, 0)];
    let rng = Random::get_system();
    let alt = rng.get_value(2) == 0;
    let ty = rng.get_value(11);
    match ty {
        1 => {
            let x = if alt { 6 }
            else {
                rel_pos.push((0, -1));
                4
            };
            for w in 1..x {
                rel_pos.push((-w, 0));
                rel_pos.push((w, 0));
                if alt {
                    rel_pos.push((-w, -1));
                    rel_pos.push((w, -1));
                }
            }
        }
        2 => {
            let x = if alt { 6 } else {
                rel_pos.push((1, 0));
                4
            };
            for w in 1..x {
                rel_pos.push((0,-w));
                rel_pos.push((0,w));
                if alt {
                    rel_pos.push((1, -w));
                    rel_pos.push((1, w));
                }
            }
        }
        3 => {
            let b = if alt { -1 } else { 1 };
            for w in 1..4 {
                for a in -w..w+1 {
                    rel_pos.push((a, b*w));
                }
            }
        }
        4 => {
            let b = if alt { -1 } else { 1 };
            for w in 1..4 {
                for a in -w..w+1 {
                    rel_pos.push((b*w, a));
                }
            }
        }
        5 => {
            let a = if alt { -1 } else { 1 };
            for x in 1..3 {
                rel_pos.push((a*x, 0));
                rel_pos.push((0, a*x));
                rel_pos.push((a*x, a*x));
            }
        }
        6 => {
            let a = if alt { -1 } else { 1 };
            let b = if rng.get_value(2) == 0 { -1 } else { 1 };
            for x in 1..3 {
                rel_pos.push((a*x, b*x));
                rel_pos.push((a*x, -1*b*x));
            }
        }
        7 => {
            rel_pos.push((3, 0));
            rel_pos.push((-3, 0));
            rel_pos.push((0, 3));
            rel_pos.push((0, -3));
            for x in 1..4 { // Diamond
                for y in 1..4 {
                    if (x + y) == 3 {
                        rel_pos.push((x, y));
                        rel_pos.push((-1*x, -1*y));
                        rel_pos.push((x, -1*y));
                        rel_pos.push((-1*x, y));
                    }
                }
            }
        }
        8 => {
            for x in 1..5 { // Plus
                rel_pos.push((-x, 0));
                rel_pos.push((x, 0));
                rel_pos.push((0, x));
                rel_pos.push((0, -x));
            }
        }
        9 => {  // Cross
            for x in 1..4 {
                rel_pos.push((x, x));
                rel_pos.push((-x, -x));
                rel_pos.push((x, -x));
                rel_pos.push((-x, x));

            }
        }
        10 => { // 4x4 Square
            let b = if alt { -1 } else { 1 };
            for y in 0..4 {
                for x in 0..4 {
                    if y == 0 && x == 0 { continue; }
                    rel_pos.push((x, b*y));
                }
            }
        }
        _ => {
            for w in 1..3 {
                rel_pos.push((-w, 0));
                rel_pos.push((w, 0));
                rel_pos.push((0, -w));
                rel_pos.push((0, w));
                rel_pos.push((-w, w));
                rel_pos.push((-w, -w));
                rel_pos.push((w, w));
                rel_pos.push((w, -w));
            }
        }
    }
    rel_pos
}



