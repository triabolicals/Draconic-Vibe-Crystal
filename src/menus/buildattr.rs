use engage::{
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
    menu::{BasicMenuItem, BasicMenuItemAttribute},
};
use unity::prelude::OptionalMethod;
use crate::config::DVCVariables;

pub fn job_gauge_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if DVCVariables::is_main_menu()  { BasicMenuItemAttribute::Enable }
    else if !DVCVariables::random_enabled() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number(DVCVariables::JOB_KEY) > 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub fn skill_gauge_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if DVCVariables::is_main_menu() { BasicMenuItemAttribute::Enable }
    else if GameVariableManager::get_bool(DVCVariables::SKILL_KEY) && GameUserData::get_sequence() > 3 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

pub fn hub_item_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if DVCVariables::is_main_menu() { BasicMenuItemAttribute::Enable }
    else if GameVariableManager::get_number(DVCVariables::ITEM_KEY) != 0 && GameUserData::get_sequence() != 5 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}
pub fn not_in_map_sortie_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if DVCVariables::is_main_menu() {BasicMenuItemAttribute::Enable  }
    else if GameUserData::get_sequence() < 4 || !DVCVariables::random_enabled() { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub fn not_in_map_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if DVCVariables::is_main_menu() {BasicMenuItemAttribute::Enable  }
    else if !DVCVariables::random_enabled() || GameUserData::get_sequence() == 3 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub fn build_attribute_normal(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if !DVCVariables::random_enabled() && !DVCVariables::is_main_menu() { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}
pub fn build_attribute_not_in_map2(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() < 4 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}
pub fn dlc_build_attr(_this: &BasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if crate::utils::dlc_check() { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
}