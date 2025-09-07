use engage::{
    force::*, gamedata::{Gamedata, PersonData}, 
    gameuserdata::GameUserData, gamevariable::GameVariableManager, 
    godpool::GodPool, map::inspectors::*, proc::ProcInst, 
    random::Random, script::{DynValue, *}, 
    sequence::{gmap_sequence::GmapSequence, mapsequence::MapSequence}, util::get_singleton_proc_instance,
    hub::hubsequence::HubSequence,
    proc::{ProcVoidFunction, desc::*},

};
use engage::unitpool::UnitPool;
use unity::prelude::*;
use crate::{utils::*, enums::PIDS, randomizer::{*, person::PLAYABLE, RANDOMIZER_STATUS}, config::DVCVariables};
use crate::randomizer::person::{switch_person, switch_person_reverse};

pub extern "C" fn dvc_alear_is_female(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) -> &'static DynValue{
    DynValue::new_number(GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) as f64)
}
pub extern "C" fn is_alear_female(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) -> &'static DynValue {
    let r = GameVariableManager::get_number(DVCVariables::LUEUR_GENDER) == 2;
    DynValue::new_boolean(r)
}

extern "C" fn hub_sequence_map_opening(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    if get_singleton_proc_instance::<HubSequence>().is_some() && (GameUserData::get_sequence() == 4 || GameUserData::get_sequence() == 5) {
        unsafe { hub_sequence_map_opening_event(proc, None); }
        item::hub::hub_item_randomized();
    }
}
extern "C" fn gmap_sequence_startup(proc: &mut ProcInst, optional_method: OptionalMethod) {
    unsafe { event_sequence_startup(proc, optional_method); }
    emblem::emblem_gmap_spot_adjust();
}
extern "C" fn map_sequence_map_opening(proc: &mut ProcInst, optional_method: OptionalMethod) {
    unsafe { event_sequence_map_opening(proc, optional_method); }
    adjust_person_map_inspectors();
    move_unit_to_deploy_spot(false);
}

extern "C" fn map_sequence_sortie_desc(proc: &mut ProcInst, optional_method: OptionalMethod) {
    move_unit_to_deploy_spot(false);
    unsafe { map_sequence_sortie(proc, optional_method); }
}
pub fn map_opening_proc_edit() {
    if let Some(hub_sequence) = get_singleton_proc_instance::<HubSequence>() {
        unsafe {
          (*hub_sequence.proc.descs.get())[28] = ProcDesc::call(ProcVoidFunction::new(None, hub_sequence_map_opening));
        }
    }
    if let Some(singleton_proc) = get_singleton_proc_instance::<GmapSequence>(){
        unsafe {
            (*singleton_proc.proc.descs.get())[12] = ProcDesc::call(ProcVoidFunction::new(None, gmap_sequence_startup));
        }
    }
    if let Some(singleton_proc) = get_singleton_proc_instance::<MapSequence>() {
        unsafe {
            (*singleton_proc.proc.descs.get())[45] = ProcDesc::call(ProcVoidFunction::new(None, map_sequence_map_opening));
            (*singleton_proc.proc.descs.get())[51] = ProcDesc::call(ProcVoidFunction::new(None, map_sequence_sortie_desc));
        }
    }
    get_nested_virtual_methods_mut("App", "SortieUnitSelect", "UnitMenuItem", "YCall")
        .map(|method| method.method_ptr = crate::assets::accmenu::unit_item_y_call as _);
}

