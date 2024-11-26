use super::*;

pub static mut GENERIC_ACC: Vec<(i32, i32, String)> = Vec::new();

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
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent);
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
    let asset_table = AssetTable::get_list_mut().unwrap();
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


}

pub fn accesorize_enemy_unit(enemy: &Unit) {
    let accessory_list = &mut unsafe { unit_get_accessory_list(enemy, None) }.unit_accessory_array;
    let length = accessory_list.len();
    if !GameVariableManager::get_bool("G_EnemyOutfits") { 
        for x in 0..length { accessory_list[x].index = 0; }
        return;
    }
    let rng = Random::get_game();
    let gender = if enemy.edit.is_enabled() { enemy.edit.gender  } 
    else { enemy.person.get_gender() };
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
    if let Some(data) = asset_data.assets.iter().find(|x| x.index == index) {
        let asset = data.asset.replace("X", &gen_str);
        result.dress_model = asset.into();
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
    new_accessory.model = Some(model.clone().into() );
    new_accessory.locator = Some(location.clone().into());
    unsafe { try_add_accessory_list(list, new_accessory, None); }
}
pub fn clear_accessory_from_list(list: &mut List<AssetTableAccessory>, model: &str) {
    for x in 0..list.len() {
        if let Some(accessory_model) = list[x].model {
            if accessory_model.contains(model) {
                list[x].model = Some("null".into());
                //list[x].locator = Some("null".into());
            }
        }
    }
}

pub fn change_accessory(list: &mut List<AssetTableAccessory>, model: &str, locator: &str){
    if let Some(acc) = list.iter_mut().find(|f| if f.locator.is_some() { f.locator.unwrap().contains(locator) } else { false }) {
        acc.model = Some(model.clone().into());
    }
    else { add_accessory_to_list(list, model, locator); }
}

pub fn accessory_at_locator(list: &List<AssetTableAccessory>, locator: &str) -> Option<String> {
    if let Some(acc) = list.iter().find(|f| if f.locator.is_some() { f.locator.unwrap().contains(locator) } else { false }) {
        if let Some(model) = acc.model {
            let str = model.to_string();
            if str.len() == 0 { return None; }
            else { return Some(str.clone()); }
        }
    }
    None
}

pub fn add_generic_unit_acc(unit: &Unit, list: &mut List<AssetTableAccessory>) {
    let ident = unit.ident;
    let generic_list = unsafe { &mut GENERIC_ACC };
    let locators = ["c_head_loc", "c_spine2_jnt", "c_head2_loc", "c_spine1_jnt"];

    if generic_list.iter().any(|v| v.0 == ident) {
        generic_list.iter()
            .filter(|v| v.0 == ident)
            .for_each(|v|{
                change_accessory(list, v.2.as_str(), locators[ v.1 as usize]);
                //println!("Replace acc with {} for #{}, {}", v.2, ident, Mess::get(unit.get_job().jid));
            }
        );
        return;
    }
    println!("Searching for #{}, {} at {}, {}", ident, Mess::get(unit.get_job().name), unit.x, unit.z);
    for x in 0..4 {
        if let Some(model) = accessory_at_locator(list, locators[x]) {
            println!("Added: {} for #{}, {} at {}, {}", model, ident, Mess::get(unit.get_job().name), unit.x, unit.z);
            if !generic_list.iter().any(|v| v.0 == ident && v.1 == x as i32) {
                generic_list.push( ( ident, x as i32, model.clone() ) ); 
            }
        }
    }
}
pub fn clear_generic_acc() {
    println!("Clearing Generic Accessory List");
    unsafe { GENERIC_ACC.clear() };
}
