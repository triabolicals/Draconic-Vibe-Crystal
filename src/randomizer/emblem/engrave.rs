use std::sync::OnceLock;
use engage::sequence::configsequence::ConfigSequence;
use engage::titlebar::TitleBar;
use crate::menus;
use super::*;
pub static ENGRAVE_STATS: OnceLock<[i8; 255]> = OnceLock::new();

const SCORE: [&str; 2] = ["Low", "High"];
pub struct EngraveSettings;
impl ConfigBasicMenuItemSwitchMethods for EngraveSettings {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry("EngraveSetting", GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY));
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engrave_settings } else { GameVariableManager::get_number("EngraveSetting") };
        let result = ConfigBasicMenuItem::change_key_value_i(value & 255, 0, 5, 1);
        if (value & 255) != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engrave_settings = result; }
            else {
                let new_value = (value & !255) | result;
                GameVariableManager::set_number("EngraveSetting", new_value);
            }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().engrave_settings } else { GameVariableManager::get_number("EngraveSetting") }.into();

        let changed = DVCVariables::changed_setting_text("EngraveSetting", DVCVariables::ENGRAVE_KEY);
        this.command_text = format!("{}{}", changed, engrave_setting_text(value)).into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text =
            if DVCVariables::is_main_menu() {
                if CONFIG.lock().unwrap().engrave_settings == 5 { "Custom engrave score range. Press A to change." } else { "Sets the level of randomness for engraves" }.into()
            }
            else {
                let current_setting = GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY);
                if ( current_setting & 255) == (GameVariableManager::get_number("EngraveSetting") & 255) {
                    if (current_setting & 255) != 5 { format!("Current Level: {}", engrave_setting_text(current_setting)).into() }
                    else {
                        let high = ((current_setting >> 16) & 255) - 100;
                        let low = ((current_setting >> 8) & 255) - 100;
                        format!("Custom Score Range: {} to {}. + to set score range.", low, high).into()
                    }
                }
                else if (GameVariableManager::get_number("EngraveSetting") & 255) != 5 { format!("Current Level: {} (A to Change)", engrave_setting_text(current_setting)).into() }
                else {
                    let high = ((current_setting >> 16) & 255) - 100;
                    let low = ((current_setting >> 8) & 255) - 100;
                    format!("Custom Score Range: {} to {}. Set score +. A to Change.", low, high).into()
                }
            }
    }
    extern "C" fn a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        if DVCVariables::is_main_menu() {
            if CONFIG.lock().unwrap().engrave_settings == 5 { bind_to_engrave_score(this) } else { BasicMenuResult::new() }
        }
        else if GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY) == GameVariableManager::get_number("EngraveSetting") { BasicMenuResult::new() }
        else {
            let text =
                format!("Change Engrave Randomization Level:\n\tFrom '{}' to '{}'?",
                        engrave_setting_text( GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY)),
                        engrave_setting_text( GameVariableManager::get_number("EngraveSetting")),
                );
            YesNoDialog::bind::<EngraveConfirm>(this.menu, text, "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
    }
}

