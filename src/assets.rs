use concat_string::concat_string;
use bitflags::Flags;
use std::sync::RwLock;
use data::SEARCH_LIST;
use unity::{
    prelude::*,
    system::List,
};
use crate::{assets::conditions::ConditionFlags};
use std::sync::OnceLock;
use engage::{
    dialog::yesno::*, force::*, gamedata::{accessory::*, assettable::*, item::ItemData, skill::*, unit::*, *}, 
    gamevariable::GameVariableManager, menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItemSwitchMethods}, *}, mess::Mess, random::Random 
};
use crate::{
    config::DVCVariables, enums::*, utils::str_contains, CONFIG,
    randomizer::{names::EMBLEM_NAMES, person::PLAYABLE},
};
use animation::*;

// Move to Accessory
pub static ACCESSORY_COUNT: OnceLock<i32> = OnceLock::new();
pub static ASSET_STATUS: RwLock<AssetStatus> = RwLock::new(AssetStatus{
    engaging_count: 0, 
    engage_atk_eirika: 11, 
    engage_atk_3h: 12, 
    engage_atk_chrom: 18, 
    engage_atk_type: 0,
    unit: 0,
    link_unit1: 0,
    link_unit2: 0,
    link_god: 0,
    darkness: false,
    condition_flag: ConditionFlags::empty(),
});

pub struct AssetStatus {
    pub engaging_count: i32,
    pub engage_atk_eirika: i32,
    pub engage_atk_3h: i32,
    pub engage_atk_chrom: i32,
    pub engage_atk_type: i32,
    pub unit: i32,
    pub link_unit1: i32,
    pub link_unit2: i32,
    pub link_god: i32,
    pub darkness: bool,
    pub condition_flag: ConditionFlags,
}
impl AssetStatus {
    pub fn reset(&mut self) { 
        self.engaging_count = 0;
        self.engage_atk_3h = 12;
        self.engage_atk_eirika = 11;
        self.engage_atk_chrom = 22;
        self.engage_atk_type = 0;
        self.unit = 0;
        self.link_unit1 = 0;
        self.link_unit2 =  0;
        self.link_god = 0;
        self.condition_flag.clear();
    }
    pub fn reset_engage_atk(&mut self) {
        self.engage_atk_3h = 12;
        self.engage_atk_eirika = 11;
        self.engage_atk_chrom = 22;
        self.engage_atk_type = 0;
        self.unit = 0;
        self.darkness = false;
        self.link_unit1 = 0;
        self.link_unit2 =  0;
        self.link_god = 0;
    }
}


pub fn get_accessory_count() { ACCESSORY_COUNT.get_or_init(|| unsafe { accessory_list_get_count(0, None) } ); }

#[unity::from_offset("App", "UnitAccessoryList", "get_Count")]
fn accessory_list_get_count(this: u64, method_info: OptionalMethod) -> i32;

pub mod accessory;
pub mod data;
pub mod animation;
pub mod bust;
pub mod emblem;
pub mod transform;
pub mod accmenu;
pub mod dress;
pub mod conditions;

// pub mod dress;

#[unity::class("Combat", "AnimSetDB")]
pub struct AnimSetDB{
    pub parent: StructBaseFields,
    pub name: &'static Il2CppString,
    pub atk1: Option<&'static Il2CppString>,
    pub atk2: Option<&'static Il2CppString>,
    pub atk3: Option<&'static Il2CppString>,
    pub atk4: Option<&'static Il2CppString>,
    pub atk5: Option<&'static Il2CppString>,
    pub atkc: Option<&'static Il2CppString>,
    pub atkt: Option<&'static Il2CppString>, 
}
impl Gamedata for AnimSetDB {}



pub fn get_unit_outfit_mode(unit: &Unit) -> i32 {
    if unit.person.get_asset_force() != 0 { return 0; }
    if !PLAYABLE.get().unwrap().iter().any(|&x| x == unit.person.parent.index) { return 0;}

    let key = format!("G_O{}", unit.person.pid);
    if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry(key.as_str(), 1); return 0; }
    return GameVariableManager::get_number(key.as_str());
}

