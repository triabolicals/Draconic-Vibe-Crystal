use super::*;
use animation::MONSTERS;
use engage::{battle::BattleCalculator, combat::*};
use engage::battle::BattleSideType;
use engage::unit::{Unit, UnitItem};

pub const MONSTER_PERSONS: [&str; 8] = [
    "PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼",
    "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"
];
pub const MONSTER_ACT: [&str; 8] = [
    "Fyd2DM-Mg1_c715_M", "Fyd2DM-Mg1_c707_M", "Wlf2CM-Ft1_c715_N", "Wlf2CM-Ft1_c707_N", "Sds1AT-Mg1_c049_M", 
    "Mrp0AT-Ft1_c000_N", "Mrp0AT-Ft1_c706_N", "Sds1AT-Mg1_c049_M"
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
pub fn has_enemy_tiki(unit: &Unit) -> bool {
    if let Some(god_unit) = unit.god_link.or(unit.god_unit) { god_unit.data.gid.str_contains("敵チキ") }
    else { unit.person.pid.str_contains("チキ") }
}

pub fn is_tiki_engage(unit: &Unit) -> bool {
    unit.status.value & 8388608 != 0 && unit.god_unit.is_some_and(|g_unit| g_unit.data.gid.contains("チキ"))
}
pub fn has_fake_tiki(unit: &Unit) -> bool {
    unit.god_unit.is_some_and(|x| x.data.force_type == 1 && x.data.main_data.mid.str_contains("Tiki") && x.data.main_data.flag.value & 12 != 0 && x.data.gid.str_contains("M0"))
}
pub fn do_transformation(this: &mut CombatRecord, calc_side: BattleSideType) {
    let calculator = this.get_calculator();
    let side = CombatSide::convert_from(calc_side, this.is_enemy_attack != 0) as usize;
    if let Some(unit) = this.game_status[side].get_unit(){
        if unit.person.get_job().is_some_and(|v|{ let job = v.jid.to_string(); unit.person.gender == 0 || MONSTERS.iter().any(|&monster| monster == job) }) { return; }
        if unit.person.pid.str_contains("チキ") { return; }
        let distance = crate::utils::clamp_value(this.map_distance, 1, 2);
        let status = AssetTableConditionFlags::get_state(unit);
        if let Some(g_unit) = unit.god_link.or(unit.god_unit){
            // if using Emblem Tiki and engaged + must use engaged weapons
            if (g_unit.data.parent.hash == 1978213856 || g_unit.data.parent.hash == -252721213) && unit.is_engaging() && g_unit.data.flag.value & 16 != 0 { return; }
            if (g_unit.data.gid.to_string().contains("M0") || g_unit.data.force_type == 1) && g_unit.data.mid.str_contains("Tiki"){
                if status >= AssetTableStates::EngageAttack {   // Is Engage Attacking
                    let asset_index = emblem::get_emblem_attack_index(unit);
                    this.combat_style |= 1 << 22;
                    let conditions = Array::new_from_element_class(Il2CppString::class(), 3).unwrap();
                    conditions[0] = "エンゲージ技".into();
                    let result;
                    if asset_index == 13 || asset_index > 20 {
                        conditions[1] = "敵チキ".into();
                        conditions[2] = "AID_チキ竜化".into();
                        result = AssetTableResult::get_from_pid(2, "PID_E001_Boss_竜化", conditions);
                    }
                    else {
                        conditions[1] =
                            if asset_index == 1 { "敵シグルド" }
                            else if asset_index == 17 { "敵カミラ" }
                            else if asset_index == 12 { "エーデルガルト" } else { EMBLEM_ASSET[asset_index] }.into();

                        conditions[2] = "女装".into();
                        result = AssetTableResult::get_from_god_unit(2, g_unit, conditions);
                    }
                    result.sound.voice = None;
                    import_assign_appearance(this.dragonize[side], side as i32, calc_side, distance, calculator, result);
                    this.game_status[side].appearance = CharacterAppearance::create_from_result(result, distance);
                    return;
                }
                else if this.game_status[side].weapon.as_ref().is_some_and(|w| w.item.kind == 9 || w.item.iid.to_string().contains("チキ")) {
                    if this.combat_style & (1 << 28) == 0 { this.combat_style |= 1 << 22; }
                    let conditions = Array::new_from_element_class(Il2CppString::class(), 3).unwrap();
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
                    this.dragonize[side].import(side as i32, calculator, calc_side, distance);
                    this.game_status[side].appearance = CharacterAppearance::create_from_result(result, distance);
                    return;
                }
            }
        }
        if status >= AssetTableStates::EngageAttack { return; }
        if this.game_status[side].weapon.as_ref().is_some_and(|w| w.item.kind == 9 && (w.item.flag.value & 0x8000000 == 0) && w.item.iid.to_string().contains("チキ"))
        {
            let conditions = Array::new_from_element_class(Il2CppString::class(), 1).unwrap();
            conditions[0] = "EID_チキ".into();
            let result = AssetTableResult::get_from_pid(2, "PID_G001_チキ_竜化", conditions);
            this.combat_style |= 1 << 22;
            this.dragonize[side].import(side as i32, calculator, calc_side, distance);
            this.game_status[side].appearance = CharacterAppearance::create_from_result(result, distance);
            return; 
        }
        if let Some(monster_data) = get_outfit_data().dress.transform.iter().find(|x|{
            println!("{} = {}: {}", unit.job.parent.hash, x.hash, x.hash == unit.job.parent.hash);
            !x.is_transform && unit.job.parent.hash == x.hash
        }){
            this.combat_style |= 1 << 22;
            this.dragonize[side].import(side as i32, calculator, calc_side, distance);
            this.game_status[side].appearance = CharacterAppearance::create_from_result(monster_data.get_result(2, unit), distance);
        }
        else {
            let can_dragon_stone = unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && unit.job.get_max_weapon_level(9) > 1;
            if this.game_status[side].weapon.is_some_and(|i| is_dragonstone(Some(i.item)) || (i.item.kind == 9 && can_dragon_stone)){
                this.combat_style |= 1 << 22;
                this.dragonize[side].import(side as i32, calculator, calc_side, distance);
                this.game_status[side].appearance = CharacterAppearance::create_from_result(get_transform_result(unit), distance);
                return;
            }
        }
    }
}
pub fn get_transformation2(unit: &Unit, weapon: &UnitItem) -> Option<&'static mut AssetTableResult> {
    let job = unit.job.parent.hash;
    let can_dragon_stone = unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && unit.job.get_max_weapon_level(9) > 1;
    if is_dragonstone(Some(weapon.item)) || (weapon.item.kind == 9 &&  can_dragon_stone){ Some(get_transform_result(unit)) }
    else {
        let db = get_outfit_data();
        db.dress.transform.iter().find(|x| !x.is_transform && job == x.hash).map(|x| x.get_result(2, unit))
    }
    /*
    let mut dark_tiki = false;
    let jid = unit.job.jid.to_string();
    let conditions = Array::new_from_element_class(Il2CppString::class(), 3).unwrap();
    let is_tiki_engage = is_tiki_engage(unit);
    conditions[0] = "".into();
    conditions[1] = "".into();
    conditions[2] = "".into();
    let mut pid = String::new();
    let db = get_outfit_data();
    if weapon.item.kind == 9 && weapon.item.iid.str_contains("チキ") {
        if has_fake_tiki(unit) {
            pid = "PID_E001_Boss_竜化".into();
            conditions[0] = "AID_チキ竜化".into();
            dark_tiki = true;
        } else {
            conditions[0] = "AID_Person_チキ_竜化".into();
            pid = "PID_闘技場_チキ".to_string();
        }
    } 
    else if db.is_monster_class(unit) {
        conditions[0] = jid.as_str().into();
        pid = MONSTER_PERSONS[0].to_string();
        let result = AssetTableResult::get_from_pid(2, pid.as_str(), conditions);
        if db.apply_monster_asset(result, unit, 2) { return Some(result); }
    }
    if db.is_transform_class(unit) && weapon.item.kind == 9 && weapon.item.flag.value & 134217728 == 0 {
        conditions[0] = jid.as_str().into();
        pid = MONSTER_PERSONS[0].to_string();
        let result = AssetTableResult::get_from_pid(2, pid.as_str(), conditions);
        if db.apply_monster_asset(result, unit, 2) { return Some(result); }
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
        let result = AssetTableResult::get_from_pid(2, pid, conditions);
        if dark_tiki { result.sound.voice = None; }
        Some(result)
    } else { None }
    */
}

fn get_transform_result(unit: &Unit) -> &'static mut AssetTableResult {
    let db = get_outfit_data();
    let job = unit.job.parent.hash;
    let result =
    db.dress.transform.iter().find(|x| job == x.hash).map(|data| data.get_result(2, unit))
        .unwrap_or(
            AssetTableResult::get_from_pid(2,if unit.person.get_gender() == 2 { "PID_エル_竜化" } else { "PID_ラファール_竜化"}, CharacterAppearance::get_constions(None))
        );
    outfit_core::print_asset_table_result(result, 2);
    result
}
pub fn is_dragonstone(equipped: Option<&ItemData>) -> bool {
    equipped.is_some_and(|i| i.flag.value & 0x4000000 != 0 || (i.flag.value & 0x8000000 == 0 && i.kind == 9 ))
}

fn import_assign_appearance(game_status: &mut CharacterGameStatus, side: i32, calc_side: BattleSideType, distance: i32, calc: &BattleCalculator, result: &AssetTableResult) {
    game_status.import(side, calc, calc_side, distance);
    game_status.appearance = CharacterAppearance::create_from_result(result, distance);
}