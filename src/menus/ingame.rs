use crate::VERSION;
use super::*;

fn add_in_game_dvc_menu_items(config_menu: &mut ConfigMenu<ConfigBasicMenuItem>) {
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::AssetSubMenu>("Asset Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::ItemSubMenu>("Item Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::ClassSubMenu>("Class/Skill Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::EmbelmSubMenu>("Emblem Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::RecruitmentSubMenu>("Player Unit Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_command::<submenu::EnemySubMenu>("Enemy Unit Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::RandomDeploySpots>("Random Deployment Spots"));
    config_menu.add_item(randomizer::job::menu::vibe_job_rerand());
    config_menu.add_item(randomizer::bgm::vibe_bgm());
    config_menu.add_item(randomizer::map::vibe_tile());
    config_menu.add_item(randomizer::terrain::menu::vibe_energy());
    config_menu.add_item(randomizer::terrain::menu::vibe_fow());
}

pub struct TriabolicalInGameMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalInGameMenu {
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
                add_in_game_dvc_menu_items(config_menu);

                config_menu.add_item(randomizer::vibe_reseed());
                TitleBar::open_header("Draconic Vibe Crystal", super::super::VERSION, "");
                BasicMenuResult::se_cursor()
            }   
            else { BasicMenuResult::new() }
        }
        else if pad_instance.npad_state.buttons.plus() {
            output_bind(this, None);
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text =
            if GameVariableManager::get_number("G_Random_Seed") == 0 {
                "Open up In-Game Draconic Vibe Crystal settings.".to_string()
            }
            else { format!("Press + to Create Output. Seed: {}", GameVariableManager::get_number(DVCVariables::SEED)) }.into();
    }
}

pub extern "C" fn vibe2() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::super::VERSION);
    let command = ConfigBasicMenuItem::new_command::<TriabolicalInGameMenu>(title);
    command
}
fn output_bind(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
    if DVCVariables::random_enabled() {
        let text = format!("Create Output File for this Save?\n Save as 'sd:/Draconic Vibe Crystal/{}.log'",  crate::utils::get_player_name());
        YesNoDialog::bind::<WriteOutputConfirm>(this.menu, text, "Do it!", "Nah..");
    }
}
pub fn dvc_in_game_menu_create_bind(this: &mut BasicMenuItem, _method_info: OptionalMethod) ->  BasicMenuResult {
    this.menu.close_anime_all();
    // Initialize the menu
    ConfigSequence::create_bind(this.menu);

    // Register a OnDispose callback to restore the previous menu
    this.menu.proc.child.as_mut().unwrap().get_class_mut().get_virtual_method_mut("OnDispose")
        .map(|method| method.method_ptr = open_anime_all_ondispose as _)
        .unwrap();

    let create_global_config_menu = ProcVoidMethod::new(None, create_dvc_config_menu);

    // Replace CreateConfigMenu by our own implementation
    this.menu.proc.child.as_mut() .unwrap().get_descs_mut().get_mut(4)
        .map(|call| *call = ProcDesc::call(create_global_config_menu))
        .unwrap();


    BasicMenuResult::se_decide()
}

#[skyline::from_offset(0x0253a650)]
pub fn config_sequeunce_create_bint<P: Bindable + Sized>(proc: &P, method_info: OptionalMethod);

#[skyline::from_offset(0x0253a470)]
pub fn create_config_menu(this: &ConfigSequence, method_info: OptionalMethod);

extern "C" fn create_dvc_config_menu(this: &mut ConfigSequence, _method_info: OptionalMethod) {
    ConfigMenu::create_bind(this);
    let config_menu = this.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.full_menu_item_list.clear();
    add_in_game_dvc_menu_items(config_menu);
    TitleBar::open_header("Draconic Vibe Crystal", super::super::VERSION, "");
}

#[skyline::from_offset(0x02671300)]
pub fn map_sequence_sortie_human_create_sys_menu(map_sequence: &mut ProcInst, method_info: OptionalMethod);
#[skyline::from_offset(0x024eb880)]
fn sortie_top_menu_create_bind(proc: &ProcInst, method_info: OptionalMethod);


pub extern "C" fn map_system_add_dvc_add(proc: &mut ProcInst) {
    let top_menu = proc.cast_mut::<BasicMenu<BasicMenuItem>>();
    if let Some(menu) = top_menu.full_menu_item_list
        .iter_mut().find(|x| x.get_class().get_name() == "SystemMenuItem")
    {
        let menu_item_class = menu.get_class_mut().clone();
        let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(menu_item_class.clone()).unwrap();
        new_menu_item.get_class_mut().get_virtual_method_mut("GetName")
            .map(|m| m.method_ptr = dvc_name as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("ACall")
            .map(|m| m.method_ptr = dvc_in_game_menu_create_bind as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("GetHelpText").map(|m|
            m.method_ptr = dvc_help as _);
        new_menu_item.get_class_mut().get_virtual_method_mut("PlusCall").map(|m|
            m.method_ptr = output_bind as _ );

        top_menu.full_menu_item_list.insert(2, new_menu_item);
        let items = top_menu.full_menu_item_list.len() as i32;
        top_menu.set_show_row_num(items);
    }
}
pub fn dvc_name(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    format!("DVC v. {}", VERSION).into()
}
pub fn dvc_help(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    engage::mess::Mess::get("MID_MENU_H_CONFIG")
}