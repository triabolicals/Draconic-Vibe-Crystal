use super::*;
use engage::force::*;
use crate::{utils::*, CONFIG};

use super::person::PLAYABLE;
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
        this.command_text = if CONFIG.lock().unwrap().random_names { "Randomized" }
            else { "Default" }.into();
    }
}

pub struct GenericAppearance;
impl ConfigBasicMenuItemSwitchMethods for GenericAppearance {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().generic_mode } else { GameVariableManager::get_number("G_GenericMode") };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if GameUserData::get_sequence() == 0  {  CONFIG.lock().unwrap().generic_mode  = result; }
            else { GameVariableManager::set_number("G_GenericMode", result) }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        //G_GenericMode
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap(). generic_mode } else { GameVariableManager::get_number("G_GenericMode") };
        let str = if GameUserData::get_sequence() == 0 || value == 0 { "" } else { " (Press A to reseed.)"};

        this.help_text = format!("{}{}",
            match value {
                1 => { "Randomizes generic units' appearance."}
                2 => { "Randomizes generic units' colors."}
                3 => { "Randomized generic units' appearance and color."}
                _ => { "Default appearance for generic enemies." }
            }, str).into();

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap(). generic_mode } else { GameVariableManager::get_number("G_GenericMode") };
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
    let switch = ConfigBasicMenuItem::new_switch::<GenericAppearance>("Generic Units Settings");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = generic_acall as _ );
    switch
}

pub fn generic_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 { return BasicMenuResult::new(); }
    let mode = GameVariableManager::get_number("G_GenericMode");
    let msg; 
    match mode {
        1 => { msg = "Reseed generic enemy appearance?" }
        2 => { msg = "Ressed generic enemy colors?"}
        3 => { msg = "Reseed generic enemy appearance/colors?"}
        _ => { 
            return BasicMenuResult::new() }
    }
    YesNoDialog::bind::<ReseedEnemyConfirm>(this.menu, msg, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub struct ReseedEnemyConfirm;
impl TwoChoiceDialogMethods for ReseedEnemyConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        change_enemy_seed();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

fn change_enemy_seed() {
    let force_type = [ForceType::Enemy, ForceType::Ally];
    let rng = Random::get_game();
    let mode = GameVariableManager::get_number("G_GenericMode");
    for ff in force_type {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter {
            if unit.person.get_asset_force() == 0 { continue; }
            unsafe { 
                if mode & 2 == 2 { set_grow_seed(unit, rng.value(), None); }
                if mode & 1 == 1 { set_drop_seed(unit, rng.value(), None); }
            }
        }
    }
}

#[unity::from_offset("App", "Unit", "set_GrowSeed")]
fn set_grow_seed(this: &Unit, value: i32, _method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "set_DropSeed")]
fn set_drop_seed(this: &Unit, value: i32, _method_info: OptionalMethod);

#[unity::from_offset("App", "PersonData", "get_Belong")]
pub fn get_person_bid(this: &PersonData, method_info: OptionalMethod) -> Option<&Il2CppString>;

pub fn give_names_to_generics() {
    if !crate::utils::can_rand() { return; }
    let list = PersonData::get_list_mut().unwrap();
    if GameVariableManager::get_bool("G_Random_Names") {
        randomize_emblem_names();
        println!("Emblem Name Randomized");
    }
    list.iter_mut()
        .filter(|p| 
            p.get_name().is_some() && unsafe { get_person_bid(p, None ).is_some() } && p.gender != 0 
            && !PLAYABLE.lock().unwrap().iter().any(|&y| y == p.parent.index) 
            && !p.pid.to_string().contains("Boss")
            && p.get_flag().value & 2048 == 0
                //&& !name_list.male.iter().any(|&y| y == p.parent.index as i16) && !name_list.female.iter().any(|&y| y ==  p.parent.index as i16)
        )
        .for_each(|person|{
            let flag_value = person.get_flag();
            flag_value.value |= 2048;
        }
    );
}

pub fn randomize_emblem_names() {
    let name_size = if dlc_check() { 40 } else { 35 };
    let mut used: [bool; 41] = [false; 41];
    let mut is_female: [bool; 41] = [false; 41];
    for x in 1..41 {
        let person = PersonData::get(PIDS[x]).unwrap();
        is_female[x] = !(person.gender == 1 && person.get_flag().value & 32 == 0);
    }
    if GameVariableManager::get_bool("G_Random_Names") {
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