#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    let result = call_original!(dyn_value, method_info);
    if result.is_none() || !DVCVariables::random_enabled() { return result; }
    let result_string = result.unwrap();
    let str1 = result_string.to_string();
    if DVCVariables::get_flag(DVCFlags::RandomDeploySpot, false){
        if str1.contains("AI_MV_BreakDown") { return Some("AI_MV_NearestEnemy".into()); }
        if str1.contains("AI_MI_BreakDown") { return Some("AI_MI_Null".into()); }
    }
    if str1.contains("Kengen") && !GameVariableManager::get_bool("G_CustomEmblem") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let emblem_index = KENGEN.iter().position(|x| *x == str1);
        if emblem_index.is_none() { return result;}
        let gid = EMBLEM_GIDS[emblem_index.unwrap()];
        let new_index = person::pid_to_index(&gid.to_string(), false);
        if new_index < 1 || new_index >= 20 { return result; }
        return Some(KENGEN[new_index as usize].into());
    }
    if str1.contains("GID_") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M026" || cid.contains("CID_S0")  { return result; } //Do not shuffle emblems in endgame
        if GameVariableManager::exist(format!("G_R_{}", str1).as_str()) {
            Some(GameVariableManager::get_string(format!("G_R_{}", str1).as_str()))
        }
        else { result }
    }
    else if str1.contains("PID_") {
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return result; }
        if GameUserData::get_sequence() == 4 || GameUserData::get_sequence() == 5 {
            if let Some(person) = PersonData::get(str1.as_str()) {
                let playable = PLAYABLE.get().unwrap();
                if let Some(pos) = playable.iter().position(|&x| x == person.parent.index){
                    if pos > 40 && DVCVariables::get_flag(DVCFlags::CustomUnitRecruitDisable, false) { return None; }
                    if pos > 96 { return None; }
                }
            }
        }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M022" && GameUserData::get_sequence() == 3 {
            if !GameVariableManager::exist("VeyleRecruitment") {
                GameVariableManager::make_entry_norewind("VeyleRecruitment", 0);
            }
            if !GameVariableManager::get_bool("VeyleRecruitment") {
                if str1 == PIDS[VEYLE] {
                    let new_pid = DVCVariables::get_dvc_person(VEYLE as i32, false);
                    if let Some(veyle_replacement_force) =
                        engage::unitpool::UnitPool::get_from_person_mut(new_pid, true)
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
            println!("Returing PID {}", Mess::get_name(GameVariableManager::get_string(format!("G_R_{}", str1).as_str())));
            Some(GameVariableManager::get_string(format!("G_R_{}", str1).as_str()))
        }
        else { result }
    }
    else if str1.contains("IID_") { // Random Item
        if GameVariableManager::get_number(DVCVariables::ITEM_KEY) & 1 == 0  { result }
        else { Some(item::get_random_item(result.unwrap(), false)) }
    }
    else if str1.contains("TUTID_紋章士") {
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { return result; }
        let key = replace_strs(result_string, "TUTID_紋章士", "G_R_GID_");
        let new_gid = GameVariableManager::get_string(key);
        let new_tut = replace_strs(new_gid, "GID_", "TUTID_紋章士");
        return Some(new_tut);
    }
    else { result }
}

pub fn change_g_pid_lueur() {
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    if let Some(pid) = GameVariableManager::try_get_string("G_R_PID_リュール") {
        EventScript::set("g_pid_lueur", DynValue::new_string(pid));
    }
}

pub fn replace_lueur_chapter22() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" && GameUserData::get_sequence() == 3 {  change_g_pid_lueur(); }
}

pub fn post_sortie_script_adjustment() {
    if GameUserData::get_chapter().cid.to_string() == "CID_M022" {
        GameVariableManager::make_entry("VeyleRecruitment", 0);
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 || lueur_on_map() { change_g_pid_lueur(); }
    }
    if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) == 2  { remove_equip_emblems();  }
    else if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) == 1 {
        let mut emblem_list =  crate::deployment::get_emblem_list();
        if emblem_list.len() < 2 { return; }
        remove_equip_emblems();
        let rng = Random::get_game();
        while let Some(unit) = Force::get(ForceType::Player).unwrap().iter().next() {
            if emblem_list.len() > 0 {
                let value = rng.get_value(emblem_list.len() as i32) as usize;
                let god_unit = GodPool::try_get_gid(emblem_list[value].as_str(), false).unwrap();
                unit.set_god_unit(god_unit);
                emblem_list.remove(value);
            }
            else { break; }
        }
    }
    if lueur_on_map() && GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3 { return; } // if alear is on map don't change anything
    adjust_person_map_inspectors();
}

fn person_index_convert(person_index: &mut i32) {
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 || *person_index < 1 { return; }
    if let Some(person) = PersonData::try_index_get(*person_index) {
        if is_player_unit(person){
            let new_person = person::switch_person(person);
            let x = PersonData::get_index( new_person.pid );
            println!("[MapInspector Swap] {} to {}", Mess::get_name(person.pid), Mess::get_name(new_person.pid));
            *person_index = x;
        }
    }
}

