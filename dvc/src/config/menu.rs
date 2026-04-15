use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, *},
    proc::{Bindable, ProcInst},
    sequence::configsequence::ConfigSequence,
    titlebar::TitleBar,
};
use engage::game::GameUI;
use engage::gamedata::Gamedata;
use engage::menu::menus::config::{ConfigMenuContent, ConfigRoot};
use engage::resourcemanager::ResourceManager;
use unity::system::List;
pub mod items;
pub(crate) mod kind;
mod text;

pub use text::{DVCConfigText, CONFIG_TEXT};
pub use kind::*;
use crate::config::DVCFlags;
use crate::config::menu::items::DVCConfigMenuItem;
use crate::DVCVariables;
use crate::menus::dvc_header_version;

pub trait DVCCMenuItem {
    fn a_call(&self, _menu_item: &mut DVCConfigMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn minus_call(&self, _menu_item: &mut DVCConfigMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn plus_call(&self, _menu_item: &mut DVCConfigMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn custom_call(&self, _menu_item: &mut DVCConfigMenuItem) -> BasicMenuResult { BasicMenuResult::new() }
    fn build_attribute(&self, _menu_item: &DVCConfigMenuItem) -> BasicMenuItemAttribute { BasicMenuItemAttribute::Enable }
}

pub fn single_class_check() {
    if !DVCVariables::is_main_menu() {
        if DVCFlags::SingleJobEnabled.get_value() || DVCVariables::ClassMode.get_value() == 2 {
            if DVCVariables::get_single_class(false, false).is_some() {
                DVCVariables::ClassMode.set_value(2);
                DVCFlags::SingleJobEnabled.set_value(true);
            }
            else {
                DVCVariables::ClassMode.set_value(0);
                DVCFlags::SingleJobEnabled.set_value(false);
            }
        }
    }
}

pub fn create_dvc_bind<B: Bindable>(proc: &B) -> bool {
    let root = GameUI::get_root();
    if let Some(content) = ResourceManager::instantiate2("UI/Common/Config/Prefabs/ConfigRoot", root)
        .and_then(|go| go.get_component_by_type::<ConfigRoot>())
        .and_then(|config_root| config_root.config_menu_content_object.get_component_by_type::<ConfigMenuContent>())
    {
        single_class_check();
        let list = List::<DVCConfigMenuItem>::with_capacity(20).unwrap();
        DVCMenu::reset_select();
        DVCMenu::Main.get_items().iter().for_each(|k|{
            let item = DVCConfigMenuItem::new_kind(*k);
            item.menu_kind = DVCMenu::Main;
            list.add(item);
        });
        let row_count = content.get_menu_item_content_max();
        let menu = BasicMenu::new_with_content(list, content);
        let default_descs = menu.create_default_desc();
        menu.create_bind(proc, default_descs, "DVCConfigMenu");
        menu.set_show_row_num(row_count - 1);
        true
    }
    else { true }
}

extern "C" fn create_dvc_config_menu_test(this: &mut ConfigSequence, _method_info: OptionalMethod) {
    create_dvc_bind(this);
    dvc_header_version();
}

extern "C" fn config_sequence_end(this: &mut ConfigSequence, _method_info: OptionalMethod) {
    this.end_sequence();
    // if parent of ConfigSequence is a Menu then re-open the menu. BasicMenus have virtual method OpenAnimeAll
    if let Some(parent) = this.get_parent() {
        parent.klass.get_virtual_method("OpenAnimeAll").map(|method| {
            let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
            open_anime_all(parent, method.method_info);
        });
        // Change header based on what the parent proc is
    }
}
pub fn dvc_ng_menu_create_bind<B: Bindable>(this: &B){
    ConfigSequence::create_bind(this);
    // Replacing
    if let Some(descs) = this.get_child().map(|child| child.get_descs_mut()){
        // Desc[4] Method for creating the ConfigMenu
        if let Some(create_menu) = descs.get_mut(4)
            .and_then(|d| d.cast_to_method_call_mut())
        {
            create_menu.function.method_ptr = create_dvc_config_menu_test as _;
        }
        // Desc[5] Method ConfigSequence$$EndSequence
        if let Some(end_sequence) = descs.get_mut(5)
            .and_then(|d| d.cast_to_method_call_mut())
        {
            end_sequence.function.method_ptr =  config_sequence_end as _;
        }
    }
}