use unity::prelude::*;
use super::*;
use engage::menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult};
use engage::dialog::yesno::TwoChoiceDialogMethods;
use engage::util::get_instance;
use crate::randomizer::skill::menu::SkillChangeConfirm;

pub struct RandomEmblemMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemMod {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = custom::cemblem_recruitment_menu_a_call as _).unwrap();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let max = if EMBLEM_LIST.get().unwrap().len() > 23 { 4 } else { 3 };
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_mode, 0, max, 1);
        if CONFIG.lock().unwrap().emblem_mode != result {
            CONFIG.lock().unwrap().emblem_mode = result;
            this.is_command_icon = result == 3;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_mode  {
            1 => { "Random" },
            2 => { "Reverse" },
            3 => { "Custom Order" },
            4 => { "Custom Emblems"},
            _ => { "Standard"},
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().emblem_mode {
            1 => { "Emblem recruitment will be randomized." },
            2 => { "Emblem recruitment will be in reversed order" },
            3 => { "Emblem recruitment will be determined by list."},
            4 => { "Random recruitment with custom emblems."},
            _ => { "Default recruitment order for emblems." },
        }.into();
    }
}
pub struct RandomEmblemLinkMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEmblemLinkMod {
    fn init_content(_this: &mut ConfigBasicMenuItem) {}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_engage_link(true);
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_engage_link(result, true);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else {BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let main = DVCVariables::is_main_menu();
        this.is_command_icon = DVCVariables::flag_changed(DVCFlags::EngagePlus);
        this.command_text = if DVCVariables::get_engage_link(!main) { "Random Links" } else { "No Links" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Emblems are linked to characters for Engage+".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_main_menu() { return BasicMenuResult::new(); }
        if DVCVariables::flag_changed(DVCFlags::EngagePlus) { return BasicMenuResult::new();}
        YesNoDialog::bind::<EngageLinkConfirm>(this.menu, "Change Engage Link Settings?", "Do it!", "Nah..");
        BasicMenuResult::new()
    }
    extern "C" fn build_attributes(this: &mut ConfigBasicMenuItem, method_info: OptionalMethod) -> BasicMenuItemAttribute {
        crate::menus::buildattr::not_in_map_sortie_build_attr(this, method_info)
    }
}

pub struct RandomGodMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGodMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() { DVCVariables::set_temp(DVCVariables::EMBLEM_SKILL_KEY); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_god_mode();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_random_god_mode(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.is_command_icon = DVCVariables::is_temp_change(DVCVariables::EMBLEM_SKILL_KEY);
        this.command_text =
            match DVCVariables::get_random_god_mode() {
            1 => { "Skill Inheritance" },
            2 => { "Engage Atks" },
            3 => { "Inherits/Engage Atks"},
            _ => { "None" },
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            match DVCVariables::get_random_god_mode() {
                1 => { "Inheritable skills will be randomized."},
                2 => { "Engage/Linked Engage Atks will be randomized." },
                3 => { "Inheritable/Engage Atks will be randomized." },
                _ => { "No Randomization to emblem data."},
            }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_temp_change(DVCVariables::EMBLEM_SKILL_KEY) {
            YesNoDialog::bind::<EmblemSkillChangeConfirm>(this.menu, "Change Emblem Skill Setting?\nRequires Save and Reload.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
}
crate::random_confirm!(EMBLEM_SKILL_KEY, EmblemSkill);
pub struct RandomSynchoMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSynchoMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() { DVCVariables::set_temp(DVCVariables::EMBLEM_SYNC_KEY); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_god_sync_mode();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_random_god_sync_mode(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.is_command_icon = DVCVariables::is_temp_change(DVCVariables::EMBLEM_SYNC_KEY);
        this.command_text =
            match DVCVariables::get_random_god_sync_mode() {
            1 => { "Stats" },
            2 => { "Sync/Engage Skills" },
            3 => { "All Sync"},
            _ => { "None"}
        }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            match DVCVariables::get_random_god_sync_mode() {
                1 => { "Emblem stat bonuses are randomized." },
                2 => { "Sync/engage skills are randomized." },
                3 => { "Stats/sync/engage skills are randomized." },
                _ => { "No changes to sync/engage emblem data."},
            }.into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_temp_change(DVCVariables::EMBLEM_SYNC_KEY) {
            YesNoDialog::bind::<EmblemSyncChangeConfirm>(this.menu, "Change Emblem Sync Skill Setting?\nRequires Save and Reload.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
}
crate::random_confirm!(EMBLEM_SYNC_KEY, EmblemSync);
pub struct RandomEngageWepMod;
impl ConfigBasicMenuItemSwitchMethods for RandomEngageWepMod {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_engage_weapon(true);
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_random_engage_weapon(result, true);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.is_command_icon = DVCVariables::flag_changed(DVCFlags::EngageWeapons);
        this.command_text =
            DVCVariables::command_text_flag(
                DVCFlags::EngageWeapons,
                if DVCVariables::get_random_engage_weapon(true){  "Randomize" } else { "Default" }
            ).into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text =
            DVCVariables::help_text_flag(DVCFlags::EngageWeapons,
                if DVCVariables::get_random_engage_weapon(true) { "Engage weapons are randomized." }
                else { "No changes to engage weapons." }
            ).into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::flag_changed(DVCFlags::EngageWeapons) {
            YesNoDialog::bind::<EngageWeaponChangeConfirm>(this.menu, "Change Engage Weapons Setting?\nRequires Save and Reload.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else {BasicMenuResult::new() }
    }
}
pub struct EngageLinkConfirm;
impl TwoChoiceDialogMethods for EngageLinkConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        randomize_engage_links(true);
        DVCVariables::update_flag(DVCFlags::EngagePlus);
        crate::menus::utils::dialog_restore_text::<RandomEmblemLinkMod>(this, false);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}
pub struct EngageWeaponChangeConfirm;
impl TwoChoiceDialogMethods for EngageWeaponChangeConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        DVCVariables::update_flag(DVCFlags::EngageWeapons);
        crate::menus::utils::dialog_remove_command_change(this);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
}

pub extern "C" fn vibe_engage_links() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomEmblemLinkMod>("Unit-Emblem Links") }