use unity::prelude::*;
use engage::{
    random::Random,
    gamedata::{Gamedata, assettable::*, accessory::AccessoryData, unit::*},
};
use concat_string::concat_string;
use crate::CONFIG;
use itertools::Itertools;
pub struct AccAssetData {
    pub male: AccessoryList,
    pub female: AccessoryList,
    pub bust_values: Vec<(i32, f32)>,  
    pub assets: Vec<AccessoryAssets>, 
}

impl AccAssetData {
    pub fn add_data(&mut self, data: &AccessoryData) {
        match data.condition_gender {
            1 => { self.male.add_data(data); },
            2 => { self.female.add_data(data); },
            0 => { 
                self.male.add_data(data); 
                self.female.add_data(data); 
            }
            _ => {},
        }
    }
    pub fn add_asset_data(&mut self, line: String) {
        let list = AccessoryData::get_list().unwrap();
        let args: Vec<_> = line.split_whitespace().collect();
        let aid = concat_string!("AID_", args[0]);
        if let Some(acc) = list.iter().find(|&x| x.aid.contains(aid.as_str())) {
            let index = acc.parent.index;
            let gen = args[1].parse::<i32>().unwrap();
            let gstr = if gen == 1 { "M" } else if gen == 2 {"F"} else { "X" };
            if acc.mask == 1 {
                let asset = if index < 43 { concat_string!("uBody_Wear", gstr, "_", args[2]) }
                else { concat_string!("uBody_", args[2], "_c000") };
                self.assets.push(AccessoryAssets::new(index, gen, asset, 0));
                if gen == 1 {
                    let entry_index = self.male.n_entries[0];
                    self.male.n_entries[0] = entry_index + 1;
                    self.male.list.push((index, 0, entry_index));
                }
                else {
                    let entry_index = self.female.n_entries[0];
                    self.female.n_entries[0] = entry_index + 1;
                    self.female.list.push((index, 0, entry_index));
                }
                if args.len() == 5 {
                    let gen2 =  args[3].parse::<i32>().unwrap();
                    let asset2 = concat_string!("uBody_", args[4], "_c000");
                    self.assets.push(AccessoryAssets::new(index, gen2, asset2, 0));
                    if gen2 == 1 {
                        let entry_index = self.male.n_entries[0];
                        self.male.n_entries[0] = entry_index + 1;
                        self.male.list.push((index, 0, entry_index));
                    }
                    else {
                        let entry_index = self.female.n_entries[0];
                        self.female.n_entries[0] = entry_index + 1;
                        self.female.list.push((index, 0, entry_index));
                    }
                }
            }
            else {
                self.assets.push(AccessoryAssets::new(index, gen, args[2].to_string(), 1));
            }
        }
    }
    pub fn change_accessory(&self, unit: &Unit, kind: i32, up: bool) {
        let gender = super::super::unit_dress_gender(unit);
        let accessory_list = &mut unsafe { super::super::unit_get_accessory_list(unit, None) }.unit_accessory_array;
        match gender {
            1 => { accessory_list[kind as usize].index = self.male.get_next_index(kind, accessory_list[kind as usize].index, up); },
            2 => { accessory_list[kind as usize].index = self.female.get_next_index(kind, accessory_list[kind as usize].index, up); },
            _ => {},
        }
    }
    pub fn apply_bust_changes(&self) {
        let value = CONFIG.lock().unwrap().misc_option_1;
        if value  <= 0.4 { self.reset_busts(); }
        else if value >= 4.75 { self.randomized_busts(); }
        else { self.set_busts(); }
    }
    pub fn reset_busts(&self) {
        let static_fields = &mut Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().search_lists[2];
        self.bust_values.iter().for_each(|x|static_fields[x.0 as usize].scale_stuff[11] = x.1);
    }
    pub fn set_busts(&self) {
        let value = CONFIG.lock().unwrap().misc_option_1;
        let static_fields = &mut Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().search_lists[2];
        self.bust_values.iter().for_each(|x| static_fields[x.0 as usize].scale_stuff[11] = value );
    }
    pub fn randomized_busts(&self) {
        let rng = Random::get_game();
        let static_fields = &mut Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().search_lists[2];
        self.bust_values.iter()
            .for_each(|x|
                static_fields[x.0 as usize].scale_stuff[11] = 1.0 + rng.get_value(50) as f32 * 0.032
        );
    }

}

