use data::AccessoryAssets;
use concat_string::concat_string;
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
use super::names::EMBLEM_NAMES;
use std::{fs, fs::File, io::Write};

pub mod accessory;
pub mod data;
pub mod animation;
pub mod bust;
pub mod emblem;
pub mod transform;
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
        if let Some(found) = self.list.iter().find(|&x| x.1 == index && x.2 == position) { found.0 } 
        else { 0 }
    }
}



pub struct AssetData {
    pub male: AccessoryList,
    pub female: AccessoryList,
    pub bust_values: Vec<(i32, f32)>,  
    pub assets: Vec<AccessoryAssets>, 
}

pub static ASSET_DATA: Mutex<AssetData> = Mutex::new(
    AssetData{
        male: AccessoryList { list: Vec::new(), n_entries: [0; 8] },
        female: AccessoryList { list: Vec::new(), n_entries: [0; 8] },
        bust_values: Vec::new(),
        assets: Vec::new(),
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
    pub fn add_asset_data(&mut self, line: String) {
        let list = AccessoryData::get_list().unwrap();
        let args: Vec<_> = line.split_whitespace().collect();
        let aid = concat_string!("AID_", args[0]);
        if let Some(acc) = list.iter().find(|&x| str_contains(&x.aid, aid.as_str())) {
            let index = acc.parent.index;
            let gen = args[1].parse::<i32>().unwrap();
            let gstr = if gen == 1 { "M" } else if gen == 2 {"F"} else { "X" };
            if acc.mask == 1 {
                let asset = if index < 43 { concat_string!("uBody_Wear", gstr, "_", args[2]) }
                else { concat_string!("uBody_", args[2], "_c000") };
                println!("AID {}: {}", index, asset);
                self.assets.push(AccessoryAssets::new(index, gen, asset, false));
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
            }
            else {
                self.assets.push(AccessoryAssets::new(index, gen, args[2].to_string(), true));
            }
        }
    }




    pub fn apply_bust_changes(&self) {
        if CONFIG.lock().unwrap().misc_option_1 <= 0.4 { self.reset_busts(); }
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
}

#[unity::class("App", "AssetTable")]
pub struct AssetTable {
    pub parent: StructBaseFields,
    pub preset_name: Option<&'static Il2CppString>,
    pub mode: i32,
    __: i32,
    pub conditions: Option<&'static mut Array<&'static mut Il2CppString>>,
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
    pub unity_colors: [UnityColor; 8],
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
#[unity::class("App", "AsssetTable.ConditionFlags")]
pub struct AssetTableConditionFlags {}

impl Gamedata for AssetTable {}
#[repr(C)]
pub struct AssetTableStaticFields { 
    preset_name: &'static List<Il2CppString>,
    pub search_lists: &'static mut Array<&'static mut List<AssetTable>>,
    condition_indexes: *const u8,
    pub condition_flags: &'static AssetTableConditionFlags,

}

impl AssetTableConditionFlags {
    pub fn add_by_key<'a>(&self, key: impl Into<&'a Il2CppString>) {
        unsafe { condition_add_by_key(self, key.into(), None);}
    }
    pub fn add_unit(&self, unit: &Unit ) {
        unsafe { condition_add_unit(self, unit, None);}
    }

}
#[derive(Clone, Copy)]
pub struct UnityColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl UnityColor {
    pub fn set(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
    }
}

pub struct AssetTableSound {
    pub voice: Option<&'static Il2CppString>,
    pub footstep: Option<&'static Il2CppString>,
    pub material: Option<&'static Il2CppString>,
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
    pub body_anim: Option<&'static Il2CppString>,
    pub ride_anim: Option<&'static Il2CppString>,
    pub unity_colors: [UnityColor; 8],
    pub scale_stuff: [f32; 19], 
    __ : i32,
    pub sound: AssetTableSound,
    pub info_anims: Option<&'static Il2CppString>,
    pub talk_anims: Option<&'static Il2CppString>,
    pub demo_anims: Option<&'static Il2CppString>,
    pub hub_anims: Option<&'static Il2CppString>,
    pub force_id: Option<&'static Il2CppString>,
    pub weapon_id: Option<&'static Il2CppString>,
    pub body_anims: &'static mut List<Il2CppString>,
    pub accessory_list: &'static mut List<AssetTableAccessory>,
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
    println!("Unlocking Royal Classes");
    let data = data::UNIQUE_JOB_DATA.lock().unwrap();
    data.iter().for_each(|ujob|{
        if let Some(job) = JobData::get_mut(ujob.jid.as_str()) {
            let flag = job.get_flag();
            if ujob.gender == 2 { flag.value |= 4;}
            else { flag.value |= 16; }
            flag.value |= 2;
            if flag.value & 20 == 20 { flag.value -= 20; }
        }
    });
    return; 
}

pub fn auto_adjust_asset_table(is_ghast: bool) {
    CONFIG.lock().unwrap().debug = false;
    if is_ghast {
        CONFIG.lock().unwrap().auto_adjust_asset_table = true;
        CONFIG.lock().unwrap().enable_tradables_item = true;
        CONFIG.lock().unwrap().save();
    }
    else {
        CONFIG.lock().unwrap().auto_adjust_asset_table = false;
        CONFIG.lock().unwrap().save();
        if !CONFIG.lock().unwrap().auto_adjust_asset_table { return; }
    }
    let list = AssetTable::get_list_mut().unwrap();
    list.iter_mut().for_each(|entry|{
        if entry.mode == 1 { 
            for x in 0..9 { entry.scale_stuff[x] = 1.0; }
            entry.scale_stuff[18] = 0.50;
            entry.scale_stuff[17] = 0.0;
            entry.scale_stuff[16] = 2.50;
            if let Some(body) = entry.body_model {
                match body.to_string().as_str() {
                    "oBody_Tik1AT_c000" => {
                        entry.scale_stuff[16] = 1.0;
                        entry.scale_stuff[17] = 0.0;
                        entry.scale_stuff[18] = 0.50;
                    }
                    "oBody_Wng1FM_c000"|"oBody_Wng2DM_c000"|"oBody_Wng2DM_c704" => {
                        entry.scale_stuff[18] = 0.50;
                    }
                    "oBody_Mrp0AT_c706" | "oBody_Mrp0AT_c715" | "oBody_Fyd0DT_c707" | "oBody_Fyd0DT_c750" => {
                        entry.scale_stuff[16] = 2.0;
                        entry.scale_stuff[18] = 0.0;
                        entry.scale_stuff[17] = 0.0;
                    }
                    "oBody_Sds0AT_c049"|"oBody_Sds0AT_c099"|"oBody_Sds1AT_c049" => {
                        entry.scale_stuff[16] = 0.5;
                    }
                    "oBody_Sdk1AT_c504" => {
                        entry.scale_stuff[16] = 0.70;
                    }
                    "oBody_Cav2CM_c000" | "oBody_Cav2CF_c000" | "oBody_Wlf0CT_c707" | "oBody_Wlf0CT_c715" => {
                        entry.scale_stuff[16] = 2.40;
                    }
                    _ => {}
                }
            }
            if let Some(ride_model) = entry.ride_model {
                match ride_model.to_string().as_str() {
                    "oBody_Cav0BR_c000" | "oBody_Wng0ER_c000" | "oBody_Cmi0DR_c561" => {
                        entry.scale_stuff[16] = 2.10;
                    }
                    "oBody_Sig0BR_c531" | "oBody_Sig0BR_c538" => {
                        entry.scale_stuff[16] = 2.20;
                    }
                    "oBody_Cav2CR_c000"  => {
                        entry.scale_stuff[16] = 2.40;
                    }
                    _ => {}
                }
            }
            if let Some(body_act) = entry.body_anim {
                match body_act.to_string().as_str() {
                    "UAS_oBody_AM" | "UAS_oBody_AF" => {
                        entry.scale_stuff[16] = 2.60;
                    }
                    "UAS_oBody_FF" | "UAS_oBody_FM" | "UAS_oBody_BF" | "UAS_oBody_BM"  => {
                        entry.scale_stuff[16] = 2.40;
                    }
                    _ => {}
                }
            }
        }
    });
    unsafe { asset_table_on_completed_end(list[0], None); }
}

pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.person.pid.contains(PIDS[0]) {
         if unit.edit.is_enabled() { return unit.edit.gender; }
    }
    unsafe { get_dress_gender(unit.person, None) }
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

#[skyline::from_offset(0x01bafdd0)]
fn condition_add_by_key(condition: &AssetTableConditionFlags, key: &Il2CppString, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb0200)]
fn condition_add_unit(condition: &AssetTableConditionFlags, unit: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x01f266a0)]
fn get_dress_gender(person: &PersonData, method_info: OptionalMethod) -> i32; 

