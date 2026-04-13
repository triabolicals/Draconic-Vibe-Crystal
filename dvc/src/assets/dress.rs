use bitflags::Flags;
use engage::gameuserdata::GameUserData;
use outfit_core::{get_outfit_data, AssetConditions, AssetFlags, PersonalDressData, UnitAssetMenuData};
use outfit_core::CharacterAssetMode::UnitInfo;
use accessory::*;
use transform::{has_enemy_tiki};
use crate::assets::transform::is_dragonstone;
use crate::config::DVCFlags;
use crate::config::menu::DVCMenu::Asset;
use crate::randomizer::data::RandomizedGameData;
use crate::randomizer::names::get_emblem_person;
use crate::randomizer::person::is_playable_person;
use crate::utils::create_rng;
use super::*;

fn is_preview_unit(unit: &Unit) -> bool { unit.force.is_some_and(|x| (1 << x.force_type) & 25 != 0) && unit.status.value & 35184372088832 == 0 }

fn tiki_engage(result: &mut AssetTableResult, condition_unit: &Unit, mode: i32, equipped: Option<&ItemData>, condition: &mut AssetConditions) -> bool {
    if transform::is_tiki_engage(condition_unit) {
        AssetFlags::remove_unit_accessories(condition_unit);
        if DVCFlags::EngageWeapons.get_value() || condition.flags.contains(AssetFlags::Vision) ||
            ( condition.flags.contains(AssetFlags::EngageAttack) && condition_unit.get_engage_attack().is_some_and(|s| !s.sid.str_contains("SID_チキ")))
        {
            condition.remove_god_eid_conditions();
            AssetFlags::set_condition_key("EID_チキ", false);
            AssetFlags::set_condition_key("チキ", false);
            AssetFlags::set_condition_key("PID_G001_チキ", true);
            AssetFlags::set_condition_key("AID_Person_チキ", true);
            result.commit(mode,  Some(condition_unit.person), None, equipped);
            if is_dragonstone(equipped) && condition.flags.contains(AssetFlags::EngageAttack) && mode == 2 {
                result.body_anims.add("End0AF-No2_c099_N".into());
            }
        }
        else {
            result.commit(mode, Some(condition_unit.person), Some(condition_unit.get_job()), equipped);
        }
        // println!("Tiki Engaged!");
        // print_asset_table_result(result, mode);
        true
    }
    else { false }
}

fn mode_1_transformation(result: &mut AssetTableResult, condition_unit: &mut Unit, equipped: Option<&ItemData>, conditions: &Array<&Il2CppString>, flags: &mut AssetConditions) {
    let jid = condition_unit.job.jid.to_string();
    if has_enemy_tiki(condition_unit) {
        if flags.flags.contains(AssetFlags::EngageAttack){
            result.body_model = "oBody_Tik0AF_c567".into();
            result.hair_model = "oHair_h567".into();
            emblem::engage_animation_mode_1(result, emblem::get_emblem_attack_index(condition_unit) as i32, 2);
            result.scale_stuff[16] = 2.6;
        }
        else { result.setup_for_person(1, PersonData::get("PID_E001_Boss"), conditions); }
    }
    else if is_dragonstone(equipped) &&
        (( transform::is_tiki_engage(condition_unit) && DVCFlags::EngageWeapons.get_value() ) || equipped.is_some_and(|i| i.iid.str_contains("チキ") && i.kind == 9)){
        result.setup_for_person(1, PersonData::get("PID_G001_チキ_竜化"), conditions);
    }
    else if !get_outfit_data().apply_monster_asset(result, condition_unit, 1) {
        if jid == "JID_裏邪竜ノ子" || condition_unit.get_dress_gender() == Gender::Male {
            result.setup_for_person_job_item(1, PersonData::get("PID_ラファール_竜化"), Some(condition_unit.job), None, conditions);
        }
        else { result.setup_for_person_job_item(1, PersonData::get("PID_エル_竜化"), Some(condition_unit.job), None, conditions); }
    }
}

