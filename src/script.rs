use engage::{
    force::*, gamedata::{Gamedata, PersonData}, 
    gameuserdata::GameUserData, gamevariable::GameVariableManager, 
    godpool::GodPool, map::inspectors::*, proc::ProcInst, 
    random::Random, script::{DynValue, *}, 
    sequence::mapsequence::MapSequence, unitpool::UnitPool, util::get_singleton_proc_instance
};
use unity::{prelude::*, il2cpp::object::Array, };
use crate::{utils::*, enums::PIDS, randomizer::{*, person::PLAYABLE, RANDOMIZER_STATUS}, config::DVCVariables};

#[unity::hook("App", "EventSequence", "MapOpening")]
pub fn event_sequence_map_opening_hook(proc: &ProcInst, method_info: OptionalMethod) {
    call_original!(proc, method_info);
    if GameUserData::get_sequence() == 4 || GameUserData::get_sequence() == 5 { crate::randomizer::item::hub::hub_item_randomized();  }
    else if GameUserData::get_sequence() == 6 {crate::randomizer::emblem::emblem_gmap_spot_adjust(); }
    if get_singleton_proc_instance::<MapSequence>().is_some() {  adjust_person_map_inspectors(); }
}

#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    let result = call_original!(dyn_value, method_info);
    if result.is_none() || !DVCVariables::random_enabled() { return result; }
    let result_string = result.unwrap();
    let str1 = result_string.to_string();
    if str1.contains("Kengen") && !GameVariableManager::get_bool("G_CustomEmblem") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let emblem_index = KENGEN.iter().position(|x| *x == str1);
        if emblem_index.is_none() { return result;}
        let gid = EMBLEM_GIDS[emblem_index.unwrap()];
        let new_index = crate::randomizer::person::pid_to_index(&gid.to_string(), false);
        if new_index < 1 || new_index >= 20 { return result; }
        return Some(KENGEN[new_index as usize].into());
    }
    if str1.contains("GID_") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M026" || cid.contains("CID_S0")  { return result; } //Do not shuffle emblems in endgame
        let string = format!("G_R_{}", str1);
        let new_gid = GameVariableManager::get_string(&string);
        if unsafe { !is_null_empty(new_gid, None) } { return Some(new_gid); }
    }
    else if str1.contains("PID_") {
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return result; }
        if (GameUserData::get_sequence() == 4 || GameUserData::get_sequence() == 5) && GameVariableManager::get_bool(DVCVariables::CUSTOM_UNIT_RECRUIT_DISABLE) {
            if let Some(person) = PersonData::get(str1.as_str()) {
                let playable = PLAYABLE.get().unwrap();
                if let Some(pos) = playable.iter().position(|&x| x == person.parent.index){
                    if pos > 40 { 
                        println!("Unit #{} ignored", pos);
                        return None; 
                    }
                }
            }
        }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M022" {
            if !GameVariableManager::exist("VeyleRecruitment") { GameVariableManager::make_entry_norewind("VeyleRecruitment", 0); }
            if !GameVariableManager::get_bool("VeyleRecruitment") {
                if str1 == PIDS[32] {
                    let new_pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
                    let veyle_replacement = engage::unitpool::UnitPool::get_from_person_mut(new_pid, true);
                    if veyle_replacement.is_none() { return result; }
                    let force = veyle_replacement.unwrap().force.unwrap().force_type;
                    if force == 2 {  return Some(new_pid); }
                    else if force == 0 { GameVariableManager::set_bool("VeyleRecruitment", true); }
                }
            }
            return result; 
        }
        else if cid == "CID_M026" { return result; }
        let string = format!("G_R_{}", str1);
        let new_pid = GameVariableManager::get_string(&string);
        if unsafe { !is_null_empty(new_pid, None) } { return Some(new_pid);  }
    }
    else if str1.contains("IID_") { // Random Item
        if GameVariableManager::get_number(DVCVariables::ITEM_KEY) & 1 == 0  { return result; }
        else { return Some( crate::randomizer::item::get_random_item(result.unwrap(), false)); }
    }
    else if str1.contains("TUTID_紋章士") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let key = replace_strs(result_string, "TUTID_紋章士", "G_R_GID_");
        let new_gid = GameVariableManager::get_string(key);
        let new_tut = replace_strs(new_gid, "GID_", "TUTID_紋章士");
        return Some(new_tut);
    }
    return result;
}

