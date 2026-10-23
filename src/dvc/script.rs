pub use engage::{
    app::eventscript::*, EventScriptExt,
    moon_sharp::interpreter::{dynvalue::*, script::*, ITableMethods},
};
use engage::app::{IItemDataMethods, IPersonDataMethods, ScriptUnit};
use unity::prelude::*;
use crate::{
    DVCFlags, DVCVariables,
    enums::KENGEN, gamedata::EmblemPool, enums::PIDS,
    utils::override_vtable_by_slot
};
use crate::enums::EMBLEM_GIDS;
use super::*;

pub(crate) mod chapter;
pub mod common;
pub mod inspectors;

pub fn script_vtable_override() {
    override_vtable_by_slot("App", "TurnInspector", 7, inspectors::turn_common_inspector_is_eanble as _);
    override_vtable_by_slot("App", "UnitInspector", 7, inspectors::unit_inspector_is_eanble as _);
    override_vtable_by_slot("App", "FixedInspector", 7, inspectors::unit_inspector_is_eanble as _);
}
pub extern "C" fn install_dvc_script_functions(script: EventScript) {
    chapter::install_chapter_script_functions(script);
    script.register_function("PlayerGender", dvc_alear_is_female);
    script.register_function("IsAlearFemale", is_alear_female);
    if DVCVariables::get_sequence() == 4 { 
        script.register_action("UnitJoin", unit_join);
        script.register_action("UnitJoinSilent", unit_join_silent);
    }
}
extern "C" fn unit_join(args: Array<DynValue>, _: OptionalMethod) {
    let is_fx =
        args.iter().any(|v|
            if v.get_type().value == 4 {
                let s = v.get_string().to_rust_string();
                PIDS[36..].contains(&s.as_str())
            }
            else { false }
    );
    if is_fx { ScriptUnit::unit_join(args); }
}
extern "C" fn unit_join_silent(args: Array<DynValue>, _: OptionalMethod) {
    let is_fx =
        args.iter().any(|v|
            if v.get_type().value == 4 {
                let s = v.get_string().to_rust_string();
                PIDS[36..].contains(&s.as_str())
            }
            else { false }
        );
    if is_fx { 
        for i in 0..3 { ScriptUnit::unit_join_impl(args, i); } 
    }
}
extern "C" fn dvc_alear_is_female(_args: Array<DynValue>, _: OptionalMethod) -> DynValue{ DynValue::new_number(DVCVariable::LueurGender.get_value() as f64) }
extern "C" fn is_alear_female(_args: Array<DynValue>, _: OptionalMethod) -> DynValue { DynValue::new_boolean(DVCVariable::LueurGender.get_value() == 2) }

#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: DynValue,  method_info: OptionalMethod) -> Il2CppString {
    let result = call_original!(dyn_value, method_info);
    if result.is_null()  { return result; }
    let str1 = result.to_rust_string();
    let chapter = DVCVariables::get_chapter_index();
    let kengen = str1.contains("Kengen");
    let is_pid = str1.starts_with("PID_");
    let is_gid = str1.starts_with("GID_");
    if kengen || is_gid {
        if DVCVariable::EmblemRecruitment.get_value() == 0 || str1 == EMBLEM_GIDS[19] { result }
        else if kengen {
            KENGEN.iter().position(|x| *x == str1)
                .and_then(|pos| EmblemPool::get_replacement_index_2(pos as i32, false).filter(|x| *x > 1 && *x < 19 ))
                .map(|replacement| KENGEN[replacement as usize].into())
                .unwrap_or_else(|| Il2CppString::new(""))
        }
        else {
            if chapter == 26 { result }                 //Do not shuffle emblems in endgame
            else if chapter > 32 && chapter < 45 {  // Emblem Paralogue only switch if custom replacement
                if let Some(god) = EmblemPool::get_replacement_god(GodData::get(result), false).filter(|v| EmblemPool::is_custom(*v)) { god.get_gid() }
                else { result }
            }
            else if GameVariableManager::is_exist(format!("G_R_{}", str1).as_str()) { GameVariableManager::get_string(format!("G_R_{}", str1).as_str()) }
            else { result }
        }
    }
    else if is_pid {
        if DVCVariable::UnitRecruitment.get_value() == 0 { result }
        else {
            let sequence = DVCVariables::get_sequence();
            if (chapter == 22 && sequence == 3) || chapter == 26 { return result; }
            if GameVariableManager::is_exist(format!("G_R_{}", str1).as_str()) { GameVariableManager::get_string(format!("G_R_{}", str1).as_str()) } else { result }
        }
    }
    else if str1.contains("TUTID_紋章士") && DVCVariable::EmblemRecruitment.get_value() > 0 {
        let key = str1.replace("TUTID_紋章士", "G_R_GID_");
        let new_gid = GameVariableManager::get_string(key).to_string();
        return new_gid.replace("GID_", "TUTID_紋章士").into();
    }
    else if str1.starts_with("IID_") && DVCFlags::RandomEventItems.get_value() {
        if let Some(item_check) = to_opt(ItemData::get(str1.as_str().into())) {
            let flag = item_check.get_flag().m_value();
            if flag & 1 == 1 || item_check.get_kind() == ItemData_Kinds::precious() { return result; }
            let use_type = item_check.get_use_type().value as i64;
            let use_type_bits: i64 = 1 << use_type;
            if use_type_bits & 0x30001800000 != 0 { item_check.add_inventory(1); }
            GameData::get_random_item(RandomItemReplacementType::Script, false)
        }
        else { result }
    }
    else { result }
}
pub fn change_g_pid_lueur() {
    if let Some(hero) = PersonPool::get_replacement(0, false, true){
        EventScript::get_instance().m_global_table().set("g_pid_lueur", DynValue::new_string(hero.get_pid()));
    }
}