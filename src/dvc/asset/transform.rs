use engage::{
    app::{
        BattleSide_Type, IBitField32, IUnitItem,
        UnitItem, BattleCalculator, IBattleCalculatorMethods
    },
    combat::{charactergamestatus::*, combatrecord::*, Side, CombatStyle},
};
use engage::combat::WeaponStyle;
use outfit_core::anim::SpecialAttackType;
use unity::Cast;
use crate::dvc::asset::engage_attack::EngageAttackType;
use super::*;

pub fn is_tiki_dragonstone(item_data: ItemData) -> bool {
    if item_data.is_null() { false }
    else { item_data.get_kind().value == 9 && item_data.get_iid().to_rust_string().contains("チキ") }
}
pub fn is_tiki_engage(unit: Unit) -> bool {
    if !unit.is_engaging_2() || unit.get_god_unit().is_null() { false }
    else { unit.get_god_unit().m_data().get_gid().to_rust_string().contains("チキ") }
}
pub fn is_dragonstone(equipped: ItemData) -> bool {
    if equipped.is_null() { false } else {
        equipped.is_dragon() || is_tiki_dragonstone(equipped) || (equipped.get_kind().value == 9 && !equipped.is_bullet() && equipped.get_flag().m_value() & 0x40000 == 0)
    }
}
fn is_tiki_dragon_weapon(item: ItemData) -> bool {
    if item.is_null() { false }
    else { item.get_iid().to_rust_string().contains("チキ") && item.get_kind().value == 9 }
}
fn set_transform_appearance(this: CombatRecord, unit: Unit, calculator: BattleCalculator, side: i32, calc_side: BattleSide_Type, distance: i32) {
    let array = Array::<Il2CppString>::from_slice(&[Il2CppString::new("コンバット"), Il2CppString::new("竜石")]).unwrap();
    let drag = this.get_game_status_dragonize().get(side as usize);
    drag.import_4(side, calculator, calc_side, distance);
    let result = AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), unit, array);
    let female = get_outfit_data().get_dress_gender(result.get_dress_model()).value == 2;
    let transform_anim = AnimData::get_transforming_anim(unit.is_engaging_2(), female);
    result.set_body_anim(transform_anim);
    let body_anims = result.get_body_anims();
    body_anims.clear();
    AnimData::remove(result, true, true);
    let anim = format!("{}0A{}-No1_c000_N", if unit.is_engaging_2() { "Enb"} else { "Com" }, if female { "F" } else { "M" });
    body_anims.add(anim.into());
    body_anims.add(transform_anim.into());
    print_asset_table_result(result, 2);
    drag.set_appearance(CharacterAppearance::create_from_result(result, distance));
}
fn bullet_attack_result(unit: Unit, item: ItemData) -> AssetTable_Result {
    let array = Array::<Il2CppString>::from_slice(&[Il2CppString::new("コンバット"), Il2CppString::new("弾丸")]).unwrap();
    let result = AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), unit, array);
    if item.is_bullet() { AssetTable::s_condition_flags().add_3(item); }
    result.commit(AssetTable_Modes::combat());
    result.replace(AssetTable_Modes::combat());
    AnimData::remove(result, true, true);
    let body_anims = result.get_body_anims();
    body_anims.clear();
    let female = get_outfit_data().get_dress_gender(result.get_dress_model()).value == 2;
    result.get_body_anims().add(AnimData::bullet_anim(false, female).into());
    if unit.is_engaging_2() {
        result.get_body_anims().add(AnimData::bullet_anim(true, female).into());
    }
    result
}
pub fn adjust_fx_fogato_for_engage_atk(this: CombatRecord, calc_side: BattleSide_Type){
    let calculator = this.get_calculator();
    let side = Side::convert_from(calc_side, this.get_is_enemy_attack());
    let distance = utils::clamp_value(this.get_map_distance(), 1, 2);
    let game_status = this.get_game_status();
    let gs = game_status.get(side as usize);
    if gs.is_null() { return; }
    let unit = gs.get_unit();
    if unit.is_null() { return; }
    let person = unit.get_person();
    let hash = person.hash();
    let fx_fogato = hash == -1722328881 || hash == 625928415;
    if !fx_fogato { return; }
    let mut style = this.get_combat_style().value;
    if !gs.appearance().is_null(){
        if fx_fogato {
            let engage_attack = EngageAttackType::from_unit(unit);
            if !is_dragonstone(gs.get_weapon().m_item()) && engage_attack != EngageAttackType::TikiDragon{
                style &= !(1 << 22);
                if engage_attack != EngageAttackType::None {
                    let result = AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), unit, calculator.get_asset_conditions(calc_side));
                    let mut asset_conditions = AssetConditions::new(unit, 2, gs.get_weapon().m_item());
                    engage_attack.setup_engage_attack_result_unit(result, unit, engage_attack == EngageAttackType::TikiDragon, &mut asset_conditions);
                    gs.set_appearance(CharacterAppearance::create_from_result(result, distance));
                }
                else { gs.import_4(side, calculator, calc_side, distance); }
                if let Some(drag) = to_opt(this.get_game_status_dragonize().get(side as usize)) { drag.clear(); }
                this.set_combat_style(CombatStyle{value: style});
            }
        }
    }
}
fn remove_acc_for_transform(result: AssetTable_Result) {
    ["c_head_loc", "c_head2_loc", "c_spine1_jnt", "c_spine2_jnt"].into_iter().for_each(|locator|{
        result.commit_8(new_accessory(locator, "null"));
    });
    result.replace(AssetTable_Modes::combat());
}
pub fn do_transformation(this: CombatRecord, calc_side: BattleSide_Type) {
    let calculator = this.get_calculator();
    let side = Side::convert_from(calc_side, this.get_is_enemy_attack());
    let distance = utils::clamp_value(this.get_map_distance(), 1, 2);
    let game_status = this.get_game_status();
    let gs = game_status.get(side as usize);
    if gs.is_null() { return; }
    let unit = gs.get_unit();
    if unit.is_null() { return; }
    let person = unit.get_person();
    adjust_fx_fogato_for_engage_atk(this, calc_side);
    let mut style = this.get_combat_style().value;
    if !gs.appearance().is_null(){
        if !gs.appearance().assets().is_null() { if gs.appearance().get_race().value != 1 { return; } }
    }
    if person.get_pid().to_rust_string().contains("チキ") { return; }
    let state = AssetTable_ConditionFlags::get_state(unit);
    let dragon = state.value & CombatStyle::dragon_change().value != 0;
    let god_unit = unit.get_god_unit();
    match state.value {
        2 => { 
            if god_unit.is_null() { return; }
        }
        0|1 => {
            let item = gs.get_weapon();
            if item.is_null() { return; }
            let item = item.m_item();
            match SpecialAttackType::determine_from_unit(unit, item) {
                SpecialAttackType::Bullet => {
                    if let Some(drag) = to_opt(this.get_game_status_dragonize().get(side as usize)) { drag.clear(); }
                    gs.set_appearance(CharacterAppearance::create_from_result(bullet_attack_result(unit, item), distance));
                    gs.appearance().set_weapon_style(WeaponStyle::magic());
                }
                SpecialAttackType::Transform => {
                    if is_tiki_dragon_weapon(item) || unit.get_job().get_jid().contains("チキ"){
                        style |= 1 << 22;
                        set_transform_appearance(this, unit, calculator, side, calc_side, distance);
                        let result = AssetTable_Result::get_for_talk_2(unit);
                        let colors = [result.get_mask_color100(), result.get_mask_color075(), result.get_mask_color050(), result.get_mask_color025()];
                        let sound = il2str(result.get_sound().voice_id);
                        result.clear();
                        let conditions = AssetTable::s_condition_flags();
                        if !god_unit.is_null() {
                            let data = god_unit.m_data();
                            let gid = data.get_gid().to_rust_string();
                            if data.get_force_type().value == 1 && (gid.contains("M0") || gid.contains("E00")) { conditions.add_2("PID_E001_Boss_竜化"); }
                            else {
                                conditions.add_2("EID_チキ");
                                conditions.add_2("AID_Person_チキ_竜化");
                            }
                        }
                        else {
                            conditions.add_2("EID_チキ");
                            conditions.add_2("AID_Person_チキ_竜化");
                        }
                        conditions.add_3(item);
                        result.commit(AssetTable_Modes::combat());
                        result.replace(AssetTable_Modes::combat());
                        if let Some(sounds) = sound {
                            let mut s = result.get_sound();
                            s.voice_id = sounds.into();
                            result.set_sound(s);
                        }
                        if !unit.is_engaging_2() && unit.get_force_type().value == 0 {
                            result.set_mask_color100(colors[0]);
                            result.set_mask_color075(colors[1]);
                            result.set_mask_color050(colors[2]);
                            result.set_mask_color025(colors[3]);
                        }
                        gs.set_appearance(CharacterAppearance::create_from_result(result, distance));
                    }
                    else if let Some(transform) = get_transformation2(unit, gs.get_weapon()){
                        transform.set_pid(unit.get_pid().to_rust_string());
                        remove_acc_for_transform(transform);
                        if let Some(dress) = il2str(transform.get_dress_model()){
                            if dress.contains("uBody_Sdk1AT") || dress.contains("uBody_Sds1AT") { transform.set_scale_all(0.07); }
                            else if dress.contains("uBody_Sds0AT") { transform.set_scale_all(0.4); }
                            else if dress.contains("AT") && !dress.contains("Tik") { transform.set_scale_all(0.7); }
                        }
                        style |= 1 << 22;
                        set_transform_appearance(this, unit, calculator, side, calc_side, distance);
                        gs.set_appearance(CharacterAppearance::create_from_result(transform, distance));
                    }
                }
                _ => {
                    if let Some(drag) = to_opt(this.get_game_status_dragonize().get(side as usize)) {
                        drag.clear();
                    }
                }
            }
        }
        _ => {}
    }
    if dragon { style |= 1 << 22; }
    this.set_combat_style(CombatStyle{value: style});
}

pub fn get_transformation2(unit: Unit, weapon: UnitItem) -> Option<AssetTable_Result> {
    let item = weapon.m_item();
    let kind = SpecialAttackType::determine_from_unit(unit, weapon.m_item());
    match kind {
        SpecialAttackType::Transform => {
            get_outfit_data().get_combat_transformation(unit, item)
                .or_else(||
                    Some(
                        AssetTable_Result::get_from_pid(
                            AssetTable_Modes::combat(),if unit.get_gender().value == 2 { "PID_エル_竜化" } else { "PID_ラファール_竜化"},
                            CharacterAppearance::conditions()
                        )
                    )
            )
        }
        SpecialAttackType::Bullet => { if item.is_bullet() { None } else { Some(bullet_attack_result(unit, item)) } }
        SpecialAttackType::NormalAttack => { None }
    }
}