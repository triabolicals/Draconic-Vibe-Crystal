use unity::prelude::*;
use engage::{
    dialog::yesno::*,
    force::*,
    random::Random,
    gamevariable::GameVariableManager,
    mess::Mess,
    gamedata::{*, accessory::*, item::ItemData},
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItem}},
    gamedata::{unit::*, skill::*,},
};

pub mod accessory;
pub mod animation;
pub mod bust;

use crate::{utils::str_contains, CONFIG};
use unity::system::List;
use crate::enums::*;
static mut ASSET_SIZE: usize = 0;
use std::sync::Mutex;

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
        let found = self.list.iter().find(|&x| x.1 == index && x.2 == position);
        if found.is_none() { 0 }
        else { found.unwrap().0 }
    }
}



pub struct AssetData {
    pub male: AccessoryList,
    pub female: AccessoryList,
    pub bust_values: Vec<(i32, f32)>,  
}

pub static ASSET_DATA: Mutex<AssetData> = Mutex::new(
    AssetData{
        male: AccessoryList { list: Vec::new(), n_entries: [0; 8] },
        female: AccessoryList { list: Vec::new(), n_entries: [0; 8] },
        bust_values: Vec::new(),
    }
);

impl AssetData {
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
    pub fn apply_bust_changes(&self) {
        if CONFIG.lock().unwrap().misc_option_1 <= 0.4 { self.reset_busts(); }
        else { self.set_busts(); }
    }
    pub fn reset_busts(&self) {
        let static_fields = &mut Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().search_lists[2];
        for x in &self.bust_values {
            static_fields[x.0 as usize].scale_stuff[11] = x.1;
        }
    }
    pub fn set_busts(&self) {
        let value = CONFIG.lock().unwrap().misc_option_1;
        let static_fields = &mut Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().search_lists[2];
        for x in &self.bust_values {
            static_fields[x.0 as usize].scale_stuff[11] = value;
        }
    }
}

