use data::AccessoryAssets;
use concat_string::concat_string;
use unity::prelude::*;
use engage::{
    sortie::SortieSelectionUnitManager,
    dialog::yesno::*, 
    force::*, 
    gamedata::{assettable::*, accessory::*, item::ItemData, skill::*, unit::*, *}, 
    gamevariable::GameVariableManager, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItemSwitchMethods}, *}, 
    mess::Mess,
    proc::Bindable, 
    random::Random,
};
use itertools::Itertools;
use super::{names::EMBLEM_NAMES, person::PLAYABLE};

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

pub struct AssetData {
    pub male: AccessoryList,
    pub female: AccessoryList,
    pub bust_values: Vec<(i32, f32)>,  
    pub assets: Vec<AccessoryAssets>, 
}

pub static ASSET_DATA: Mutex<AssetData> = Mutex::new(
    AssetData{
        male: AccessoryList { list: Vec::new(), n_entries: [0; 8]},
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
            // println!("Accessory Name/ Index: {} / {}, LINE: {}", Mess::get(acc.name), acc.parent.index, line);
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
        let gender = unit_dress_gender(unit);
        let accessory_list = &mut unsafe { unit_get_accessory_list(unit, None) }.unit_accessory_array;
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
                static_fields[x.0 as usize].scale_stuff[11] = 1.0 + rng.get_value(50) as f32 * 0.03
        );
    }

}



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
    list[0].on_completed_end();
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



#[skyline::from_offset(0x01f266a0)]
fn get_dress_gender(person: &PersonData, method_info: OptionalMethod) -> i32; 

pub fn get_weapon_mode_2_hands() {}

#[unity::class("App", "UnitMenuItem")]
pub struct UnitMenuItem {
    pub menu: u64,
    pub junk: [u8; 0x4c],
    pub unit: &'static mut Unit,
}

pub fn reload_unit_info(unit: &Unit) -> i32 {
    unsafe {
        help_set_unit(0, None, false, false, false, None, None);
        help_set_unit(1, None, false, false, false, None, None);
        help_set_unit(0, Some(unit), false, false, false, None, None);
    }
    return 0x80;
}

#[skyline::from_offset(0x01f61b10)]
pub fn accessory_count(lol: u64, method_info: OptionalMethod) -> i32; 

pub fn get_unit_outfit_mode(unit: &Unit) -> i32 {
    if unit.person.get_asset_force() != 0 { return 0; }
    if !PLAYABLE.lock().unwrap().iter().any(|&x| x == unit.person.parent.index) { return 0;}

    let key = format!("G_O{}", unit.person.pid);
    if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry(key.as_str(), 1); return 0; }
    return GameVariableManager::get_number(key.as_str());
}

#[skyline::from_offset(0x01f86a50)]
fn help_set_unit(side: i32, unit: Option<&Unit>, relax: bool, reverse_rotation: bool, is_delay_load: bool, action: OptionalMethod, method_info: OptionalMethod);

#[skyline::from_offset(0x024622f0)]
fn create_basic_menu_content(method_info: OptionalMethod) -> &'static BasicMenuContent; 

#[skyline::from_offset(0x0245e330)]
fn transform_as_sub_menu(this: &BasicMenu<BasicMenuItem>, parent: &BasicMenu<BasicMenuItem>, parent_item: &BasicMenuItem, method_info: OptionalMethod);

fn unit_accessory_sub_menu_create_bind(menu: &mut BasicMenuItem){
    let list = menu.menu.full_menu_item_list.get_class();
    let new_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(list).unwrap();
    let count;
    if unsafe { accessory_count(0, None) >= 6 } {
        new_list.items = Il2CppArray::new(10).unwrap();
        count = 7;
    }
    else {
        new_list.items = Il2CppArray::new(4).unwrap();
        count = 4;
    }
    for _x in 0..count {
        let cock = get_base_menu_item_class().clone();
        let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(cock).unwrap();
        // new_menu_item.get_class_mut().get_virtual_method_mut("CustomCall").map(|method| method.method_ptr = as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("GetName").map(|method| method.method_ptr = unit_access_sub_menu_name as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("LCall").map(|method| method.method_ptr = unit_access_sub_menu_l_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("RCall").map(|method| method.method_ptr = unit_access_sub_menu_r_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("PlusCall").map(|method| method.method_ptr = unit_access_sub_menu_plus_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("YCall").map(|method| method.method_ptr = unit_access_sub_menu_y_call as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("MinusCall").map(|method| method.method_ptr = unit_access_sub_menu_minus_call as _);
        new_list.add(new_menu_item);
    }
    let content = unsafe { create_basic_menu_content(None) };
    let new_menu = BasicMenu::new(new_list, content);
    let descs = new_menu.create_default_desc();
    new_menu.bind_parent_menu();
    new_menu.create_bind(menu.menu, descs, "UnitAccessorySubMenu");
    new_menu.set_transform_as_sub_menu(menu.menu, menu);
    new_menu.set_show_row_num(count);
}