fn boss_dress_setup_conditions(condition_unit: &Unit, conditions: &mut AssetConditions, appearance: Option<&PersonalDressData>) {
    // Past Alear
    let pid = condition_unit.person.pid.to_string();
    if condition_unit.person.get_asset_force() != 0 && condition_unit.person.get_job().map(|j| j.parent.hash).unwrap_or(condition_unit.job.parent.hash) == 185671037 {
        if let Some(lueur_replacement) =  PersonData::get(DVCVariables::get_dvc_person(0, false)).filter(|p| p.parent.index > 1){
            conditions.flags.set_gender(lueur_replacement.get_dress_gender());
            AssetFlags::set_person_conditions(condition_unit.person, false);
            if lueur_replacement.gender == 1 && lueur_replacement.flag.value & 32 == 0 {
                condition_unit.edit.set_gender(1);
                condition_unit.person.set_gender(Gender::Male);
                conditions.flags.set_gender(Gender::Male);
            }
            else {
                condition_unit.person.set_gender(Gender::Female);
                condition_unit.edit.set_gender(2);
                conditions.flags.set_gender(Gender::Female);
            }
            AssetFlags::set_person_conditions(lueur_replacement, true);
            conditions.flags.set(AssetFlags::NPC, true);
        }
    }
    else if pid.contains("ヴェイル") && pid != PIDS[32] {   // Veyle
        if let Some(veyle_replacement) = PersonData::get(DVCVariables::get_dvc_person(32, false)).filter(|p| p.pid.to_string() != PIDS[32]){
            AssetFlags::set_person_conditions(condition_unit.person, false);
            AssetFlags::set_person_conditions(veyle_replacement, true);
            if veyle_replacement.parent.index == 1 {
                AssetFlags::set_condition_key("JID_邪竜ノ子", true);
                conditions.flags.set_gender(if DVCVariables::is_lueur_female() { Gender::Female } else { Gender::Male });
            }
            conditions.flags.set(AssetFlags::NPC, true);
        }
    }
    else if DVCFlags::RandomBossesNPCs.get_value() && !is_playable_person(condition_unit.person) {  // Bosses
        if let Some(appearance) = appearance.as_ref() {
            AssetFlags::set_person_conditions(condition_unit.person, false);
            condition_unit.edit.set_name(Mess::get(appearance.mpid.as_str()));
            condition_unit.edit.set_gender(if appearance.is_female { 2 } else { 1 });
            conditions.flags.set_gender(if appearance.is_female { Gender::Female } else { Gender::Male });
            conditions.flags.set(AssetFlags::NPC, true);
        }
    }
    if condition_unit.person.parent.index > 1 && condition_unit.person.flag.value & 128 == 0 { AssetFlags::set_condition_key("MPID_Lueur", false); }
    if condition_unit.person.flag.value & 1024 != 0 && !pid.contains("ヴェイル") { AssetFlags::set_condition_key("MPID_Lueur", true); }
}