#[unity::class("App", "AssetTable")]
pub struct AssetTable {
    pub parent: StructBaseFields,
    pub preset_name: Option<&'static Il2CppString>,
    pub mode: i32,
    __: i32,
    pub conditions: Option<&'static mut Array<&'static Il2CppString>>,
    pub body_model: Option<&'static Il2CppString>,
    pub dress_model: Option<&'static Il2CppString>,
    pub head_model: Option<&'static Il2CppString>,
    pub hair_model: Option<&'static Il2CppString>,
    pub ride_model: Option<&'static Il2CppString>,
    pub ride_dress_model: Option<&'static Il2CppString>,
    pub left_hand: Option<&'static Il2CppString>,
    pub right_hand: Option<&'static Il2CppString>,
    pub trail: Option<&'static Il2CppString>,
    pub magic: Option<&'static Il2CppString>,
    pub body_anim: Option<&'static Il2CppString>, 
    pub ride_anim: Option<&'static Il2CppString>,
    pub info_anim: Option<&'static Il2CppString>,
    pub talk_anim: Option<&'static Il2CppString>,
    pub demo_anim: Option<&'static Il2CppString>,
    pub hub_anim: Option<&'static Il2CppString>,
    pub hair_r: u8,
    pub hair_g: u8,
    pub hair_b: u8,
    pub grad_r: u8,
    pub grad_g: u8,
    pub grad_b: u8,
    pub skin_r: u8,
    pub skin_g: u8,
    pub skin_b: u8,
    pub toon_r: u8,
    pub toon_g: u8,
    pub toon_b: u8,
    pub mask_color_100_r: u8,
    pub mask_color_100_g: u8,
    pub mask_color_100_b: u8,
    pub mask_color_075_r: u8,
    pub mask_color_075_g: u8,
    pub mask_color_075_b: u8,
    pub mask_color_050_r: u8,
    pub mask_color_050_g: u8,
    pub mask_color_050_b: u8,
    pub mask_color_025_r: u8,
    pub mask_color_025_g: u8,
    pub mask_color_025_b: u8,
    unity_colors: [u64; 16],
    pub accessories: [&'static mut AssetTableAccessory; 8],
    pub accessory_list: &'static List<AssetTableAccessory>,
    pub scale_stuff: [f32; 19], 
    ___: i32,
    pub voice: Option<&'static Il2CppString>,
    pub foot_steps: Option<&'static Il2CppString>,
    pub material: Option<&'static Il2CppString>,
    pub comment: Option<&'static Il2CppString>,
    //ConditionIndexes
}
impl Gamedata for AssetTable {}
pub struct AssetTableStaticFields { 
    preset_name: &'static List<Il2CppString>,
    pub search_lists: &'static mut Array<&'static mut List<AssetTable>>,
}


#[unity::class("App", "AssetTable.Result")]
pub struct AssetTableResult {
    pub pid: &'static Il2CppString,
    pub jid: &'static Il2CppString,
    pub body_model: &'static Il2CppString,
    pub dress_model: &'static Il2CppString,
    pub head_model: &'static Il2CppString,
    pub hair_model: &'static Il2CppString,
    pub ride_model: &'static Il2CppString,
    pub ride_dress_model: &'static Il2CppString,
    pub left_hand: &'static Il2CppString,
    pub right_hand: &'static Il2CppString,
    pub trail: &'static Il2CppString,
    pub magic: &'static Il2CppString,
    pub body_anim: &'static Il2CppString, 
    pub ride_anim: &'static Il2CppString,
    unity_colors: [u64; 16],
    pub scale_stuff: [f32; 19], 
}

#[unity::class("App", "AssetTableAccessory")]
pub struct AssetTableAccessory {
    pub locator: Option<&'static Il2CppString>,
    pub model: Option<&'static Il2CppString>, 
}
#[unity::from_offset("App","AssetTable", "set_Conditions")]
pub fn asset_table_set_conditions(this: &AssetTable, value: &Array<&Il2CppString>, method_info: OptionalMethod);

#[unity::from_offset("App","AssetTable", "set_Conditions")]
pub fn asset_table_mut_set_conditions(this: &mut AssetTable, value: &Array<&Il2CppString>, method_info: OptionalMethod);

#[unity::from_offset("App","AssetTable", "get_Conditions")]
pub fn asset_table_get_conditions(this: &AssetTable, method_info: OptionalMethod) -> &'static mut Array<&'static Il2CppString>;

#[unity::from_offset("App","AssetTable", ".ctor")]
pub fn asset_table_ctor(this: &AssetTable, method_info: OptionalMethod);

#[unity::from_offset("App","AssetTable", ".cctor")]
pub fn asset_table_cctor( method_info: OptionalMethod);

#[unity::from_offset("App","AssetTable", "OnCompletedEnd")]
pub fn asset_table_on_completed_end(this: &AssetTable, method_info: OptionalMethod);

#[unity::from_offset("App","AssetTable", "OnBuild")]
pub fn asset_table_on_build(this: &AssetTable, method_info: OptionalMethod);


//Unlock royal classes if asset table entry is found
pub fn unlock_royal_classes(){
    let list = AssetTable::get_list().unwrap();
    let job_list = JobData::get_list().unwrap();
    
    for j in 0..job_list.len() {
        let current_job = &job_list[j as usize];  
        let job = current_job.jid.get_string().unwrap();
        let flag = current_job.get_flag();
        if flag.value & 1 == 0 {continue; }    // If not reclassable, skip
        if flag.value & 2 != 0 {continue; } // If already reclassable by everyone, skip
        for x in 0..list.len(){
                //Search all assettable entries
            let asset_entry = &list[x];
            if asset_entry.body_model.is_none() || asset_entry.conditions.is_none() { continue; }
            let mut job_conditions: [bool; 3] = [false; 3];
            let conditions = asset_entry.conditions.as_ref().unwrap(); 
            for y in 0..conditions.len() {
                if conditions[y].get_string().is_err() { continue; }
                if conditions[y].get_string().unwrap() == job { job_conditions[0] = true; }
                if conditions[y].get_string().unwrap() == "女装" { job_conditions[1] = true;}  //Females
                if conditions[y].get_string().unwrap() == "男装" { job_conditions[2] = true;}  // Dudes
            }
            if job_conditions[0] {
                if job_conditions[1] {
                    flag.value = flag.value | 2;
                    flag.value = flag.value | 4;
                }
                else if job_conditions[2] {
                    flag.value = flag.value | 2;
                    flag.value = flag.value | 16;
                }
            }
        }
        // If both Male and Female are flagged, disable flags
        if flag.value & 4 != 0 && flag.value & 16 != 0 {    flag.value = 3; }
    }
}

pub fn auto_adjust_asset_table() {
    if !CONFIG.lock().unwrap().auto_adjust_asset_table { return; }
    let list = AssetTable::get_list_mut().unwrap();
    for j in 0..list.len() {
        if list[j].mode != 1 { continue; }
        list[j].scale_stuff[16] = 2.50;
        list[j].scale_stuff[15] = 0.0;
        list[j].scale_stuff[14] = 0.0;
        for x in 0..9 {
         list[j].scale_stuff[x] = 1.0;

        }
    }
    unsafe { asset_table_on_completed_end(list[0], None); }
}


#[skyline::from_offset(0x01bb0100)]
pub fn unit_god_get_state(this: &Unit, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x1bb2260)]
pub fn get_body_anims(this: &AssetTableResult, method_info: OptionalMethod) -> &'static mut List<Il2CppString>;

