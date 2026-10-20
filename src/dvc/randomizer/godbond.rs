use super::*;
use engage::{app::{godbond::*, godbondholder::*, godbondholderpool::*, godunit::*}};
use crate::{utils::to_opt, dvc::unit::for_each_unit};

#[unity::hook("App", "GodBondHolder", "Get")]
pub fn god_bond_holder_get(this: GodBondHolder, unit: Unit, method_info: OptionalMethod) -> GodBond {
    if unit.is_null() || this.is_null() || this.m_data().get_force_type().value == 1 || unit.get_force_type().value == 1 { call_original!(this, unit, method_info) }
    else {
        if DVCVariables::is_random_map() || DVCVariable::Continuous.get_value() == 1 {
            let god_bond = call_original!(this, UnitPool::get_hero(false), method_info);
            if !god_bond.is_null() { unit.inherit_aptitude_2(god_bond); }
            god_bond
        }
        else if PersonPool::get_person_index(unit.get_person().hash()).is_some_and(|v| v < 41) { call_original!(this, unit, method_info) }
        else {
            let god_bond = call_original!(this, UnitPool::get_hero(false), method_info);
            if !god_bond.is_null() { unit.inherit_aptitude_2(god_bond); }
            god_bond
        }
    }
}
pub fn update_lueur_bonds() {
    if PersonPool::get_replacement_index(0, false).is_some_and(|v| v != 0 ){
        if let Some(g_unit) = to_opt(GodPool::try_get("GID_リュール", false)){
            for_each_unit(9u32, |unit|{
                if let Some(bond) = to_opt(g_unit.get_bond(unit)) {
                    if bond.m_level() < 20 {
                        bond.set_m_level(20);
                        unit.inherit_aptitude_2(bond);
                    }
                }
            });
        }
    }
}
fn set_bond_levels(this: GodUnit) {
    if this.m_data().get_force_type().value != 0 { return; }
    let level = 
        if DVCVariables::is_main_chapter_complete(22) { 
            if to_opt(this.m_data().get_unlock_level_cap_var_name()).is_some_and(|key| GameVariableManager::get_bool(key)) { 20 } else { 10 } 
        } 
        else if DVCVariables::is_main_chapter_complete(20) { 
            if to_opt(this.m_data().get_unlock_level_cap_var_name()).is_some_and(|key| GameVariableManager::get_bool(key)) { 15 } else { 10 } 
        } 
        else if DVCVariables::is_main_chapter_complete(5) { 10 } 
        else { 5 };
    
    PersonPool::PIDS.iter().for_each(|pid|{
        if let Some(unit) = to_opt(UnitPool::get_from_person_2(format!("PID_{}", pid), false)) {
            if let Some(g_bond) = to_opt(this.get_bond(unit)) {
                g_bond.set_m_level(level);
                unit.inherit_aptitude_2(g_bond);
            }
        }
    });
}
pub fn check_fix_god_bonds(this: GodUnit) {
    if this.m_bonds().is_null() {
        let bonds = GodBondHolderPool::get_instance().create_or_get(this.m_data().get_main_data());
        if !bonds.is_null() {
            this.set_m_bonds(bonds);
            set_bond_levels(this);
        }
    }
}