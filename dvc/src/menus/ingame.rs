use engage::{
    menu::{
        menu_item::config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods},
    },
    mess::Mess,
};
use crate::VERSION;
use super::*;

pub struct TriabolicalInGameMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalInGameMenu {
    extern "C" fn custom_call(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        if pad_instance.npad_state.buttons.plus() { BasicMenuResult::se_cursor() }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.command_text = Mess::get("MID_CONFIG_BGM_CHANGE_DECIDE");
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = "Open up In-Game Draconic Vibe Crystal settings.".into();
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        this.menu.close_anime_all();
        create_dvc_bind(this.menu);
        BasicMenuResult::se_cursor()
    }
}
/*
fn output_bind(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
    if DVCVariables::random_enabled() {
        let text = format!("Create Output File for this Save?\n Save as 'sd:/Draconic Vibe Crystal/{}.log'",  crate::utils::get_player_name());
        YesNoDialog::bind::<WriteOutputConfirm>(this.menu, text, "Do it!", "Nah..");
    }
}
 */
pub extern "C" fn vibe2() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("{} {}", draconic_vibe_name(), VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalInGameMenu>(title)
}

pub fn draconic_vibe_name() -> &'static str {
    match unsafe { get_lang(None) } {
        0 => "竜の振動水晶",
        2|5 => "Cristal de Vibration Draconique",
        3|6 => "Gema de Vibración Dracónica",
        7 => "Drachenvibrationskristall",
        8 => "Gemma di Vibrazione del Drago",
        9 => "龍之氣息水晶",
        10 => "龙之气息水晶",
        11 => "용의 분위기석",
        _ => "Draconic Vibe Crystal",
    }
}

#[skyline::from_offset(0x01bdbc80)]
fn get_lang(method_info: OptionalMethod) -> i32;