#[skyline::from_offset(0x01a21460)]
pub fn get_engage_attack(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[skyline::from_offset(0x03785820)]
pub fn copy_str(string: &Il2CppString, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[skyline::from_offset(0x01bb2270)]
pub fn asset_table_result_accessory_list(this: &AssetTableResult, method_info: OptionalMethod) -> &'static mut List<AssetTableAccessory>;

#[skyline::from_offset(0x01baf640)]
pub fn try_add_accessory_list(this: &mut List<AssetTableAccessory>, accessory: &AssetTableAccessory, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb5a90)]
pub fn get_for_talk(pid: &Il2CppString, method_info: OptionalMethod) -> &'static mut AssetTableResult;

// Fixing Engage Attack Animation (kinda)

#[skyline::from_offset(0x01bb2120)]
fn get_volume_bust(this: &AssetTableResult, method_info: OptionalMethod) -> f32;

#[skyline::from_offset(0x01bb2130)]
fn set_volume_bust(this: &AssetTableResult, value: f32, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb1ff0)]
fn set_volume_legs(this: &AssetTableResult, value: f32, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb1fe0)]
fn get_volume_legs(this: &AssetTableResult, method_info: OptionalMethod) -> f32;

#[skyline::from_offset(0x01a4dff0)]
fn unit_get_accessory_list(this: &Unit, method_info: OptionalMethod) -> &'static mut UnitAccessoryList;

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup(this: &mut AssetTableResult, mode: i32, god_data: Option<&GodData>, is_darkness: bool, conditions: &Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    if mode > 10 {
        return call_original!(this, mode-10, god_data, is_darkness, conditions, method_info);
    }
    if god_data.is_none() {
        return call_original!(this, mode, god_data, is_darkness, conditions, method_info);
    }
    let gid = god_data.unwrap().gid.get_string().unwrap(); 
    let is_enemy_emblem = crate::randomizer::emblem::enemy::ENEMY_EMBLEMS.iter().find(|&x| x.0 == gid);
    if is_enemy_emblem.is_some() {
        let emblem_index = is_enemy_emblem.unwrap().1;
        let new_emblem = crate::randomizer::emblem::EMBLEM_ORDER.lock().unwrap()[emblem_index as usize] as usize;
        if new_emblem > 19 { return call_original!(this, mode, god_data, is_darkness, conditions, method_info);  }
        let replace_god = if new_emblem < 12 { GodData::get( EMBLEM_GIDS[new_emblem]).unwrap() }
            else { GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[new_emblem])).unwrap() };

        let is_m002 = gid == "GID_M002_シグルド";
        return call_original!(this, mode, Some(replace_god), !is_m002, conditions, method_info);
    }
    else { return call_original!(this, mode, god_data, is_darkness, conditions, method_info); }


}