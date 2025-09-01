use super::*;
use engage::force::*;
use crate::{utils::*, CONFIG};
use crate::assets::animation::MONSTERS;
use crate::randomizer::person::{ENEMY_PERSONS, PERSONS_LIST};

pub static mut EMBLEM_NAMES: [i32; 25] = [-1; 25];
pub static mut NPCS_NAMES: [i32; 25] = [-1; 25];
pub struct RandomNameMods;
impl ConfigBasicMenuItemSwitchMethods for RandomNameMods {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry(DVCVariables::EMBLEM_NAME_KEY, 0);
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_names();
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_random_names(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else {BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if DVCVariables::get_random_names() { "Randomized" } else { "Default" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if DVCVariables::get_random_names() {"Emblem will have random names and appearances if possible." }
            else { "Emblem will have their default name and appearances." }.into();
    }
}

pub struct GenericAppearance;
impl ConfigBasicMenuItemSwitchMethods for GenericAppearance {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry_norewind(DVCVariables::GENERIC_APPEARANCE_KEY, 0);
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().generic_mode }
            else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if DVCVariables::is_main_menu()  {  CONFIG.lock().unwrap().generic_mode  = result; }
            else { GameVariableManager::set_number(DVCVariables::GENERIC_APPEARANCE_KEY, result) }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap(). generic_mode }
        else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };
        this.command_text =
            match value {
                1 => { "Appearance"}
                2 => { "Colors"}
                3 => { "All"}
                _ => { "Default" }
            }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        //G_GenericMode
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap(). generic_mode }
        else { GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) };

        let str = if DVCVariables::is_main_menu() || value == 0 { "" } else { " (Press A to reseed.)"};

        this.help_text = format!("{}{}",
            match value {
                1 => { "Randomizes generic units' appearance."}
                2 => { "Randomizes generic units' colors."}
                3 => { "Randomized generic units' appearance and color."}
                _ => { "Default appearance for generic enemies." }
            }, str).into();
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
    BasicMenuResult::new()
}
pub struct ReseedEnemyConfirm;
impl TwoChoiceDialogMethods for ReseedEnemyConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        change_enemy_seed();
        crate::assets::accessory::change_enemy_outfits();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        BasicMenuResult::new().with_close_this(true)
    }
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
pub fn get_new_npc_person(index: usize) -> Option<&'static mut PersonData> {
    unsafe {
        NPCS_NAMES.get(index).filter(|x| **x >= 0)
            .and_then(|x| PERSONS_LIST.get().unwrap().get(*x as usize))
            .and_then(|x| PersonData::try_get_hash_mut(x.0))
    }
}
pub fn get_new_npc_person_name(index: usize) -> Option<String> {
    unsafe {
        NPCS_NAMES.get(index).filter(|x| **x >= 0)
            .and_then(|x| PERSONS_LIST.get().unwrap().get(*x as usize))
            .map(|x| x.2.clone())
    }
}
pub fn get_emblem_person(mid: &Il2CppString) -> Option<&'static PersonData> {
    if !GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) { return None; }
    let mut key = format!("G_GN_{}", mid);
    if mid.str_contains("Lueur") {
        if DVCVariables::is_lueur_female() { key.push('F'); } else { key.push('M'); }
    }
    let hash = GameVariableManager::get_number(key.as_str());
    PersonData::try_get_hash(hash)
}
pub fn randomize_emblem_npc_names() {
    let emblem_name_count = GameVariableManager::find_starts_with("G_GN_").len();
    let emblem_count = EMBLEM_LIST.get().unwrap().len() + 1;
    if emblem_count > emblem_name_count {
        let mut males: Vec<_> =
            PLAYABLE.get().unwrap().iter().enumerate().filter(|(i, x)| *i > 0 && PersonData::try_index_get(**x)
                .is_some_and(|p|
                    p.jid.is_some_and(|x| !MONSTERS.iter().any(|m| x.str_contains(m))) &&
                        p.get_flag().value & 128 == 0 && (p.gender == 1 && p.get_flag().value & 32 == 0) || (p.gender == 2 && p.get_flag().value & 32 != 0)))
                .map(|(_, p)| PersonData::try_index_get(*p).map(|x| x.parent.hash).unwrap()).collect();

        let mut females: Vec<_> =
            PLAYABLE.get().unwrap().iter().enumerate().filter(|(i, x)| *i > 0 && PersonData::try_index_get(**x)
                .is_some_and(|p|
                    p.jid.is_some_and(|x| !MONSTERS.iter().any(|m| x.str_contains(m))) &&
                        p.get_flag().value & 128 == 0 && (p.gender == 2 && p.get_flag().value & 32 == 0) || (p.gender == 1 && p.get_flag().value & 32 != 0)))
                .map(|(_, p)| PersonData::try_index_get(*p).map(|x| x.parent.hash).unwrap()).collect();

        let rng = get_rng();
        EMBLEM_LIST.get().unwrap().iter()
            .map(|&hash| GodData::try_get_hash(hash).unwrap())
            .for_each(|god| {
                let god_female = god.female == 1;
                let is_lueur = god.parent.index == 13;
                let name_key = format!("G_GN_{}", god.mid);
                if god_female || god.parent.index == 13 {
                    let pool_size = females.len();
                    if pool_size > 1 {
                        let value = rng.get_value(pool_size as i32);
                        if is_lueur { GameVariableManager::make_entry(format!("G_GN_{}F", god.mid).as_str(), females[value as usize]); }
                        else { GameVariableManager::make_entry(name_key.as_str(), females[value as usize]); }
                        females.remove(value as usize);
                    }
                }
                if !god_female || god.parent.index == 13 {
                    let pool_size = males.len();
                    if pool_size > 1 {
                        let value = rng.get_value(pool_size as i32);
                        if is_lueur { GameVariableManager::make_entry(format!("G_GN_{}M", god.mid).as_str(), males[value as usize]); }
                        else { GameVariableManager::make_entry(name_key.as_str(),  males[value as usize]); }
                        males.remove(value as usize);
                    }
                }
            });
    }
    if let Some(enemies) = ENEMY_PERSONS.get(){
        enemies.iter().filter(|x| x.0 >= 150).filter_map(|x| PersonData::try_index_get_mut(x.1) )
            .for_each(|person| { person.get_flag().value |= 2048; });

        if let Some(npcs) =  PERSONS_LIST.get(){
            let rng = get_rng();
            let mut npcs_indexes = npcs.iter().enumerate()
                .map(|x| (x.0, x.1.1)).collect::<Vec<_>>();

            for enemy_index in 0..17 {
                if let Some(x) = enemies.iter().find(|x| x.0 == (enemy_index + 150))
                    .and_then(|x| PersonData::try_index_get(x.1)).map(|x| x.gender)
                {
                    let count = npcs_indexes.len() as i32;
                    if count > 3 {
                        loop {
                            let sel = rng.get_value(count) as usize;
                            if npcs_indexes[sel].1 == x {
                                unsafe {
                                    NPCS_NAMES[enemy_index as usize] = npcs_indexes[sel].0 as i32;
                                }
                                npcs_indexes.remove(sel);
                                break;
                            }
                        }
                    }
                }
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