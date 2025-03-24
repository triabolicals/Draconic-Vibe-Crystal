use data::job::Mount;

use super::*;
use crate::config::DVCVariables;

pub struct RandomAssets;
impl ConfigBasicMenuItemSwitchMethods for RandomAssets {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry(DVCVariables::ASSETS, 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let main = DVCVariables::is_main_menu();
        let value = if main { CONFIG.lock().unwrap().assets }
            else { GameVariableManager::get_number(DVCVariables::ASSETS) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if main { CONFIG.lock().unwrap().assets = result; }
            else { GameVariableManager::set_number(DVCVariables::ASSETS, result);  }

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().assets }
            else { GameVariableManager::get_number(DVCVariables::ASSETS) };

        this.help_text = match value {
            1 => { "Weapons assets are randomized"  }
            2 => { "Info animations are randomized for player units."}
            3 => { "Weapons / player info animations are randomized"}
            _ => { "No assets are randomized." }
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().assets }
        else { GameVariableManager::get_number(DVCVariables::ASSETS)  };
            
        this.command_text =  match value {
            1 => { "Weapons"}
            2 => { "Info"}
            3 => { "Weapon+Info"}
            _ => { "None"}
        }.into();
    }
}

pub struct RandomEnemyOutfits;
impl ConfigBasicMenuItemSwitchMethods for RandomEnemyOutfits {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("EnemyOutfits", GameVariableManager::get_number(DVCVariables::ENEMY_OUTFIT_KEY));
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = GameVariableManager::get_bool("EnemyOutfits");
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            GameVariableManager::set_bool("EnemyOutfits", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let str1 =  if GameVariableManager::get_bool("EnemyOutfits") { "Enemies will wear random outfits. "  }
                    else { "Enemies will wear their regular outfits." };

        this.help_text = if GameVariableManager::get_bool("EnemyOutfits") != GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) {
            format!("{} (Press A to Confirm)", str1) }
        else { str1.to_string() }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let changed = if GameVariableManager::get_bool("EnemyOutfits") != GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) { "*"} else { "" };
        this.command_text = format!("{}{}", changed,
            if GameVariableManager::get_bool("EnemyOutfits") { "Randomized"  } 
            else { "Normal "}
        ).into();
    }
}

pub fn outfits_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_bool("EnemyOutfits") == GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) { return BasicMenuResult::new();}
    let str1 = if GameVariableManager::get_bool("EnemyOutfits") { "Randomized enemy outfits?"}
        else { "Revert enemies to their default outfits?"};

    YesNoDialog::bind::<OutfitConfirm>(this.menu, str1, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub extern "C" fn vibe_enemy_outfit() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<RandomEnemyOutfits>("Random Enemy Outfits");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = outfits_setting_acall as _ );
    switch
}

pub struct OutfitConfirm;
impl TwoChoiceDialogMethods for OutfitConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_bool(DVCVariables::ENEMY_OUTFIT_KEY, GameVariableManager::get_bool("EnemyOutfits"));
        change_enemy_outfits();

        let menu =  unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        RandomEnemyOutfits::set_help_text(menu.menu_item_list[index as usize], None);
        RandomEnemyOutfits::set_command_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}



pub fn accesorize_enemy_unit(enemy: &Unit) {
    let accessory_list = &mut unsafe { unit_get_accessory_list(enemy, None) }.unit_accessory_array;
    let length = accessory_list.len();
    if !GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) && enemy.person.get_asset_force() != 0 { 
        for x in 0..length { accessory_list[x].index = 0; }
        return;
    }
    for x in 0..length {
        if x == 4 { continue; }
        random_unit_accessory(enemy, x as i32, true);
    }
}

pub fn change_enemy_outfits() {
    let force_type = [ForceType::Enemy, ForceType::Ally];
    for ff in force_type {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter {
            if unit.person.get_asset_force() == 0 { continue; }
            accesorize_enemy_unit(unit);
        }
    }
}

pub fn set_accessories_generic(result: &mut AssetTableResult, aid: Option<&Il2CppString>, gender: i32) {
    if let Some(generic_aid) = aid {
        let belong = generic_aid.to_string();
        match belong.as_str() {
            "AID_異形兵" => {
                if gender == 1 { add_accessory_to_list(result.accessory_list, "uAcc_spine2_BoneMrp1AM", "c_spine2_jnt"); }
                else { add_accessory_to_list(result.accessory_list, "uAcc_spine2_BoneMrp1AF", "c_spine2_jnt"); }
            }
            _ => {},
        }
    }
}

