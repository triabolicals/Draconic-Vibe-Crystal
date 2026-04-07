use data::SEARCH_LIST;
use unity::prelude::*;
use engage::{
    unit::*,
    dialog::yesno::*, gamedata::{assettable::*, item::ItemData, skill::*, *}, 
    gamevariable::GameVariableManager, random::Random 
};
use engage::gameuserdata::GameUserData;
use engage::mess::Mess;
use crate::{config::DVCVariables, enums::*};
pub use::outfit_core::Mount;
use engage::combat::*;
use outfit_core::{get_outfit_data, print_asset_table_result, AssetFlags, CharacterAssetMode, UnitAssetMenuData};

pub mod accessory;
pub mod data;
pub mod animation;
pub mod emblem;
pub mod transform;
pub mod dress;
pub mod gmap;
pub(crate) mod engage_attack;

use animation::*;
use crate::assets::{transform::{has_fake_tiki, is_dragonstone}, data::search::search_by_iid};
use crate::config::{DVCFlags};
use std::io::Write;
use outfit_core::anim::AnimData;
use crate::assets::engage_attack::{adjust_engage_attack_animation, lueur_engage_atk};

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
impl AnimSetDB {
    pub fn get_engage1(&self) -> Option<&'static Il2CppString> { unsafe { anim_get_engage_1(self, None) } }
}
#[skyline::from_offset(0x1c8f340)]
fn anim_get_engage_1(this: &AnimSetDB, optional_method: OptionalMethod) -> Option<&'static Il2CppString>;

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(
    this: &mut AssetTableResult,
    mode: i32,
    unit: &mut Unit,
    equipped: Option<&ItemData>,
    conditions: &mut Array<&'static Il2CppString>,
    method_info: OptionalMethod) -> &'static mut AssetTableResult
{
    let result = call_original!(this, mode, unit, equipped, conditions, method_info);
    let conditions = dress::commit_for_unit_dress(result, mode, unit, equipped, conditions);  // Pre-set Conditions
    if unit.job.jid.str_contains("紋章士") { return result; }
    if mode == 3 {
        if unit.person.parent.index > 1 { result.scale_stuff[16] = 4.8; }
        return result;
    }
    if mode == 2 && conditions.flags.contains(AssetFlags::Engaging) {
        AnimData::remove(result, true, true);
        result.ride_model = None;
        result.ride_dress_model = None;
        if unit.person.is_hero() {
            if conditions.flags.contains(AssetFlags::Male) { result.body_anims.add("Tsf0AM-No1_c001_N".into()); }
            else { result.body_anims.add("Tsf0AF-No1_c051_N".into()); }
        }
        else if unit.person.parent.index == 1 { // Alear is not hero so set default engaging 1/2 anims
            result.body_anims.clear();
            if conditions.flags.contains(AssetFlags::Male) { result.body_anims.add("Com0AM-No1_c000_N".into()); }
            else { result.body_anims.add("Com0AF-No1_c000_N".into()); }
        }
    }
    if (conditions.flags.contains(AssetFlags::MapTransform) && mode == 1) || conditions.flags.contains(AssetFlags::Monster) { return result; }
    else if mode == 2 && (conditions.flags.contains(AssetFlags::EngAtkCoopMain) || conditions.flags.contains(AssetFlags::EngAtkCoopMain)) { // DragonBlast or BondBlast
        lueur_engage_atk(result, unit, &conditions);
    }
    else if conditions.flags.contains(AssetFlags::EngageAttack){
        adjust_engage_attack_animation(this, unit, equipped, &conditions);
        if has_fake_tiki(unit) {
            if unit_dress_gender(unit) == 1 { result.body_anims.add( "Sds0AM-No2_c049_N".into()); }
            else {  result.body_anims.add( "Sds0AF-No2_c099_N".into()); }
        }
    }
    else if conditions.emblem_unit {

    }
    else {  // anim correction
        if conditions.flags.contains(AssetFlags::EngageTiki) { return result; }
        get_outfit_data().correct_anims(result, unit, conditions.profile_flag, &conditions)
    }
    if conditions.character_mode == CharacterAssetMode::Combat {  // weapon asset
        edit_asset_weapon(result, false, 2, equipped);
    }
    if crate::DeploymentConfig::get().debug { print_asset_table_result(result, mode); }
    let pid = unit.person.pid.to_string();
    if ( mode == 2 && GameUserData::get_chapter().cid.str_contains("G00") && unit.force.is_some_and(|f| f.force_type != 0) ) || pid.contains("_チキ") {}
    result
}