pub struct EngraveConfirm;
impl TwoChoiceDialogMethods for EngraveConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        random_engrave_by_setting( GameVariableManager::get_number("EngraveSetting"), false);
        GameVariableManager::set_number(DVCVariables::ENGRAVE_KEY, GameVariableManager::get_number("EngraveSetting"));
        menus::utils::dialog_restore_text::<EngraveSettings>(this, false);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn random_engrave_by_setting(setting: i32, loaded: bool) {
    if loaded && setting == 0 { return; }
    match setting & 255 {
        1 => { randomized_engraves2(0, 50, 0); },
        2 => { randomized_engraves2(-75, 75, 25); }, 
        3 => { randomized_engraves2(-150, 150, 50); },
        4 => { randomized_engraves2(-400, 400, 100); },
        5 => {
            let engrave_limits = [(setting  >> 8) & 255, (setting  >> 16) & 255];
            println!("Engrave Limits: {} to {}", engrave_limits[0], engrave_limits[1]);
            randomized_engraves2(engrave_limits[0]*3, engrave_limits[1]*3, 10);
        },
        _ => { randomized_engraves2(0, 0, 0); },
    }
}
pub fn engrave_setting_plus_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if !DVCVariables::is_main_menu() && GameVariableManager::get_number("EngraveSetting") & 255 == 5 {
        bind_to_engrave_score(this)
    }
    else { BasicMenuResult::new() }
}
fn bind_to_engrave_score(this: &mut ConfigBasicMenuItem) -> BasicMenuResult{
    ConfigMenu::create_bind(this.menu);
    if !DVCVariables::is_main_menu() {
        GameVariableManager::set_number("EngraveSetting", GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY));
    }

    this.menu.close_anime_all();
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose")
        .map(|method| method.method_ptr = menus::submenu::open_anime_all_ondispose_to_dvc_main as _).unwrap();
    config_menu.get_class_mut().get_virtual_method_mut("BCall")
        .map(|method| method.method_ptr = engrave_score_menu_b_call as _).unwrap();
    config_menu.full_menu_item_list.clear();
    for x in 0..2{
        let switch = ConfigBasicMenuItem::new_switch::<EngraveScoreGauge>(format!("{} Engrave Score Limit", SCORE[x]));
        config_menu.add_item(switch);
    }
    for x in 0..2 {
        let item = &mut config_menu.full_menu_item_list[x as usize];
        item.index = x;
        EngraveScoreGauge::set_command_text(item, None);
        EngraveScoreGauge::set_help_text(item, None);
        item.update_text();
    }
    TitleBar::open_header("Draconic Vibe Crystal", "Engrave Custom Score Limit", "");
    BasicMenuResult::se_cursor()
}
fn engrave_setting_text(choice: i32) -> String {
    let choice = choice & 255;
    match choice {
        1 => { "Low" },
        2 => { "Medium"},
        3 => { "High"},
        4 => { "Chaotic"},
        5 => { "Custom"},
        _ => { "None"},
    }.to_string()
}


pub extern "C" fn vibe_engrave() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<EngraveSettings>("Engrave Randomization Level");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = menus::buildattr::build_attribute_normal as _);
    switch.get_class_mut().get_virtual_method_mut("PlusCall").map(|method| method.method_ptr = engrave_setting_plus_call as _);
    switch
}
pub fn gauge_get_score(is_high: bool) -> i32 {
    let v =
    match (is_high, DVCVariables::is_main_menu()) {
        (true, true) => { CONFIG.lock().unwrap().engrave_upper_score }
        (false, true) => { CONFIG.lock().unwrap().engrave_lower_score }
        (true, false) => {
            ((GameVariableManager::get_number("EngraveSetting") >> 16) & 255) - 100
        }
        (false, false) => {
            ((GameVariableManager::get_number("EngraveSetting") >> 8) & 255) - 100
        }
    };
    println!("Engrave Score: {}", v);
    v
}
pub struct EngraveScoreGauge;
impl ConfigBasicMenuItemSwitchMethods for EngraveScoreGauge {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_high = this.index as usize == 1;
        let (h_score, l_score) =
        if DVCVariables::is_main_menu() {
            let high = CONFIG.lock().unwrap().engrave_upper_score;
            let low = CONFIG.lock().unwrap().engrave_lower_score;
            (high+ 100,  low + 100)
        }
        else {
            let v = GameVariableManager::get_number("EngraveSetting");
            ((v >> 16) & 255, (v >> 8) & 255)
        };
        let (score, score_limit_min, score_limit_max) =
            if is_high { (h_score, max(l_score+10, 0), 200) }
            else { (l_score, 0, h_score-10) };

