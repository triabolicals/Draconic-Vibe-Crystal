use super::*;

fn add_in_game_dvc_menu_items(config_menu: &mut ConfigMenu<ConfigBasicMenuItem>) {
    config_menu.add_item(deployment::vibe_deployment());
    config_menu.add_item(deployment::vibe_emblem_deployment());
    config_menu.add_item(randomizer::terrain::menu::vibe_energy());
    config_menu.add_item(randomizer::terrain::menu::vibe_fow());
    config_menu.add_item(randomizer::assets::accessory::vibe_enemy_outfit());
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::assets::accessory::RandomAssets>("Randomized Assets"));
    config_menu.add_item(randomizer::job::vibe_job_rerand());
    config_menu.add_item(randomizer::item::unit_items::vibe_prw());
    config_menu.add_item(randomizer::assets::bust::vibe_bust());
    config_menu.add_item(randomizer::names::vibe_generic());
    config_menu.add_item(randomizer::grow::vibe_pgmode());
    config_menu.add_item(autolevel::menu::vibe_autolevel());
    config_menu.add_item(autolevel::menu::autobench());
    config_menu.add_item(randomizer::job::vibe_custom_job());
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomCC>("Random Reclassing"));
    config_menu.add_item(randomizer::skill::learn::vibe_learn_skill());
    config_menu.add_item(randomizer::emblem::emblem_skill::vibe_rand_esc());
    config_menu.add_item(randomizer::skill::menu::vibe_rand_spc());
    config_menu.add_item(randomizer::skill::menu::vibe_skill_gauge());
    config_menu.add_item(randomizer::job::vibe_job_gauge());
    config_menu.add_item(randomizer::item::vibe_drops());
    config_menu.add_item(randomizer::item::hub::vibe_hub_items());
    config_menu.add_item(randomizer::item::vibe_item_gauge());
    config_menu.add_item(randomizer::bgm::vibe_bgm());
    config_menu.add_item(randomizer::styles::vibe_styles());
    config_menu.add_item(randomizer::interact::vibe_interaction());
    config_menu.add_item(randomizer::emblem::engrave::vibe_engrave());
    config_menu.add_item(autolevel::enemy::vibe_enemy_emblem());
    config_menu.add_item(autolevel::revival::vibe_enemy_stones());
    config_menu.add_item(randomizer::emblem::menu::vibe_engage_links());
    config_menu.add_item(randomizer::map::vibe_tile());
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
            if DVCVariables::random_enabled() {
                let text = format!("Create Output File for this Save?\n Save as 'sd:/Draconic Vibe Crystal/{}.log'",  crate::utils::get_player_name());
                YesNoDialog::bind::<WriteOutputConfirm>(this.menu, text, "Do it!", "Nah..");
            }
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = if GameVariableManager::get_number("G_Random_Seed") == 0 {  
            "Open up In-Game Draconic Vibe Crystal settings.".to_string() 
        }
        else { format!("Press + to Create Output. Seed: {}", GameVariableManager::get_number(DVCVariables::SEED))
        }.into();
    }
}

pub extern "C" fn vibe2() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::super::VERSION);
    let command = ConfigBasicMenuItem::new_command::<TriabolicalInGameMenu>(title);
    command
}

pub fn dvc_in_game_menu_create_bind(this: &mut BasicMenuItem, _method_info: OptionalMethod) ->  BasicMenuResult {
    this.menu.close_anime_all();
    if engage::sortie::SortieTopMenuManager::get_instance().is_some() {
        this.menu.save_select(SortieTopMenuManager::get_menu_select());
    }
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

pub fn dvc_minus_calls() {
    Il2CppClass::from_name("App", "SortieTopMenu").unwrap().get_nested_types().iter().for_each(|cc|{
        Il2CppClass::from_il2cpptype(cc.get_type()).unwrap()
            .get_virtual_method_mut("MinusCall").map(|method| method.method_ptr = dvc_in_game_menu_create_bind as _);
    });
    Il2CppClass::from_name("App", "MapSystemMenu").unwrap().get_nested_types().iter().for_each(|cc|{
        Il2CppClass::from_il2cpptype(cc.get_type()).unwrap().get_virtual_method_mut("MinusCall").map(|method| method.method_ptr = dvc_in_game_menu_create_bind as _);
    });
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
