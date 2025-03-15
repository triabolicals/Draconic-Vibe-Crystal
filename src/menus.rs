use unity::prelude::*;
use engage::{
    dialog::yesno::*, gamevariable::*, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult, *},
    pad::Pad, proc::{desc::ProcDesc, Bindable, ProcInst, ProcVoidMethod},
    sequence::configsequence::ConfigSequence, 
    sortie::SortieTopMenuManager, 
    titlebar::TitleBar, 
    util::get_instance
};
use crate::{autolevel, deployment, continuous, ironman, randomizer};
use super::{DVCVariables, CONFIG};

pub mod ingame;
pub mod global;
pub mod submenu;
pub mod buildattr;

extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
}
// Functions to hide the option when conditions are met

fn add_dvc_menu_options(config_menu: &mut ConfigMenu<ConfigBasicMenuItem>){
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<continuous::ContiniousMode>("Continuous/Map Modes"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::RecruitmentSubMenu>("Deployment / Recruitment"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::EmbelmSubMenu>("Emblem Randomization"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::ClassSubMenu>("Class / Skills Randomization"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::ItemSubMenu>("Item Randomization"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::EnemySubMenu>("Enemy Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::grow::RandomGrowMod>("Random Growth Mode"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::grow::PersonalGrowMode>("Personal Growth Mode"));
    config_menu.add_item(randomizer::terrain::menu::vibe_energy());
    config_menu.add_item(randomizer::terrain::menu::vibe_fow());
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<autolevel::menu::AutolevelMod>("Level Scale Units")); 
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::bgm::RandomBGMMod>("Map BGM Setting")); 
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::assets::accessory::RandomAssets>("Random Assets"));
    config_menu.add_item(randomizer::map::vibe_tile());
}

pub struct TriabolicalMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalMenu {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.a() {
            if pad_instance.npad_state.buttons.a() {
            // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
                let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
                    close_anime_all(this.menu, method.method_info);
                });
                ConfigMenu::create_bind(this.menu);
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();

                config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
                config_menu.full_menu_item_list.clear();
                add_dvc_menu_options(config_menu);
                BasicMenuResult::se_cursor()
            }   
            else { BasicMenuResult::new() }
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "All will be Revealed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.help_text = "Open up the Draconic Vibe Crystal settings.".into(); }
}

extern "C" fn vibe() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalMenu>(title)
}

pub struct WriteOutputConfirm;
impl TwoChoiceDialogMethods for WriteOutputConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomizer::write_seed_output_file();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}


pub fn install_vibe() { 
    cobapi::install_global_game_setting(global::vibe_stats);
    cobapi::install_global_game_setting(global::vibe_enable);
    cobapi::install_global_game_setting(global::vibe_post_save);
    cobapi::install_global_game_setting(vibe); 
    cobapi::install_game_setting(ingame::vibe2);
}

pub fn save_config_settings(_this: &BasicMenu<BasicMenuItem>, _method_info: OptionalMethod) -> i32 {
    if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().save(); }
    return 0x201;
}

pub fn menu_calls_install() {
    ingame::dvc_minus_calls();
    let cooking_menu = Il2CppClass::from_name("App", "HubPlayTalkAfter").unwrap().get_nested_types().iter().find(|x| x.get_name() == "CookingMenu").unwrap();
    let cooking_menu_mut = Il2CppClass::from_il2cpptype(cooking_menu.get_type()).unwrap();
    cooking_menu_mut.get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::message::cooking_menu_build_attribute as _);
    println!("Replaced Virtual Method of CookingMenu");

    if let Some(cc) = Il2CppClass::from_name("App", "ClassChangeJobMenu").unwrap().get_nested_types().iter().find(|x| x.get_name() == "ClassChangeJobMenuItem"){
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("ACall").map(|method| method.method_ptr = crate::randomizer::job::class_change_a_call_random_cc as _);
        println!("Replaced ACall of ClassChangeJobMenuItem");
    }
    if let Some(cc) = Il2CppClass::from_name("App", "HubMenu").unwrap().get_nested_types().iter().find(|x| x.get_name() == "NextItem"){
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("GetHelpText").map(|method| method.method_ptr = crate::continuous::hub_menu_next_help_text as _);
    }

    crate::deployment::sortie::sortie_deployment_menu_install();
    crate::continuous::sortie::sortie_continious_menu_install();

}

extern "C" fn create_dvc_config_menu_test(this: &mut ConfigSequence, _method_info: OptionalMethod) {
    // Initialize the menu
    ConfigMenu::create_bind(this);
    let config_menu = this.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();

    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(global::vibe_enable());
    config_menu.add_item(global::vibe_dlc());
    config_menu.add_item(randomizer::vibe_seed());
    add_dvc_menu_options(config_menu);
    
    TitleBar::open_header("Draconic Vibe Crystal", super::VERSION, "");
}

pub fn dvc_ng_menu_create_bind(this: &ProcInst) {

    ConfigSequence::create_bind(this);

    let create_global_config_menu = ProcVoidMethod::new(None, create_dvc_config_menu_test);

    // Replace CreateConfigMenu by our own implementation
    this.child.as_ref().unwrap().get_descs_mut().get_mut(4)
        .map(|call| *call = ProcDesc::call(create_global_config_menu))
        .unwrap();

}