pub fn adjust_person_map_inspectors() {
    let inspectors = MapInspectors::get_instance();
    let free_deploy = GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3;
    let is_set = RANDOMIZER_STATUS.read().unwrap().inspectors_set;
    if !is_set{
        inspectors.inspectors.iter_mut()
            .for_each(|inspector| adjust_inspector(inspector, free_deploy));
        RANDOMIZER_STATUS.try_write()
            .map(|mut status| status.inspectors_set = true).unwrap();
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
        MapInspectorKind::BreakdownEnemy => {
            let inspector = inspector.cast_mut::<PokeInspector>();
            if DVCVariables::get_flag(DVCFlags::RandomDeploySpot, false) && GameUserData::get_chapter().cid.str_contains("M008") {
                inspector.x = -1;
                inspector.z = -1;
                inspector.h = -1;
                inspector.w = -1;
            }
            person_index_convert(&mut inspector.person);
        }
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

fn move_unit_to_deploy_spot(is_ally: bool) {
    if DVCVariables::get_flag(DVCFlags::RandomDeploySpot, false) && !GameVariableManager::get_bool("RDSet") {
        GameVariableManager::make_entry("RDSet", 1);
        for x in 0..41 {
            // Alear, Vander, Framme, Clanne are listed under the replacement pid in DisposData
            let person =
                if x == 0 || GameVariableManager::get_bool("DDFanClub") { PersonData::get(DVCVariables::get_dvc_person(x, false)) }
                else { PersonData::get(PIDS[x as usize]) };
            if x > 4 && person.is_some_and(|x| x.pid == DVCVariables::get_dvc_person(0, false)) { continue; }
            if let Some(data) = person.and_then(|p| get_dispos_data_by_person(p, if is_ally { 2 } else { 0 })){
                if let Some(unit) = UnitPool::get_from_person_mut(DVCVariables::get_dvc_person(x, false), false) {
                    if unit.x != data.dispos_x as u8 || unit.z != data.dispos_y as u8 {
                        let array: &mut Array<_> = Array::from_slice(
                            vec![
                                DynValue::new_string(PIDS[x as usize].into()),
                                DynValue::new_number(data.dispos_x as f64),
                                DynValue::new_number(data.dispos_y as f64),
                            ]
                        ).unwrap();
                        unsafe {
                            script_set_pos_unit(array, None);
                            script_move_unit(array, None);
                        }
                        unit.x = data.dispos_x as u8;
                        unit.z = data.dispos_y as u8;
                    }
                }
            }
        }
    }
}

pub extern "C" fn unit_move(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = args.try_get_unit(0).filter(|f| f.force.is_some_and(|f| f.force_type == 2)){
        if args.try_get_i32(1) != 0 {
            let person_data =
                if GameVariableManager::get_bool("DDFanClub") { &unit.person } else { switch_person(unit.person) };
            let position = GameVariableManager::get_number(format!("RD_{}", person_data.pid));
            let x = position % 100;
            let z = position / 100;
            let original = switch_person_reverse(person_data);
            if x != 0 && z != 0 {
                let array: &mut Array<_> = Array::from_slice(
                    vec![
                        DynValue::new_string(original.pid.clone()),
                        DynValue::new_number(x as f64),
                        DynValue::new_number(z as f64),
                    ]
                ).unwrap();
                unsafe {
                    script_set_pos_unit(array, None);
                    script_move_unit(array, None);
                }
                return;
            }
        }
    }
    unsafe { script_move_unit_1(args, None); }
}

fn get_dispos_data_by_person(person_data: &PersonData, force: i8) -> Option<&'static &'static mut DisposData> {
    if let Some(dispos_data) = DisposData::get_list() {
        dispos_data.iter().flat_map(|g| g.iter())
            .find(|x| x.get_person().is_some_and(|p| p.parent.hash == person_data.parent.hash) && x.force == force)
    }
    else { None }
}

#[skyline::from_offset(0x02199dc0)]
fn unit_join_original(args: &Il2CppArray<DynValue>, method_info: OptionalMethod);

#[skyline::from_offset(0x0219cff0)]
fn script_move_unit(arg: &mut Il2CppArray<&mut DynValue>, optional_method: OptionalMethod);

#[skyline::from_offset(0x0219cff0)]
fn script_move_unit_1(arg: &Il2CppArray<DynValue>, optional_method: OptionalMethod);

#[skyline::from_offset(0x0219cec0)]
fn script_set_pos_unit(arg: &mut Il2CppArray<&mut DynValue>, optional_method: OptionalMethod);

#[unity::from_offset("App", "HubSequence", "MapOpeningEvent")]
fn hub_sequence_map_opening_event(this: &ProcInst, method_info: OptionalMethod);

#[unity::from_offset("App", "MapSequence", "Sortie")]
fn map_sequence_sortie(this: &ProcInst, method_info: OptionalMethod);

#[skyline::from_offset(0x024e46d0)]
fn event_sequence_map_opening(proc: &ProcInst, optional_method: OptionalMethod);

#[skyline::from_offset(0x024e4430)]
fn event_sequence_startup(proc: &ProcInst, optional_method: OptionalMethod);

#[skyline::from_offset(0x01ed1fd0)]
fn event_turn_entry(args: &Array<&mut DynValue>, method_info: OptionalMethod);