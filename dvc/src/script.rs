use engage::{
    force::*, gamedata::{Gamedata, PersonData}, 
    gameuserdata::GameUserData, gamevariable::GameVariableManager, 
    god::GodPool, map::inspectors::*, unit::UnitPool,
    random::Random, script::{DynValue, *},
};
use unity::prelude::*;
use crate::{
    randomizer::person::switch_person, utils::*, enums::PIDS, 
    randomizer::*, config::DVCVariables
};
use crate::randomizer::status::RandomizerStatus;
pub(crate) mod chapter;
mod common;

pub extern "C" fn dvc_alear_is_female(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) -> &'static DynValue{
    DynValue::new_number(GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) as f64)
}
pub extern "C" fn is_alear_female(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) -> &'static DynValue {
    let r = GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2;
    DynValue::new_boolean(r)
}

#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    let result = call_original!(dyn_value, method_info);
    if result.is_none() || !DVCVariables::random_enabled() { return result; }
    let result_string = result.unwrap();
    let str1 = result_string.to_string();
    if str1.contains("Kengen") && !DVCFlags::CustomEmblemsRecruit.get_value() {
        if DVCVariables::EmblemRecruitment.get_value() == 0 { return result; }
        let emblem_index = KENGEN.iter().position(|x| *x == str1);
        if emblem_index.is_none() { return result;}
        let gid = EMBLEM_GIDS[emblem_index.unwrap()];
        let new_index = person::pid_to_index(&gid.to_string(), false);
        if new_index < 1 || new_index >= 20 { return result; }
        return Some(KENGEN[new_index as usize].into());
    }
    if str1.contains("GID_") {
        if DVCVariables::EmblemRecruitment.get_value() == 0 { return result; }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M026" || cid.contains("CID_S0")  { return result; } //Do not shuffle emblems in endgame
        if GameVariableManager::exist(format!("G_R_{}", str1).as_str()) {
            Some(GameVariableManager::get_string(format!("G_R_{}", str1).as_str()))
        }
        else { result }
    }
    else if str1.contains("PID_") {
        let sequence =  GameUserData::get_sequence();
        if sequence == 4 || sequence == 5 {
            if let Some(person) = PersonData::get(str1.as_str()) {
                if let Some(pos) = get_data_read().playables.iter().position(|x| x.hash == person.parent.hash) {
                    return
                        if pos < 41 || (pos >= 41 && pos < 90 && !DVCFlags::CustomUnitRecruitDisable.get_value()) {
                            if GameVariableManager::exist(format!("G_R_{}", str1).as_str()) {
                                Some(GameVariableManager::get_string(format!("G_R_{}", str1).as_str()))
                            } else { result }
                        } else { None }
                }
            }
            return result;
        }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M022" && sequence == 3 {
            if !GameVariableManager::exist("VeyleRecruitment") {
                GameVariableManager::make_entry_norewind("VeyleRecruitment", 0);
            }
            if !GameVariableManager::get_bool("VeyleRecruitment") {
                if str1 == PIDS[VEYLE] {
                    let new_pid = DVCVariables::get_dvc_person(VEYLE as i32, false);
                    if let Some(veyle_replacement_force) =
                        UnitPool::get_from_pid(new_pid, true)
                            .and_then(|unit| unit.force)
                    {
                        if veyle_replacement_force.force_type == 2 {  return Some(new_pid); }
                        else if veyle_replacement_force.force_type == 0 {
                            GameVariableManager::set_bool("VeyleRecruitment", true);
                        }
                    }
                }
            }
            return result; 
        }
        else if cid == "CID_M026" { return result; }
        if GameVariableManager::exist(format!("G_R_{}", str1).as_str()) {
            Some(GameVariableManager::get_string(format!("G_R_{}", str1).as_str()))
        }
        else { result }
    }
    else if str1.contains("IID_") { // Random Item
        if !DVCFlags::RandomEventItems.get_value() { result }
        else { Some(item::get_random_item(result.unwrap(), false)) }
    }
    else if str1.contains("TUTID_紋章士") {
        if DVCVariables::EmblemRecruitment.get_value() == 0 { return result; }
        let key = replace_strs(result_string, "TUTID_紋章士", "G_R_GID_");
        let new_gid = GameVariableManager::get_string(key);
        let new_tut = replace_strs(new_gid, "GID_", "TUTID_紋章士");
        return Some(new_tut);
    }
    else { result }
}

pub fn change_g_pid_lueur() {
    if DVCVariables::UnitRecruitment.get_value() == 0 { return; }
    if let Some(pid) = GameVariableManager::try_get_string("G_R_PID_リュール") {
        EventScript::set("g_pid_lueur", DynValue::new_string(pid));
    }
}

