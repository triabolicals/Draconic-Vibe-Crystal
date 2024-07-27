use unity::prelude::*;
use engage::{
    menu::{*, BasicMenuResult, config::{ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    gamevariable::*,
    gameuserdata::*,
    proc::ProcInst,
    mess::*,
    pad::Pad,
    util::get_instance,
};
use crate::{deploy, person, emblem, item, skill, grow};

use super::CONFIG;

extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent, method.method_info);
    });
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
                //config_menu.add_item(ConfigBasicMenuItem::new_switch::<RandomEnable>("Enable Randomization"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::continuous::ContiniousMode>("Continuous Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::ironman::IronmanMod>("Ironman Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::autolevel::AutolevelMod>("Level Scale Units")); 
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<person::RandomPersonMod>("Unit Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemMod>("Emblem Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomJobMod>("Random Classes"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<item::EnemyJobGauge>("Random Enemy Class Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomGrowMod>("Random Growth Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomBattleStyles>("Randomize Class Types"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::InteractionSettings>("Weapon Triangle Settings"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomSkillMod>("Randomize Skills"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<skill::EnemySkillGauge>("Random Enemy Skill Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomItemMod>("Randomize Obtained Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::item::ItemPriceGauge>("Randomized Item Value"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomGiftMod>("Reward/Gift Item Settings"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::shop::RandomShopMod>("Shop Setting"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::shop::RandomHubItemMod>("Exploration Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomGodMod>("Randomize Emblem Data"));       
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomSynchoMod>("Randomize Emblem Sync Data"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::emblem_skill::EmblemSkillChaos>("Emblem Skill Chaos Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::EngraveSettings>("Engrave Randomization Level"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEngageWepMod>("Engage Items/Weapons"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemLinkMod>("Engage+ Links"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::bgm::RandomBGMMod>("Randomize Map BGM")); 
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
// InGame Menu Stuff
extern "C" fn vibe_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map2 as _);
    switch
} 
extern "C" fn vibe_emblem_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map2 as _);
    switch
} 
extern "C" fn vibe_autolevel() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<crate::autolevel::AutolevelMod>("Level Scale Units");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
} 
extern "C" fn vibe_hub_items() -> &'static mut ConfigBasicMenuItem {  
    let hub_items = ConfigBasicMenuItem::new_switch::<crate::shop::RandomHubItemMod2>("Exploration Items");
    hub_items.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_hub_items as _);
    hub_items
}
extern "C" fn vibe_bgm() -> &'static mut ConfigBasicMenuItem {  
    let switch =  ConfigBasicMenuItem::new_switch::<crate::bgm::RandomBGMMod>("Randomize Map BGM");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
}
extern "C" fn vibe_engage_links() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemLinkMod>("Engage+ Links");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
}
extern "C" fn vibe_enemy_emblem() -> &'static mut ConfigBasicMenuItem { 
    let enemy_emblem = ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"); 
    enemy_emblem.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    enemy_emblem
}
extern "C" fn vibe_enemy_stones() -> &'static mut ConfigBasicMenuItem { 
    let enemy_stones = ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"); 
    enemy_stones.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    enemy_stones
}
extern "C" fn vibe_job_gauge() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_gauge::<item::EnemyJobGauge>("Random Enemy Class Rate"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_job_gauge as _);
    class_gauge
}
extern "C" fn vibe_skill_gauge() -> &'static mut ConfigBasicMenuItem {  
    let skill_gauge = ConfigBasicMenuItem::new_gauge::<skill::EnemySkillGauge>("Random Enemy Skill Rate");
    skill_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_skill_gauge as _);
    skill_gauge
}
extern "C" fn vibe_item_gauge() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_gauge::<item::ItemPriceGauge>("Randomized Item Value");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_hub_items as _);
    item_gauge
}
extern "C" fn vibe_styles() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_switch::<grow::RandomBattleStyles>("Random Class Types");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_normal as _);
    item_gauge.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = grow::battle_style_setting_acall as _ );
    item_gauge
}
extern "C" fn vibe_engrave() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<emblem::EngraveSettings>("Engrave Randomization Level");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_normal as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr =  emblem::engrave_setting_acall as _);
    switch
}
extern "C" fn vibe_interaction() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<grow::InteractionSettings>("Weapon Triangle Setting");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_normal as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr =  grow::interaction_setting_acall as _ );
    switch
}
// Function to hide the option when conditions are met
fn build_attribute_job_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number("G_Random_Job") > 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

fn build_attribute_skill_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_bool("G_Random_Skills") { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

fn build_attribute_hub_items(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
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
pub fn install_vibe() { 
    cobapi::install_global_game_setting(vibe_enable);
    cobapi::install_global_game_setting(vibe); 
    cobapi::install_game_setting(vibe_deployment);
    cobapi::install_game_setting(vibe_emblem_deployment);
    cobapi::install_game_setting(vibe_autolevel);
    cobapi::install_game_setting(vibe_skill_gauge);
    cobapi::install_game_setting(vibe_job_gauge);
    cobapi::install_game_setting(vibe_hub_items);
    cobapi::install_game_setting(vibe_item_gauge);
    cobapi::install_game_setting(vibe_bgm);
    cobapi::install_game_setting(vibe_styles);
    cobapi::install_game_setting(vibe_interaction);
    cobapi::install_game_setting(vibe_engrave);
    cobapi::install_game_setting(vibe_enemy_emblem);
    cobapi::install_game_setting(vibe_enemy_stones);
    cobapi::install_game_setting(vibe_engage_links);
}