//Unlock royal classes if asset table entry is found
pub fn unlock_royal_classes(){}

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(this: &mut AssetTableResult, mode: i32, unit: &mut Unit, equipped: Option<&ItemData>, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let result = call_original!(this, mode, unit, equipped, conditions, method_info);   // Pre-set Conditions
    let conditions_flags = dress::commit_for_unit_dress(result, mode, unit, equipped, conditions);
    if conditions_flags.contains(ConditionFlags::Transform) && mode == 1 { return result; }
    emblem::bust_modifier_randomization(result, unit.grow_seed);
    let pid = unit.person.pid.to_string();

    if conditions_flags.contains(ConditionFlags::EngageAttack) && mode == 2 && !conditions_flags.contains(ConditionFlags::Talk) {
        emblem::adjust_engage_attack_animation(result, unit, equipped);  
        return result;
    }
    if ( engage::gameuserdata::GameUserData::get_chapter().cid.to_string().contains("G00") && unit.person.get_asset_force() != 0 ) || pid.contains("_チキ")   { 
        if mode == 2 { edit_asset_weapon(result, false, equipped); }
        return result;
    }
    if conditions_flags.contains(ConditionFlags::ClassChange) || conditions_flags.contains(ConditionFlags::Ballista) || conditions_flags.contains(ConditionFlags::Vision) || conditions_flags.contains(ConditionFlags::AllyDarkEmblem) { return result; }   
    if conditions_flags.contains(ConditionFlags::Engaging) { 
        animation::adjust_engaging_animations(result, unit);
        return result;
    }

    if conditions_flags.contains(ConditionFlags::Transforming) { 
        animation::edit_result_for_monster_trans(result, unit, equipped, mode); 
        return result;
    }

    // Class Animations
    if CONFIG.lock().unwrap().debug {
        println!("{}: Job: {} Mode: {}", Mess::get_name(unit.person.pid), Mess::get_name(unit.job.jid), mode);
        if mode == 2 {
           // println!("Body / Dress: {} / {}", result.body_model ,result.dress_model);
            // if !result.ride_model.is_null() { println!("Ride Model: {}", result.ride_model); }
            // if !result.ride_dress_model.is_null() { println!("Ride Dress Model: {}", result.ride_dress_model); }
            result.body_anims.iter().for_each(|m| println!("Before Body Act: {}", m));
            set_class_animations(result, unit.job, equipped, unit, mode, conditions_flags);
            result.body_anims.iter().for_each(|m| println!("After Body Act: {}", m));
           // println!("Body / Dress: {} / {}", result.body_model ,result.dress_model);
            if !result.ride_model.is_null() { println!("Ride Model: {}", result.ride_model); }
            if !result.ride_dress_model.is_null() { println!("Ride Dress Model: {}", result.ride_dress_model); }
        }
        else {
            // println!("Body {}", result.body_model);
          //  if !result.ride_model.is_null() { println!("Ride Model: {}", result.ride_model); }
            result.body_anims.iter().for_each(|m| println!("Before Body Act: {}", m));
            set_class_animations(result, unit.job, equipped, unit, mode, conditions_flags);
            result.body_anims.iter().for_each(|m| println!("After Body Act: {}", m));
        }
    }
    else { set_class_animations(result, unit.job, equipped, unit, mode, conditions_flags); }
    // Weapon 
    if mode == 2 { edit_asset_weapon(result, false, equipped); }
    result
}