pub fn change_g_pid_lueur() {
    if !GameVariableManager::exist("G_R_PID_リュール") { return; }
    let replacement_pid = DVCVariables::get_dvc_person(0, true);
    if unsafe { crate::utils::is_null_empty(replacement_pid, None) } { return; }
    EventScript::set("g_pid_lueur", DynValue::new_string(replacement_pid));
    println!("Lueur PID was replaced for Chapter 22"); 
}

pub fn replace_lueur_chapter22() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" && GameUserData::get_sequence() == 3 {  change_g_pid_lueur(); }
}

pub fn post_sortie_script_adjustment() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" {
        GameVariableManager::make_entry("VeyleRecruitment", 0);
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 || crate::utils::lueur_on_map() { change_g_pid_lueur(); }
    }
    if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) == 2  { crate::utils::remove_equip_emblems();  }
    else if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) == 1 {
        let mut emblem_list =  crate::deployment::get_emblem_list();
        if emblem_list.len() < 2 { return; }
        crate::utils::remove_equip_emblems();
        let rng = Random::get_game();
        while let Some(unit) = Force::get(ForceType::Player).unwrap().iter().next() {
            let value = rng.get_value(emblem_list.len() as i32) as usize;
            let god_unit = GodPool::try_get_gid(emblem_list[value].as_str(), false).unwrap();
            unit.set_god_unit(god_unit);
            emblem_list.remove(value);
        }
    }

    if crate::utils::lueur_on_map() && GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3 { return; } // if alear is on map don't change anything 
    adjust_person_map_inspectors();
}

pub fn adjust_person_map_inspectors() {
    if RANDOMIZER_STATUS.read().unwrap().inspectors_set { return; }
    let inspectors = MapInspectors::get_instance();
    let full_deploy = GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3;
    if full_deploy {
        inspectors.inspectors.iter_mut().for_each(|inspector| adjust_inspector(inspector, full_deploy) );
    }
    /*
    inspectors.kind_inspectors.iter_mut().flat_map(|kind| kind.iter_mut() ).for_each(|inspector| {
        println!("Inspector Found: {}", inspector.kind as i32);
        adjust_inspector(inspector, full_deploy);
    });
    */
    
    RANDOMIZER_STATUS.try_write().map(|mut status| status.inspectors_set = true).unwrap();
}

fn adjust_inspector(inspector: &mut MapInspector, _full_deploy: bool) {
    let kind = inspector.kind;
    if kind == MapInspectorKind::Escape {
        let poke = inspector.cast_mut::<PokeInspector>();
        if poke.person == 1 { poke.person = -1; }
    }
    if kind == MapInspectorKind::Fixed  {
        let unit = inspector.cast_mut::<UnitInspector>();
        if unit.person == 1 {  unit.person = -1; }
    }
    if kind == MapInspectorKind::Talk {
        let talk = inspector.cast_mut::<EachInspector>();
        if talk.from_person == 1 {  talk.from_person = -1; }
    }
}
/* 
pub extern "C" fn unit_join(arg: &Array<DynValue>, method_info: OptionalMethod) {
    println!("Running Custom UnitJoin");
    let person1 = unit_try_join(arg, 0);
    let person2 = unit_try_join(arg, 1);
    let person3 = unit_try_join(arg, 2); 

    let proc = ScriptUtil::get_sequence();
    unsafe { unit_join_message(proc, person1, person2, person3, None); }

}

fn unit_try_join(arg: &Array<DynValue>, index: i32) -> Option<&'static PersonData> {
    if let Some(pid) = arg.try_get_string(index) {
        println!("Recruiting {}", Mess::get_name(pid));
        let key = format!("G_R_{}", pid);
        if let Some(pid_new) = GameVariableManager::try_get_string(key.as_str()) {
            if let Some(unit) = UnitPool::get_from_person_mut(pid_new, false) { //Unit is already created
                if unit.force.is_some_and(|f| f.force_type != 0 && ( f.force_type == 1 ||  f.force_type == 2 )) {
                    unit.transfer(0, false);
                    println!("Transferring {} -> {}", Mess::get_name(pid), Mess::get_name(unit.person.pid));
                    return Some(unit.person);
                }
            }
            else {
                println!("Creating and Transferring {} -> {}", Mess::get_name(pid_new),  Mess::get_name(pid),);
                if let Some(unit) = UnitUtil::join_unit(pid.to_string().as_str()) { return Some(unit.person); }
            }
        }
    }
    return None;
}

#[unity::from_offset("App", "UnitUtil", "JoinMessage")]
fn unit_join_message(proc: &ProcInst, person1: Option<&PersonData>, person2: Option<&PersonData>, person3: Option<&PersonData>, method_info: OptionalMethod);
*/