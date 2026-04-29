use unity::prelude::*;
use engage::{map::history::MapHistory, mess::Mess, menu::{BasicMenuItem, BasicMenuItemAttribute}};
use crate::{utils::get_nested_class, config::DVCFlags};
pub fn ironman_code_edits(){
    if DVCFlags::Ironman.get_value() { MapHistory::rewind_disable() } else { MapHistory::rewind_enable() }
}

pub fn vtable_edit(klass: &mut Il2CppClass, vtable_method_name: &str, function: *mut u8) {
    if let Some(v) = klass.get_virtual_method_mut(vtable_method_name) {
        v.method_ptr = function;
    }
}
pub fn map_save_menu_edits() {
    if let Some(temporary_save_item_class) = get_nested_class(Il2CppClass::from_name("App", "MapSystemMenu").unwrap(), "TemporarySaveItem") {
        vtable_edit(temporary_save_item_class, "GetName", map_system_temp_save_menu_name as _);
        vtable_edit(temporary_save_item_class, "GetHelpText", map_system_temp_save_get_help_text as _);
        vtable_edit(temporary_save_item_class, "GetMapAttribute", map_system_temp_save_build_attr as _);
        vtable_edit(temporary_save_item_class, "GetHelpText", map_system_temp_save_get_help_text as _);
    }
    if let Some(sub_menu) = get_nested_class(Il2CppClass::from_name("App", "MapSystemMenu").unwrap(), "SubSystemMenu") {
        if let Some(restart) = get_nested_class(sub_menu, "RestartItem") {
            vtable_edit(restart, "BuildAttribute", restart_menu_item_build_attr as _);
        }
        if let Some(reset) = get_nested_class(sub_menu, "ResetItem") {
            vtable_edit(reset, "BuildAttribute", reset_menu_item_build_attr as _);
        }
    }
}

fn map_system_temp_save_menu_name(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { Mess::get("MID_SORTIE_SAVE") }
fn map_system_temp_save_get_help_text(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { Mess::get("MID_SORTIE_SAVE_HELP") }
fn map_system_temp_save_build_attr(_temp_save_menu_item: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCFlags::Ironman.get_value() { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable }
}
fn restart_menu_item_build_attr(restart_item: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCFlags::Ironman.get_value(){ BasicMenuItemAttribute::Hide } else { unsafe { original_restart_item_build_attr(restart_item, None) } }
}
fn reset_menu_item_build_attr(_reset_item: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCFlags::Ironman.get_value() { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable } 
}
#[skyline::from_offset(0x01b72cb0)]
fn original_restart_item_build_attr(restart_item: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute;