        let result = ConfigBasicMenuItem::change_key_value_i(score, score_limit_min, score_limit_max, 5);
        if score != result {
            match (is_high, DVCVariables::is_main_menu()) {
                (true, true) => { CONFIG.lock().unwrap().engrave_upper_score = result - 100; }
                (false, true) => { CONFIG.lock().unwrap().engrave_lower_score = result - 100; }
                (true, false) => {
                    let v = (GameVariableManager::get_number("EngraveSetting") & !(255 << 16)) | (result << 16);
                    GameVariableManager::set_number("EngraveSetting", v);
                }
                (false, false) => {
                    let v = (GameVariableManager::get_number("EngraveSetting") & !(255 << 8)) | (result << 8);
                    GameVariableManager::set_number("EngraveSetting", v);
                }
            };
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.command_text = gauge_get_score(this.index == 1).to_string().into();
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = format!("Engrave {} Score Limit", SCORE[this.index as usize]).into();
    }
}

pub struct EngraveScoreChange;
impl TwoChoiceDialogMethods for EngraveScoreChange {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        let new_value = (GameVariableManager::get_number("EngraveSetting") & !255) | (GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY) & 255);
        GameVariableManager::set_number(DVCVariables::ENGRAVE_KEY, new_value);
        random_engrave_by_setting( GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY), false);
        menus::utils::dialog_restore_text::<EngraveSettings>(this, true);
        BasicMenuResult::new().with_close_this(true).with_close_parent(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        let v =  GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY) & !255;
        let new_value = (GameVariableManager::get_number("EngraveSetting") & 255) | v;
        GameVariableManager::set_number("EngraveSetting", new_value);
        BasicMenuResult::new().with_close_this(true).with_close_parent(true)
    }
    extern "C" fn bcall_first(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        let v =  GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY) & !255;
        let new_value = (GameVariableManager::get_number("EngraveSetting") & 255) | v;
        GameVariableManager::set_number("EngraveSetting", new_value);
        BasicMenuResult::new().with_close_this(true).with_close_parent(true)
    }
    extern "C" fn bcall_second(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        let v =  GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY) & !255;
        let new_value = (GameVariableManager::get_number("EngraveSetting") & 255) | v;
        GameVariableManager::set_number("EngraveSetting", new_value);
        BasicMenuResult::new().with_close_this(true).with_close_parent(true)
    }
}

fn engrave_score_menu_b_call(this: &mut ConfigSequence, _optional_method: OptionalMethod) -> BasicMenuResult {
    let new = GameVariableManager::get_number("EngraveSetting");
    let old = GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY);
    if !DVCVariables::is_main_menu() && ((new & !255) != (old & !255)) {
        if (new & 255) == 5 {
            YesNoDialog::bind::<EngraveScoreChange>(this, "Update engraves to the new score limit?", "Do it!", "Discard Changes");
        }
        this.get_class_mut().get_virtual_method_mut("BCall").map(|m| m.method_ptr = menus::utils::close_this_with_cancel as _);

    }
    BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
}


