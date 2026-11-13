use super::*;
use crate::dvc::DVC;
use crate::DVCConfig;
use crate::menus::DVCSettingMenu;

pub fn main_sequence_event_jump(main: ProcInst, label: i32) {
    if label == 6 {
        DVC::get().init = false;
        DVCSettingMenu::create_bind(main);
    }
    else if label == 28 {
        DVCConfig::get().save();
        DVC::get().on_new_game_start();
    }
}

