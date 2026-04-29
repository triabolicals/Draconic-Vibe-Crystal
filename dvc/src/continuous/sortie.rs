use super::*;
use engage::{
    sequence::hub::HubSequence, util::get_singleton_proc_instance,
    menu::{BasicMenuItem, BasicMenuItemAttribute, BasicMenuResult}
};
use crate::ironman::vtable_edit;

#[unity::class("App", "HubMenuSequence")]
pub struct HubMenuSequence {}
impl Bindable for HubMenuSequence {}

pub fn next_chapter_a_call(this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
    let mode = DVCVariables::Continuous.get_value();
    if mode == 1 || mode == 2 {
        if let Some(chapter) = ChapterData::try_get_hash(GameVariableManager::get_number("G_DVC_Next")){
            GameUserData::cleanup_for_chapter();
            HubSequence::next_chapter(chapter.cid);
            if let Some(hub_menu_sequence) = get_singleton_proc_instance::<HubMenuSequence>() {
                ProcInst::jump(hub_menu_sequence, 9);
            }
            return  BasicMenuResult::se_decide().with_close_all(true);
        }
    }
    unsafe { next_chapter_original_a_call(this, optional_method) }
}
pub fn next_chapter_get_name(_this: &BasicMenuItem, _optional_method: OptionalMethod) -> &'static Il2CppString {
    let mode = DVCVariables::Continuous.get_value();
    if mode == 1 || mode == 2 {
        if let Some(chapter) = ChapterData::try_get_hash(GameVariableManager::get_number("G_DVC_Next")){
            let prefix = Mess::get(format!("{}_PREFIX", chapter.name));
            let name = Mess::get(chapter.name);
            let count = get_continious_total_map_complete_count();
            return format!("{}: {} [#{}]", prefix, name, count+1).into()
        }
    }
    Mess::get("MID_Hub_Next_Go")
}

pub fn sortie_top_menu_back_get_map_attr(this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    let c_mode = DVCVariables::Continuous.get_value();
    if c_mode == 1 || c_mode == 2 { BasicMenuItemAttribute::Hide }
    else { unsafe { sortie_top_menu_back_get_map_attr_original(this, None) } }
}

pub fn sortie_top_menu_base_b_call(this: &BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let c_mode = DVCVariables::Continuous.get_value();
    if c_mode == 1 || c_mode == 2 { BasicMenuResult::new() }
    else { unsafe { sortie_top_menu_base_b_call_original(this, None) } }
}

pub fn sortie_continious_menu_install() {
    let sortie_top_menu_class = Il2CppClass::from_name("App", "SortieTopMenu").unwrap();
    if let Some(cc) = sortie_top_menu_class.get_nested_types().iter().find(|x| x.get_name() == "BackMenuItem") {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("GetMapAttribute").map(|method| method.method_ptr = sortie_top_menu_back_get_map_attr as _);
    }
    if let Some(cc) = sortie_top_menu_class.get_nested_types().iter()
        .find(|x| x.get_name() == "SortieTopMenuBase"){
            let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
            menu_mut.get_virtual_method_mut("BCall").map(|method| method.method_ptr = sortie_top_menu_base_b_call as _);
        }
    let dialog_next_class = Il2CppClass::from_name("App", "DialogItemNextChapter").unwrap();
    vtable_edit(dialog_next_class, "ACall", next_chapter_a_call as _);
    vtable_edit(dialog_next_class, "GetName", next_chapter_get_name as _);
}

#[skyline::from_offset(0x01d76320)]
fn sortie_top_menu_back_get_map_attr_original(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute;

#[skyline::from_offset(0x01d78d40)]
fn sortie_top_menu_base_b_call_original(this: &BasicMenuItem, method_info: OptionalMethod) -> BasicMenuResult;

#[skyline::from_offset(0x1cecaa0)]
fn next_chapter_original_a_call(this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult;