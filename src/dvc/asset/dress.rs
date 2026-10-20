use bitflags::Flags;
use unity::system::string::IIl2CppStringMethods;
use engage::app::UnitUtil;
use crate::dvc::asset::{engage_attack::EngageAttackType, transform::{is_dragonstone}};
use super::*;
fn tiki_engage(result: AssetTable_Result, condition_unit: Unit, mode: i32, equipped: ItemData, condition: &mut AssetConditions) -> bool {
    if transform::is_tiki_engage(condition_unit) {
        AssetFlags::remove_unit_accessories(condition_unit);
        let engage_atk = EngageAttackType::from_unit(condition_unit);
        if DVCFlags::EngageWeapons.get_value() || condition.flags.contains(AssetFlags::Vision) ||
            ( condition.flags.contains(AssetFlags::EngageAttack) && engage_atk != EngageAttackType::TikiDragon)
        {
            condition.remove_god_eid_conditions();
            AssetFlags::set_condition_key("EID_チキ", false);
            AssetFlags::set_condition_key("チキ", false);
            AssetFlags::set_condition_key("PID_G001_チキ", true);
            AssetFlags::set_condition_key("AID_Person_チキ", true);
            result.commit_2(AssetTable_Modes{value: mode}, condition_unit.get_person(), condition_unit.get_job(), equipped);
            if DVCFlags::GodNames.get_value() {
                if let Some(appear) = AppearanceManager::get_appearance_emblem_2(13, false) {
                    let db = get_outfit_data();
                    appear.apply_appearance(result, mode, false, None, &db.hashes, true);
                }
            }
            if condition.flags.contains(AssetFlags::Vision) {
                if condition.mode == 1 { result.set_body_anim("UAS_Enb0AF"); }
                else {
                    let body_anims = result.get_body_anims();
                    body_anims.clear();
                    body_anims.add("Com0AF-No1_c000_N".into());
                    body_anims.add("Com0AF-Sw1_c000_N".into());
                    body_anims.add("Enb0AF-Sw1_c000_N".into());
                }
            }
            else if engage_atk != EngageAttackType::None { engage_atk.setup_engage_attack_result_unit(result, condition_unit, true, condition); }
            else if is_dragonstone(equipped) && condition.flags.contains(AssetFlags::EngageAttack) && mode == 2 {
                result.get_body_anims().add("End0AF-No2_c099_N".into());
            }
        }
        else {
            let mode1 = AssetTable_Modes{value: condition.mode};
            result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped);
            result.commit(mode1);
            result.replace(mode1);
            condition.flags.set(AssetFlags::EngageTiki, true);
        }
        true
    }
    else { false }
}

fn mode_1_transformation(result: AssetTable_Result, condition_unit: Unit, equipped: ItemData, conditions: Array<Il2CppString>, flags: &mut AssetConditions) {
    if condition_unit.get_job().get_jid().contains("_チキ") || (condition_unit.is_engaging_2() && flags.engaged.as_ref().is_some_and(|v| v.contains("チキ"))){
        result.setup_4(AssetTable_Modes::onmap(), PersonData::get("PID_G001_チキ_竜化".into()), conditions);
    }
    else {
        let job_hash = condition_unit.get_job().hash();
        let db = get_outfit_data();
        if let Some(data) = db.dress.transform.iter().find(|v| v.hash == job_hash && !v.is_transform) {
            data.set_result(1, condition_unit, equipped, result);
        }
        else if let Some(data) = db.dress.transform.iter().find(|v| v.hash == job_hash && v.is_transform) {
            data.set_result(1, condition_unit, equipped, result); }
        else if condition_unit.get_dress_gender().value == 1 {
            result.setup_5(AssetTable_Modes::onmap(), PersonData::get("PID_ラファール_竜化".into()), condition_unit.get_job(), ItemData::null(), conditions);
        }
        else {
            result.setup_5(AssetTable_Modes::onmap(), PersonData::get("PID_エル_竜化".into()), condition_unit.get_job(), ItemData::null(), conditions);
        }
        #[cfg(feature = "debug")] {
            println!("Transform Mode 1 for {}", condition_unit.get_name());
            print_asset_table_result(result, 1);
        }
        if let Some(body) = il2str(result.get_body_model()) {
            if body.contains("Mrp") { result.set_map_scale_all(1.0); }
            else if body.contains("1AT") && !body.contains("Tik") {
                result.set_map_scale_all(0.25);
                result.set_map_scale_wing(0.25);
                result.set_map_scale_head(0.25);
            }
            else if body.contains("Fyd") {
                result.set_map_scale_all(1.0);
                result.set_map_scale_wing(0.25);
            }
        }
    }
}

