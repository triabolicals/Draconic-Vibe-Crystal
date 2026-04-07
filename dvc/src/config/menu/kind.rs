use super::*;

mod command;
mod submenu;
pub(crate) mod order;

use engage::dialog::BasicDialog2;
use engage::gamedata::JobData;
use engage::gamemessage::GameMessage;
use engage::gameuserdata::GameUserData;
use engage::menu::{BasicMenuItemAttribute, BasicMenuResult};
use engage::mess::Mess;
use engage::pad::{NpadButton, Pad};
use unity::prelude::OptionalMethod;
use unity::system::action::Action;
use crate::config::DVCFlags;


pub use command::*;
pub use submenu::*;
pub use order::*;
use crate::config::DVCFlags::PostChapterAutolevel;
use crate::DVCVariables;
use crate::DVCVariables::SingleJob;
use crate::randomizer::data::RandomizedGameData;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum DVCMenuItemKind {
    Menu(DVCMenu),
    Variable(DVCVariables),
    Gauge(DVCVariables),
    Flag(DVCFlags),
    Command(DVCCommand),
    Order(RecruitmentOrder),
    SingleJob,
}

impl DVCCMenuItem for DVCMenuItemKind {
    fn a_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        match self {
            DVCMenuItemKind::Variable(variables) => {
                if variables.can_a_call() && (variables.get_value() != item.dvc_value) {
                    let action = Action::new_method_mut(Some(item), change_variable);
                    let message = DVCConfigText::change_text(item.title.to_string().as_str(), item.command_text);
                    BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
                    BasicMenuResult::se_cursor()
                }
                else { BasicMenuResult::se_miss() }
            }
            DVCMenuItemKind::Flag(flags) => {
                match flags {
                    PostChapterAutolevel => {
                        if flags.can_a_call(item.dvc_value != 0) {
                            if let Some((level, count)) = crate::autolevel::autolevel_party() {
                                let message = format!("{} unit{} leveled up to Lv. {}", count, if count > 1 { "s" } else { "" }, level);
                                GameMessage::create_key_wait(item.menu, message);
                                return BasicMenuResult::se_cursor();
                            }
                        }
                        BasicMenuResult::new()
                    }
                    DVCFlags::BGM => {
                        if flags.can_a_call(item.dvc_value != 0) {
                            RandomizedGameData::a_call_menu_action(*self);
                            BasicMenuResult::se_cursor()
                        }
                        else { BasicMenuResult::new() }
                    }
                    _ => {
                        if flags.can_a_call(item.dvc_value != 0){
                            let action = Action::new_method_mut(Some(item), change_variable);
                            let message = DVCConfigText::change_text(item.title.to_string().as_str(), item.command_text);
                            BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
                            BasicMenuResult::se_cursor()
                        }
                        else { BasicMenuResult::se_miss() }
                    }
                }
            }
            DVCMenuItemKind::SingleJob => {
                if DVCVariables::is_main_menu() || (item.dvc_value == SingleJob.get_value() && DVCFlags::SingleJobEnabled.get_value()) {
                    BasicMenuResult::se_miss()
                }
                else {
                    let action = Action::new_method_mut(Some(item), change_variable);
                    let message =
                    match item.dvc_value {
                        1 => { format!("Change the class line to\n'{} (DLC)'?", Mess::get_name("MJID_ShadowLordR")) }
                        _ => {
                            if let Some(job) = JobData::try_get_hash(item.dvc_value) {
                                format!("Change the class line to\n'{}'?", Mess::get_name(job.jid))
                            }
                            else { "???".to_string() }
                        }
                    };
                    BasicDialog2::create_confirm_cancel_bind(item.menu, message, Some(action));
                    BasicMenuResult::se_cursor()
                }
            }
            DVCMenuItemKind::Menu(menu) => menu.a_call(item),
            DVCMenuItemKind::Command(command) => { command.a_call(item) }
            _ => { BasicMenuResult::se_miss() }
        }
    }
    fn minus_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        match self {
            DVCMenuItemKind::Order(order) => { order.minus_call(item) }
            _ => BasicMenuResult::new(),
        }
    }
    fn plus_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        match self {
            DVCMenuItemKind::Order(order) => { order.plus_call(item) }
            _ => BasicMenuResult::new(),
        }
    }
    fn custom_call(&self, item: &mut DVCConfigMenuItem) -> BasicMenuResult {
        if item.attribute & 2 != 0 { BasicMenuResult::new() }
        else {
            match self {
                DVCMenuItemKind::Order(order) => { order.custom_call(item) }
                DVCMenuItemKind::Gauge(gauge) => {
                    if let Some(increase) = get_change(true){
                        let v = item.dvc_value;
                        let v2 = gauge.increment(v, increase);
                        item.gauge_ratio = (v2 as f32) / 100.0;
                        item.dvc_value = v2;
                        item.update_config_text();
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::new() }
                }
                DVCMenuItemKind::Variable(DVCVariables::ClassMode) => {
                    let current = item.dvc_value;
                    if let Some(increase) = get_change(true) {
                        if !DVCVariables::is_main_menu() {
                            if current == 2 && DVCFlags::SingleJobEnabled.get_value() {
                                item.padding = increase as u8;  // Increase/Decrease
                                let action = Action::new_method_mut(Some(item), oops_all_change);
                                BasicDialog2::create_confirm_cancel_bind(item.menu, "Disable 'Opps All' mode?", Some(action));
                                return BasicMenuResult::se_cursor()
                            }
                        }
                        let new_value = DVCVariables::ClassMode.increment(current, increase);
                        item.dvc_value = new_value;
                        item.update_config_text();
                        DVCMenu::rebuild_menu_variable_change(item);
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::new() }
                }
                DVCMenuItemKind::Variable(variables) => {
                    if let Some(increase) = get_change(true) {
                        let current = item.dvc_value;
                        let new_value = variables.increment(current, increase);
                        item.is_command = variables.get_value() != new_value;
                        item.dvc_value = new_value;
                        item.update_config_text();
                        DVCMenu::rebuild_menu_variable_change(item);
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::new() }
                }
                DVCMenuItemKind::Flag(flags) => {
                    if get_change(true).is_some(){
                        // println!("Current: {}", item.dvc_value);
                        let current = item.dvc_value != 0;
                        item.dvc_value = if current { 0 } else { 1 };
                        if flags.need_confirm_to_change() { item.is_command = flags.get_value() != !current; }
                        else {
                            flags.set_value(!current);
                            item.is_command = flags.can_a_call(!current);
                        }
                        // println!("Flag: #{} DVC Value: {} / Var: {}", *flags as i32, item.dvc_value, flags.get_value());
                        item.update_config_text();
                        DVCMenu::rebuild_menu_variable_change(item);
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::new() }
                }
                Self::SingleJob => {
                    if let Some(increase) = get_change(true) {
                        let current_value = item.dvc_value;
                        let new_value = crate::randomizer::job::single::get_next_class(current_value, increase);
                        item.dvc_value = new_value;
                        if DVCVariables::is_main_menu() { SingleJob.set_value(new_value); }
                        item.update_config_text();
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::new() }
                }
                _ => BasicMenuResult::new(),
            }
        }
    }
}

