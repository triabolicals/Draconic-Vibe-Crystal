use super::*;

pub struct RandomHubItemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomHubItemMod{
    fn init_content(_this: &mut ConfigBasicMenuItem){  GameVariableManager::make_entry(DVCVariables::HUB_ITEM_KEY, 0);  }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().exploration_items } 
            else { GameVariableManager::get_number(DVCVariables::HUB_ITEM_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().exploration_items = result; }
            else { GameVariableManager::set_number(DVCVariables::HUB_ITEM_KEY, result); }

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().exploration_items } 
            else { GameVariableManager::get_number(DVCVariables::HUB_ITEM_KEY) };
        this.help_text = match value {
            1 => { "Excludes gift items from exploration." },
            2 => { "Excludes food items from exploration."},
            3 => { "Excludes gift and food items from exploration."},
            _ => { "Exploration items includes both gift and food items."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value =  if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().exploration_items } 
            else { GameVariableManager::get_number(DVCVariables::HUB_ITEM_KEY) };
        this.command_text = match value {
            1 => {  "No Gifts" },
            2 => {  "No Food" },
            3 => {  "No Gift/Food"},
            _ => {  "Default"},
        }.into();
    }
}

pub extern "C" fn vibe_hub_items() -> &'static mut ConfigBasicMenuItem {  
    let hub_items = ConfigBasicMenuItem::new_switch::<RandomHubItemMod>("Exploration Items");
    hub_items.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::hub_item_build_attr as _);
    hub_items
}

pub fn hub_item_randomized() {
    if !DVCVariables::random_enabled() || GameVariableManager::get_number(DVCVariables::ITEM_KEY) & 2 == 0  { return; }
    if let Some(hub_sequence) = get_singleton_proc_instance::<HubSequence>() {
        println!("Randomizing Hub Items");
        let rng = Random::get_system();
        hub_sequence.get_locator_group().as_mut().unwrap().access_list.iter_mut()
            .flat_map(| access | access.access_data.as_mut() )
            .for_each(| access|{
                if access.aid.is_some_and(|aid| aid.to_string().contains("IID")) {
                    if ItemData::get(access.aid.unwrap()).is_some_and(|item| item.kind < 14 && item.kind > 16 ) {
                        access.aid = Some(super::random_item(2, false));
                        access.item_count += rng.get_value(5);
                    }
                }
                else if rng.get_value(3) < 1 {
                    access.talk_item = Some(super::random_item(2, false));
                    access.item_count += rng.get_value(3);
                }
            }
        );
    }
}