pub fn commit_for_unit_dress(
    result: &mut AssetTableResult,
    mode: i32,
    unit: &mut Unit,
    equipped: Option<&ItemData>,
    con: &mut Array<&'static Il2CppString>,
) -> AssetConditions
{
    let mut conditionss = AssetConditions::new(Some(unit), mode, equipped);
    if let Some(gid) = unit.person.aid.filter(|aid| aid.str_contains("GID_")) {
        if let Some(god) = GodData::get(gid) {
            let gender = if god.female == 1 { Gender::Female } else { Gender::Male };
            conditionss.flags.set_gender(gender);
            AssetFlags::set_condition_key(unit.person.pid, false);
            if let Some(name) = unit.person.name {
                AssetFlags::set_condition_key(name, false);
            }
            AssetFlags::set_condition_key(god.asset_id, true);
            AssetFlags::set_condition_key(god.gid, true);
            AssetFlags::set_condition_key(god.mid, true);
            result.commit_mode(mode);
            result.replace(mode);
            return conditionss;
        }
    }
    if (unit.person.gender != 2 &&  unit.person.gender != 1) || unit.person.bmap_size > 1 {
        if conditionss.flags.contains(AssetFlags::Vision) {
            result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, con);
            conditionss.flags.set_gender(Gender::Female);
            conditionss.flags.set(AssetFlags::Monster, false);
        }
        else {
            result.commit(mode, Some(unit.person), Some(unit.job), equipped);
            conditionss.flags.set(AssetFlags::Monster, true);
        }
        return conditionss;
    }
    if m022_god_dress(result, unit, mode, con) || random_emblem_name_asset_switch(result, unit, mode, equipped, con){
        conditionss.emblem_unit = true;
        return conditionss;
    }
    let condition_unit = if conditionss.flags.contains(AssetFlags::Vision) { UnitUtil::get_vision_owner(unit).unwrap_or(unit) } else { &unit  };

    if tiki_engage(result, condition_unit, mode, equipped, &mut conditionss) { return conditionss; }
    // Boss Randomization / Past Alear Edit / Veyle Edit
    let rand = RandomizedGameData::get_read();
    let appearance = rand.person_appearance.get_unit_appearance(condition_unit);
    boss_dress_setup_conditions(condition_unit, &mut conditionss, appearance);
    let promoted = condition_unit.job.rank > 1 || condition_unit.level > 20;
    if conditionss.flags.contains(AssetFlags::MapTransform) && mode == 1 {
        mode_1_transformation(result, unit, equipped, con, &mut conditionss);
        return conditionss;
    }
    let db = get_outfit_data();
    let engaged = condition_unit.status.value & 8388608 != 0;

    if let Some(data) = UnitAssetMenuData::get_unit_data(condition_unit) {
        let profile_flag = data.get_active_flag(engaged);
        // println!("{}: Profile {} Mode: {}", condition_unit.get_name(), profile_flag, mode);
        if let Some(god) = condition_unit.god_link.or(condition_unit.god_unit).filter(|_| engaged  ) {
            if profile_flag & 256 != 0 { conditionss.flags.set_condition_flag(AssetFlags::Engaged, false); }
            if profile_flag & 6 == 2 { conditionss.remove_god_eid_conditions(); }
            else if profile_flag & 6 == 4 {
                let gid = god.data.gid.to_string();
                conditionss.remove_god_eid_conditions();
                AssetFlags::set_person_conditions(condition_unit.person, false);
                AssetFlags::set_condition_key(gid, true);
                AssetFlags::set_condition_key(god.data.mid, true);
                AssetFlags::set_condition_key(god.data.asset_id, true);
                conditionss.flags.set_condition_flag(AssetFlags::Engaged, false);
                let gender = if god.data.female == 1 { Gender::Female } else { Gender::Male };
                conditionss.flags.set_gender(gender);
                result.commit(mode, Some(condition_unit.person), None, equipped);
                db.correct_anims(result, unit, profile_flag, &conditionss);
                return conditionss;
            }
        }
        if UnitAssetMenuData::get().is_preview {
            let shop_combat = UnitAssetMenuData::get().is_shop_combat;
            AssetFlags::set_condition_key("拠点", !shop_combat);
            AssetFlags::set_condition_key("私服", !shop_combat);
            if shop_combat { AssetFlags::remove_unit_accessories(condition_unit); }
            result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        }
        else {             
            result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
            db.accessory_conditions.commit_accessories(result, condition_unit, mode);
        }
        if profile_flag & 8 != 0 {
            if let Some(appear) = appearance.as_ref() {
                appear.apply_appearance(result, mode, promoted, None, &db.hashes, false);
            }
        }
        else if GameUserData::get_sequence() != 4 || conditionss.character_mode == UnitInfo|| (UnitAssetMenuData::get().is_preview && UnitAssetMenuData::get().is_shop_combat) {
            if GameVariableManager::get_number(format!("G_JG_{}", condition_unit.person.pid)) == unit.job.parent.hash &&
                (DVCVariables::ClassMode.get_value()& 1 != 0 || DVCVariables::is_changed_recruitment_order(false))
                && conditionss.random_dress.is_off()
            {
                if let Some(dress) = db.dress.get_personal_dress(condition_unit) {
                    dress.apply(result, mode, promoted, db.anims.get_mount_type(unit, get_unit_dress(unit)), &db.hashes);
                }
            }
        }
        db.adjust_dress(result, &condition_unit, &conditionss);
        UnitAssetMenuData::set_assets(result, condition_unit, &conditionss);
        conditionss.profile_flag = profile_flag;
    }
    else {
        result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        if !result.dress_model.is_null() { if result.dress_model.contains("AT_c") { return conditionss; }}
        if !result.body_model.is_null() { if result.body_model.str_contains("AT_c") { return conditionss; }}
        db.adjust_dress(result, &condition_unit, &conditionss);
        if unit.person.asset_force != 0 {
            let generic_mode = DVCVariables::GenericAppearance.get_value();
            if DVCFlags::RandomBossesNPCs.get_value() && appearance.is_some(){ // Switch Appearances
                if let Some(appearance) = appearance.as_ref() { appearance.apply_appearance(result, mode, promoted, None, &db.hashes, false); }
                else { db.adjust_dress(result, &condition_unit, &conditionss); }
            }
            else {
                db.adjust_dress(result, &condition_unit, &conditionss);
                if conditionss.flags.is_generic() && generic_mode != 0 && mode == 2 {   // Generic
                    let rng = Random::new(unit.grow_seed as u32);
                    if generic_mode & 1 != 0 { db.assign_random_head_hair(result, rng); }
                    if generic_mode & 2 != 0 { change_color_by_rng(result, rng); }
                }
            }
        }
        if condition_unit.check_status(UnitStatusField::Engaging) { // Engaged
            if let Some(asset) = conditionss.engaged.as_ref()
                .and_then(|gid| GodData::get(gid)).map(|god| god.asset_id)
                .and_then(|asset| db.dress.get_engaged_dress(asset))
            {
                asset.apply(result, mode, unit.get_dress_gender());
            }
        }
        conditionss.profile_flag = 0;
    }
    if conditionss.flags.contains(AssetFlags::CombatTranforming) { AnimData::remove(result, true, true); }
    if DVCFlags::RandomUnitInfo.get_value() {}

    if mode == 2 {
        if DVCVariables::BodyScaling.get_value() != 0 {  random_body_scale(result, Some(unit.grow_seed), false); }
        if condition_unit.person.get_job().map(|j| j.parent.hash).unwrap_or(condition_unit.job.parent.hash) == 185671037 { lueur_fell_child_hair(result); }
    }
    if DVCVariables::BodyScaling.get_value() != 0 && mode == 2 { random_body_scale(result, Some(unit.grow_seed), false); }
    /*
    if crate::DeploymentConfig::get().debug {
        println!("Dress Complete {}: Job: {} Mode: {}", Mess::get_name(unit.person.pid), Mess::get_name(unit.job.jid), mode);
        print_asset_table_result(result, mode);
    }
     */
    conditionss
}
pub fn change_result_colors_by_unit(unit: &Unit, result: &mut AssetTableResult) {
    let value = if unit.person.get_asset_force() == 0 {
        (unit.person.parent.hash as u32 >> 2) +
            (DVCVariables::get_seed() as u32 >> 2) +
            ( unit.job.parent.hash as u32 >> 2)
    }
    else { unit.grow_seed as u32 };
    let rng = Random::instantiate().unwrap();
    rng.ctor(value);
    change_color_by_rng(result, rng);
}
pub fn change_color_by_rng(result: &mut AssetTableResult, rng: &Random) {
    let index: [usize; 6] = [0, 1, 4, 5, 6, 7];
    for x in index {
        let value2 = rng.value();
        result.unity_colors[x].r = ( value2 & 255 ) as f32 / 255.0;
        result.unity_colors[x].g = (( value2 >> 4 ) & 255 ) as f32 / 255.0;
        result.unity_colors[x].b = (( value2 >> 8 ) & 255 ) as f32 / 255.0;
    } 
}
fn m022_god_dress(result: &mut AssetTableResult, unit: &Unit, mode: i32, conditions: &mut Array<&'static Il2CppString>) -> bool {
    let pid = unit.person.pid.to_string();
    if pid.contains("M022_紋章士") && DVCVariables::EmblemRecruitment.get_value() != 0 {
        if let Some(god) = EMBLEM_ASSET.iter().position(|x| pid.contains(x))
            .and_then(|x| DVCVariables::get_god_from_index(x as i32, true))
        {
            emblem::asset_table_result_god_setup(result, mode, Some(god), true, conditions, None);
            return true;
        }
    }
    false
}
fn random_emblem_name_asset_switch(
    result: &mut AssetTableResult,
    condition_unit: &Unit,
    mode: i32,
    equipped: Option<&ItemData>,
    conditions: &mut Array<&'static Il2CppString>
) -> bool {
    let pid =  condition_unit.person.pid.to_string();
    if condition_unit.person.summon_rank > 2 { return false;}
    if PIDS.iter().any(|x| pid == *x) || condition_unit.person.name.is_none() || condition_unit.person.flag.value & 1536 != 0 { return false; }
    let name = condition_unit.person.name.unwrap().to_string();

    if DVCFlags::GodNames.get_value() && !pid.contains(PIDS[0]) && !pid.contains("M022") {
        if condition_unit.force.is_some_and(|f| f.force_type == 1) {
            let mid = name.replace("MPID", "MGID");
            if let Some(person) = get_emblem_person(mid.into()) {
                result.setup_for_person_job_item(mode, Some(person), person.get_job(), equipped, conditions);
                condition_unit.edit.set_name(Mess::get_name(person.pid));
                condition_unit.edit.set_gender(person.gender);
                clear_accessory_from_list(result.accessory_list, "Eff_EmblemAura");
                return true;
            }
        }
    }
    condition_unit.edit.set_gender(0);
    false
}

