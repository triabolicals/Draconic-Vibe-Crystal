use super::*;
use animation::MONSTERS;
use engage::combat::{CombatRecord, CharacterGameStatus};
use item::UnitItem;

pub const MONSTER_PERSONS: [&str; 8] = ["PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼", "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"];
pub const SCALE: [f32; 8] = [  1.0, 1.0, 1.0, 1.0, 0.40, 1.0, 1.0, 0.40];
pub static mut OLD_EMBLEM: String = String::new();


pub struct CombatRecordDisplayClass85 {
    pub this: &'static CombatRecord,
    pub calc: u64,
}
pub fn is_monster_class(unit: &Unit) -> bool {
    let jid = unit.get_job().jid.to_string();
    if unit.person.get_bmap_size() > 1 || unit.person.gender == 0 { false }
    else { super::animation::MONSTERS.iter().any(|&monster| monster == jid)  }

}
pub fn is_emblem_class(unit: &Unit) -> bool {
    let pid = unit.person.pid.to_string();
    let jid = unit.get_job().jid.to_string();
    let hash = unit.get_job().parent.hash;
    if !crate::randomizer::job::JOB_HASH.iter().any(|&h| h == hash) { return false; }
    if unit.person.get_bmap_size() > 1  { return false; }
    if let Some(pos1) = EMBLEM_ASSET.iter().position(|&x1| format!("JID_紋章士_{}", x1) == jid) {
        if pos1 == 23 || pos1 == 19 { return false; }   // No Lueur / Ephiram
        if let Some(pos2) = EMBLEM_ASSET.iter().position(|&x2| pid.contains(x2)) { return pos1 != pos2;  }
        else { return true; }
    }
    return false;
}
pub fn has_enemy_tiki(unit: &Unit) -> bool {
    if let Some(god_unit) = unit.god_unit { god_unit.data.gid.to_string().contains("敵チキ") }
    else if let Some(god_unit) = unit.god_link { god_unit.data.gid.to_string().contains("敵チキ") }
    else { unit.person.pid.to_string().contains("チキ") }
}

#[skyline::from_offset(0x0247ce20)]
fn combat_side_convert_from(side_type: i32, is_reversed: bool, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x027def50)]
fn character_game_status_is_valid(gs: &CharacterGameStatus, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x027e0880)]
fn combat_character_game_status_import(this:&mut CharacterGameStatus, side: i32, calc: u64, side_type: i32, distance: i32, method_info: OptionalMethod);

fn adjust_emblem_zone(this: &mut CharacterGameStatus) {
    if this.emblem_identifier.is_some() && this.unit.is_some() {
        let unit = this.unit.unwrap();
        if unit.get_god_state() > 1 {
            if let Some(engage_attack) = unit.get_engage_attack()  {
                let sid = engage_attack.sid.to_string();
                let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
                    else if sid.contains("三級長エンゲージ技＋") { 20 }
                    else if sid.contains("三級長エンゲージ") { 12 }
                    else { 50 };
                if emblem_index > 20 { return; }
                let new_emblem_id = 
                    match emblem_index {
                        12|20 => { "Ede" },
                        16 => { "Sen"},
                        17 => { "Cam"},
                        _ => { ENGAGE_PREFIX[emblem_index] }
                    };
                this.emblem_identifier = Some(new_emblem_id.into());
            }
        }
    }
}

