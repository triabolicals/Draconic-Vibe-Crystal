use concat_string::concat_string;
use unity::{
    prelude::*,
    system::List,
};
use engage::{
    dialog::yesno::*, force::*, gamedata::{accessory::*, assettable::*, item::ItemData, skill::*, unit::*, *}, 
    gamevariable::GameVariableManager, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItemSwitchMethods}, *}, 
    mess::Mess,  
    random::Random, 
};
use crate::{
    utils::str_contains, CONFIG,
    config::DVCVariables,
    enums::*,
};
use {animation::*, data::item::*};
use super::{names::EMBLEM_NAMES, person::PLAYABLE};

pub mod accessory;
pub mod data;
pub mod animation;
pub mod bust;
pub mod emblem;
pub mod transform;
pub mod accmenu;
// pub mod dress;

#[unity::class("Combat", "AnimSetDB")]
pub struct AnimSetDB{
    pub parent: StructBaseFields,
    pub name: &'static Il2CppString,
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
pub fn unlock_royal_classes(){
    println!("Unlocking Royal Classes");
    let data = data::UNIQUE_JOB_DATA.get().unwrap();
    data.iter().for_each(|ujob|{
        if let Some(job) = JobData::try_get_hash(ujob.job_hash) {
            let flag = job.get_flag();
            if ujob.gender == 2 { flag.value |= 5;}
            else { flag.value |= 17; }
            flag.value |= 3;
            if flag.value & 20 == 20 { flag.value -= 20; }
        }
    });
    return; 
}
/*
pub fn auto_adjust_asset_table(is_ghast: bool) {
    return;
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
*/

pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.person.pid.to_string() == PIDS[0] { 
        if unit.edit.is_enabled() { return unit.edit.gender; }
    }
    unsafe { get_dress_gender(unit.person, None) }
}

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(this: &mut AssetTableResult, mode: i32, unit: &mut Unit, equipped: Option<&ItemData>, conditions: &mut Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let mut result;
    let sequence = engage::gameuserdata::GameUserData::get_sequence();
    if sequence == 0 {
        let result = call_original!(this, mode, unit, equipped, conditions, None);
        change_unique_class(unit.job, result, mode, unit_dress_gender(unit), equipped, false);
        return result;
    }
    let mut job_hash = 0;
    if unit.person.gender > 0 &&
        ( transform::is_emblem_class(unit) || transform::is_monster_class(unit) || ( equipped.is_some_and(|item| item.iid.to_string().contains("チキ")) && unit.status.value & 25165824 == 0)) {
        job_hash = unit.get_job().parent.index;
        if let Some(jid) = unit.person.jid { unit.job = JobData::get_mut(jid).unwrap(); }
        else {unit.job = JobData::get_mut("JID_ソードマスター").unwrap(); }
    }
    // unsafe { clear_result(this, None) };
    let outfit = get_unit_outfit_mode(unit);
    let is_hub = conditions.iter().any(|x| x.to_string() == "私服");
    let is_talk = conditions.iter().any(|x| x.to_string() == "会話" || x.to_string() == "情報" );

    // PlayerOutfit Mode
    let is_rosado = unit.person.parent.hash == 469588104;
    if is_rosado { if unit.edit.gender > 0 {  unit.edit.set_gender(2); } }
    if unit.person.get_asset_force() == 0 {
        if is_hub { result = call_original!(this, mode, unit, equipped, conditions, method_info); }
        else if outfit == 0 {
            let mut accessories = [-1; 16];
            for x in 0..unit.accessory_list.unit_accessory_array.len() {
                accessories[x] = unit.accessory_list.unit_accessory_array[x].index;
                unit.accessory_list.unit_accessory_array[x].index = 0;
            }
            result = call_original!(this, mode, unit, equipped, conditions, method_info);
            for x in 0..unit.accessory_list.unit_accessory_array.len() { unit.accessory_list.unit_accessory_array[x].index = accessories[x]; }
        }
        else if outfit == 1 {
            if unit.accessory_list.unit_accessory_array[0].index > 0 {
                let static_fields = &Il2CppClass::from_name("App", "AssetTable").unwrap().get_static_fields_mut::<AssetTableStaticFields>().condition_flags;
                unsafe { add_condition(static_fields, "私服".into(), None);}
            }
            result = call_original!(this, mode, unit, equipped, conditions, method_info);
        }
        else { result = call_original!(this, mode, unit, equipped, conditions, method_info); }
    }
    else { result = call_original!(this, mode, unit, equipped, conditions, method_info); }
    if is_rosado { if unit.edit.gender > 0 {  unit.edit.set_gender(1); } }
    let pid = unit.person.pid.to_string();
    if job_hash > 0 { 
        unit.job = JobData::try_index_get_mut(job_hash).unwrap(); 
        edit_result_for_monster_trans(this, unit, equipped, mode);
    }
    else if transform::has_enemy_tiki(unit) || equipped.is_some_and(|item| item.iid.to_string().contains("チキ")) {
        transform::enemy_tiki_emblem_transformation(result, unit, equipped, mode);
    }
    // Class Change / Co-Op Engage Attack or Loading into Map
    if conditions.iter().any(|con| con.to_string() == "エンゲージ開始") && GameVariableManager::get_bool(DVCVariables::ENGAGE_P_KEY) {
        if let Some(body) = result.body_anims.iter_mut().find(|act| act.to_string().contains("Tsf0A")){  
            unsafe { LINK_COUNT = 1; }
        }
        else if unsafe { LINK_COUNT == 0 } {
            if unit_dress_gender(unit) == 1 { result.body_anims.add(Il2CppString::new_static("Tsf0AM-No1_c001_N")); }
            else { result.body_anims.add(Il2CppString::new_static("Tsf0AF-No1_c051_N")); }
            unsafe { LINK_COUNT += 1; }
        }
    }
    if ( sequence == 7 && mode == 2 ) || conditions.iter().any(|con|{ let condition = con.to_string(); condition == "クラスチェンジ中" || condition == "エンゲージ開始"  }) {
        emblem::random_engage_voice(result);
        unique_class_dress(unit.job, result, unit_dress_gender(unit), mode, unit, false, true);
        return result; 
    }

    if is_hub && sequence == 4 { 
        for x in 0..4 {
            if unit.accessory_list.unit_accessory_array[x].index > 0 { 
                accessory::set_accessories_for_unit(unit, result); 
                break;
            }
        }
        return result; 
    }

    if pid.contains("_残像") { // Lyn Doubles
        illusion_double_assets(result, unit, mode, equipped, conditions); 
        return result; 
    }
    let is_engage = unit.status.value & 8388608 != 0;

    let state = unit.get_god_state();
    let generic_mode =  GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);

    if unsafe { !crate::utils::is_null_empty(result.dress_model, None) } && (is_engage || state != 2 ) { //Tiki Engage Mode 2
        if result.dress_model.to_string().contains("uBody_Tik1AT") { 
            if unit.person.get_flag().value & 2048 != 0 && generic_mode & 2 == 2  { change_hair_change(unit, result); }
            if unit.status.value & 16777216 != 0 {
                emblem::adjust_engage_attack_animation(result, unit, equipped, mode); 
            }
            if is_talk { result.scale_stuff[0] = 0.5; }
            return result; 
        }
    }
    if unsafe { !crate::utils::is_null_empty(result.body_model, None) } && ( is_engage || state != 2) { //Tiki Engage Mode 1
        if result.body_model.to_string().contains("Tik1AT") { 
            if unit.person.get_flag().value & 2048 != 0 && generic_mode & 2 == 2  { change_hair_change(unit, result); }
            result.body_anim = Some("UAS_Ent0AT".into());
            result.body_anims.add(Il2CppString::new_static("UAS_Ent0AT"));
            result.body_anims.iter().for_each(|act| println!("Tiki Body Act: {}", act));
            return result; 
        }
    }

    if unit.person.get_flag().value & 2048 != 0 {  // Generic Appearance
        let can_accessorize = crate::randomizer::RANDOMIZER_STATUS.read().unwrap().accessory;
        if generic_mode & 1 == 1 && mode == 2 && can_accessorize { data::HEAD_DATA.get().unwrap().replace_by_rng(unit, result);  }
        if generic_mode & 2 == 2 { change_hair_change(unit, result); }
        if can_accessorize { accessory::set_accessories_for_unit(unit, result); }
    }
    else if GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) && !pid.contains(PIDS[0]) {
        let name = unit.person.get_name().unwrap().to_string();
        if let Some(emblem_index) = RINGS.iter().position(|x| name == concat_string!("MPID_", x) || name == concat_string!("MGID_", x) ){
            let index = if emblem_index > 18 { emblem_index + 1 }
                else { emblem_index };
            if unsafe { EMBLEM_NAMES[index] } != -1 {
                let emblem_person = PersonData::get(PIDS[ unsafe { EMBLEM_NAMES[index] } as usize ]);
                result = unsafe { transform::asset_table_result_setup_person(result, mode, emblem_person, emblem_person.unwrap().get_job(), equipped, conditions, method_info) };
                accessory::clear_accessory_from_list(result.accessory_list, "Eff_EmblemAura");
            }
        }
    }

    if ( engage::gameuserdata::GameUserData::get_chapter().cid.to_string().contains("G00") && unit.person.get_asset_force() != 0 ) || pid.contains("_チキ")   { 
        if unit.status.value & 16777216 != 0 {  emblem::adjust_engage_attack_animation(result, unit, equipped, mode);  } 
        return result;
    }   // Ignore Divine Paralogue

    if GameVariableManager::get_number("G_RandAsset") > 1 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 { data::HEAD_DATA.get().unwrap().random_aoc(unit, result); }
    // Dance Command
    if conditions.iter().any(|con| con.to_string() == "踊り") && !conditions.iter().any(|con| con.to_string() == "砲台") {
        dancing_animation(result, unit, mode);
        return result;
    }

    if mode == 2 && conditions.iter().any(|con| con.to_string() == "砲台") && !conditions.iter().any(|con| con.to_string() == "踊り") {
        unique_class_dress(unit.job, result, unit_dress_gender(unit), mode, unit, is_engage, false);
        let body = if unit_dress_gender(unit) == 1 { "Bat0AM-Bw1_c000_L" } else { "Bat0AF-Bw1_c000_L" };
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();
        result.left_hand = "uWep_Ft00".into();
        result.right_hand = "uWep_Ft00".into();
        result.trail = "null".into();
        result.body_anims.add(Il2CppString::new_static(body));
        return result;
    }

    if state >= 2  { 
        unique_class_dress(unit.job, result, unit_dress_gender(unit), mode, unit, is_engage, false);
        if is_talk && is_engage && unit.god_unit.is_some_and(|god| god.data.gid.contains("チキ")) { result.scale_stuff[0] = 0.5; }
        emblem::adjust_engage_attack_animation(result, unit, equipped, mode);  
        return result;
    }
    if state == 0 { //Not Engage
        if let Some(name) = unit.person.get_name() { 
            let name_ = name.to_string(); 
            if job_hash == 0 {
                if name_.contains("Sfoglia") && mode == 1 { result.body_model = "oBody_Wng2DF_c000".into(); }
                else if name_.contains("MPID_Eve") && mode == 1 { result.body_model = "oBody_Rod2AF_c000".into(); } 
                if pid.contains("M022_紋章士") && GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0 {
                    if let Some(emblem_index) = EMBLEM_ASSET.iter().position(|x| pid.contains(x)) {
                        let gid = GameVariableManager::get_string(format!("G_R_{}", EMBLEM_GIDS[emblem_index]).as_str());
                        if let Some(emblem_index2) = EMBLEM_ASSET.iter().position(|x| gid.contains(x)){
                            if emblem_index2 >= 12 {    // DLC Dark Emblems
                                let gid2 = format!("GID_E006_敵{}", EMBLEM_ASSET[emblem_index2]);
                                return emblem::asset_table_result_god_setup(this, mode+10, GodData::get(gid2), true, conditions, method_info);
                            }
                        }
                        return emblem::asset_table_result_god_setup(this, mode+10, GodData::get(gid), true, conditions, method_info);  // Custom Emblems?
                    }
                }
                if !result.body_anims.iter().any(|body| body.to_string().contains("Ent0AT")) { correct_animations(unit, result, mode, equipped);   }
            }
            if equipped.is_some_and(|item| item.iid.contains("チキ")) {
                transform::enemy_tiki_emblem_transformation(result, unit, equipped, mode);
                return result;
            }
            let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 };
            let job = unit.get_job().jid.to_string();
            if kind == 6 && ( job.contains("メリュジーヌ") || job.contains("邪竜ノ娘") ) {
                result.right_hand = "null".into();
                result.left_hand = "null".into();
            }
            else if kind < 9 && mode == 2 { edit_asset_weapon(result, false, equipped); }
            unique_class_dress(unit.job, result, unit_dress_gender(unit), mode, unit, is_engage, false);
            return result;
        }
    }
    if state == 1 {
        unique_class_dress(unit.job, result, unit_dress_gender(unit), mode, unit, is_engage, false);
        if is_talk && unit.god_unit.is_some_and(|god| god.data.gid.contains("チキ")) { result.scale_stuff[0] = 0.5; }
        let kind = if equipped.is_some() { equipped.unwrap().kind } else { 0 } as usize;
        if mode == 2 && kind == 9 {
            if unit.job.jid.to_string().contains("裏邪竜ノ子") { result.body_anims.add(Il2CppString::new_static("Sds0AM-No2_c049_N")); }
            else if unit.job.jid.to_string().contains("裏邪竜ノ娘") && kind == 9 { result.body_anims.add(Il2CppString::new_static("Sds0AF-No2_c099_N")); } 
        }
        else if !result.body_anims.iter().any(|body| body.to_string().contains("Enb0")) {
            let gender = unit_dress_gender(unit);
            if gender != 1 && gender != 2 { return result; } //prevent monsters
            result.ride_model = "null".into();
            result.ride_dress_model = "null".into();
            result.ride_anim = None;
            let gender_str = if gender == 2 { "F" } else { "M" };
            if mode == 1 {
                result.body_anims.add(Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )) );
                result.body_anim = Some( Il2CppString::new_static( concat_string!("UAS_Enb0A", gender_str )));
            }
            else {
                let act = concat_string!("Enb0A", gender_str, "-", animation::WEP_PRE[kind], "1_c000_N");
                result.body_anims.iter_mut().filter(|body| body.to_string().contains(animation::WEP_PRE[kind]) ).for_each(|body| *body = Il2CppString::new_static(act.clone()));
            }
        }
        if mode == 2 { edit_asset_weapon(result, false, equipped); }
    }
    result
}
pub fn edit_asset_weapon(result: &mut AssetTableResult, equipped: bool, item: Option<&ItemData>) {
    if let Some(w_item) = item {
        if w_item.kind == 9 { return; }
        let weapons = data::WEAPON_ASSET.get().unwrap();
        if GameVariableManager::get_number("G_RandAsset") & 1 != 0  {
            let rng = Random::get_system();
            match w_item.kind {
                6 => {  //Magic
                    let weapon = weapons.get_random(6, rng);
                    if weapon.right_hand != "none" { 
                        result.right_hand = weapon.right_hand.clone().into();
                        if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    }
                    if weapon.kind == 6 { result.magic = concat_string!("MG_", MAGIC[rng.get_value(31) as usize]).into();  }
                    else if weapon.kind == 7 { result.magic = concat_string!("RD_", animation::ROD[rng.get_value(16) as usize]).into();  }
                }
                4 => {  // Bow
                    if rng.get_value(15) <= 1 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else { replace_weapon_hands( result, weapons.get_random(4, rng), false); }  //Bow
                    replace_weapon_hands( result, weapons.get_random(4, rng), true);    //Arrow
                }
                1|2|3|5 => {    //Melee Weapons
                    let weapon = weapons.get_random(w_item.kind, rng);
                    if rng.get_value(15) == 0 { result.right_hand = "uBody_Msc0AT_c000".into(); }
                    else if weapon.right_hand != "none" { 
                        if !weapon.right_hand.contains("Mg") {
                            result.right_hand = weapon.right_hand.as_str().into();
                        }
                    }
                    else if weapon.left_hand != "none" { result.right_hand = weapon.left_hand.as_str().into(); }
                    if weapon.kind == 7 { result.magic = concat_string!("RD_", animation::ROD[rng.get_value(16) as usize]).into();  }
                }
                _ => {}
            }
        }
        else if equipped {
            if let Some(weapon) = weapons.get_index(w_item.parent.index){
                replace_weapon_hands(result, weapon, false);
                replace_weapon_hands(result, weapon, true);
            }
        }
    }
}