pub fn change_variable(menu_item: &mut DVCConfigMenuItem, _optional_method: OptionalMethod) {
    match menu_item.menu_item_kind {
        DVCMenuItemKind::Variable(variables) => {
            variables.set_value(menu_item.dvc_value);
            menu_item.is_command = false;
        }
        DVCMenuItemKind::Flag(flags) => {
            flags.set_value(menu_item.dvc_value != 0);
            if flags == PostChapterAutolevel && flags.get_value() {
                if let Some((level, count)) = crate::autolevel::autolevel_party() {
                    let message = format!("{} unit{} leveled up to Lv. {}", count, if count > 1 { "s" } else { "" }, level);
                    GameMessage::create_key_wait(menu_item.menu, message);
                    return;
                }
            }
            else {
                menu_item.is_command = false;
                menu_item.update_config_text();
            }
        }
        DVCMenuItemKind::SingleJob => {
            menu_item.is_command = false;
            SingleJob.set_value(menu_item.dvc_value);
            DVCFlags::SingleJobEnabled.set_value(true);
            DVCVariables::ClassMode.set_value(2);
        }
        _ => {}
    }
    RandomizedGameData::a_call_menu_action(menu_item.menu_item_kind.clone());
    menu_item.update_text();
}
fn oops_all_change(menu_item: &mut DVCConfigMenuItem, _: OptionalMethod) {
    let increase = menu_item.padding != 0;
    menu_item.dvc_value = if increase { 0 } else { 1 };
    DVCVariables::ClassMode.set_value(menu_item.dvc_value);
    DVCFlags::SingleJobEnabled.set_value(false);
    menu_item.update_config_text();
    DVCMenu::rebuild_menu_variable_change(menu_item);
}
fn get_change(trigger: bool) -> Option<bool> {
    let left;
    let right;
    if trigger {
        left = Pad::is_trigger(NpadButton::left_key());
        right = Pad::is_trigger(NpadButton::right_key());

    }
    else {
        left = Pad::is_button(NpadButton::left_key());
        right = Pad::is_button(NpadButton::right_key());
    }
    if left && !right { Some(false) }
    else if right && !left { Some(true) }
    else { None }
}