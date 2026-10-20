use engage::{app::{BasicMenuItem, BasicMenuItem_Attribute}, Mess};
use unity::{Il2CppString, OptionalMethod};
use crate::config::DVCFlags;
use crate::utils::override_vtable;

pub fn iron_man_vtable_edit() {
    let namespace = "App";
    let class = "MapSystemMenu.TemporarySaveItem";
    override_vtable(namespace, class, "GetName", map_system_temp_save_menu_name as _);
    override_vtable(namespace, class, "GetHelpText", map_system_temp_save_get_help_text as _);
    override_vtable(namespace, class, "GetMapAttribute", map_system_temp_save_build_attr as _);
    override_vtable(namespace, "MapSystemMenu.SubSystemMenu.RestartItem", "BuildAttribute", restart_menu_item_build_attr as _);
    override_vtable(namespace, "MapSystemMenu.SubSystemMenu.ResetItem", "BuildAttribute", map_system_temp_save_build_attr as _);
}

fn map_system_temp_save_menu_name(_temp_save_menu_item: BasicMenuItem, _: OptionalMethod) -> Il2CppString { Mess::get("MID_SORTIE_SAVE") }
fn map_system_temp_save_get_help_text(_temp_save_menu_item: BasicMenuItem, _: OptionalMethod) -> Il2CppString { Mess::get("MID_SORTIE_SAVE_HELP") }
fn map_system_temp_save_build_attr(_temp_save_menu_item: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItem_Attribute {
    if DVCFlags::Ironman.get_value() { BasicMenuItem_Attribute::hide() } else { BasicMenuItem_Attribute::enable() }
}
fn restart_menu_item_build_attr(restart_item: BasicMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    if DVCFlags::Ironman.get_value(){ BasicMenuItem_Attribute::hide() } else { unsafe { original_restart_item_build_attr(restart_item, None) } }
}
#[skyline::from_offset(0x01b72cb0)]
fn original_restart_item_build_attr(restart_item: BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItem_Attribute;