pub fn edit_asset_weapon(result: &mut AssetTableResult, equipped: bool, mode: i32, item: Option<&ItemData>) {
    if mode == 2 && !equipped {
        if result.body_anims
            .iter()
            .any(|x| (x.str_contains("Msn") && x.str_contains("DF-Mg1")) || x.str_contains("Sdp0AF-Mg1") || x.str_contains("Sdk0AM-Mg1"))
        {
            result.right_hand = "uWep_Mg00".into();
            return;
        }
    }
    if UnitAssetMenuData::get().is_preview { return; }
    if item.is_some_and(|i| i.kind == 8) && mode == 2 { result.right_hand = "uWep_Ft00".into(); }
    if equipped {
        if let Some(asset_table) = item.and_then(|item| search_by_iid(item.iid, mode)) { result.commit_asset_table(asset_table); }
    }
    if !result.right_hand.is_null() { if result.right_hand.str_contains("00") { return; } }
    if !result.left_hand.is_null() { if result.left_hand.str_contains("00") { return; } }
    if let Some(w_item) = item {
        if w_item.kind == 9 { return; }
        let weapons = &SEARCH_LIST.get().unwrap().items;
        if DVCFlags::RandomWeaponAsset.get_value() {
            let rng = Random::get_system();
            match w_item.kind {
                6 => {  //Magic
                    if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else {
                        let weapon = weapons.get_random(6, rng);
                        let _ = AssetTable::try_index_get(weapon.asset_entry).map(|entry| result.commit_asset_table(entry));
                        if weapon.kind == 6 { result.magic = format!("MG_{}", MAGIC[rng.get_value(31) as usize]).into();  }
                        else if weapon.kind == 7 {
                            if !result.left_hand.is_null() {
                                result.right_hand = result.left_hand;
                                result.left_hand = "null".into();
                            }
                            result.magic = format!("RD_{}", ROD[rng.get_value(16) as usize]).into();
                        }
                    }
                }
                4 => {  // Bow
                    if rng.get_value(15) <= 1 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else if let Some(right) = AssetTable::try_index_get(weapons.get_random(4, rng).asset_entry).and_then(|asset|asset.right_hand) {
                        result.right_hand = right;
                    }
                    else if let Some(left) = AssetTable::try_index_get(weapons.get_random(4, rng).asset_entry).and_then(|asset|asset.left_hand) {
                        result.left_hand = left;
                    }
                }
                1|2|3|5 => {
                    if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else {
                        let weapon = weapons.get_random(w_item.kind, rng);
                        let _ = AssetTable::try_index_get(weapon.asset_entry).map(|entry| result.commit_asset_table(entry));
                        if weapon.kind == 7 { 
                            result.magic = format!("RD_{}", ROD[rng.get_value(16) as usize]).into();
                            if !result.left_hand.is_null() {
                                result.right_hand = result.left_hand;
                                result.left_hand = "null".into();
                            }
                         }
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.person.pid.to_string() == PIDS[0] || unit.person.flag.value & 128 != 0 {  
        if unit.edit.is_enabled() { return unit.edit.gender; }
    }
    if unit.person.flag.value & 32 != 0 {
        if unit.person.gender == 1 { 2 } else { 1 }
    }
    else { unit.person.gender }
}
pub fn get_unit_dress(unit: &Unit) -> Gender {
    let dress = unit_dress_gender(unit);
    if dress == 1 { Gender::Male } else if dress == 2 { Gender::Female } else { Gender::Other }
}

pub fn is_tiki_engage(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() { this.dress_model.to_string().contains("Tik1AT") }
    else if !this.body_model.is_null() { this.body_model.to_string().contains("Tik1AT") }
    else { false }
}