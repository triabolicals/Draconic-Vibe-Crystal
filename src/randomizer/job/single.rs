use engage::dialog::yesno::{BasicDialogItemNo, BasicDialogItemYes, TwoChoiceDialogMethods, YesNoDialog};
use engage::gamedata::{Gamedata, JobData};
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::menu::BasicMenuResult;
use engage::menu::config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods};
use engage::mess::Mess;
use engage::pad::Pad;
use engage::unitpool::UnitPool;
use engage::util::get_instance;
use unity::prelude::OptionalMethod;
use crate::CONFIG;
use crate::config::DVCVariables;
use crate::menus::utils;
use crate::randomizer::job::unit_change_to_random_class;
use crate::utils::dlc_check;

pub struct SingleJob;

impl ConfigBasicMenuItemSwitchMethods for SingleJob {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry(DVCVariables::SINGLE_CLASS, 0);
            GameVariableManager::make_entry("SINGLE_CLASS", GameVariableManager::get_number(DVCVariables::SINGLE_CLASS));
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad = get_instance::<Pad>();
        if GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3 {
            return BasicMenuResult::new();
        }
        if pad.npad_state.buttons.left() && !pad.old_buttons.left() {
            get_next_class(false);
            Self::set_command_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        else if pad.npad_state.buttons.right() && !pad.old_buttons.right() {
            get_next_class(true);
            Self::set_command_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        else if !DVCVariables::is_main_menu(){
            let new_class = GameVariableManager::get_number("SINGLE_CLASS");
            if new_class != GameVariableManager::get_number(DVCVariables::SINGLE_CLASS) {
                if pad.npad_state.buttons.a() {
                    let message =
                        if let Some(class) = JobData::try_get_hash(new_class) {
                            format!("Change to {}?", Mess::get_name(class.jid))
                        } else { "Turn off Single Class? (Not Reversible)".to_string() };
                    YesNoDialog::bind::<ChangeSingleClassConfirm>(this.menu, message, "Do it!", "Nah..");
                    return BasicMenuResult::se_cursor();
                }
            }
        }
        BasicMenuResult::new()
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let is_main = DVCVariables::is_main_menu();
        let hash =
            if is_main { CONFIG.lock().unwrap().single_class }
            else { GameVariableManager::get_number("SINGLE_CLASS") };
        let changed =
            if is_main { "".to_string() }
            else { DVCVariables::changed_setting_text("SINGLE_CLASS", DVCVariables::SINGLE_CLASS) };

        if let Some(job) = JobData::try_get_hash(hash) { this.command_text = format!("{}: {}{}", job.parent.index, Mess::get_name(job.jid), changed).into(); }
        else {
            if is_main { CONFIG.lock().unwrap().single_class = 0; }
            this.command_text = format!("None{}", changed).into(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3 {
            this.help_text = "Cannot change this setting while in-map.".into();
        }
        else { this.help_text = "Class line that all playable units will use.".into(); }
    }
}


fn get_next_class(increase: bool) {
    let job_count = JobData::get_count();
    let hash =
        if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().single_class }
        else { GameVariableManager::get_number("SINGLE_CLASS") };

    let mut current_index = JobData::try_get_hash(hash)
        .filter(|x| x.flag.value & 23 == 3 && (x.is_high() || x.max_level >= 40))
        .map(|j| j.parent.index).unwrap_or(0);

    loop {
        if increase {
            if current_index < 0 { current_index = 1; }
            else if current_index < job_count { current_index += 1 }
            else { current_index = 0}
        }
        else {
            if current_index >= job_count { current_index = 0; }
            else if current_index > 1 { current_index -= 1 }
            else { current_index = 0 }
        }
        if current_index <= 0 || current_index >= job_count {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().single_class = 0; }
            else { GameVariableManager::set_number("SINGLE_CLASS", 0); }
            return;
        }
        if let Some(job) = JobData::try_index_get(current_index).filter(|x| x.flag.value & 23 == 3 && (x.is_high() || x.max_level >= 40)) {
            let jid = job.jid.to_string();
            if (jid == "JID_マージカノン" || jid == "JID_エンチャント") && !dlc_check() { continue; }
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().single_class = job.parent.hash; }
            else { GameVariableManager::set_number("SINGLE_CLASS", job.parent.hash); };
            return;
        }
    }
}

pub fn single_class_exists() {
    if GameVariableManager::get_number(DVCVariables::SINGLE_CLASS) != 0 {
        let hash = GameVariableManager::get_number(DVCVariables::SINGLE_CLASS);
        if !JobData::try_get_hash(hash).is_some_and( |x|x.flag.value & 23 == 3 && (x.is_high() || x.max_level >= 40)) {
            GameVariableManager::set_number(DVCVariables::SINGLE_CLASS, 0);
        }
    }
}

pub struct ChangeSingleClassConfirm;
impl TwoChoiceDialogMethods for ChangeSingleClassConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        let new = GameVariableManager::get_number("SINGLE_CLASS");
        GameVariableManager::set_number(DVCVariables::SINGLE_CLASS, new);

        if JobData::try_get_hash(new).is_some() {
            for x in 1..250 {
                if let Some(unit) = UnitPool::get_by_index(x)
                    .filter(|u| u.force.is_some_and(|f| (1 << f.force_type) & 57 != 0))
                {
                    unit_change_to_random_class(unit);
                }
            }
        }
        utils::dialog_restore_text::<SingleJob>(this);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        BasicMenuResult::new().with_close_this(true)
    }
}
