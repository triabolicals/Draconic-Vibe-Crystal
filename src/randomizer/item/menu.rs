use engage::gamevariable::GameVariableManager;
use engage::menu::{BasicMenuItemAttribute, BasicMenuResult};
use engage::menu::config::{ConfigBasicMenuItem, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItemSwitchMethods};
use unity::prelude::OptionalMethod;
use crate::CONFIG;
use crate::config::DVCVariables;
use crate::utils::can_rand;

pub struct EnemyDropGauge;
impl ConfigBasicMenuItemGaugeMethods  for EnemyDropGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_drop_rate  as f32 / 100.0 }
        else { GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY) as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        let gauge = if is_main { CONFIG.lock().unwrap().enemy_drop_rate  }
        else { GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(gauge, 0, 100, 10);
        if gauge != result {
            if is_main {CONFIG.lock().unwrap().enemy_drop_rate  = result; }
            else { GameVariableManager::set_number(DVCVariables::ITEM_DROP_GAUGE_KEY, result ); }
            this.gauge_ratio = 0.01 * result as f32;
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_drop_rate }
        else { GameVariableManager::get_number(DVCVariables::ITEM_DROP_GAUGE_KEY)};
        this.help_text =
            if value == 0 { "Enemy units will not randomly drop items.".into() }
            else { format!("Chance of enemy units dropping random items: {}%",  value).into() }
    }
}

pub struct ItemPriceGauge;
impl ConfigBasicMenuItemGaugeMethods  for ItemPriceGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().replaced_item_price as f32 / 100.0 }
        else { GameVariableManager::get_number(DVCVariables::ITEM_GAUGE_KEY) as f32 / 100.0 };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().replaced_item_price }
        else { GameVariableManager::get_number(DVCVariables::ITEM_GAUGE_KEY) };

        let result = ConfigBasicMenuItem::change_key_value_i(gauge, 0, 100, 10);
        if gauge != result {
            if DVCVariables::is_main_menu() {CONFIG.lock().unwrap().replaced_item_price = result; }
            else { GameVariableManager::set_number(DVCVariables::ITEM_GAUGE_KEY, result ); }
            this.gauge_ratio = 0.01 * result as f32;
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Minimum value of new item as a percentage of original item's value.".into();
    }
}
pub extern "C" fn vibe_item_gauge() -> &'static mut ConfigBasicMenuItem {
    let item_gauge = ConfigBasicMenuItem::new_gauge::<ItemPriceGauge>("Randomized Item Value");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::hub_item_build_attr as _);
    item_gauge
}
pub struct RandomItemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomItemMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_item();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value!= result {
            DVCVariables::set_random_item(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match DVCVariables::get_random_item() {
            1 => { "Events"},
            2 => { "Drops"},
            3 => { "Events/Drops"}
            _ => { "None "}
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match DVCVariables::get_random_item() {
            1 => { "Items obtained from chests/villages will be random." },
            2 => { "Item drops from enemies will be random." },
            3 => { "Item obtained from events and enemy drops will be random." },
            _ => { "No changes made to item events or item drops." },
        }.into();
    }
}

pub struct RandomGiftMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGiftMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry(DVCVariables::GIFTS_KEY, 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_gift_items();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            DVCVariables::set_random_gift_items(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_gift_items();
        this.command_text =
            match  value {
                1 => { "No Rare Items" }
                2 => { "With Rare Items" }
                _ => { "No Randomization" }
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let value = DVCVariables::get_random_gift_items();
        this.help_text =
            match value {
                1 => { "No rare items will be included when randomizing gift item lists." }
                2 => { "Rare items will be included when randomizing gift item lists." }
                _ => { "No randomization done to gift items." }
            }.into();
    }
}

pub struct PlayerRandomWeapons;
impl ConfigBasicMenuItemSwitchMethods for PlayerRandomWeapons {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry_norewind(DVCVariables::PLAYER_INVENTORY, 0);
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let state = DVCVariables::get_random_inventory();
        let result = ConfigBasicMenuItem::change_key_value_i(state, 0, 3,1);
        if state != result {
            DVCVariables::set_random_inventory(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = DVCVariables::get_random_inventory();
        this.command_text = match state {
            1 => "Player",
            2 => "Enemy/NPC",
            3 => "All",
            _ => "Default",
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let state = DVCVariables::get_random_inventory();
        this.help_text = match state {
            1 => "Player units will start with random inventories",
            2 => "Enemies and NPCs will start with random inventories",
            3 => "All units will start with random inventories",
            _ => "Units will start with normal inventories.",
        }.into();
    }
}
fn prw_build_attrs(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if can_rand() { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_prw() ->  &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<PlayerRandomWeapons>("Unit Starting Inventory");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = prw_build_attrs as _);
    switch
}

pub extern "C" fn vibe_drops() -> &'static mut ConfigBasicMenuItem {
    let item_gauge = ConfigBasicMenuItem::new_gauge::<EnemyDropGauge>("Enemy Item Drop Rate");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    item_gauge
}