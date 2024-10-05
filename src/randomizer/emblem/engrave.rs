use super::*;

pub struct EngraveSettings;
impl ConfigBasicMenuItemSwitchMethods for EngraveSettings {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        GameVariableManager::make_entry("EngraveSetting", GameVariableManager::get_number("G_EngraveSetting"));
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value =  if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engrave_settings } 
                     else { GameVariableManager::get_number("EngraveSetting") };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 5, 1);
            if value != result {
                if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().engrave_settings = result;  }
                else { GameVariableManager::set_number("EngraveSetting", result); }
                Self::set_command_text(this, None);
                Self::set_help_text(this, None);
                this.update_text();
                return BasicMenuResult::se_cursor();
            } else {return BasicMenuResult::new(); }
        } 
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if GameUserData::get_sequence() == 0 { this.help_text = "Sets the level of randomness for engraves".into(); }
        else {
            let current_setting = GameVariableManager::get_number("G_EngraveSetting");
            this.help_text = if GameVariableManager::get_number("G_EngraveSetting") == GameVariableManager::get_number("EngraveSetting") {
                format!("Current Level: {}",  engrave_setting_text(current_setting)) }
            else { format!("Current Level: {} (Press A to change.)",  engrave_setting_text(current_setting)) }.into();
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().engrave_settings }
        else { GameVariableManager::get_number("EngraveSetting") }.into();

        this.command_text = engrave_setting_text( value ).into();
    }
}

pub struct EngraveConfirm;
impl TwoChoiceDialogMethods for EngraveConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        random_engrave_by_setting( GameVariableManager::get_number("EngraveSetting") );
        GameVariableManager::set_number("G_EngraveSetting", GameVariableManager::get_number("EngraveSetting"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent) };
        let index = menu.select_index;
        EngraveSettings::set_help_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn random_engrave_by_setting(setting: i32) {
    match setting {
        1 => { randomized_engraves2(0, 50, 0); },
        2 => { randomized_engraves2(-75, 75, 25); }, 
        3 => { randomized_engraves2(-150, 150, 50); },
        4 => { randomized_engraves2(-400, 400, 100); },
        5 => {
            let engrave_limits = CONFIG.lock().unwrap().get_engrave_limits();
            randomized_engraves2(engrave_limits.0 as i32, engrave_limits.1 as i32, 10);
        },
        _ => { randomized_engraves2(0, 0, 0); },
    }
}

pub fn engrave_setting_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 0 {return BasicMenuResult::new(); }
    if GameVariableManager::get_number("G_EngraveSetting") == GameVariableManager::get_number("EngraveSetting") { return BasicMenuResult::new();}
    let text = format!("Change Engrave Randomization Level:\n\tFrom '{}' to '{}'?",
        engrave_setting_text( GameVariableManager::get_number("G_EngraveSetting")), 
        engrave_setting_text( GameVariableManager::get_number("EngraveSetting")), 
    );
    YesNoDialog::bind::<EngraveConfirm>(this.menu, text, "Do it!", "Nah..");
    return BasicMenuResult::new();
}
fn engrave_setting_text(choice: i32) -> String {
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
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_normal as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = engrave_setting_acall as _);
    switch
}


pub fn get_engrave_stats() {
    for x in 0..50 { if ENGRAVE_STATS.lock().unwrap()[x] != 0 { return; } }
    let mut max_engrave_stat: [i8; 6] = [0; 6];
    let mut min_engrave_stat: [i8; 6] = [100; 6];
    for x in 0..20 { 
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let index = x*6 as usize;
        ENGRAVE_STATS.lock().unwrap()[index] = god.get_engrave_avoid();
        ENGRAVE_STATS.lock().unwrap()[index + 1] = god.get_engrave_critical();
        ENGRAVE_STATS.lock().unwrap()[index + 2] = god.get_engrave_hit();
        ENGRAVE_STATS.lock().unwrap()[index + 3] = god.get_engrave_power();
        ENGRAVE_STATS.lock().unwrap()[index + 4] = god.get_engrave_secure();
        ENGRAVE_STATS.lock().unwrap()[index + 5] = god.get_engrave_weight();
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
    }
    for x in 0..5 { 
        if x == 3 { //Might
            ENGRAVE_STATS.lock().unwrap()[120+x] += 5;
            ENGRAVE_STATS.lock().unwrap()[126+x] -= 5;
        }
        else {
            ENGRAVE_STATS.lock().unwrap()[120+x] = ( max_engrave_stat[x] / 5) + 2;
            ENGRAVE_STATS.lock().unwrap()[126+x] = min_engrave_stat[x] / 5 ;
        }
    }
    //Weight Limit from -5 to 10
    ENGRAVE_STATS.lock().unwrap()[125] = 10;
    ENGRAVE_STATS.lock().unwrap()[131] = -5;
}

pub fn randomized_engraves2(lower: i32, upper: i32, bandwidth: i32) {
    if lower == upper || !crate::utils::can_rand() {
        for x in 0..20 { 
            let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
            for i in 0..6 {
                let index = ( x*6 + i ) as usize;
                let value = ENGRAVE_STATS.lock().unwrap()[index];
                god.set_engrave(i, value);
            }
        }
        println!("Engraves stats are reset");
        return;
    }
    let mut max_engrave_stat: [i8; 6] = [0; 6];
    let mut min_engrave_stat: [i8; 6] = [0; 6];
    for x in 0..6 {
        max_engrave_stat[x as usize] = ENGRAVE_STATS.lock().unwrap()[120+x];
        min_engrave_stat[x as usize] = ENGRAVE_STATS.lock().unwrap()[126+x];
    }
    println!("Engraving Score Limits: Upper: {}, Lower: {}", upper, lower);
    let rng = crate::utils::get_rng();
    let band_lower = ( upper - lower ) / 2 - bandwidth;
    let band_higher  = ( upper - lower ) / 2 + bandwidth;
    for x in 0..20 { 
        let mut values: [i8; 6] = [0;6];
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
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
            if total >= lower && total <= upper  { break; }
        }
        for i in 0..6 { god.set_engrave(i,values[i as usize]); }
    }
}