pub fn get_engrave_stats() {
    ENGRAVE_STATS.get_or_init(||{
        let mut stats: [i8; 255] = [0; 255];
        let mut max_engrave_stat: [i8; 6] = [0; 6];
        let mut min_engrave_stat: [i8; 6] = [100; 6];
        EMBLEM_LIST.get().unwrap().iter().flat_map(|&x| GodData::try_get_hash(x)).enumerate().for_each(|(x, god)|{
            let index = x*6;
            stats[index] = god.get_engrave_avoid();
            stats[index + 1] = god.get_engrave_critical();
            stats[index + 2] = god.get_engrave_hit();
            stats[index + 3] = god.get_engrave_power();
            stats[index + 4] = god.get_engrave_secure();
            stats[index + 5] = god.get_engrave_weight();
            if max_engrave_stat[0] < god.get_engrave_avoid() { max_engrave_stat[0] = god.get_engrave_avoid(); }
            if max_engrave_stat[1] < god.get_engrave_critical() { max_engrave_stat[1] = god.get_engrave_critical();}
            if max_engrave_stat[2] < god.get_engrave_hit() { max_engrave_stat[2] = god.get_engrave_hit(); }
            if max_engrave_stat[3] < god.get_engrave_power() { max_engrave_stat[3] = god.get_engrave_power(); }
            if max_engrave_stat[4] < god.get_engrave_secure() { max_engrave_stat[4] = god.get_engrave_secure(); } 
            if max_engrave_stat[5] > god.get_engrave_weight() { max_engrave_stat[5] = god.get_engrave_weight(); }  
            if min_engrave_stat[0] > god.get_engrave_avoid() { min_engrave_stat[0] = god.get_engrave_avoid(); }
            if min_engrave_stat[1] > god.get_engrave_critical() { min_engrave_stat[1] = god.get_engrave_critical();}
            if min_engrave_stat[2] > god.get_engrave_hit() { min_engrave_stat[2] = god.get_engrave_hit(); }
            if min_engrave_stat[3] > god.get_engrave_power() { min_engrave_stat[3] = god.get_engrave_power(); }
            if min_engrave_stat[4] > god.get_engrave_secure() { min_engrave_stat[4] = god.get_engrave_secure(); }  
            if min_engrave_stat[5] > god.get_engrave_weight() { min_engrave_stat[5] = god.get_engrave_weight(); }  
        });
        for x in 0..5 { 
            if x == 3 { //Might
                stats[240+x] += 5;
                stats[246+x] -= 5;
            }
            else {
                stats[240+x] = ( max_engrave_stat[x] / 5) + 2;
                stats[246+x] = min_engrave_stat[x] / 5 ;
            }
        }
    //Weight Limit from -5 to 10
        stats[245] = 10;
        stats[241] = -5;
        println!("Engrave Stats Initalized");
        stats
    });
}

pub fn randomized_engraves2(lower: i32, upper: i32, bandwidth: i32) {
    if lower == upper || !DVCVariables::random_enabled() {
        if let Some(stats) = ENGRAVE_STATS.get() {
            EMBLEM_LIST.get().unwrap().iter().flat_map(|&x| GodData::try_get_hash(x)).enumerate().for_each(|(x, god)|{
                for i in 0..6 {
                    let index = x*6 + i;
                    let value = stats[index];
                    god.set_engrave(i as i32, value);
                }
            });
            println!("Engraves stats are reset");
        }
        return;
    }
    let mut max_engrave_stat: [i8; 6] = [0; 6];
    let mut min_engrave_stat: [i8; 6] = [0; 6];
    for x in 0..6 {
        max_engrave_stat[x] = ENGRAVE_STATS.get().map_or(0, |v| v[240+x]);
        min_engrave_stat[x] = ENGRAVE_STATS.get().map_or(0, |v| v[246+x]);
    }
    let rng = crate::utils::get_rng();
    let band_lower = ( upper - lower ) / 2 - bandwidth;
    let band_higher  = ( upper - lower ) / 2 + bandwidth;
    EMBLEM_LIST.get().unwrap().iter().flat_map(|&x| GodData::try_get_hash(x)).for_each(|god|{
        let mut values: [i8; 6] = [0;6];
        let mut count = 0;
        loop {
            for i in 0..6 {
                values[i as usize] =  if i == 3 || i == 5 { 1 }
                else { 5 }*rng.get_min_max( min_engrave_stat[i as usize] as i32, max_engrave_stat[i as usize ] as i32) as i8; 
            }
            let total = 
            if values[0] < 0 { 2* (values[1] as i32 ) } else { values[0] as i32} +
            if values[1] < 0 { 2* (values[1] as i32 ) } else { values[1] as i32} +
            if values[2] < 0 { 2 * (values[2] as i32 ) } else { values[2] as i32} + 
            if values[3] < 0 { 15*values[3] as i32  } else { 10*values[3] as i32} +
            if values[4] < 0 { 2 * (values[4] as i32 ) } else { values[4] as i32 } -
            if values[5] < 0 { 10* values[5] as i32 } else { 20*values[5] as i32 };
            if band_lower <= total && band_higher >= total { continue; }
            if (total >= lower && total <= upper) || count > 10 { break; }
            count += 1;
        }
        for i in 0..6 { god.set_engrave(i,values[i as usize]); }
    });
}