pub struct AccessoryList {
    pub list: Vec<(i32, i32, i32)>,
    pub n_entries: [i32; 8],
}

impl AccessoryList {
    pub fn add_data(&mut self, data: &AccessoryData) {
        let index = data.parent.index;
        let kind = data.kind;
        if kind < 0 || kind > 7 { return; }
        let entry_index = self.n_entries[kind as usize];
        self.n_entries[kind as usize] = entry_index + 1;
        self.list.push((index, kind, entry_index) );
    }
    pub fn get_index(&self, index: i32, rng: &Random) -> i32 {
        if index < 0 || index > 7 { return 0; }
        if self.n_entries[ index as usize ] == 0 { return 0; }
        let position = rng.get_value( self.n_entries[ index as usize]);
        if let Some(found) = self.list.iter().find(|&x| x.1 == index && x.2 == position) { found.0 } 
        else { 0 }
    }
    pub fn get_next_index(&self, index: i32, current: i32, increase: bool) -> i32 {
        if index < 0 || index > 7 || self.n_entries[ index as usize ] == 0 { return 0; }
        let list: Vec<_> = self.list.iter().filter(|&x| x.1 == index).sorted_by(|a,b| Ord::cmp(&a.0, &b.0) ).map(|c| c.0 ).collect();
        let length = list.len();
        if length == 0 { return 0; }
        let new_acc = 
            if current == 0 {
                if increase { 
                    if list[0] == 0 { list[1] }
                    else { list[0] }
                }
                else { list[length-1] }
            }
            else if list.iter().any(|&x| x == current) {
                if increase {
                    if let Some(&new) = list.iter().find(|&&x| current < x) { new }
                    else { 0 }
                }
                else {
                    if let Some(&new) = list.iter().filter(|&&x| x < current).max() {
                        new
                    }
                    else { 0 }
                }
            }
            else {
                if increase { list[0] }
                else { list[length-1] }
            };
        return new_acc;
    }
}


pub struct AccessoryAssets {
    pub index: i32,
    pub gender: i32,
    pub asset: String,
    pub locator: i32,
}

impl AccessoryAssets {
    pub fn new(index: i32, gen: i32, asset: String, loc: i32) -> Self {
        Self { index: index, gender: gen, asset: asset.clone(), locator: loc }
    }
}

pub fn get_all_accesories() -> AccAssetData {
    let accessory_list = AccessoryData::get_list().unwrap(); 
    let mut asset_data =     
        AccAssetData{
            male: AccessoryList { list: Vec::new(), n_entries: [0; 8]},
            female: AccessoryList { list: Vec::new(), n_entries: [0; 8] },
            bust_values: Vec::new(),
            assets: Vec::new(),
        };
    include_str!("data/accessories.txt").lines().into_iter()
        .for_each(|line|{
            let new_line = line.to_string();
            asset_data.add_asset_data(new_line);
        }
    );
    if asset_data.assets.len() == 0 {
        for x in 43..accessory_list.len() { asset_data.add_data(accessory_list[x]);  }
        println!("Accessory Assets Added");
    }
    let static_fields = &Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields::<AssetTableStaticFields>().search_lists[2];
    for x in 0..static_fields.len() {
        let volume_bust = static_fields[x as usize].scale_stuff[11];
        if volume_bust > 0.6 {
            asset_data.bust_values.push( (x as i32, volume_bust) ); 
        }
        if  asset_data.bust_values.len() > 550 { break; }
    }

    asset_data
}