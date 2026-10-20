use super::*;
use engage::{
    app::{dieevent::*, UnitUtil, godunit::*, goddata::*, unit::*}, Mess
};
use crate::utils::to_opt;

pub fn die_event_edit(proc: ProcInst) {
    if let Some(die) = proc.try_cast::<DieEvent>() {
        let descs = die.m_descs();
        descs.set(3, Proc::call_2(ProcVoidMethod::from_fn(die, unit_die_talk).unwrap()));
        descs.set(4, Proc::call_2(ProcVoidMethod::from_fn(die, god_die_talk).unwrap()));
    }
}

extern "C" fn god_die_talk(die_event: DieEvent, _: OptionalMethod) {
    let unit = die_event.m_unit();
    if unit.is_null() { return; }
    if !unit.is_null() && UnitUtil::get_die_type(unit).value == 3 && to_opt(unit.get_god_unit()).is_some_and(|d| !d.get_is_darkness()){
        if unit.is_hero() {
            let god_unit = unit.get_god_unit().m_data().get_main_data().get_ascii_name();
            let mid = format!("MID_DIE_Lueur_{}", god_unit);
            if Mess::is_exist(mid.as_str()) { engage::app::Talk::start_bind(die_event, mid, false); }
        }
    }
}
extern "C" fn unit_die_talk(die_event: DieEvent, _: OptionalMethod) {
    let unit = die_event.m_unit();
    if unit.is_null() { return; }
    let mid = UnitUtil::get_die_message_2(unit);
    if !mid.is_null() { die_event.die_talk(); }
    else if unit.is_hero() {
        let message = UnitUtil::get_die_message(unit, "Lueur");
        if !message.is_null() { engage::app::Talk::start_bind(die_event, mid, false); }
    }
}