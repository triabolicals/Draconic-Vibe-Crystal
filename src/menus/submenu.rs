use super::*;

pub extern "C" fn open_anime_all_ondispose_to_dvc_main(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
    TitleBar::open_header("Draconic Vibe Crystal", super::super::VERSION, "");
}
pub extern "C" fn open_anime_all_ondispose_to_dvc_main2(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
    TitleBar::open_header("Draconic Vibe Crystal", super::super::VERSION, "");
}

pub struct RecruitmentSubMenu;
impl ConfigBasicMenuItemCommandMethods for RecruitmentSubMenu {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = recruitment_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal deployment and recruitment settings.".into();
    }
}
pub fn recruitment_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::DeploymentMod>("Deployment Mode"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<deployment::EmblemMod>("Emblem Deployment Mode"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<ironman::IronmanMod>("Ironman Mode"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::person::RandomPersonMod>("Unit Recruitment Order"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menuitem::RandomEmblemMod>("Emblem Recruitment Order"));
    config_menu.add_item(randomizer::person::vibe_custom_units());
    config_menu.add_item(randomizer::person::vibe_custom_slot_disable());
    TitleBar::open_header("Draconic Vibe Crystal", "Deployment and Recruitment Settings", "");
    BasicMenuResult::se_cursor()
}

pub struct EmbelmSubMenu;
impl ConfigBasicMenuItemCommandMethods for EmbelmSubMenu {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = emblem_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal emblem settings.".into();
    }
}
pub fn emblem_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::names::RandomNameMods>("Random Emblem Names"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menuitem::RandomGodMod>("Emblem Engage/Inherit Skills"));       
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menuitem::RandomSynchoMod>("Emblem Sync Skills"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_skill::EmblemSkillChaos>("Emblem Skill Chaos Mode"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::engrave::EngraveSettings>("Engrave Randomization Level"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menuitem::RandomEngageWepMod>("Engage Items / Weapons"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::emblem_item::EmblemWeaponProfs>("Emblem Proficiencies Setting"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::emblem::menuitem::RandomEmblemLinkMod>("Unit-Emblem Links"));
    TitleBar::open_header("Draconic Vibe Crystal", "Emblem Settings", "");
    BasicMenuResult::se_cursor()
}

pub struct ClassSubMenu;
impl ConfigBasicMenuItemCommandMethods for ClassSubMenu {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = class_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal class and skills settings.".into();
    }
}
pub fn class_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(randomizer::job::vibe_custom_job());
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::battle_styles::RandomBattleStyles>("Class Types Setting"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomCC>("Reclassing Setting"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::job::RandomJobMod>("Random Classes"));    
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::menuitem::RandomSkillMod>("Randomize Skills"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::skill::menuitem::RandomSkillCost>("Skill Inheritance / SP Cost"));
    config_menu.add_item(randomizer::skill::learn::vibe_learn_skill());
    TitleBar::open_header("Draconic Vibe Crystal", "Class / Skill Settings", "");
    BasicMenuResult::se_cursor()
}

pub struct ItemSubMenu;
impl ConfigBasicMenuItemCommandMethods for ItemSubMenu {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = item_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal item settings.".into();
    }
}
pub fn item_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::interact::InteractionSettings>("Weapon Triangle Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::RandomItemMod>("Item Randomization"));
    config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::item::ItemPriceGauge>("Item Replacement Value"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::unit_items::PlayerRandomWeapons>("Player Starting Inventory"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::RandomGiftMod>("Reward/Gift Item Settings"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::shop::RandomShopMod>("Shop Setting"));
    config_menu.add_item(ConfigBasicMenuItem::new_switch::<randomizer::item::hub::RandomHubItemMod>("Exploration Items"));
    TitleBar::open_header("Draconic Vibe Crystal", "Items Settings", "");
    BasicMenuResult::se_cursor()
}

pub struct EnemySubMenu;
impl ConfigBasicMenuItemCommandMethods for EnemySubMenu {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = enemy_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new() }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "View Settings".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = "View Draconic Vibe Crystal growth settings.".into();
    }
}
pub fn enemy_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.full_menu_item_list.clear();
    config_menu.add_item(randomizer::names::vibe_generic());
    config_menu.add_item(ConfigBasicMenuItem::new_gauge::<randomizer::skill::menuitem::EnemySkillGauge>("Random Enemy Skill Rate"));
    config_menu.add_item(randomizer::item::vibe_drops());
    config_menu.add_item(randomizer::job::vibe_job_gauge());
    config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"));
    config_menu.add_item(ConfigBasicMenuItem::new_gauge::<autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"));
    TitleBar::open_header("Draconic Vibe Crystal", "Enemy Settings", "");
    BasicMenuResult::se_cursor()
}