fn hair_adjustment(result: &mut AssetTableResult) {
    if !result.hair_model.is_null() {
        if !result.hair_model.contains("null") {
            if result.accessory_list.list.iter()
                .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("Hair"))) {
                result.hair_model = "uHair_null".into();
            }
        }
    }
}
fn eve_sforgia_correction(result: &mut AssetTableResult, mode: i32) {
    if mode != 1 { return; }
    if !result.body_model.is_null() {
        let body = result.body_model.to_string();
        if body.contains("c451") { result.body_model = body.replace("c451", "c000").into(); }
        if body.contains("c151") { result.body_model = body.replace("c151", "c000").into(); }
    }
}
pub(crate) fn random_body_scale(result: &mut AssetTableResult, hash: Option<i32>, is_unit_info: bool) {
    let scale_mode = DVCVariables::BodyScaling.get_value();
    if scale_mode == 0 { return; }
    let rng = 
    if let Some(hash) = hash { create_rng(hash, 2) }
    else { Random::get_system() };
    if scale_mode & 1 != 0 {
        let v = outfit_core::get_random_scaling(9, rng);
        result.scale_stuff[9] = v as f32 / 100.0;
    }
    if scale_mode & 2 != 0 {
        for x in 0..13 {
            if x == 9 { continue; }
            let v = outfit_core::get_random_scaling(x, rng);
            result.scale_stuff[x as usize] = v as f32 / 100.0;
        }
    }
}
pub fn replace_with_engage_hair(result: &mut AssetTableResult, mode: i32) {
    if !result.hair_model.is_null() && !result.hair_model.contains("null") {
        if let Some(engaged) = get_outfit_data().hashes.get_engaged_hair(result.hair_model) { result.hair_model = engaged; }
    }
    else {
        if let Some(acc) = result.accessory_list.list.iter().find(|x| x.model.is_some_and(|m| m.str_contains("Hair"))) {
            let locator = acc.locator.unwrap().to_string();
            if let Some(hair) = get_outfit_data().hashes.get_engaged_hair(acc.model.unwrap()) {
                change_accessory(result.accessory_list, hair.to_string().as_str(), locator.as_str());
            }
        }
    }
    result.replace(mode);
}
pub fn lueur_fell_child_hair(result: &mut AssetTableResult) {
    result.unity_colors[0].r = 0.631372549;
    result.unity_colors[0].g = 0.1647058824;
    result.unity_colors[0].b = 0.1882352941;
    result.unity_colors[1].r = 0.631372549;
    result.unity_colors[1].g = 0.1647058824;
    result.unity_colors[1].b = 0.1882352941;
    replace_with_engage_hair(result, 2);
}
pub fn lueur_god_hair(result: &mut AssetTableResult) {
    result.unity_colors[0].r = 0.1529411765;
    result.unity_colors[0].g = 0.2745098039;
    result.unity_colors[0].b = 0.462745098;
    result.unity_colors[1].r = 0.09019607843;
    result.unity_colors[1].g = 0.1764705882;
    result.unity_colors[1].b = 0.3490196078;
    replace_with_engage_hair(result, 2);
}