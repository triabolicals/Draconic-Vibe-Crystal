
use super::*;
use engage::{
    app::{
        godunitmenuitem::*, godunitselectmenuitem::*,
        telopmanager::*, ringselectmenu::*, resourceobject::*, resourcemanager_2::*,
        IGodUnit,
        IGodUnitMethods,
        IRingSelectMenuItemContent,
        IUnitMenuItemSetter,
        RingSelectMenuItemContent,
        RingSelectMenu_RingMenuItem,
        BasicMenuItem,
        IProcInst,
        ISortieUnitSelect_UnitMenuItem,
        ISpriteAtlasManager_2Methods,
        IUnitSelectMenuItemContent,
        ProcInst,
        SortieUnitSelect_UnitMenuItem, UnitSelectMenuItemContent
    },
    unity_engine::ui::image::*,
    ProcVoidMethodExt,
};
mod bondlevel;
// mod engagepair;
// mod engagequartett;
use crate::{dvc::asset::AppearanceManager, utils::{override_vtable, to_opt}};
use crate::DVCFlags::GodNames;

pub fn override_facethumb_menu_item_build() {
    override_vtable("App", "RingSelectMenuItemContent", "Build", ring_select_menu_item_content_build as _);
    override_vtable("App", "GodUnitSelectMenuItem", "Build", god_select_menu_content_build as _);
    override_vtable("App", "UnitSelectMenuItemContent", "Build", unit_select_menu_item_content_build as _);
}

pub fn telop_proc_edit(this: ProcInst) {
    if bondlevel::telop_bond_level_desc_edit(this) { return; }
    // if engagepair::telop_engage_pair_desc_edit(this) { return; }
    // engagequartett::telop_engage_quartette_edit(this);
}
pub fn unit_select_menu_item_content_build(this: UnitSelectMenuItemContent, menu_item: BasicMenuItem, _: OptionalMethod) {
    unsafe { UnitSelectMenuItemContent::build(this, menu_item) };
    if let Some(unit_menu_item) = menu_item.try_cast::<SortieUnitSelect_UnitMenuItem>() {
        let unit = unit_menu_item.m_unit();
        let person = unit.get_person();
        if let Some(replacement) = PersonPool::get_enemy_player_replacement(person, true) {
            if let Some(sprite) = to_opt(FaceThumbnail::get_2(replacement)) { this.m_setter().m_face().set_m_sprite(sprite); }
            else { this.m_setter().m_face().set_m_sprite(FaceThumbnail::s_face_thumb().get("Phantom")); }
        }
    }
}
pub fn get_bond_face(this: Unit) -> Il2CppString {
    if AppearanceManager::is_playable_rand_app(this.get_person().hash()) {
        if let Some(app) = AppearanceManager::get_appearance_person(this.get_person(), Some(this.get_dress_gender().value == 2))
            .and_then(|a| a.get_bond_face_path())
        {
            return app.into();
        }
    }
    if this.get_person().index() == 1 {
        if this.is_female() { "Telop/LevelUp/FaceThumb/LueurW".into() } else { "Telop/LevelUp/FaceThumb/Lueur".into() }
    }
    else if this.is_hero() {
        let ascii = this.get_person().get_ascii_name();
        format!("Telop/LevelUp/FaceThumb/{}", ascii).into()
    }
    else { TelopManager::get_bond_level_face_path(this) }
}
pub fn get_god_face(this: GodData) -> Il2CppString {
    if GodNames.get_value() {
        if let Some(app) = AppearanceManager::get_appearance_emblem(this.hash(), false).and_then(|a| a.get_bond_face_path()){
            return app.into();
        }
    }
    if this.is_hero() {
        if let Some(p) = PersonPool::get_replacement(0, false, true){
            let path = format!("Telop/LevelUp/FaceThumb/{}",p.get_ascii_name());
            if ResourceManager_2::file_exists(path.as_str()) { return path.into(); }
        }
        if DVCVariables::is_lueur_female() { "Telop/LevelUp/FaceThumb/GodLueurW".into() } else { "Telop/LevelUp/FaceThumb/GodLueur".into() }
    }
    else { TelopManager::get_bond_level_face_path_3(this) }
}

pub fn set_god_face(image: Image, data: GodData) {
    if GodNames.get_value() || (data.is_hero() && PersonPool::get_replacement_index(0, false).is_some_and(|v| v != 0)){
        if let Some(app) = AppearanceManager::get_appearance_emblem(data.hash(), false).and_then(|app| app.get_thumbnail_key()) {
            image.set_sprite(app);
        }
    }
}

pub fn ring_select_menu_item_content_build(this: RingSelectMenuItemContent, menu_item: RingSelectMenu_RingMenuItem, _: OptionalMethod) {
    if let Some(method) = this.get_class().raw().get_method_from_name("Build", 1){
        let build_invoke = unsafe { std::mem::transmute::<_, fn(RingSelectMenuItemContent, RingSelectMenu_RingMenuItem, &MethodInfo)>(method.method_ptr) };
        build_invoke(this, menu_item, method);
    }
    if menu_item.m_is_god() {
        let god = menu_item.get_god();
        if !god.is_null() { set_god_face(this.m_face(), god.m_data()); }
    }
}

pub fn god_select_menu_content_build(this: GodUnitSelectMenuItem, menu_item: GodUnitMenuItem, _optional_method: OptionalMethod) {
    if let Some(method) = this.get_class().raw().get_method_from_name("Build", 1){
        let build_invoke = unsafe { std::mem::transmute::<_, fn(GodUnitSelectMenuItem, GodUnitMenuItem, &MethodInfo)>(method.method_ptr) };
        build_invoke(this, menu_item, method);
    }
    let god = menu_item.get_optional_god_data();
    if !god.is_null() { set_god_face(this.m_setter().m_face(), god); }
}




