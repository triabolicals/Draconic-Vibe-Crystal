use crate::assets::emblem;

use super::*;
use animation::MONSTERS;
use engage::{battle::BattleCalculator, combat::{CharacterGameStatus, CombatRecord}};
use engage::combat::CharacterAppearance;

pub const MONSTER_PERSONS: [&str; 8] = [
    "PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼",
    "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"
];
pub const SCALE: [f32; 8] = [  1.0, 1.0, 1.0, 1.0, 0.40, 1.0, 1.0, 0.40];


pub struct CombatRecordDisplayClass85 {
    pub this: &'static CombatRecord,
    pub calc: &'static BattleCalculator,
}

#[repr(C)]
pub struct CombatRecordDisplayClass87 {
    pub calc: &'static BattleCalculator,
    pub pre_index: i32,
    pub this: &'static CombatRecord,

}
pub fn is_monster_class(unit: &Unit) -> bool {
    let jid = unit.get_job().jid.to_string();
    if unit.person.get_bmap_size() > 1 || unit.person.gender == 0 { false }
    else { MONSTERS.iter().any(|&monster| monster == jid)  }
}
pub fn is_emblem_class(unit: &Unit) -> bool {
    let pid = unit.person.pid.to_string();
    let jid = unit.get_job().jid.to_string();
    let hash = unit.get_job().parent.hash;
    if !crate::randomizer::job::JOB_HASH.iter().any(|&h| h == hash) { return false; }
    if unit.person.get_bmap_size() > 1  { return false; }
    if let Some(pos1) = EMBLEM_ASSET.iter().position(|&x1| format!("JID_紋章士_{}", x1) == jid) {
        if pos1 == 23 || pos1 == 19 { return false; }   // No Lueur / Ephraim
        if let Some(pos2) = EMBLEM_ASSET.iter().position(|&x2| pid.contains(x2)) { return pos1 != pos2;  }
        else { return true; }
    }
    false
}
pub fn has_enemy_tiki(unit: &Unit) -> bool {
    if let Some(god_unit) = unit.god_unit { god_unit.data.gid.to_string().contains("敵チキ") }
    else if let Some(god_unit) = unit.god_link { god_unit.data.gid.to_string().contains("敵チキ") }
    else { unit.person.pid.to_string().contains("チキ") }
}

pub fn is_tiki_engage(unit: &Unit) -> bool {
    unit.status.value & 8388608 != 0 &&
        unit.god_unit.is_some_and(|g_unit| g_unit.data.gid.contains("GID_チキ") || g_unit.data.gid.contains("相手チキ"))
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
                    else if sid.contains("三級長エンゲージ技") { 20 }
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

#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon2(this: &mut CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass85, method_info: OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let side = unsafe { combat_side_convert_from(calc_side, this.is_enemy_attack != 0, None) };
    adjust_emblem_zone(this.game_status[side as usize]);

    let item = this.game_status[side as usize].weapon.as_ref().map(|u_item| u_item.item);
    if this.game_status[side as usize].unit.is_some() {
        if this.game_status[side as usize].unit.unwrap().person.pid.str_contains("チキ_竜化") { return; }
        let distance = crate::utils::clamp_value(this.map_distance, 1, 2);
        let status = this.game_status[side as usize].unit.unwrap().get_god_state();
        if let Some(g_unit) = this.game_status[side as usize]
            .unit.unwrap().god_link.or(this.game_status[side as usize].unit.unwrap().god_unit)
        {
            if g_unit.data.gid.to_string().contains("敵チキ") || (g_unit.data.mid.str_contains("Tiki") && !g_unit.data.gid.str_contains("チキ")) {
                // Enemy Tiki Emblem
                if status >= 2 {   // Is Engage Attacking 
                    let asset_index = super::emblem::get_emblem_attack_index(this.game_status[side as usize].unit.unwrap());
                    this.combat_style |= 1 << 22;
                    let conditions = Array::<&Il2CppString>::new_specific(PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(), 3).unwrap();
                    conditions[0] = "エンゲージ技".into();
                    let result;
                    if asset_index == 13 || asset_index > 20 {
                        conditions[1] = "敵チキ".into();
                        conditions[2] = "AID_チキ竜化".into();
                        result = AssetTableResult::get_from_pid(2, "PID_E001_Boss_竜化", conditions);
                    }
                    else {
                        conditions[1] =
                            if asset_index == 1 { "敵シグルド" } else if asset_index == 17 { "敵カミラ " } else if asset_index == 12 { "エーデルガルト" } else { EMBLEM_ASSET[asset_index] }.into();
                        conditions[2] = "女装".into();
                        result = AssetTableResult::get_from_god_unit(2, g_unit, conditions);
                        emblem::adjust_engage_attack_animation(result, this.game_status[side as usize].unit.as_ref().unwrap(), item, ConditionFlags::Female);
                    }
                    result.sound.voice = None;
                    import_assign_appearance(this.dragonize[side as usize], side, calc_side, distance, param_3.calc, result);
                    this.game_status[side as usize].appearance = CharacterAppearance::create_from_result(result, distance);
                    return;
                }
                else if this.game_status[side as usize].weapon.as_ref().is_some_and(|w| w.item.kind == 9 || w.item.iid.to_string().contains("チキ")) {
                    if this.combat_style & (1 << 28) == 0 { this.combat_style |= 1 << 22; }
                    let conditions = Array::<&Il2CppString>::new_specific(PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(), 1).unwrap();
                    let result;
                    if g_unit.data.gid.to_string().contains("M002") {
                        conditions[0] = "チキ".into();
                        result = AssetTableResult::get_from_pid(2, "PID_G001_チキ_竜化", conditions);
                        result.sound.voice = Some("Lumiere".into());
                    }
                    else {
                        conditions[0] = "敵チキ".into();
                        result = AssetTableResult::get_from_pid(2, "PID_E001_Boss_竜化", conditions);
                        result.sound.voice = None;
                    }
                    result.body_anims.add(Il2CppString::new_static("Ent0AT-Ft1_c000_N"));
                    if this.combat_style & (1 << 28) == 0 { this.combat_style |= 1 << 22; }
                    this.dragonize[side as usize].import(side, param_3.calc, calc_side, distance);
                    this.game_status[side as usize].appearance = CharacterAppearance::create_from_result(result, distance);
                    return;
                }
            }
        }
        if status >= 2 { return; }
        if let Some(result) = add_transformation(this.dragonize[side as usize], side, calc_side, distance, param_3.calc) {
            this.combat_style |= 1 << 22;
            this.game_status[side as usize].appearance = CharacterAppearance::create_from_result(result, distance);
        }
    }
}
#[skyline::hook(offset=0x02928bc0)]
pub fn transformation_chain_atk(this: &mut CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass87, method_info: OptionalMethod) {
    let chain_atk_index = this.chain_attack_count;
    call_original!(this, calc_side, param_3, method_info);
    if chain_atk_index < this.chain_attack_count && chain_atk_index < this.chain_atk.len() as i32  {
        let side = this.chain_atk[chain_atk_index as usize].side;
        if let Some(result) = add_transformation(this.chain_atk[chain_atk_index  as usize], side, calc_side, 1, param_3.calc) {
            this.chain_atk[chain_atk_index  as usize].appearance = CharacterAppearance::create_from_result(result, 1);
        }
    }
}