pub fn replace_lueur_chapter22() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" && GameUserData::get_sequence() == 3 {
        change_g_pid_lueur();
    }
}

pub fn post_sortie_script_adjustment() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" {
        GameVariableManager::make_entry("VeyleRecruitment", 0);
        GameVariableManager::make_entry("TalkPID", 0);
        if DVCVariables::UnitRecruitment.get_value()  != 0 || lueur_on_map() { change_g_pid_lueur(); }
    }
    let emblem_deploy_mode = DVCVariables::EmblemDeployment.get_value();
    if emblem_deploy_mode == 1 || emblem_deploy_mode == 2 { remove_equip_emblems(); }
    if emblem_deploy_mode == 1 {
        let mut emblem_list =  crate::deployment::get_emblem_list();
        if emblem_list.len() < 2 { return; }
        let rng = Random::get_game();
        let mut iter = Force::get(ForceType::Player).unwrap().iter();
        while let Some(unit) = iter.next() {
            if emblem_list.len() > 0 {
                let value = rng.get_value(emblem_list.len() as i32) as usize;
                let god_unit = GodPool::try_get_gid(emblem_list[value].as_str(), false).unwrap();
                unit.try_connect_god_unit(god_unit);
                emblem_list.remove(value);
            }
            else { break; }
        }
    }
    if lueur_on_map() && DVCVariables::UnitDeployment.get_value() == 3 { return; } // if alear is on map don't change anything
    adjust_person_map_inspectors();
}

fn person_index_convert(person_index: &mut i32) {
    if DVCVariables::UnitRecruitment.get_value()  == 0 || *person_index < 1 { return; }
    if let Some(person) = PersonData::try_index_get(*person_index) {
        if is_player_unit(person){
            if let Some(person) = switch_person(person).map(|p| p.parent.index){
                *person_index = person;
            }
        }
    }
}

pub fn adjust_person_map_inspectors() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M026" {
        // Expand the temp position when placing big dragon
        EventScript::set("temp_x_start", DynValue::new_number(1.0));
        EventScript::set("temp_x_end", DynValue::new_number(29.0));
        EventScript::set("temp_z_start", DynValue::new_number(17.0));
        EventScript::set("temp_z_end", DynValue::new_number(23.0));
        return;
    }
    let inspectors = MapInspectors::get_instance();
    let free_deploy = DVCVariables::UnitDeployment.get_value() == 3;
    let status = RandomizerStatus::get();
    if !status.inspectors_set {
        inspectors.inspectors.iter_mut()
            .for_each(|inspector| adjust_inspector(inspector, free_deploy));
        status.inspectors_set = true;
    }
}

fn adjust_inspector(inspector: &mut MapInspector, free_deploy: bool) {
    let kind = inspector.kind;
    if free_deploy {
        if kind == MapInspectorKind::Escape {
            let poke = inspector.cast_mut::<PokeInspector>();
            if poke.person == 1 { poke.person = -1; }
            else { person_index_convert(&mut poke.person); }
            return;
        }
        if kind == MapInspectorKind::Fixed  {
            let unit = inspector.cast_mut::<UnitInspector>();
            if unit.person == 1 {  unit.person = -1; }
            else { person_index_convert(&mut unit.person); }
            return;
        }
        if kind == MapInspectorKind::Talk {
            let talk = inspector.cast_mut::<EachInspector>();
            if talk.from_person == 1 { talk.from_person = -1; }
            person_index_convert(&mut talk.to_person);
            return;
        }
    }

    match kind {
        MapInspectorKind::UnitCommandPrepare | MapInspectorKind::TargetSelect | MapInspectorKind::EngageAfter | MapInspectorKind::EngageBefore|
        MapInspectorKind::Pickup => {
            let inspector = inspector.cast_mut::<PersonInspector>();
            person_index_convert(&mut inspector.person);
        }
        MapInspectorKind::Die | MapInspectorKind::ReviveBefore | MapInspectorKind::ReviveAfter | MapInspectorKind::Fixed => {
            let inspector = inspector.cast_mut::<UnitInspector>();
            person_index_convert(&mut inspector.person);
        }
        MapInspectorKind::BattleTalk | MapInspectorKind::BattleAfter | MapInspectorKind::BattleBefore | MapInspectorKind::Talk => {
            let inspector = inspector.cast_mut::<EachInspector>();
            person_index_convert(&mut inspector.from_person);
            person_index_convert(&mut inspector.to_person);
        }
        MapInspectorKind::UnitCommandInterrupt => {
            let inspector = inspector.cast_mut::<InterruptInspector>();
            person_index_convert(&mut inspector.person);
        }
       MapInspectorKind::Escape | MapInspectorKind::Breakdown => {
           let inspector = inspector.cast_mut::<PokeInspector>();
           person_index_convert(&mut inspector.person);
        }
        _ => {}
    }
}