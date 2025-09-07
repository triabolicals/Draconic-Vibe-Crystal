use super::*;

pub struct BenchAutoLevelOption;
impl ConfigBasicMenuItemSwitchMethods for BenchAutoLevelOption {
    fn init_content(_this: &mut ConfigBasicMenuItem)    {}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle =  DVCVariables::get_flag(DVCFlags::PostChapterAutolevel, false);
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);
        if toggle != result {
            DVCVariables::set_flag(DVCFlags::PostChapterAutolevel, result, false);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if !DVCVariables::get_flag(DVCFlags::PostChapterAutolevel, false) { "Disabled" } else { "Enabled" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Units will auto-level to around the deployed level average.".into();
    }
}
pub extern "C" fn autobench() -> &'static mut ConfigBasicMenuItem {
    ConfigBasicMenuItem::new_switch::<BenchAutoLevelOption>("Post Chapter Autoleveling")
}

pub struct AutolevelMod;
impl ConfigBasicMenuItemSwitchMethods for AutolevelMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = DVCVariables::get_autolevel(false);
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            DVCVariables::set_autolevel(result, false);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }

    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if DVCVariables::get_autolevel(false) { "Enable" } else { "Disable" }.into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Units/enemies will be scaled to army's power.".into();
    }
    extern "C" fn build_attributes(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        if DVCVariables::is_random_map() { BasicMenuItemAttribute::Hide }
        else { crate::menus::buildattr::not_in_map_sortie_build_attr(this, None) }
    }
}
pub extern "C" fn vibe_autolevel() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<AutolevelMod>("Level Scale Units") }