pub fn get_weapon_mode_2_hands() {

    let asset_table: Vec<_> = AssetTable::get_list().unwrap().iter().filter(|asset| 
        {
            let mut sum = 0.0;
            for x in 0..14 { sum += asset.scale_stuff[x]; }
            sum > 1.0 && asset.mode != 1

        }
        
        ).collect();

    let filename = "sd:/Draconic Vibe Crystal/Scale.txt";
    let mut file = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
    asset_table.iter()
        .for_each(|asset|{
            let sum = asset.unity_colors[0].r + asset.unity_colors[0].g + asset.unity_colors[0].b;
            let skin: [i32; 3] = [ (asset.unity_colors[0].r * 255.0 ) as i32, (asset.unity_colors[0].g * 255.0 ) as i32, (asset.unity_colors[0].b * 255.0 ) as i32 ];
            let mut hands = "Scale: ".to_string();
            for x in 0..14 {
                hands = format!("{} {}", hands, asset.scale_stuff[x]);
            }
            if let Some(head) = asset.head_model {
                hands = format!("{}\tHead: {}", hands, head);
            }
            if sum > 1.0 {
                hands = format!("{}\tSkin: {} {} {}", hands, skin[0], skin[1], skin[2]);
            }
            if let Some(conditions) = &asset.conditions {
                hands = format!("{}\tConditions: ", hands);
                conditions.iter().for_each(|con|{
                    hands = format!("{} {}", hands, con);
                });
            }
            writeln!(&mut file, "Mode: {} | {}", asset.mode, hands).unwrap();
        }
    );

    /* 
    ItemData::get_list().unwrap().iter().for_each(|item|{
        let iid = item.iid.to_string();
        let mut hands = String::new();
        if let Some(found) = asset_table.iter().find(|&asset|{
            if let Some(cons) = &asset.conditions {
                cons.iter().any(|str| str.contains(iid.as_str()))
            }
            else { false }
        }){
            if let Some(right) = found.right_hand {
                if let Some(left) = found.left_hand {
                    hands = format!("{}\t{}", right.to_string(), left.to_string());
                }
                else {
                    hands = right.to_string();
                }
                writeln!(&mut file, "{}\t{}", iid, hands).unwrap();
            }
            else if let Some(left) = found.left_hand {
                hands = left.to_string();
                writeln!(&mut file, "{}\t{}", iid, hands).unwrap();
            }

        }
    }); 
    */
}