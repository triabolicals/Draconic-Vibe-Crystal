use engage::gamedata::GodData;
use engage::god::GodUnit;
use engage::menu::{BasicMenu, BasicMenuItem};
use engage::menu::content::common::UnitMenuItemSetter;
use engage::menu::menu_item::BasicMenuItemContentFields;
use engage::spriteatlasmanager::FaceThumbnail;
use engage::tmpro::TextMeshProUGUI;
use engage::unit::UnitRing;
use unity::engine::ui::{Image, IsImage};
use unity::prelude::*;
use crate::config::DVCFlags;
use crate::DVCVariables;
use crate::randomizer::data::GameData;
use crate::randomizer::names::AppearanceRandomizer;

#[unity::class("App", "RingSelectMenuItemContent")]
pub struct RingSelectMenuItemContent {
    pub parent: BasicMenuItemContentFields,
    pub name: &'static mut TextMeshProUGUI,
    pub empty_text: &'static mut TextMeshProUGUI,
    pub face: &'static mut Image,
    pub symbol: &'static mut Image,
    pub select_bg: &'static mut Image,
    pub frame: &'static mut Image,
}
#[repr(C)]
pub struct RingMenuItem {
    klass: &'static Il2CppClass,
    monitor: u64,
    pub menu: &'static mut BasicMenu<BasicMenuItem>,
    pub menu_item_content: &'static mut RingSelectMenuItemContent,
    pub name: &'static Il2CppString,
    pub index: i32,
    full_index: i32,
    pub attribute: i32,
    pub cursor_color: unity::engine::Color,
    pub active_text_color: unity::engine::Color,
    pub inactive_text_color: unity::engine::Color,
    pub is_god: bool,
    pub god_unit: Option<&'static GodUnit>,
    pub ring: Option<&'static UnitRing>,
    pub ring_sort: i32,
}
#[unity::class("App", "GodUnitMenuItem")]
pub struct GodUnitMenuItem {}
impl GodUnitMenuItem {
    #[unity::class_method(7)] pub fn get_god_name(&self) -> &'static Il2CppString; // Offset: 0x23472D0 Flags: 0
    #[unity::class_method(8)] pub fn get_optional_god_data(&self) -> Option<&'static GodData>; // Offset: 0x23473F0 Flags: 0
}

#[unity::class("App", "GodUnitSelectMenuItem")]
pub struct GodUnitSelectMenuItem {
    pub parent: BasicMenuItemContentFields,
    pub setter: &'static mut UnitMenuItemSetter,
}

pub fn set_god_face(image: &Image, data: &GodData) {
    if DVCFlags::GodNames.get_value() {
        let sprite =
            if DVCFlags::GodNames.get_value() {
                GameData::get_playable_emblem_hashes().iter().position(|&x| x == data.parent.hash)
                    .and_then(|x| AppearanceRandomizer::get_emblem_app_person_index(x as i32))
                    .and_then(|p| FaceThumbnail::get_from_person(p.1))
            }
            else if data.gid.str_contains("リュール") {
                DVCVariables::get_dvc_person_data(0, false)
                    .filter(|p| p.parent.index > 1)
                    .and_then(|p| FaceThumbnail::get_from_person(p))
            }
            else { None };
        if let Some(sprite) = sprite { image.set_sprite2(sprite); }
    }
}
pub fn ring_select_menu_item_content_build(this: &mut RingSelectMenuItemContent, menu_item: &RingMenuItem, _optional_method: OptionalMethod) {
    if let Some(method) = this.klass.get_method_from_name("Build", 1).ok() {
        let build_invoke = unsafe { std::mem::transmute::<_, fn(&mut RingSelectMenuItemContent, &RingMenuItem, &MethodInfo)>(method.method_ptr) };
        build_invoke(this, menu_item, method);
    }
    if menu_item.is_god { if let Some(g_unit) = menu_item.god_unit.as_ref(){ set_god_face(this.face, g_unit.data); } }
}

pub fn god_select_menu_content_build(this: &mut GodUnitSelectMenuItem, menu_item: &GodUnitMenuItem, _optional_method: OptionalMethod) {
    if let Some(method) = this.klass.get_method_from_name("Build", 1).ok() {
        let build_invoke = unsafe { std::mem::transmute::<_, fn(&mut GodUnitSelectMenuItem, &GodUnitMenuItem, &MethodInfo)>(method.method_ptr) };
        build_invoke(this, menu_item, method);
    }
    if let Some(data) = menu_item.get_optional_god_data() { set_god_face(this.setter.face, data); }
}