pub fn commit_for_unit_dress(
    result: AssetTable_Result,
    mode: i32,
    unit: Unit,
    equipped: ItemData,
    con: Array<Il2CppString>,
) -> AssetConditions
{
    let mode1 = AssetTable_Modes{value: mode};
    let condition_unit = if !UnitUtil::get_vision_owner(unit).is_null() { UnitUtil::get_vision_owner(unit) } else { unit };
    let mut conditionss = AssetConditions::new(condition_unit, mode, equipped);
    if conditionss.flags.contains(AssetFlags::ThreeStar) || conditionss.flags.contains(AssetFlags::FiveStar) {
        if !to_opt(condition_unit.get_engage_attack()).is_some_and(|v| v.get_sid().contains("ヴェロニカ")) && !condition_unit.get_pid().contains("ヴェロニカ") {
            conditionss.flags.set(AssetFlags::ThreeStar, false);
            conditionss.flags.set(AssetFlags::FiveStar, false);
        }
    }
    if let Some(emblem) = PersonPool::M022_EMBLEMS.iter().position(|v| *v == unit.get_person().hash()) {
        if let Some(replacement) = EmblemPool::get_replacement_god_3(emblem as i32 + 1, false) {
            emblem::asset_table_result_god_setup(result, mode, replacement, true, con, None);
            result.set_pid(replacement.get_gid());
            conditionss.flags.set(AssetFlags::Monster, true);
            return conditionss;
        }
    }
    else if let Some(god) = EmblemPool::get_paralogue_custom_emblem(unit.get_person()) {
        conditionss.flags.set_gender(unit.get_dress_gender());
        emblem::asset_table_result_god_setup(result, mode, god, false, con, None);
        if !equipped.is_null() {
            AssetFlags::set_condition_key(equipped.get_iid(), true);
            if !equipped.get_aid().is_null() { AssetFlags::set_condition_key(equipped.get_aid(), true); }
        }
        result.commit(mode1);
        result.replace(mode1);
        if mode == 1{
            let body_mode = result.get_body_model().get_hash_code();
            let db = get_outfit_data();
            if !db.hashes.o_body.contains_key(&body_mode){
                db.random_body(result, mode, Random_2::new_2(god.hash() as u32), god.get_female() != 0);
            }
        }
        conditionss.flags.set(AssetFlags::Monster, true);
        return conditionss;
    }
    if conditionss.flags.contains(AssetFlags::Monster){
        if conditionss.flags.contains(AssetFlags::Vision) {
            conditionss.flags.set_gender(Gender::female());
            result.clear();
            result.setup_5(mode1, PersonData::get("PID_S004_リン".into()), JobData::get("JID_紋章士_リン".into()), equipped, con);
        }
        else {
            result.commit_2(mode1, unit.get_person(), unit.get_job(), equipped);
            result.replace(mode1);
        }
        return conditionss;
    }
    let condition_unit =
        if conditionss.flags.contains(AssetFlags::Vision) && !UnitUtil::get_vision_owner(unit).is_null(){ UnitUtil::get_vision_owner(unit) }
        else { unit  };
    AppearanceManager::swap_asset_condition_enemy_player_unit(condition_unit, &mut conditionss);
    if condition_unit.get_job().hash() == LUEUR_CLASS[2] { AssetFlags::set_condition_key(condition_unit.get_job().get_jid(), false); }
    if tiki_engage(result, condition_unit, mode, equipped, &mut conditionss) {
        conditionss.flags.set(AssetFlags::EngageTiki, true);
        return conditionss;
    }
    if is_dragonstone(equipped){
        if conditionss.flags.contains(AssetFlags::EngageAttack) {
            conditionss.flags.set(AssetFlags::CombatTranforming, false);
        }
        else if mode == 2 { conditionss.flags.set(AssetFlags::CombatTranforming, true); }
    }
    if conditionss.flags.contains(AssetFlags::MapTransform) && mode == 1 {
        mode_1_transformation(result, unit, equipped, con, &mut conditionss);
        return conditionss;
    }
    let db = get_outfit_data();
    let engaged = condition_unit.is_engaging_2();
    if let Some(data) = UnitAssetMenuData::get_unit_data(condition_unit) {
        let profile_flag = data.get_active_flag(engaged);
        let god_unit = condition_unit.get_god_unit();
        if  !condition_unit.is_null() && engaged {
            let god = god_unit.m_data();
            if profile_flag & 256 != 0 { conditionss.flags.set_condition_flag(AssetFlags::Engaged, false); }
            if profile_flag & 6 == 2 { conditionss.remove_god_eid_conditions(); }
            else if profile_flag & 6 == 4 {
                let gid = god.get_gid().to_string();
                conditionss.remove_god_eid_conditions();
                AssetFlags::set_person_conditions(condition_unit.get_person(), false);
                AssetFlags::set_condition_key(gid, true);
                AssetFlags::set_condition_key(god.get_mid(), true);
                AssetFlags::set_condition_key(god.get_asset_id(), true);
                conditionss.flags.set_condition_flag(AssetFlags::Engaged, false);
                let gender = if god.get_female() == 1 { Gender::female() } else { Gender::male() };
                conditionss.flags.set_gender(gender);
                result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped);
                db.correct_anims(result, unit, profile_flag, &conditionss);
                return conditionss;
            }
        }
        if UnitAssetMenuData::get().is_preview {
            let shop_combat = UnitAssetMenuData::get().is_shop_combat;
            AssetFlags::set_condition_key("拠点", !shop_combat);
            AssetFlags::set_condition_key("私服", !shop_combat);
            if shop_combat { AssetFlags::remove_unit_accessories(condition_unit); }
        }
        result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped);
        if profile_flag & 8 == 0 {
            let job_hash = GameVariableManager::get_number(GrowthData::get_starting_class_key(unit.get_person()));
            if job_hash == condition_unit.get_job().hash() && conditionss.random_dress.is_off() && !engaged && !conditionss.flags.contains(AssetFlags::CausalClothes){
                if let Some(dress) = db.dress.get_personal_dress(condition_unit) {
                    dress.apply(result, mode, false, db.anims.get_mount_type(unit, unit.get_dress_gender()), &db.hashes);
                }
            }
        }
        else { AppearanceManager::get_person_appearance(unit, result, mode); }
        if engaged && god_unit.is_hero() { AppearanceManager::apply_hero_mask_to_hair(result, condition_unit.is_hero(), mode); }
        db.adjust_dress(result, condition_unit, &conditionss);
        UnitAssetMenuData::set_assets(result, condition_unit, &conditionss);
        conditionss.profile_flag = profile_flag;
    }
    else {
        if condition_unit.get_person().hash() == PersonPool::M002_LUMERA && condition_unit.is_engaging_2() {
            if let Some(sigurd_replacement) = EmblemPool::get_replacement_god_3(1, false){
                let conditions = AssetTable::s_condition_flags();
                conditions.clear();
                let eid = sigurd_replacement.get_gid().to_rust_string().replace("GID_", "EID_");
                conditions.add_7(condition_unit.get_person(), condition_unit.get_job(), Force_Type::enemy());
                conditions.add_3(equipped);
                conditions.add_8(condition_unit.m_mask_skill());
                conditions.add(con);
                if !eid.contains("チキ") {
                    conditions.add_9(AssetTable_States::engaging(), sigurd_replacement, false);
                    conditionss.engaged = Some(eid);
                    result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped);
                    result.replace(mode1);
                }
                else {
                    result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped);
                    conditionss.engaged = None;
                    if mode == 1 { result.set_body_model("oBody_Tik0AF_c560"); }
                    else { result.set_dress_model("uBody_Tik0AF_c560"); }
                }
            }
            random_aoc(result, condition_unit);
            return conditionss;
        }
        else { result.commit_2(mode1, condition_unit.get_person(), condition_unit.get_job(), equipped); }
        if il2str(result.get_dress_model()).is_some_and(|v| v.contains("AT_c")) || il2str(result.get_body_model()).is_some_and(|v| v.contains("AT_c")) { return conditionss; }
        let asset_force = unit.get_person().get_asset_force().value;
        if asset_force != 0 {
            let generic_mode = DVCVariable::GenericAppearance.get_value();
            if conditionss.flags.is_generic() && generic_mode != 0 && mode == 2 {   // Generic
                let rng = Random_2::new_2(unit.get_grow_seed());
                if generic_mode & 1 != 0 { db.assign_random_head_hair(result, rng); }
                if generic_mode & 2 != 0 { change_color_by_rng(result, rng); }
            }
            else { AppearanceManager::get_person_appearance(condition_unit, result, mode); }
        }
        db.adjust_dress(result, condition_unit, &conditionss);
        if PersonPool::is_nil(condition_unit.get_person()) {
            if let Some(replacement) = PersonPool::get_replacement(37, false, true){
                if let Some(dress) = db.dress.get_personal_dress_by_person(replacement, unit.get_dress_gender().value == 2) {
                    dress.apply(result, mode, false, None, &db.hashes);
                }
            }
        }
        if condition_unit.is_engaging_2(){ // Engaged
            if let Some(asset) = conditionss.engaged.as_ref()
                .and_then(|gid| to_opt(GodData::get(gid.as_str().into())))
                .map(|god| god.get_asset_id())
                .and_then(|asset| db.dress.get_engaged_dress(asset))
            {
                asset.apply(result, mode, unit.get_dress_gender());
            }
            if to_opt(condition_unit.get_god_unit()).is_some_and(|h| h.is_hero()) {
                AppearanceManager::apply_hero_mask_to_hair(result, condition_unit.is_hero(), mode);
            }
        }
        conditionss.profile_flag = 0;
    }
    if conditionss.flags.contains(AssetFlags::CombatTranforming) { AnimData::remove(result, true, true); }
    if DVCFlags::RandomUnitInfo.get_value() {
        let gender = db.get_dress_gender(result.get_dress_model());
        let hash = if conditionss.flags.is_generic() { condition_unit.m_ident() } else { condition_unit.get_person().hash() };
        if gender.value != 0 {
            let rng = DVCVariables::create_hash_rng(hash, condition_unit.is_engaging_2() as i32 + 1);
            if let Some(info) = db.hashes.random_aoc(gender.value == 2, rng) { result.set_m_info_anim(info.as_str().into()); }
        }
    }
    AppearanceManager::evil_enemy_asset_addition(result, condition_unit.get_person(), mode);
    if DVCVariable::BodyScaling.get_value() != 0 && mode == 2 { random_body_scale(result, Some(unit.get_grow_seed() as i32), false); }
    conditionss
}
pub fn change_color_by_rng(result: AssetTable_Result, rng: Random_2) {
    let index: [usize; 6] = [0, 1, 4, 5, 6, 7];
    for x in index {
        let value2 = rng.get_value();
        set_color_by_i32(result, x, value2);
    }
}
fn random_aoc(result: AssetTable_Result, unit: Unit) {
    let db = get_outfit_data();
    if DVCFlags::RandomUnitInfo.get_value() {
        let rng = Random_2::new_2(unit.get_grow_seed());
        let gender = db.get_dress_gender(result.get_dress_model()).value;
        if gender == 1 || gender == 2 {
            let set = if gender == 2 { &db.hashes.info_f } else { &db.hashes.info_m };
            if let Some(aoc) = set.get_random_element(rng).and_then(|i| db.hashes.aoc.get(i)) { result.set_m_info_anim(aoc.as_str().into()); }
        }
    }
}