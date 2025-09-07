use crate::randomizer::emblem::EMBLEM_LIST;
use super::*;

pub extern "C" fn open_anime_all_ondispose_to_dvc_main(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnime").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
    TitleBar::open_header("Draconic Vibe Crystal", super::super::VERSION, "");
}

pub struct RecruitmentSubMenu;
impl ConfigBasicMenuItemCommandMethods for RecruitmentSubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text =
            if DVCVariables::is_main_menu() { "View Draconic Vibe Crystal deployment and recruitment settings." } else { "View Draconic Vibe Crystal player/emblem related settings." }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose")
            .map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        if DVCVariables::is_main_menu() {
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::DeploymentMod>("Deployment Mode"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::EmblemMod>("Emblem Deployment Mode"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<ironman::IronmanMod>("Ironman Mode"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::person::RandomPersonMod>("Unit Recruitment Order"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menu::RandomEmblemMod>("Emblem Recruitment Order"));
            config_menu.add_item(randomizer::person::vibe_custom_units());
            config_menu.add_item(randomizer::person::vibe_custom_slot_disable());
            TitleBar::open_header("Draconic Vibe Crystal", "Deployment and Recruitment Settings", "");
        }
        else {
            config_menu.add_item(deployment::vibe_deployment());
            config_menu.add_item(deployment::vibe_emblem_deployment());
            config_menu.add_item(autolevel::menu::vibe_autolevel());
            config_menu.add_item(autolevel::menu::autobench());
            config_menu.add_item(randomizer::grow::vibe_pgmode());
            TitleBar::open_header("Draconic Vibe Crystal", "Player Unit/Emblem Settings", "");
        }

        BasicMenuResult::se_cursor()
    }
}


pub struct EmbelmSubMenu;
impl ConfigBasicMenuItemCommandMethods for EmbelmSubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal emblem settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::names::RandomNameMods>("Random Emblem Names"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menu::RandomGodMod>("Emblem Engage/Inherit Skills"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menu::RandomSynchoMod>("Emblem Sync Skills"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_skill::EmblemSkillChaos>("Emblem Skill Chaos Mode"));
        config_menu.add_item(randomizer::emblem::engrave::vibe_engrave());
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menu::RandomEngageWepMod>("Engage Items / Weapons"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_item::EmblemWeaponProfs>("Emblem Proficiencies Setting"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menu::RandomEmblemLinkMod>("Unit-Emblem Links"));
        TitleBar::open_header("Draconic Vibe Crystal", "Emblem Settings", "");
        BasicMenuResult::se_cursor()
    }
}
pub struct ClassSubMenu;
impl ConfigBasicMenuItemCommandMethods for ClassSubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal class and skills settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        config_menu.add_item(randomizer::job::menu::vibe_custom_job());
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::menu::RandomJobMod>("Random Classes"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::styles::RandomBattleStyles>("Class Types Setting"));
        if DVCVariables::is_main_menu() {
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::menu::RandomCC>("Re-Classing Setting"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::single::SingleJob>("Opps! All"));
        }
        else {
            if DVCVariables::get_single_class(false).is_some() || DVCVariables::get_random_reclass() == 0 {
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::single::SingleJob>("Opps! All"));
            }
            else { config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::menu::RandomCC>("Re-Classing Settings")); }
        }
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::menu::RandomSkillMod>("Randomize Skills"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::bond::BondRingSetting>("Random Bond Ring Skills"));
        config_menu.add_item(randomizer::skill::menu::vibe_rand_spc());
        config_menu.add_item(randomizer::emblem::emblem_skill::vibe_rand_esc());
        config_menu.add_item(randomizer::skill::learn::vibe_equip_job_learn_skills());
        config_menu.add_item(randomizer::skill::learn::vibe_learn_skill());
        TitleBar::open_header("Draconic Vibe Crystal", "Class / Skill Settings", "");
        BasicMenuResult::se_cursor()
    }
}

pub struct ItemSubMenu;
impl ConfigBasicMenuItemCommandMethods for ItemSubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal item settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        config_menu.add_item(crate::randomizer::styles::vibe_styles());
        config_menu.add_item(crate::randomizer::interact::vibe_interaction());
        if DVCVariables::is_main_menu() {
            config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::item::menu::ItemPriceGauge>("Item Replacement Value"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::menu::PlayerRandomWeapons>("Player Starting Inventory"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::menu::RandomGiftMod>("Reward/Gift Item Settings"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::shop::RandomShopMod>("Shop Setting"));
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::hub::RandomHubItemMod>("Exploration Items"));
        }
        else {
            config_menu.add_item(randomizer::item::menu::vibe_prw());   // I
            config_menu.add_item(randomizer::item::menu::vibe_drops()); // I
            config_menu.add_item(randomizer::item::hub::vibe_hub_items());  // I
            config_menu.add_item(randomizer::item::menu::vibe_item_gauge());    // I
        }
        TitleBar::open_header("Draconic Vibe Crystal", "Items Settings", "");
        BasicMenuResult::se_cursor()
    }
}
pub struct EnemySubMenu;
impl ConfigBasicMenuItemCommandMethods for EnemySubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal enemy settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::person::RandomBosses>("NPC/Bosses Setting"));
        config_menu.add_item(randomizer::names::vibe_generic());
        if DVCVariables::is_main_menu() {
            config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::skill::menu::EnemySkillGauge>("Random Enemy Skill Rate"));
            config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::enemy::EnemyEmblemGauge>("Enemy Emblem Rate"));
            config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::revival::EnemyRevivalStones>("Enemy Revival Stone Rate"));
        }
        else {
            config_menu.add_item(randomizer::item::menu::vibe_drops());
            config_menu.add_item(randomizer::job::menu::vibe_job_gauge());
            config_menu.add_item(autolevel::enemy::vibe_enemy_emblem());
            config_menu.add_item(autolevel::revival::vibe_enemy_stones());
        }
        TitleBar::open_header("Draconic Vibe Crystal", "Enemy Settings", "");
        BasicMenuResult::se_cursor()
    }
}
pub struct AssetSubMenu;
impl ConfigBasicMenuItemCommandMethods for AssetSubMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = "View Draconic Vibe Crystal Asset Settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        ConfigMenu::create_bind(this.menu);
        let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
        config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
        config_menu.full_menu_item_list.clear();
        if DVCVariables::is_main_menu() {
            config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::assets::accessory::RandomPlayerAppearance>("Random Player Appearance"));
        }
        else { config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::names::RandomNameMods>("Random Emblem Names")); }
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::assets::accessory::EmblemAppearance>("Emblem Appearance Settings"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::person::RandomBosses>("NPC/Bosses Setting"));
        config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::assets::accessory::RandomClassOutfits>("Random Class Outfits"));
        config_menu.add_item(randomizer::names::vibe_generic());
        if !DVCVariables::is_main_menu() { config_menu.add_item(crate::assets::accessory::vibe_enemy_outfit()); }

        config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::assets::accessory::RandomAssets>("Randomized Assets"));
        config_menu.add_item(crate::assets::bust::vibe_bust());
        TitleBar::open_header("Draconic Vibe Crystal", "Asset Settings", "");
        BasicMenuResult::se_cursor()
    }
}