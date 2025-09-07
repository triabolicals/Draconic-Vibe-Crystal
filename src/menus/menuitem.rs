pub trait DVCConfigMenuItem {
    fn get_help_text(value: i32) -> &'static str;
    fn get_command_text(value: i32) -> &'static str;
    fn get_change_value(initial: i32) -> i32;
    fn get_value() -> i32;
    fn requires_reload() -> bool;
}
/*
macro_rules! create_dvc_menu {
    ($name:ident, $dvc_key:ident) => {
        paste! {
            pub struct [<$name ConfigMenuItem>];
            fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() { DVCVariables::set_temp(DVCVariables::SKILL_KEY); }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_random_skill();
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            DVCVariables::set_random_skill(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        if DVCVariables::can_update_var(DVCVariables::SKILL_KEY){
            YesNoDialog::bind::<SkillChangeConfirm>(this.menu, "Change Skill Setting?\nRequires Save and Reload.", "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_skill();
        let command_text =
            match value {
                1 => "Personal",
                2 => "Class",
                3 => "Personal + Class",
                _ => "Default"
            };
        this.command_text = DVCVariables::changed_setting_command(DVCVariables::SKILL_KEY, command_text).into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = DVCVariables::get_random_skill();
        let help =
            match value {
                1 => "Personal skills are randomized",
                2 => "Class learn skills are randomized.",
                3 => "Personal/Learn skills are randomized.",
                _ => "No changes to personal and class skills.",
            };
        this.help_text = DVCVariables::changed_setting_help(DVCVariables::SKILL_KEY, help).into();
    }

        }
    };
}

 */