use super::*;
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
    for x2 in 0..asset_table.len() {
        let asset_entry = &asset_table[x2];
        if asset_entry.conditions.is_none() { continue; }
        let conditions = unsafe { asset_table_get_conditions( asset_entry, None) };
        let mut find_somniel_condition_index = -1;
        for y in 0..conditions.len() {
            let con = conditions[y].get_string();
            if con.is_err() {  continue; }
            if crate::utils::str_contains(conditions[y], "PID_") {
                find_somniel_condition_index = -1;
                break;
            }
            let cond = con.unwrap();
            if cond == "私服" { find_somniel_condition_index = y as i32;  }
        }
        if find_somniel_condition_index != -1  {
            conditions[ find_somniel_condition_index as usize ] = "私服 | 緑軍 | 赤軍".into(); 
        }
    }
    // Skipping Character unique outfits
    for x in 43..accessory_list.len() {
        ASSET_DATA.lock().unwrap().add_data(accessory_list[x]);
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
            for x in 0..4 {
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


