use super::*;
use engage::menu::{BasicMenuItem, BasicMenuItemAttribute};

pub fn sortie_top_menu_back_get_map_attr(this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    let c_mode = GameVariableManager::get_number(DVCVariables::CONTINIOUS);
    if c_mode > 0 && c_mode < 4 {
        BasicMenuItemAttribute::Hide
    }
    else { unsafe { sortie_top_menu_back_get_map_attr_original(this, None) } }
}

pub fn sortie_top_menu_base_b_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let c_mode = GameVariableManager::get_number(DVCVariables::CONTINIOUS);
    if c_mode > 0 && c_mode < 4 { BasicMenuResult::new() }
    else { unsafe { sortie_top_menu_base_b_call_original(this, None) } }
}

pub fn sortie_continious_menu_install() {
    let sortie_top_menu_class = Il2CppClass::from_name("App", "SortieTopMenu").unwrap();
    if let Some(cc) = sortie_top_menu_class.get_nested_types().iter().find(|x| x.get_name() == "BackMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("GetMapAttribute").map(|method| method.method_ptr = sortie_top_menu_back_get_map_attr as _);
    }
    sortie_top_menu_class.get_nested_types().iter()
        .for_each(|class|{
            let menu_mut = Il2CppClass::from_il2cpptype(class.get_type()).unwrap();
            menu_mut.get_virtual_method_mut("BCall").map(|method| method.method_ptr = sortie_top_menu_base_b_call as _);
        }
    );
}

#[skyline::from_offset(0x01d76320)]
fn sortie_top_menu_back_get_map_attr_original(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute;

#[skyline::from_offset(0x01d78d40)]
fn sortie_top_menu_base_b_call_original(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuResult;