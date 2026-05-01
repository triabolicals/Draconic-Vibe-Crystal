use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::proc::{*, desc::*};
use unity::il2cpp::object::Array;
use unity::prelude::{MethodInfo, OptionalMethod};
use crate::{continuous, ironman, randomizer, DVCVariables, DVCConfig};
use std::io::Write;
mod hubsequence;
mod mapsequence;
mod gmapsequence;
mod mainsequence;
mod summon;
mod unitgrow;
mod levelup;
mod eventdemo;

pub use gmapsequence::*;
pub fn replace_desc_void_function(desc: &mut Array<&mut ProcDesc>, method_name: &str, function: *mut u8) {
    let method_name = method_name.to_string();
    if let Some(void_method) = desc.iter_mut()
        .flat_map(|d| d.cast_to_method_call_mut())
        .find(|d| d.function.method.as_ref().is_some_and(|m| m.get_name().is_some_and(|s| s == method_name)))
    {
        void_method.function.method_ptr = function;
    }
}

pub fn proc_bind_desc_edit(proc: &mut ProcInst) {
    let hashcode = proc.hashcode;
    let name = proc.name.map(|v| v.to_string());
    let descs = proc.descs.get_mut();
    if let Some(name) = name {
        if name.contains("TelopManager") {
            if name.contains("ProcBondLevelUp") {
                replace_desc_void_function(descs, "LoadFace", crate::sprite::telop::proc_bond_level_up_load_face as _);
                replace_desc_void_function(descs, "ReleaseFace", crate::sprite::telop::proc_bond_level_up_release_face as _);
            }
            else if name.contains("ProcBondEngagePair") {
                replace_desc_void_function(descs, "LoadFace", crate::sprite::telop::proc_bond_engage_pair_load_face as _);
                replace_desc_void_function(descs, "ReleaseFace", crate::sprite::telop::proc_bond_engage_pair_release_face as _);
            }
            return;
        }
    }
    match hashcode {
        -186334910 => {

        }
        MAIN_SEQUENCE => { mainsequence::main_sequence_desc_edit(descs); }
        1959640519 => { // LevelUpSequence
            descs[0] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::job::chaos::level_up_prepare));
            descs[9] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::job::chaos::level_up_reflect));
        }
        GMAP_SEQUENCE => { gmap_sequence_desc_edit(descs); }
        SORTIE_SEQUENCE => { descs[1] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::sortie_play_bgm)); }
        WELL_SEQUENCE => { descs[19] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::item::well::well_get_item_rng)); }
        HUB_SEQUENCE => { hubsequence::hub_sequence_desc_edit(descs); }
        MAP_SEQUENCE_BATTLE => { descs[60] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::map_sequence_battle_pre_bgm)); }
        MAP_SEQUENCE_BATTLE_ACTION => { descs[1] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::map_sequence_battle_action_pre_bgm)); }
        COMBAT_COMBAT_SEQUENCE => { descs[4] = ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::combat_sequence_pre_bgm)); }
        MAP_SEQUENCE => {
            if GameUserData::get_sequence() > 3 && !GameUserData::is_evil_map() { GameUserData::get_status().value &= !8192; }
            mapsequence::map_sequence_desc_edit(descs);
            ironman::map_save_menu_edits();
            continuous::update_next_chapter();  // For Chapter 11/22 Continue Flag
            ironman::ironman_code_edits();
            randomizer::bgm::randomize_bgm_map();
            continuous::random::continous_rand_emblem_adjustment();
        }
        EVENT_DEMO_SEQUENCE => { eventdemo::event_demo_function_edit(); }
        HUB_MENU_SEQUENCE => {
            let con = DVCVariables::Continuous.get_value();
            if (con == 1 || con == 2) && GameVariableManager::get_bool("G_Cleared_M004"){
                continuous::set_next_chapter();
                descs[21] = ProcDesc::call(ProcVoidFunction::new(None, hubsequence::hub_menu_sequence_next_map_bind));
                descs[24] = ProcDesc::call(ProcVoidFunction::new(None, hubsequence::hub_menu_sequence_next_map_bind));
            }
        }
        31745184 => {   // MapEngageSummon
            descs[13] = ProcDesc::call(ProcVoidFunction::new(None, summon::commit_summon));
        }
        UNIT_GROW_SEQUENCE => {
            descs[0] = ProcDesc::call(ProcVoidFunction::new(None, unitgrow::unit_grow_sequence_prepare));
            descs[1] = ProcDesc::call(ProcVoidFunction::new(None, unitgrow::unit_grow_gain_exp));
        }
        1918405982 => { randomizer::latertalk::edit_later_talk_data(); }
        UNIT_SELECT_SUB_MENU => { outfit_core::add_sub_unit_menu_item(proc); }
        _ => {} 
    }
}
/*
pub fn print_desc(desc: &mut Array<&mut ProcDesc>, proc_name: &str, hash: i32){
    if let Ok(mut file) = File::options().create(true).write(true).truncate(true).open(format!("sd:/Classes/ProcDescs/{}.txt", proc_name)) {
        writeln!(file, "{}: {}", proc_name, hash).unwrap();
        desc.iter().enumerate().for_each(|(i, p)| {
            let ty = p.ty as i32;
            let class = p.get_class().get_name();
            let s = format!("{}\t{} ({})", i, class, ty);

            match ty {
                0 | 1 | 4 | 5 | 7 | 9 | 15 | 6 => { writeln!(&mut file, "{}", s).unwrap(); }
                2 => {
                    let jump = p.cast::<ProcDescLabel>().label;
                    writeln!(&mut file, "{}: {}", s, jump).unwrap();
                }
                3 => {
                    let label = p.cast::<ProcDescLabel>().label;
                    writeln!(&mut file, "{} Label: {}", i, label).unwrap();
                }
                _ => {
                    if class.contains("sync") || class.contains("Log") || class.contains("Sound") {
                        writeln!(&mut file, "{}", s).unwrap();
                    }
                    else {
                        let call = p.cast::<ProcDescCallEdit>();
                        if let Some(method) = call.function.method.as_ref().and_then(|m| m.get_name()) {
                            writeln!(&mut file, "{}: {}", s, method).unwrap();
                        }
                        else {
                            writeln!(&mut file, "{}: unknown method", s).unwrap();
                        }
                    }
                }
            }
        });
    }
}
*/
pub fn call_proc_original_method(proc: &ProcInst, method_name: &str) {
    if let Some(method) = proc.klass.get_method_from_name(method_name, 0).ok() {
        let method_call = unsafe { std::mem::transmute::<_, fn(&ProcInst, &MethodInfo)>(method.method_ptr) };
        method_call(proc, method);
    }
    else { println!("Unable to call method '{}' for {}", method_name, proc.klass.get_name()); }
}
pub extern "C" fn nothing_proc(_proc: &mut ProcInst, _method_info: OptionalMethod) {}