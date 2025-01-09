use engage::gameuserdata::GameUserData;
use super::*;


pub struct RandomAssets;
impl ConfigBasicMenuItemSwitchMethods for RandomAssets {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("G_RandAsset", 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = GameVariableManager::get_number("G_RandAsset");
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            GameVariableManager::set_number("G_RandAsset", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match GameVariableManager::get_number("G_RandAsset") {
            1 => { "Weapons assets are randomized"  }
            2 => { "Info animations are randomized for player units."}
            3 => { "Weapons / player info animations are randomized"}
            _ => { "No assets are randomized." }
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text =  match GameVariableManager::get_number("G_RandAsset") {
            1 => { "Weapons"}
            2 => { "Info"}
            3 => { "Weapon+Info"}
            _ => { "None"}
        }.into();
    }
}

pub struct PlayerOutfits;
impl ConfigBasicMenuItemSwitchMethods for PlayerOutfits {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry("G_PlayerOutfit", 0); }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = GameVariableManager::get_bool("G_PlayerOutfit");
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            GameVariableManager::set_bool("G_PlayerOutfit", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            if GameVariableManager::get_bool("G_PlayerOutfit") { "Playable units will wear their equipped somniel outfits."  }
            else { "Playable units will be in their default outfits for battle." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = 
            if GameVariableManager::get_bool("G_PlayerOutfit") { "Casual" } 
            else { "Default" }.into();
    }
}

pub struct RandomEnemyOutfits;
impl ConfigBasicMenuItemSwitchMethods for RandomEnemyOutfits {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("G_EnemyOutfits", 0);
        GameVariableManager::make_entry("EnemyOutfits", GameVariableManager::get_number("G_EnemyOutfits"));
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

        this.help_text = if GameVariableManager::get_bool("EnemyOutfits") != GameVariableManager::get_bool("G_EnemyOutfits") {
            format!("{} (Press A to Confirm)", str1) }
        else { str1.to_string() }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if GameVariableManager::get_bool("EnemyOutfits") { "Randomized"  } 
                            else { "Normal "}.into();
    }
}

pub fn outfits_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_bool("EnemyOutfits") == GameVariableManager::get_bool("G_EnemyOutfits") { return BasicMenuResult::new();}
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
        GameVariableManager::set_bool("G_EnemyOutfits", GameVariableManager::get_bool("EnemyOutfits"));
        change_enemy_outfits();
        unsafe { 
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap());
            let index = menu.select_index;
            RandomEnemyOutfits::set_help_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn gather_all_accesories(){
    let accessory_list = AccessoryData::get_list().unwrap(); 
    /*
    if asset_table.len() < 5000 {
        for x2 in 3500..asset_table.len() {
            let asset_entry = &asset_table[x2];
            if asset_entry.conditions.is_none() { continue; }
            let conditions = unsafe { asset_table_get_conditions( asset_entry, None) };
            if let Some(con) = conditions.iter_mut().find(|str| str.contains("私服")) {
                *con = "!エンゲージ中".into();  
            }
        }
        unsafe { asset_table_on_completed_end(asset_table[0], None); }
    }
    */
    println!("Adding Accessory Assets...");
    let assets = include_str!("data/accessories.txt").lines();
    let mut asset_data = ASSET_DATA.lock().unwrap();
    if asset_data.assets.len() == 0 {
        assets.into_iter().for_each(|line|{
            let new_line = line.to_string();
            asset_data.add_asset_data(new_line);
        });
        println!("Accessory Assets Added");
        // Skipping Character unique outfits
        for x in 43..accessory_list.len() { asset_data.add_data(accessory_list[x]);  }
    }
    for x in 0..8 {
        println!("Male Entries for Accessory Type {}, {}", x, asset_data.male.n_entries[x]);
        println!("Female Entries for Accessory Type {}, {}", x, asset_data.female.n_entries[x]);
    }


}

pub fn accesorize_enemy_unit(enemy: &Unit) {
    let accessory_list = &mut unsafe { unit_get_accessory_list(enemy, None) }.unit_accessory_array;
    let length = accessory_list.len();
    if !GameVariableManager::get_bool("G_EnemyOutfits") && enemy.person.get_asset_force() != 0 { 
        for x in 0..length { accessory_list[x].index = 0; }
        return;
    }
    let rng = Random::get_game();
    let gender = unit_dress_gender(enemy);
    match gender  {
        1 => { 
            for x in 0..length {
                if x == 4 { continue; }
                accessory_list[x].index = ASSET_DATA.lock().unwrap().male.get_index(x as i32, rng);
            }
        },
        2 => {
            for x in 0..length {
                if x == 4 { continue; }
                accessory_list[x].index = ASSET_DATA.lock().unwrap().female.get_index(x as i32, rng);
            }
        },
        _ => {}, 
    }
}

fn change_enemy_outfits() {
    let force_type = [ForceType::Enemy, ForceType::Ally];
    for ff in force_type {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter {
            if unit.person.get_asset_force() == 0 { continue; }
            accesorize_enemy_unit(unit);
        }
    }
}

pub fn set_accessories_for_unit(unit: &Unit, result: &mut AssetTableResult) {
    let asset_data = ASSET_DATA.lock().unwrap();
    let index = unit.accessory_list.unit_accessory_array[0].index;
    let gender = unit_dress_gender(unit);
    if gender > 2 || gender == 0 { return; }
    let gen_str = if gender == 1 { "M" } else { "F" };
    let mode = get_unit_outfit_mode(unit);
    if mode != 0 || GameUserData::get_sequence() == 4 || unit.person.get_asset_force() != 0 {
        for x in 1..unit.accessory_list.unit_accessory_array.len() {
            if unit.accessory_list.unit_accessory_array[x].index <= 0 { continue; }
            let index = unit.accessory_list.unit_accessory_array[x].index;
            if let Some(data) = asset_data.assets.iter().find(|l| l.index == index) {
                if data.locator == 1 { change_accessory(result.accessory_list, data.asset.replace("X", &gen_str).as_str(), "c_head_loc"); }
                else if data.locator == 2 { change_accessory(result.accessory_list, data.asset.replace("X", &gen_str).as_str(), "c_head2_loc"); }
            }
        }
    }
    if ( mode == 2 || unit.person.get_asset_force() != 0 ) && GameUserData::get_sequence() != 4 { return; } 
    if mode == 1 || GameUserData::get_sequence() == 4 || unit.person.get_asset_force() != 0 {
        if unit.accessory_list.unit_accessory_array[0].index > 0 {
            if let Some(data) = asset_data.assets.iter().find(|x| x.index == index && x.gender == gender) {
                let asset = data.asset.replace("X", &gen_str);
                result.dress_model = asset.into();
            }
        }
    }
}

pub fn set_accessories_generic(result: &mut AssetTableResult, aid: &Il2CppString, gender: i32) {
    let belong = aid.to_string();
    match belong.as_str() {
        "AID_異形兵" => {
            if gender == 1 { add_accessory_to_list(&mut result.accessory_list, "uAcc_spine2_BoneMrp1AM", "c_spine2_jnt"); }
            else { add_accessory_to_list(&mut result.accessory_list, "uAcc_spine2_BoneMrp1AF", "c_spine2_jnt"); }
        }
        _ => {},
    }
}

pub fn add_accessory_to_list(list: &mut List<AssetTableAccessory>, model: &str, location: &str) {
    let accessory_class = Il2CppClass::from_name("App", "AssetTable").unwrap().get_nested_types().iter().find(|x| x.get_name() == "Accessory").unwrap();
    let new_accessory = Il2CppObject::<AssetTableAccessory>::from_class( accessory_class ).unwrap();
    new_accessory.model = Some(model.into() );
    new_accessory.locator = Some(location.into());
    unsafe { try_add_accessory_list(list, new_accessory, None); }
}
pub fn clear_accessory_from_list(list: &mut List<AssetTableAccessory>, model: &str) {
    for x in 0..list.len() {
        if let Some(accessory_model) = list[x].model {
            if accessory_model.contains(model) {
                list[x].model = Some("null".into());
            }
        }
    }
}

pub fn change_accessory(list: &mut List<AssetTableAccessory>, model: &str, locator: &str){
    if model != "null" {
        // check if accessory exists 
        if list.iter_mut().any(|f| f.model.filter(|m| m.to_string().contains(model)).is_some() ) { return; }
    }
    // check if locator exists then replace the model
    if let Some(acc) = list.iter_mut().find(|f| f.locator.filter(|m| m.to_string().contains(locator)).is_some() ) {
        acc.model = Some(model.clone().into());
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
    if increase {
        if let Some(new_index) = accessories.iter()
            .filter(|acc| acc.get_num() > 0 && acc.can_equip(unit) && acc.kind == kind && acc.parent.index > index )
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
            .filter(|acc| acc.get_num() > 0 && acc.can_equip(unit) && acc.kind == kind && acc.parent.index > index )
            .map(|acc| acc.parent.index).max() 
            {
                accessory.unit_accessory_array[kind as usize].index = new_index;
                return true;
            }
    }
    else {
        if let Some(new_index) = accessories.iter()
            .filter(|acc| acc.get_num() > 0 && acc.can_equip(unit) && acc.kind == kind && acc.parent.index < index )
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