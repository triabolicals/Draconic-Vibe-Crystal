use engage::menu::{BasicMenuItem, BasicMenuItemAttribute};
use engage::menu::menu_item::sortie::SortieUnitSelectUnitMenuItem;
use engage::sortie::SortieSelectionUnitManager;
use engage::unit::{Gender, UnitPool};
use crate::get_nested_il2cpp_class;
use crate::ironman::vtable_edit;
use crate::utils::get_nested_class;
use super::*;

pub fn sortie_deployment_menu_install() {

    vtable_edit(
        get_nested_il2cpp_class!("App", "SortieUnitSelect", "UnitMenuItem"),
        "BuildAttribute",
        sortie_unit_select_build_attr as _
    );
    if let Some(sortie_top) = Il2CppClass::from_name("App", "SortieTopMenu").ok() {
        if let Some(klass) = get_nested_class(sortie_top, "SelectionUnitMenuItem") {
            vtable_edit(klass, "GetMapAttribute",  sortie_menu_selection_unit_map_attr as _);
        }
        if let Some(klass) = get_nested_class(sortie_top, "GodMenuItem") {
            vtable_edit(klass, "GetMapAttribute",  sortie_menu_god_menu_map_attr as _);
        }
    }
}

pub fn sortie_menu_god_menu_map_attr(_this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    let v = DVCVariables::EmblemDeployment.get_value();
    if v > 0 && v < 3 { BasicMenuItemAttribute::Hide }
    else if UnitPool::get_count(9) == 0 { BasicMenuItemAttribute::Disable }
    else { BasicMenuItemAttribute::Enable  }
}

pub fn sortie_menu_selection_unit_map_attr(_this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if UnitPool::get_count(9) == 0 { BasicMenuItemAttribute::Disable }
    else {
        let unit_deployment_mode = DVCVariables::UnitDeployment.get_value();
        match unit_deployment_mode {
            1|2|5 => BasicMenuItemAttribute::Hide,
            3|6|7 => BasicMenuItemAttribute::Enable,
            _ => {
                let status = GameUserData::get_status().value;
                if status & 12352 == 64 && DVCVariables::Continuous.get_value() == 0 { BasicMenuItemAttribute::Disable }
                else { BasicMenuItemAttribute::Enable }
            }
        }
    }
}

pub fn sortie_unit_select_build_attr(this: &SortieUnitSelectUnitMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    let result = this.build_attribute();
    let mode = DVCVariables::UnitDeployment.get_value();
    if SortieSelectionUnitManager::is_sortie_mode() && mode > 5 {
        let gender = if mode == 7 { Gender::Female } else { Gender::Male };
        if this.unit.as_ref().is_some_and(|u| u.get_gender() != gender) {
            return BasicMenuItemAttribute::Hide ; 
        }
    }
    result
}