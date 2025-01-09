use unity::prelude::*;
use engage::{
    dialog::yesno::*, 
    gameuserdata::*, 
    gamevariable::*, 
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult, *}, 
    pad::Pad, 
    proc::ProcInst, 
    util::get_instance
};
use crate::{deployment, randomizer, ironman, continuous, autolevel};
use super::CONFIG;

extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
}
// Functions to hide the option when conditions are met
pub fn build_attribute_job_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0  { BasicMenuItemAttribute::Enable }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number("G_Random_Job") > 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub fn build_attribute_skill_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 {
        if CONFIG.lock().unwrap().random_skill { BasicMenuItemAttribute::Enable }
        else { BasicMenuItemAttribute::Disable }
    }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_bool("G_Random_Skills") { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub fn build_attribute_hub_items(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number("G_Random_Item") != 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}
pub fn build_attribute_not_in_map(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else if !crate::utils::can_rand() && !GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}
pub fn build_attribute_normal(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if !crate::utils::can_rand() && !GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}
pub fn build_attribute_not_in_map2(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub struct TriabolicalMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalMenu {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.b() {CONFIG.lock().unwrap().save();}
        if pad_instance.npad_state.buttons.a() {
            if pad_instance.npad_state.buttons.a() {
            // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
                let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
                    close_anime_all(this.menu, method.method_info);
                });
                ConfigMenu::create_bind(this.menu);
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

                config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
                config_menu.full_menu_item_list.clear();
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<continuous::ContiniousMode>("Continuous Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::DeploymentMod>("Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::EmblemMod>("Emblem Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<ironman::IronmanMod>("Ironman Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::names::RandomNameMods>("Random Emblem Names"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::names::GenericAppearance>("Random Generic Units Setting"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<autolevel::AutolevelMod>("Level Scale Units")); 
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::person::RandomPersonMod>("Unit Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::RandomEmblemMod>("Emblem Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomJobMod>("Random Classes"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomCC>("Random Reclassing"));
                config_menu.add_item(randomizer::job::vibe_custom_job());
                config_menu.add_item(randomizer::job::vibe_job_gauge());
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::grow::RandomGrowMod>("Random Growth Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::battle_styles::RandomBattleStyles>("Randomize Class Types"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::interact::InteractionSettings>("Weapon Triangle Settings"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::RandomSkillMod>("Randomize Skills"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::RandomSkillCost>("Skill Inheritance SP Cost"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::skill::EnemySkillGauge>("Random Enemy Skill Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::RandomItemMod>("Randomize Obtained Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::item::ItemPriceGauge>("Randomized Item Value"));
                config_menu.add_item(randomizer::item::vibe_drops());
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::RandomGiftMod>("Reward/Gift Item Settings"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::shop::RandomShopMod>("Shop Setting"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::shop::RandomHubItemMod>("Exploration Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::RandomGodMod>("Randomize Emblem Skill Data"));       
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::RandomSynchoMod>("Randomize Emblem Sync Data"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_skill::EmblemSkillChaos>("Emblem Skill Chaos Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::engrave::EngraveSettings>("Engrave Randomization Level"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::RandomEngageWepMod>("Engage Items/Weapons"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_item::EmblemWeaponProfs>("Emblem Proficiencies Setting"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::RandomEmblemLinkMod>("Unit-Emblem Links"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::bgm::RandomBGMMod>("Map BGM Setting")); 
                BasicMenuResult::se_cursor()
            }   
            else { BasicMenuResult::new() }
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "All will be Revealed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.help_text = "Open up the Draconic Vibe Crystal settings.".into(); }
}
extern "C" fn vibe_enable() -> &'static mut ConfigBasicMenuItem { 
    ConfigBasicMenuItem::new_switch::<RandomEnable>("DVC Randomization") 
}
extern "C" fn vibe() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalMenu>(title)
}
extern "C" fn vibe_post_save() -> &'static mut ConfigBasicMenuItem { 
    ConfigBasicMenuItem::new_switch::<RandoSave>("Randomize Save Files")
}
pub struct RandomEnable;
impl ConfigBasicMenuItemSwitchMethods for RandomEnable {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().randomized;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().randomized = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().randomized { "Enables randomization settings on a new save." } 
            else {"Disables randomization settings on a new save." }.into();
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().randomized { "Enable" } else { "Disable" }.into();
    }
}

pub struct RandoSave;
impl ConfigBasicMenuItemSwitchMethods for RandoSave {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().apply_rando_post_new_game;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().apply_rando_post_new_game = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().apply_rando_post_new_game { "Apply disabled randomization settings to saves." } 
            else { "No actions done to previous save files." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().apply_rando_post_new_game { "Enable" } else { "Disable" }.into();
    }
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
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

                config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
                config_menu.full_menu_item_list.clear();
                config_menu.add_item(deployment::vibe_deployment());
                config_menu.add_item(deployment::vibe_emblem_deployment());
                config_menu.add_item(deployment::fulldeploy::vibe_energy());
                config_menu.add_item(randomizer::assets::accessory::vibe_enemy_outfit());
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::assets::accessory::RandomAssets>("Randomized Assets"));
                config_menu.add_item(randomizer::job:: vibe_job_rerand());
                config_menu.add_item(randomizer::item::unit_items::vibe_prw());
                config_menu.add_item(randomizer::assets::bust::vibe_bust());
                config_menu.add_item(randomizer::names::vibe_generic());
                config_menu.add_item(randomizer::grow::vibe_pgmode());
                config_menu.add_item(autolevel::vibe_autolevel());
                config_menu.add_item(autolevel::autobench());
                config_menu.add_item(randomizer::job::vibe_custom_job());
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomCC>("Random Reclassing"));
                config_menu.add_item(randomizer::skill::vibe_skill_gauge());
                config_menu.add_item(randomizer::job::vibe_job_gauge());
                config_menu.add_item(randomizer::item::vibe_drops());
                config_menu.add_item(randomizer::item::shop::vibe_hub_items());
                config_menu.add_item(randomizer::item::vibe_item_gauge());
                config_menu.add_item(randomizer::bgm::vibe_bgm());
                config_menu.add_item(randomizer::battle_styles::vibe_styles());
                config_menu.add_item(randomizer::interact::vibe_interaction());
                config_menu.add_item(randomizer::emblem::engrave::vibe_engrave());
                config_menu.add_item(autolevel::vibe_enemy_emblem());
                config_menu.add_item(autolevel::vibe_enemy_stones());
                config_menu.add_item(randomizer::emblem::vibe_engage_links());

                config_menu.add_item(randomizer::vibe_reseed());
                BasicMenuResult::se_cursor()
            }   
            else { BasicMenuResult::new() }
        }
        else if pad_instance.npad_state.buttons.plus() {
            if crate::utils::can_rand() {
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
        else {
            format!("Press + to Create Output. Seed: {}", GameVariableManager::get_number("G_Random_Seed"))
        }.into();
    }
}

pub struct WriteOutputConfirm;
impl TwoChoiceDialogMethods for WriteOutputConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomizer::write_seed_output_file();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}
extern "C" fn vibe2() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalInGameMenu>(title)
}

pub fn install_vibe() { 
    cobapi::install_global_game_setting(vibe_enable);
    cobapi::install_global_game_setting(vibe_post_save);
    cobapi::install_global_game_setting(randomizer::person::vibe_custom_units);
    cobapi::install_global_game_setting(vibe); 
    cobapi::install_global_game_setting(randomizer::vibe_seed);
    cobapi::install_game_setting(vibe2);
}