pub fn add_accessory_to_list(list: &mut AssetTableAccessoryList, model: &str, location: &str) {
    let accessory_class = Il2CppClass::from_name("App", "AssetTable").unwrap().get_nested_types().iter().find(|x| x.get_name() == "Accessory").unwrap();
    let new_accessory = Il2CppObject::<AssetTableAccessory>::from_class( accessory_class ).unwrap();
    new_accessory.model = Some(model.into() );
    new_accessory.locator = Some(location.into());
    list.try_add(new_accessory);
}
pub fn clear_accessory_from_list(list: &mut AssetTableAccessoryList, model: &str) {
    for x in 0..list.list.len() {
        if let Some(accessory_model) = list.list[x].model {
            if accessory_model.to_string().contains(model) {
                list.list[x].model = Some("null".into());
            }
        }
    }
}

pub fn accessory_clear_all(list: &mut AssetTableAccessoryList) {
    list.list.iter_mut().for_each(|acc| { acc.model = None; });
}


pub fn clear_accessory_at_locator(list: &mut AssetTableAccessoryList, locator: &str) {
    if let Some(acc) = list.list.iter_mut()
        .find(|acc| acc.locator.is_some_and(|loc| loc.to_string() == locator)) {
            acc.model = Some("null".into());
        }
}
pub fn change_accessory(list: &mut AssetTableAccessoryList, model: &str, locator: &str){
    if model != "null" {
        // check if accessory exists 
        if list.list.iter_mut().any(|f| f.model.filter(|m| m.to_string().contains(model)).is_some() ) { return; }
    }
    // check if locator exists then replace the model
    if let Some(acc) = list.list.iter_mut().find(|f| f.locator.is_some_and(|m|m.to_string().contains(locator))) {
        acc.model = Some(model.into());
    }
    else { add_accessory_to_list(list, model, locator); }
}

pub fn accessory_at_locator(list: &List<AssetTableAccessory>, locator: &str) -> Option<String> {
    if let Some(acc) = list.iter().find(|f| f.locator.filter(|m| m.to_string().contains(locator)).is_some() ) {
        if let Some(model) = acc.model {
            let str = model.to_string();
            if str.len() == 0 { return None; }
            else { return Some(str.clone()); }
        }
    }
    None
}

pub fn next_unit_accessory(unit: &Unit, kind: i32, increase: bool) -> bool {
    let accessory = unsafe { unit_get_accessory_list(unit, None)};
    let index = accessory.unit_accessory_array[kind as usize].index;
    let accessories = AccessoryData::get_list().unwrap();
    let dress_gender = unit_dress_gender(unit);


    if increase {
        if let Some(new_index) = accessories.iter()
            .filter(|acc| 
                acc.get_num() > 0 && 
                acc.kind == kind && 
                acc.parent.index > index &&
                ( acc.condition_gender == 0 || acc.condition_gender == dress_gender )
            )
            .map(|acc| acc.parent.index).min() 
        {
            accessory.unit_accessory_array[kind as usize].index = new_index;
            return true;
        }
        else if index != 0 {
            accessory.unit_accessory_array[kind as usize].index = 0;
            return true;
        }
    }
    else if index == 0 {
        if let Some(new_index) = accessories.iter()
            .filter(|acc| 
                acc.get_num() > 0 && 
                ( acc.condition_gender == 0 || acc.condition_gender == dress_gender ) &&
                acc.kind == kind && 
                acc.parent.index > index 
            )
            .map(|acc| acc.parent.index).max() 
            {
                accessory.unit_accessory_array[kind as usize].index = new_index;
                return true;
            }
    }
    else {
        if let Some(new_index) = accessories.iter()
            .filter(|acc|
                acc.get_num() > 0 && 
                acc.kind == kind && 
                acc.parent.index < index &&
                ( acc.condition_gender == 0 || acc.condition_gender == dress_gender )
            )
            .map(|acc| acc.parent.index).max() {
                accessory.unit_accessory_array[kind as usize].index = new_index;
                return true;
            }
        else if index != 0 {
            accessory.unit_accessory_array[kind as usize].index = 0;
            return true;
        }
    }
    return false;
}

pub fn random_unit_accessory(unit: &Unit, kind: i32, is_enemy: bool) -> bool {
    let accessory = unsafe { unit_get_accessory_list(unit, None)};
    let index = accessory.unit_accessory_array[kind as usize].index;
    let accessories = AccessoryData::get_list().unwrap();
    let mut dress_gender = unit_dress_gender(unit);
    if SEARCH_LIST.get().unwrap().job.iter().find(|j| j.mound == Mount::Pegasus && j.job_hash == unit.job.parent.hash).is_some() { dress_gender = 2; }

    let rng = Random::get_game();
    let access: Vec<_> = accessories.iter()
        .filter(|acc| (is_enemy || acc.get_num() > 0) && acc.kind == kind && acc.parent.index > index && ( acc.condition_gender == 0 || acc.condition_gender == dress_gender ) )
        .map(|acc| acc.parent.index)
        .collect();

    let len = access.len();
    if len < 2 { false }
    else {
        let index = rng.get_value( len as i32);
        accessory.unit_accessory_array[kind as usize].index = access[index as usize];
        true
    }
}