fn get_base_menu_item_class() -> &'static mut Il2CppClass {
    let menu = Il2CppClass::from_name("App", "UnitSelectSubMenu").unwrap().get_nested_types().iter().find(|x| x.get_name() == "BaseMenuItem").unwrap();
    Il2CppClass::from_il2cpptype(menu.get_type()).unwrap()
}

pub fn y_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    unit_accessory_sub_menu_create_bind(this);
    return 0x80;
}

pub fn unit_access_sub_menu_name(this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    let accessory_index = if this.index < 4 { this.index } else { this.index + 1 };
    let unit = SortieSelectionUnitManager::get_unit();
    let mode = get_unit_outfit_mode(unit);
    let slot = &unit.accessory_list.unit_accessory_array[accessory_index as usize];
    if slot.index == 0 { return "--------".into(); }
    if let Some(acc) = AccessoryData::try_index_get(slot.index) { 
        if ( mode == 1 && accessory_index != 5) || (mode == 2 && accessory_index != 0 ) { return format!("[{}]: {}", Mess::get("MID_SORTIE_SKILL_CATEGORY_EQUIPED"), Mess::get(acc.name)).into(); }
        else {
            return Mess::get(acc.name);
        }
    }
    else { return "--------".into(); }
}

pub fn unit_access_sub_menu_r_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32{
    let kind = if this.index < 4 { this.index }
        else { this.index + 1 };

    let unit = SortieSelectionUnitManager::get_unit();
    if GameVariableManager::get_number("G_Continuous") !=  0 {
        ASSET_DATA.lock().unwrap().change_accessory(unit, kind, true);
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else if accessory::next_unit_accessory(unit, kind, true) {
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else { return 0x800; }
}

pub fn unit_access_sub_menu_l_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let kind = if this.index < 4 { this.index }
        else { this.index + 1 };

    let unit = SortieSelectionUnitManager::get_unit();
    if GameVariableManager::get_number("G_Continuous") !=  0 {
        ASSET_DATA.lock().unwrap().change_accessory(unit, kind, false);
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else if accessory::next_unit_accessory(unit, kind, false) {
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else { return 0x800; }
}

pub fn unit_access_sub_menu_plus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let unit = SortieSelectionUnitManager::get_unit();
    let mode = get_unit_outfit_mode(unit);
    let key = format!("G_O{}", unit.person.pid);
    match mode {
        0 => { GameVariableManager::set_number(key.as_str(), 1); }
        1 => {
            if unsafe { accessory_count(0, None) >= 6 } { GameVariableManager::set_number(key.as_str(), 2); }
            else { GameVariableManager::set_number(key.as_str(), 0); }
        }
        _ => { GameVariableManager::set_number(key.as_str(), 0); }
    }
    this.menu.full_menu_item_list.iter().for_each(|item| item.rebuild_text());

    return reload_unit_info(unit);
}

pub fn unit_access_sub_menu_minus_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let unit = SortieSelectionUnitManager::get_unit();
    let accessory_list = &mut unsafe { unit_get_accessory_list(unit, None) }.unit_accessory_array;
    let kind = if this.index < 4 { this.index } else { this.index + 1 };
    if accessory_list[kind as usize].index != 0 {
        accessory_list[kind as usize].index = 0;
        this.rebuild_text();
        return reload_unit_info(unit);
    }
    else {  return 0;  }

}

pub fn unit_access_sub_menu_y_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    if GameVariableManager::get_number("G_Continuous") ==  0 { return 0; } 
    let unit = SortieSelectionUnitManager::get_unit();
    let accessory_list = &mut unsafe { unit_get_accessory_list(unit, None) }.unit_accessory_array;
    let rng = Random::get_game();
    let kind = if this.index < 4 { this.index } else { this.index + 1 };
    match unit_dress_gender(unit) {
        1 => { accessory_list[kind as usize].index = ASSET_DATA.lock().unwrap().male.get_index(kind, rng);},
        2 => { accessory_list[kind as usize].index = ASSET_DATA.lock().unwrap().female.get_index(kind, rng); }
        _ => {}, 
    }
    this.rebuild_text();
    return reload_unit_info(unit);
}

pub fn install_dvc_outfit() {
    if let Some(cc) = Il2CppClass::from_name("App", "SortieUnitSelect").unwrap().get_nested_types().iter().find(|x| x.get_name() == "UnitMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("YCall").map(|method| method.method_ptr = crate::randomizer::assets::y_call as _);
        println!("Replaced Added YCall to UnitMenuItem");
    }
}