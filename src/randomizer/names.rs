use super::*;
use engage::force::*;
use crate::{utils::*, CONFIG};
pub static mut EMBLEM_NAMES: [i32; 25] = [-1; 25];

pub struct RandomNameMods;
impl ConfigBasicMenuItemSwitchMethods for RandomNameMods {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_names);
        if CONFIG.lock().unwrap().random_names!= result {
            CONFIG.lock().unwrap().random_names  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().random_names {"Emblem will have random names and appearances if possible." }
            else { "Emblem will have their default name and appearances." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_names { "Randomized" } else { "Default" }.into();
    }
}

pub struct GenericAppearance;
impl ConfigBasicMenuItemSwitchMethods for GenericAppearance {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().generic_mode } else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if DVCVariables::is_main_menu()  {  CONFIG.lock().unwrap().generic_mode  = result; }
            else { GameVariableManager::set_number(DVCVariables::GENERIC_APPEARANCE_KEY, result) }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        //G_GenericMode
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap(). generic_mode } else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };
        let str = if DVCVariables::is_main_menu() || value == 0 { "" } else { " (Press A to reseed.)"};

        this.help_text = format!("{}{}",
            match value {
                1 => { "Randomizes generic units' appearance."}
                2 => { "Randomizes generic units' colors."}
                3 => { "Randomized generic units' appearance and color."}
                _ => { "Default appearance for generic enemies." }
            }, str).into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap(). generic_mode } else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };
        this.command_text = 
            match value {
                1 => { "Appearance"}
                2 => { "Colors"}
                3 => { "All"}
                _ => { "Default" }
            }.into();
    }
}

pub extern "C" fn vibe_generic() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<GenericAppearance>("Generic Enemy Appearance");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = generic_acall as _ );
    switch
}

pub fn generic_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::is_main_menu() { return BasicMenuResult::new(); }
    let mode = GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
    let msg; 
    match mode {
        1 => { msg = "Reseed generic enemy appearance?" }
        2 => { msg = "Reseed generic enemy colors?"}
        3 => { msg = "Reseed generic enemy appearance/colors?"}
        _ => { return BasicMenuResult::new() }
    }
    YesNoDialog::bind::<ReseedEnemyConfirm>(this.menu, msg, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub struct ReseedEnemyConfirm;
impl TwoChoiceDialogMethods for ReseedEnemyConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        change_enemy_seed();
        crate::assets::accessory::change_enemy_outfits();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

fn change_enemy_seed() {
    let rng = Random::get_game();
    let mode = GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY);
    Force::get(ForceType::Enemy).unwrap().iter()
        .for_each(|unit|{
            if unit.person.get_asset_force() > 0 {
                unsafe { 
                    if mode & 2 != 0 { set_grow_seed(unit, rng.value(), None); }
                    if mode & 1 != 0 { set_drop_seed(unit, rng.value(), None); }
                }
            }
        }
    );
}

#[unity::from_offset("App", "Unit", "set_GrowSeed")]
fn set_grow_seed(this: &Unit, value: i32, _method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "set_DropSeed")]
fn set_drop_seed(this: &Unit, value: i32, _method_info: OptionalMethod);

#[unity::from_offset("App", "PersonData", "get_Belong")]
pub fn get_person_bid(this: &PersonData, method_info: OptionalMethod) -> Option<&Il2CppString>;

pub fn randomize_emblem_names() {
    let name_size = if dlc_check() { 40 } else { 35 };
    let mut used: [bool; 41] = [false; 41];
    let mut is_female: [bool; 41] = [false; 41];
    for x in 1..41 {
        let person = PersonData::get(PIDS[x]).unwrap();
        is_female[x] = !(person.gender == 1 && person.get_flag().value & 32 == 0);
    }
    if GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) {
        let rng = get_rng();
        let mut emblem_count = 0;
        EMBLEM_ASSET.iter().for_each(|&gid|{
            if let Some(god) = GodData::get_mut(format!("GID_{}", gid)) {
                let god_female = god.female == 1;
                loop {
                    let value = rng.get_value(name_size) + 1;
                    if !used[value as usize] && is_female[value as usize] == god_female {
                        unsafe { EMBLEM_NAMES[ emblem_count] = value; }
                        god.mid = MPIDS[ value as usize ].into();
                        emblem_count += 1;
                        used[value as usize] = true;
                        break;
                    }
                }
            }
        });
    }
}