use unity::prelude::*;
use engage::{
    dialog::yesno::*,
    menu::{BasicMenuResult, *},
    pad::Pad, proc::{Bindable, ProcInst},
    titlebar::TitleBar, 
    util::get_instance
};
use engage::gamedata::Gamedata;
use engage::menu::menu_item::config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods};
use engage::menu::menus::config::ConfigMenu;
use engage::mess::Mess;
use crate::{deployment, continuous, randomizer, get_nested_il2cpp_class};
use crate::config::menu::create_dvc_bind;
use crate::ironman::vtable_edit;
use crate::utils::{get_nested_class, return_4};
use super::{DVCVariables, CONFIG};
pub mod ingame;
use crate::menus::ingame::draconic_vibe_name;

/*
pub extern "C" fn open_anime_parent(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnime").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
}

pub extern "C" fn close_dvc_menu(this: &mut ProcInst, _method_info: OptionalMethod) {
    if let Some(parent) = this.parent.as_ref() {
        if parent.name.unwrap().to_string().contains("Config") {
            TitleBar::open_header(
                Mess::get("MID_MENU_CONFIG").to_string().as_str(), 
                Mess::get("MID_MENU_CONFIG_HELP").to_string().as_str(), 
                "KHID_環境設定_項目選択"
            );
        }
        open_anime_parent(this, None);
        return;
    }
    open_anime_all_ondispose(this, None);
}
fn add_dvc_menu_options(config_menu: &mut ConfigMenu<DVCConfigMenuItem>){
    let s = if GameUserData::get_sequence() == 0 { 17 } else { 13 };
    for x in 1..s {
        if let Some(item) = DVCConfigMenuItem::new(x) {
            config_menu.add_item(item);
        }
    }

}
 */

pub struct TriabolicalMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "All will be Revealed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.help_text = "Open up the Draconic Vibe Crystal settings.".into(); }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if create_dvc_bind(this.menu) {
            this.menu.close_anime_all();
            BasicMenuResult::close_decide()
        }
        else { BasicMenuResult::se_miss() }
    }
}

extern "C" fn vibe() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("{} {}", draconic_vibe_name(), super::VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalMenu>(title)
}
pub struct WriteOutputConfirm;
impl TwoChoiceDialogMethods for WriteOutputConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomizer::write_seed_output_file();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub fn install_vibe() {
    cobapi::install_global_game_setting(vibe);
    cobapi::install_game_setting(ingame::vibe2);
}

pub fn save_config_settings(_this: &BasicMenu<BasicMenuItem>, _method_info: OptionalMethod) -> i32 {
    if DVCVariables::is_main_menu() {crate::DeploymentConfig::get().save(); }
    return 0x201;
}

pub fn menu_calls_install() {
    if let Some(cc) = get_nested_class(Il2CppClass::from_name("App", "HubPlayTalkAfter").unwrap(), "CookingMenu"){
        vtable_edit(cc, "BuildAttribute", crate::message::cooking_menu_build_attribute as _ );
    }
    if let Some(cc) = Il2CppClass::from_name("App", "ClassChangeJobMenu").ok() {
        vtable_edit(cc, "AfterBuild", randomizer::skill::learn::class_change_job_menu_after_build as _);
        if let Some(item) = get_nested_class(cc, "ClassChangeJobMenuItem") {
            vtable_edit(item, "ACall", randomizer::job::reclass::class_change_a_call_random_cc as _);
            vtable_edit(item, "BuildAttribute", randomizer::job::reclass::class_change_job_menu_item_build_attr as _);
            vtable_edit(item, "OnSelect", randomizer::skill::learn::class_change_job_menu_item_on_select as _);
            vtable_edit(item, "CustomCall", randomizer::skill::learn::class_change_job_menu_item_custom_call as _);
        }
    }
    // vtable_edit(Il2CppClass::from_name("App", "GmapSequence").unwrap(), "OnCreate", crate::procs::gmap_sequence_on_create as _);
    let enter_chapter = get_nested_il2cpp_class!("App", "GmapMenuSequence", "GmapMenu", "EnterChapterItem");
    vtable_edit(enter_chapter, "BuildAttribute", crate::procs::enter_chapter_build_attribute as _);

    vtable_edit( get_nested_il2cpp_class!("App", "GmapMenuSequence", "GmapMenu", "RankingItem"), "ACall", crate::procs::enter_chapter_build_attribute as _);
    randomizer::item::shop::random_shop_install();
    vtable_edit(Il2CppClass::from_name("", "ConfigMenu").unwrap(), "BCall", my_b_call as _);
    if let Some(cc) = get_nested_class(Il2CppClass::from_name("App", "MapItemMenu").unwrap(), "NextItem"){
        vtable_edit(cc, "GetHelpText", continuous::hub_menu_next_help_text as _);
    }
    if let Some(cc) = get_nested_class(Il2CppClass::from_name("App", "RingListSkillMenu").unwrap(), "MenuItem_WeaponTalent") {
       vtable_edit(cc, "BuildAttribute", randomizer::emblem::menu::weapon_talent_build_attr as _);
        // vtable_edit(cc, "BuildAttribute", randomizer::job::reclass::class_change_a_call_random_cc as _);
    }
    deployment::sortie::sortie_deployment_menu_install();
    continuous::sortie::sortie_continious_menu_install();
    crate::ironman::map_save_menu_edits();
    crate::sprite::install_sprite_menu_methods();
}
pub fn my_b_call(this: &ConfigMenu<()>, _: OptionalMethod) -> BasicMenuResult {
    BasicMenuResult::close_cancel()
}
pub fn show(_this: &BasicMenuItem, _optional_method: OptionalMethod) -> i32 { 1 }
pub fn dvc_header_version() { TitleBar::open_header(draconic_vibe_name(), super::VERSION, ""); }