pub fn edit_asset_weapon(result: &mut AssetTableResult, equipped: bool, item: Option<&ItemData>) {
    if !result.right_hand.is_null() { if result.right_hand.to_string().contains("00") { return; } }
    if !result.left_hand.is_null() { if result.left_hand.to_string().contains("00") { return; } }
    if let Some(w_item) = item {
        if w_item.kind == 9 { return; }
        let weapons = &SEARCH_LIST.get().unwrap().items;
        if GameVariableManager::get_number("G_RandAsset") & 1 != 0  {
            let rng = Random::get_system();
            match w_item.kind {
                6 => {  //Magic
                    if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else {
                        let weapon = weapons.get_random(6, rng);
                        let _ = AssetTable::try_index_get(weapon.asset_entry).map(|entry| result.commit_asset_table(entry));
                        if weapon.kind == 6 { result.magic = concat_string!("MG_", MAGIC[rng.get_value(31) as usize]).into();  }
                        else if weapon.kind == 7 { result.magic = concat_string!("RD_", animation::ROD[rng.get_value(16) as usize]).into();  }
                    }
                }
                4 => {  // Bow
                    if rng.get_value(15) <= 1 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else {
                        let _ = AssetTable::try_index_get(weapons.get_random(4, rng).asset_entry).map(|asset| asset.right_hand.map(|right| result.right_hand = right));
                    }
                    let _ = AssetTable::try_index_get( weapons.get_random(4, rng).asset_entry ).map(|asset| asset.left_hand.map(|left| result.left_hand = left));
                }
                1|2|3|5 => {
                    if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else {
                        let weapon = weapons.get_random(w_item.kind, rng);
                        let _ = AssetTable::try_index_get(weapon.asset_entry).map(|entry| result.commit_asset_table(entry));
                        if weapon.kind == 7 { result.magic = concat_string!("RD_", animation::ROD[rng.get_value(16) as usize]).into();  }
                    }
                }
                _ => {}
            }
        }
        else if equipped {
            let _ = weapons.get_index(w_item.parent.index)
                .map(|item| AssetTable::try_index_get(item.asset_entry).map(|asset| result.commit_asset_table(asset)));
        }   
    }
}



pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.person.pid.to_string() == PIDS[0] { 
        if unit.edit.is_enabled() { return unit.edit.gender; }
    }
    unsafe { get_dress_gender(unit.person, None) }
}

pub fn is_sword_fighter_outfit(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() { if this.dress_model.to_string().contains("Swd0A") { return true } }
    if !this.body_model.is_null() { if this.body_model.to_string().contains("Swd0A") { return true } }
    return false;
}

pub fn is_tiki_engage(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() { if this.dress_model.to_string().contains("Tik1AT") { return true } }
    if !this.body_model.is_null() { if this.body_model.to_string().contains("Tik1AT") { return true } }
    return false;
}

pub fn result_commit_scaling(result: &mut AssetTableResult, data: &AssetTable) {
    for x in 0..9 { if data.scale_stuff[x] > 0.0 { result.scale_stuff[x] = data.scale_stuff[x]; } }
    if data.scale_stuff[11] > 0.0 { result.scale_stuff[9] = data.scale_stuff[11]; }   //  VBust
    if data.scale_stuff[12] > 0.0 { result.scale_stuff[10] = data.scale_stuff[12]; }  //  VAbdomen
    if data.scale_stuff[13] > 0.0 { result.scale_stuff[11] = data.scale_stuff[13]; }  // VTorso
    if data.scale_stuff[9] > 0.0 { result.scale_stuff[12] = data.scale_stuff[9]; }    // VArms
    if data.scale_stuff[10] > 0.0 { result.scale_stuff[13] = data.scale_stuff[10]; }  // VLeg               
    for x in 14..19 { if data.scale_stuff[x] > 0.0 { result.scale_stuff[x] = data.scale_stuff[x]; } }
}

#[skyline::from_offset(0x01baf640)]
pub fn try_add_accessory_list(this: &mut List<AssetTableAccessory>, accessory: &AssetTableAccessory, method_info: OptionalMethod);

#[skyline::from_offset(0x01a4dff0)]
fn unit_get_accessory_list(this: &Unit, method_info: OptionalMethod) -> &'static mut UnitAccessoryList;

#[skyline::from_offset(0x01f266a0)]
fn get_dress_gender(person: &PersonData, method_info: OptionalMethod) -> i32; 

pub fn install_dvc_outfit() {
    if let Some(cc) = Il2CppClass::from_name("App", "SortieUnitSelect").unwrap().get_nested_types().iter().find(|x| x.get_name() == "UnitMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("YCall").map(|method| method.method_ptr = accmenu::unit_menu_item_y_call as _);
        println!("Replaced Added YCall to UnitMenuItem");
    }
}

