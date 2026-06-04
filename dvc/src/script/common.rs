/*
use engage::unit::UnitUtil;
use crate::procs::call_proc_original_method;
use super::*;


pub(crate) extern "C" fn unit_join(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let mut person: [Option<&PersonData>; 3] = [None, None, None];
    for x in 0..3 {
        if let Some(pid) = args.try_get_string(x as i32) { person[x] = try_unit_join(pid); }
    }
    let event_sequence = ScriptUtil::get_sequence();
    UnitUtil::join_message(event_sequence, person[0], person[1], person[2]);
    call_proc_original_method(event_sequence, "YieldCoroutine");
}

fn try_unit_join(pid: &Il2CppString) -> Option<&'static PersonData>{
    let key_r = format!("G_R2_{}", pid);
    let pid =
        if GameVariableManager::exist(key_r.as_str()) { GameVariableManager::get_string(key_r.as_str()) }
        else { pid };
    let person = PersonData::get(pid);
    if let Some(person) = person.as_ref() {
        if let Some(unit) = UnitPool::get_from_person_force_mask(person, 0x7F) {
            let force = if unit.force.is_none_or(|f| 2 < (f.force_type & 31)) { ForceType::Absent } else { ForceType::Player };
            ScriptUnit::unit_transfer_impl(unit, force);
            Some(unit.person)
        }
        else { UnitUtil::join_unit_person(person).map(|u| u.get_person()) }
    }
    else { None }
}

 */