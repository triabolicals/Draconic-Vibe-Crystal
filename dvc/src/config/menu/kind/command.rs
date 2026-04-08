use engage::gamemessage::GameMessage;
use engage::gamevariable::GameVariableManager;
use engage::keyboard::SoftwareKeyboard;
use unity::system::action::Action1;
use crate::{DeploymentConfig, CONFIG};
use crate::randomizer::data::GameData;
use crate::randomizer::RANDOMIZER_STATUS;
use crate::randomizer::status::RandomizerStatus;
use crate::utils::{can_rand, get_random_number_for_seed};
use super::*;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum DVCCommand {
    SetSeed,
    ReRandJob,
}

impl DVCCMenuItem for DVCCommand {
    fn a_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        match self {
            DVCCommand::SetSeed => {
                let action = Action1::<Il2CppString>::new_with_method_mut(Some(item), set_string_value);
                let v =
                    if DeploymentConfig::get().seed == 0 { get_random_number_for_seed() }
                    else { DeploymentConfig::get().seed };
                
                SoftwareKeyboard::create_bind(
                    item.menu,
                    10,
                    Some(format!("{}", v).into()),
                    Some("Enter text / numeric value for DVC seed.".into()),
                    Some(format!("Number Range: 1 to {}.", u32::MAX).into()),
                    0,
                    Some(action),
                );
            }
            DVCCommand::ReRandJob => {
                let action = Action::new_method_mut(Some(item), re_rand_job);
                let message =
                    if GameVariableManager::get_bool("G_Cleared_M003") { "Re-Randomize unrecruited player classes?" }
                    else { "Re-Randomize player unit classes?" };

                BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
            }
        }
        BasicMenuResult::se_cursor()
    }
    fn plus_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        match self {
            DVCCommand::SetSeed => {
                let v = get_random_number_for_seed();
                if DVCVariables::Seed.get_value() != v as i32 {
                    item.dvc_value = v as i32;
                    let message = format!("Set Seed to {}?", v);
                    let action = Action::new_method_mut(Some(item), set_seed_yes);
                    BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
                    BasicMenuResult::se_cursor()
                } else { BasicMenuResult::new() }
            }
            _ => { BasicMenuResult::new() }
        }
    }
    fn build_attribute(&self, _item: &DVCConfigMenuItem) -> BasicMenuItemAttribute {
        match self {
            DVCCommand::SetSeed => { BasicMenuItemAttribute::Enable }
            DVCCommand::ReRandJob => { crate::randomizer::job::re_rand_jobs_build_attr() }
        }
    }
}
fn set_string_value(item: &mut DVCConfigMenuItem, value: &Il2CppString, _: OptionalMethod) {
    if value.is_null() { return; }
    let v = value.to_string();
    if v.len() == 0 { return; }
    let seed = if let Ok(seed) = v.parse::<u32>() { seed } else { engage::ut::Ut::hash_fnv_1_string(value) as u32 };
    if seed == 0 || seed == DVCVariables::Seed.get_value() as u32 { return; }
    if !DVCVariables::is_main_menu() {
        item.dvc_value = seed as i32;
        let message =
            if DVCVariables::Seed.get_value() == 0 { format!("Set Randomizer Seed to:\n{}?", seed) }
            else { format!("Change save file seed to:\n{}?", seed) };

        let action = Action::new_method_mut(Some(item), set_seed_yes);
        BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
    }
}
fn set_seed_yes(item: &mut DVCConfigMenuItem, _: OptionalMethod) {
    let not_random = DVCVariables::Seed.get_value() == 0;
    update_seed(item.dvc_value as u32);
    if not_random {
        DeploymentConfig::get().create_game_variables(false);
        DVCMenu::Main.rebuild_menu(item, true);
    }
    else { item.update_config_text(); }
}
fn update_seed(new_seed: u32) {
    if new_seed == DVCVariables::Seed.get_value() as u32 { return; }
    DVCVariables::Seed.set_value(new_seed as i32);
    if !DVCVariables::is_main_menu() {
        RandomizerStatus::set_init(false);
        GameVariableManager::find_starts_with("G_P_PID").iter()
            .for_each(|person_key| GameVariableManager::set_number(person_key.to_string().as_str(), 0));

        GameVariableManager::find_starts_with("G_L_JID").iter()
            .for_each(|job_key| GameVariableManager::set_number(job_key.to_string().as_str(), 0));

        let data = GameData::get();
        let mut rando = RandomizedGameData::get_write();
        data.emblem_pool.reset_all();
        rando.randomize(data);
        rando.commit(data);
        println!("Seed updated: {}", new_seed);
        let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.seed = GameVariableManager::get_number(DVCVariables::SEED));
    }
}
fn re_rand_job(item: &mut DVCConfigMenuItem, _: OptionalMethod) {
    let units = crate::randomizer::job::rerandomize_jobs();
    if units.len() == 0 { return; }
    else {
        let mut str = String::new();
        units.iter().for_each(|unit| {
            if !str.is_empty() { str += "\n"; }
            str += unit.as_str();
        });
        GameMessage::create_key_wait(item.menu, str);
    }
}