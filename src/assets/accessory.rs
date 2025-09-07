use data::job::Mount;
use crate::assets::dress::IDS;
use super::*;
use crate::config::{DVCFlags, DVCVariables};

pub struct RandomPlayerAppearance;
impl ConfigBasicMenuItemSwitchMethods for RandomPlayerAppearance {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().player_appearance;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().player_appearance = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().player_appearance { "Enable" } else { "Disable" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            if CONFIG.lock().unwrap().player_appearance { "Original playable characters will have random appearances." }
            else { "Original playable characters will have default appearances." }.into();
    }
}
pub struct EmblemAppearance;
impl ConfigBasicMenuItemSwitchMethods for EmblemAppearance {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        if !DVCVariables::is_main_menu() { GameVariableManager::make_entry(DVCVariables::EMBLEM_APPEAR_KEY, 0); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_emblem_appearance();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_emblem_appearance(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =
            match DVCVariables::get_emblem_appearance() {
                1 => "Outfits",
                2 => "Color",
                3 => "Outfits+Color",
                _ => "None",
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            match DVCVariables::get_emblem_appearance() {
                1 => "Emblem outfits will be randomized",
                2 => "Emblem colors will be randomized.",
                3 => "Both emblem outfits and colors will be randomized.",
                _ => "No changes to emblem appearances.",
            }.into();
    }
}
pub struct RandomClassOutfits;
impl ConfigBasicMenuItemSwitchMethods for RandomClassOutfits {
    fn init_content(_this: &mut ConfigBasicMenuItem) { GameVariableManager::make_entry(DVCVariables::RANDOM_CLASS_OUTFITS, 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_class_outfits();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 2, 1);
        if value != result {
            DVCVariables::set_random_class_outfits(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_class_outfits();
        this.command_text =
            match value {
                1 => "Static",
                2 => "Random",
                _ => "None",
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_class_outfits();
        this.help_text =
            match value {
                1 => "Units will have a random set of class outfits.",
                2 => "Class outfits will be randomized based on unit's current growth seed.",
                _ => "Class outfits are not randomized.",
            }.into();
    }
}

pub struct RandomAssets;
impl ConfigBasicMenuItemSwitchMethods for RandomAssets {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry(DVCVariables::ASSETS, 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_assets();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_assets(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_assets();
        this.command_text =  match value {
            1 => { "Weapons"}
            2 => { "Info"}
            3 => { "Weapon+Info"}
            _ => { "None"}
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_assets();
        this.help_text = match value {
            1 => { "Weapons assets are randomized"  }
            2 => { "Info animations are randomized."}
            3 => { "Weapons / Info animations are randomized."}
            _ => { "No assets are randomized." }
        }.into();
    }
}
pub struct RandomEnemyOutfits;
impl ConfigBasicMenuItemSwitchMethods for RandomEnemyOutfits {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_flag(DVCFlags::EnemyOutfits, true);
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_flag(DVCFlags::EnemyOutfits, result,GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.is_command_icon = (GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3) && DVCVariables::flag_changed(DVCFlags::EnemyOutfits);
        this.command_text = if DVCVariables::get_flag(DVCFlags::EnemyOutfits, this.is_command_icon){ "Randomized" } else { "Normal "}.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let is_map = GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3;
        this.help_text =
            if DVCVariables::get_flag(DVCFlags::EnemyOutfits, is_map) { "Enemies will wear random outfits." }
            else { "Enemies will wear their regular outfits." }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if !DVCVariables::flag_changed(DVCFlags::EnemyOutfits) { return BasicMenuResult::new(); }
        if GameUserData::get_sequence() != 2 && GameUserData::get_sequence() != 3 {
            return BasicMenuResult::new();
        }
        let str1 = if DVCVariables::get_flag(DVCFlags::EnemyOutfits, true) { "Randomized enemy outfits?"}
        else { "Revert enemies to their default outfits?"};

        YesNoDialog::bind::<OutfitConfirm>(this.menu, str1, "Do it!", "Nah..");
        BasicMenuResult::new()
    }
}

pub extern "C" fn vibe_enemy_outfit() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomEnemyOutfits>("Random Enemy Outfits") }

pub struct OutfitConfirm;
impl TwoChoiceDialogMethods for OutfitConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        DVCVariables::update_flag(DVCFlags::EnemyOutfits);
        change_enemy_outfits();
        crate::menus::utils::dialog_restore_text::<RandomEnemyOutfits>(this, false);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub fn accesorize_enemy_unit(enemy: &Unit) {
    if enemy.force.is_some_and(|f| f.force_type != 1 ) { return; }
    let accessory_list = &mut unsafe { unit_get_accessory_list(enemy, None) }.unit_accessory_array;
    let length = accessory_list.len();
    if !DVCVariables::get_flag(DVCFlags::EnemyOutfits, false) && enemy.person.get_asset_force() != 0 {
        for x in 0..length { accessory_list[x].index = 0; }
        return;
    }
    for x in 0..length {
        if x == 4  || x > 5 { continue; }
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
            if accessory_model.str_contains(model) {
                list.list[x].model = Some("null".into());
            }
        }
    }
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
        if list.list.iter_mut().any(|f| f.model.filter(|m| m.str_contains(model)).is_some() ) { return; }
    }
    // check if locator exists then replace the model
    if let Some(acc) = list.list.iter_mut().find(|f| f.locator.is_some_and(|m|m.str_contains(locator))) {
        acc.model = Some(model.into());
    }
    else { add_accessory_to_list(list, model, locator); }
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
                accessory_gender_check(acc, unit) &&
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
                accessory_gender_check(acc, unit) &&
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
                accessory_gender_check(acc, unit) &&
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
    false
}

pub fn random_unit_accessory(unit: &Unit, kind: i32, is_enemy: bool) -> bool {
    let accessory = unsafe { unit_get_accessory_list(unit, None)};
    let accessories = AccessoryData::get_list().unwrap();
    let mut dress_gender = unit_dress_gender(unit);
    if SEARCH_LIST.get().unwrap().job.iter()
        .find(|j| j.mount == Mount::Pegasus && j.job_hash == unit.job.parent.hash).is_some() { dress_gender = 2; }

    let rng = Random::get_game();
    let access: Vec<_> = accessories.iter()
        .filter(|acc|
            (is_enemy || acc.get_num() > 0) &&
                acc.kind == kind &&
                ( acc.condition_gender == 0 || acc.condition_gender == dress_gender )
            && accessory_gender_check(acc, unit)
        )
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

pub(crate) fn accessory_gender_check(acc: &AccessoryData, unit: &Unit) -> bool {
    let dress_gender = unit_dress_gender(unit);
    if acc.kind != 0 && acc.kind != 5 { true }
    else {
        if acc.parent.index < 43 {
            match acc.parent.index {
                1 => { dress_gender == 1 }
                2 => {  dress_gender == 2  }
                3..43 => {
                    let id = IDS[ acc.parent.index as usize ];
                    if ( id == 303 || (id % 100) >= 50 ) && dress_gender == 2 { true }
                    else if ( id != 303 && (id % 100) < 50 ) && dress_gender == 1 { true }
                    else { false }
                }
                _ => { true }
            }
        }
        else { true }
    }
}