fn replace_weapon_hands(result: &mut AssetTableResult, weapon: &WeaponAsset, left: bool) {
    if !left {
        if weapon.right_hand != "none" {  result.right_hand = weapon.right_hand.as_str().into(); }
    }
    else {
        if weapon.left_hand != "none" {  result.left_hand = weapon.left_hand.as_str().into(); }
    }
}

#[skyline::from_offset(0x01baf640)]
pub fn try_add_accessory_list(this: &mut List<AssetTableAccessory>, accessory: &AssetTableAccessory, method_info: OptionalMethod);

// Fixing Engage Attack Animation (kinda)

#[skyline::from_offset(0x01a4dff0)]
fn unit_get_accessory_list(this: &Unit, method_info: OptionalMethod) -> &'static mut UnitAccessoryList;

#[skyline::from_offset(0x01f266a0)]
fn get_dress_gender(person: &PersonData, method_info: OptionalMethod) -> i32; 

pub fn install_dvc_outfit() {
    if let Some(cc) = Il2CppClass::from_name("App", "SortieUnitSelect").unwrap().get_nested_types().iter().find(|x| x.get_name() == "UnitMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("YCall").map(|method| method.method_ptr = crate::randomizer::assets::accmenu::unit_menu_item_y_call as _);
        println!("Replaced Added YCall to UnitMenuItem");
    }
}