fn add_transformation(game_status: &mut CharacterGameStatus, side : i32, calc_side: i32, distance: i32, calc: &BattleCalculator) -> Option<&'static AssetTableResult> {
    if let Some(unit) = game_status.unit.as_ref() {
        let jid = unit.job.jid.to_string();
        let conditions = Array::<&Il2CppString>::new_specific(PersonData::get(PIDS[0]).unwrap().get_common_sids().unwrap().get_class(), 3).unwrap();
        let is_tiki_engage = is_tiki_engage(unit);
        conditions[0] = "".into();
        conditions[1] = "".into();
        conditions[2] = "".into();
        let mut pid = String::new();
        if let Some(weapon) = game_status.weapon.as_ref() {
            if weapon.item.iid.str_contains("チキ") && !is_tiki_engage {
                if unit.get_god_state() == 1 { return None; }
                conditions[0] = "AID_Person_チキ_竜化".into();
                pid = "PID_闘技場_チキ".to_string();
            } else if is_monster_class(unit) {
                if let Some(pos) = MONSTERS.iter().position(|monster_jid| *monster_jid == jid) {
                    conditions[0] = jid.into();
                    pid = MONSTER_PERSONS[pos].to_string();
                }
            }
            if weapon.item.flag.value & 0x4000000 != 0 && pid.is_empty() {
                let job_hash = unit.job.parent.hash;
                if let Some(job_data) = SEARCH_LIST.get().unwrap().job.iter().find(|x| x.job_hash == job_hash && x.transform.len() > 0) {
                    let result = AssetTableResult::get_from_pid(2, pid.as_str(), conditions);
                    result.clear();
                    job_data.transform.iter()
                        .flat_map(|&i| AssetTable::try_index_get(i)).filter(|entry| entry.mode == 2)
                        .for_each(|entry| { result.commit_asset_table(entry); });

                    if !result.dress_model.is_null() && result.body_anims.len() > 0 {
                        game_status.import(side, calc, calc_side, distance);
                        return Some(AssetTableResult::get_from_pid(2, pid, conditions));
                    }
                }
            }
            if pid.is_empty() && weapon.item.kind == 9 && weapon.item.flag.value & 0x4000000 != 0 {
                if unit_dress_gender(unit) == 2 {
                    conditions[0] = "AID_エル竜化".into();
                    pid = "PID_エル_竜化".to_string()
                } else {
                    conditions[0] = "AID_ラファール竜化".into();
                    pid = "PID_ラファール_竜化".to_string();
                }
            }
            if !pid.is_empty() {
                game_status.import(side, calc, calc_side, distance);
                return Some(AssetTableResult::get_from_pid(2, pid, conditions));
            }
        }
    }
    None
}
fn import_assign_appearance(game_status: &mut CharacterGameStatus, side: i32, calc_side: i32, distance: i32, calc: &BattleCalculator, result: &AssetTableResult) {
    game_status.import(side, calc, calc_side, distance);
    game_status.appearance = CharacterAppearance::create_from_result(result, distance);
}