#[skyline::from_offset(0x01bb4290)]
pub fn asset_table_result_setup_person(this: &AssetTableResult, mode: i32, person: Option<&PersonData>, job: Option<&JobData>, equipped: Option<&ItemData>, conditions: &Array<&Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult;

pub fn enemy_tiki_emblem_transformation(result: &mut AssetTableResult, unit: &Unit, equipped: Option<&ItemData>, mode: i32) {
    let kind = if equipped.is_none() { 0 } else { equipped.unwrap().kind } as usize;
    let gender = unit_dress_gender(unit);
    let gen = if gender == 1 { "M" } else { "F" };
    let state = unit.get_god_state();
    if unsafe { !crate::utils::is_null_empty(result.body_model, None) } {
        if result.body_model.to_string().contains("Tik1AT") { return; }
    }
    if unsafe { !crate::utils::is_null_empty(result.dress_model, None) } {
        if result.dress_model.to_string().contains("Tik1AT") { return; }
    }
    if unit.god_unit.is_some_and(|g| g.data.gid.contains("チキ") )&& state == 1 {
        emblem::tiki_engage(result, unit, mode);
        return;
    }
    if kind == 9 || state == 2 || equipped.is_some_and(|i| i.iid.to_string().contains("チキ")) {
        println!("Enemy Tiki Body: {}", result.body_model);
        result.ride_model = "null".into();
        result.ride_dress_model = "null".into();
        result.ride_anim = None;
        result.body_anims.clear();

        if mode == 1 { 
            result.body_anims.add( Il2CppString::new_static(concat_string!("UAS_oBody_A", gen)));
            result.body_anim = Some(concat_string!("UAS_oBody_A", gen).into());
            result.scale_stuff[16] = 2.6;
            super::accessory::change_accessory(result.accessory_list, "null", "l_shld_loc");  // Remove Shield
        }
        else { 
            result.body_anims.add( Il2CppString::new_static(concat_string!("Com0A", gen, "-No1_c000_N")));
            if kind < 10 {
                if kind == 9 { result.body_anims.add( Il2CppString::new_static(concat_string!(super::animation::INF_ACT[kind], gen, "-Ft1_c000_N"))); }
                else { result.body_anims.add( Il2CppString::new_static(concat_string!(super::animation::INF_ACT[kind], gen, "-", super::animation::WEP_PRE[kind], "1_c000_N"))); }
            }
            super::accessory::change_accessory(result.accessory_list, "null", "c_hip_loc");   //Remove Feet Mount
            super::accessory::change_accessory(result.accessory_list, "null", "l_shld_loc");  //Remove Shield
            if gender == 1 { result.body_anims.add( Il2CppString::new_static("Sds0AM-No2_c049_N")); }
            else {  result.body_anims.add( Il2CppString::new_static("Sds0AF-No2_c099_N")); }
        }
    }
}

#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon2(this: &mut CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass85, method_info: OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let side = unsafe { combat_side_convert_from(calc_side, this.is_enemy_attack,  None)};
    adjust_emblem_zone(this.game_status[side as usize]);
    let game_status = &this.game_status[side as usize];

    if let Some(unit) = game_status.unit.as_ref() {
        if unit.person.pid.to_string().contains("チキ_竜化") { return; }
        let distance = crate::utils::clamp_value(this.map_distance, 1, 2);
        let status = this.game_status[side as usize].unit.unwrap().get_god_state();
        if let Some(gunit) = unit.god_unit {
            if gunit.data.gid.to_string().contains("敵チキ") || ( gunit.data.mid.to_string().contains("Tiki") && !gunit.data.gid.contains("チキ") ) {  // Enemy Tiki Emblem
                if status == 2 || status == 3 {   // Is Engage Attacking 
                    let asset_index = super::emblem::get_emblem_attack_index(unit);
                    this.combat_style |= 1 << 22; 
                    let conditions = Array::<&Il2CppString>::new_specific( PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(), 3).unwrap();
                    conditions[0] = "エンゲージ技".into();
                    let result;
                    if asset_index == 13 || asset_index > 20 {
                        conditions[1] = "敵チキ".into();
                        conditions[2] = "AID_チキ竜化".into();
                        result = AssetTableResult::get_from_pid(2, "PID_E001_Boss_竜化",  conditions);
                    }
                    else {
                        conditions[1] = if asset_index == 20 {  "エーデルガルト" } else { EMBLEM_ASSET[asset_index] }.into();
                        conditions[2] = "女装".into();
                        result = AssetTableResult::get_from_god_unit(2, gunit, conditions)
                    }
                    unsafe { combat_character_game_status_import(this.dragonize[side as usize], side, param_3.calc, calc_side, distance, None) };
                    result.sound.voice = None;
                    this.game_status[side as usize].appearance = unsafe { create_from_result(result, distance, None) };
                }
                else if game_status.weapon.as_ref().is_some_and(|w| w.item.kind == 9 || w.item.iid.to_string().contains("チキ")) {
                    if this.combat_style & (1 << 28 ) == 0 { this.combat_style |= 1 << 22; }
                    let conditions = Array::<&Il2CppString>::new_specific( PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(),1).unwrap();
                    let result;
                    if gunit.data.gid.to_string().contains("M002") {
                        conditions[0] = "チキ".into();
                        result = AssetTableResult::get_from_pid(2, "PID_G001_チキ_竜化", conditions);
                        result.sound.voice = Some("Lumiere".into());
                    }
                    else {
                        conditions[0] = "敵チキ".into();
                        result = AssetTableResult::get_from_pid(2, "PID_E001_Boss_竜化",  conditions);
                    }
                    result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft1_c000_N"));
                    result.sound.voice = None;
                    unsafe { combat_character_game_status_import(this.dragonize[side as usize], side, param_3.calc, calc_side, distance, None) };
                    this.game_status[side as usize].appearance = unsafe { create_from_result(result, distance, None) };
                }
                return;
            }
        }
        let jid = unit.job.jid.to_string();
        let conditions = Array::<&Il2CppString>::new_specific( PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(), 1).unwrap();
        conditions[0] = "".into();
        let mut pid: String = "none".to_string();
        if is_emblem_class(unit) {
            if let Some(emblem) = EMBLEM_ASSET.iter().find(|&x1| jid.contains(x1)) {
                pid = format!("PID_闘技場_{}", emblem);
                if *emblem == "チキ" { conditions[0] = "AID_Person_チキ_竜化".into(); }
                else { conditions[0] = format!("AID_Person_{}", emblem).into(); }
            }
        }
        else if let Some(weapon) = &game_status.weapon {
            println!("Item Index: {}", weapon.item.parent.index);
            if weapon.item.iid.to_string().contains("チキ") && status == 0 {
                conditions[0] = "AID_Person_チキ_竜化".into();
                pid = "PID_闘技場_チキ".to_string();
            }
            else if weapon.item.kind == 9 {
                println!("Kind 0");
                if jid == "JID_裏邪竜ノ娘" {
                    conditions[0] = "AID_エル竜化".into();
                    pid = "PID_エル_竜化".to_string();
                }
                else if jid == "JID_裏邪竜ノ子" {
                    conditions[0] = "AID_ラファール竜化".into();
                    pid = "PID_ラファール_竜化".to_string();
                }
                else if is_monster_class(unit) {
                    if let Some(pos) = MONSTERS.iter().position(|mjid| *mjid == jid) {
                        conditions[0] = jid.into();
                        pid = MONSTER_PERSONS[pos].to_string();
                    }
                }
            }
        }
        if pid != "none" {
            if this.combat_style & (1 << 28 ) == 0 {
                this.combat_style |= 1 << 22;
            }
            let result = AssetTableResult::get_from_pid(2, pid, conditions);
            unsafe { combat_character_game_status_import(this.dragonize[side as usize], side, param_3.calc, calc_side, distance, None) };
            this.game_status[side as usize].appearance = unsafe { create_from_result(result, distance, None) };
        }
    }
}


#[skyline::from_offset(0x01bb5be0)]
fn asset_table_result_get_from_pid(mode: i32, pid: &Il2CppString, conditions: &Array<&Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult;

#[unity::from_offset("App", "UnitItem", "GetData")]
fn unititem_get_data(this: &UnitItem, method_info: OptionalMethod) -> Option<&'static ItemData>;


#[skyline::from_offset(0x02b0ed80)]
fn create_from_result(result: &AssetTableResult, map_distance: i32, method_info: OptionalMethod) -> u64;
