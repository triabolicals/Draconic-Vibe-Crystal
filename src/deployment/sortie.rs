use engage::menu::{BasicMenuItem, BasicMenuItemAttribute};
use super::*;

pub fn sortie_menu_god_menu_map_attr(_this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number(DVCVariables::EMBLEM_DEPLOYMENT_KEY) > 0 { BasicMenuItemAttribute::Hide }
    else if UnitPool::get_count(9) == 0 { BasicMenuItemAttribute::Disable }
    else { BasicMenuItemAttribute::Enable  }
}

pub fn sortie_menu_selection_unit_map_attr(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_bool("UnitDeploy") { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) >= 3 { BasicMenuItemAttribute::Enable  }
    else {  unsafe { original_unit_selection_map_attr(this, method_info) } }
}

pub fn sortie_deployment_menu_install() {
    let sortie_top_menu_class = Il2CppClass::from_name("App", "SortieTopMenu").unwrap();
    if let Some(cc) = sortie_top_menu_class.get_nested_types().iter().find(|x| x.get_name() == "SelectionUnitMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("GetMapAttribute").map(|method| method.method_ptr = sortie_menu_selection_unit_map_attr as _);
    }
    if let Some(cc) = sortie_top_menu_class.get_nested_types().iter().find(|x| x.get_name() == "GodMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("GetMapAttribute").map(|method| method.method_ptr = sortie_menu_god_menu_map_attr as _);
    }
}

#[skyline::from_offset(0x01d78400)]
fn original_unit_selection_map_attr(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute;