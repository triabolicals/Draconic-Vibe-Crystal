use super::*;
use std::sync::Mutex;
use engage::{
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult, *}, 
    pad::Pad, 
    proc::ProcInst,
    titlebar::TitleBar, 
    util::get_instance
};
pub static CUSTOM_RECRUITMENT_TABLE: Mutex<[i32; 41]> = Mutex::new([0; 41]);

pub fn get_update_position_inc(index: i32) -> i32 {
    let table = CUSTOM_RECRUITMENT_TABLE.lock().unwrap(); 
    let current = table[index as usize];
    let mut available: Vec<i32> = Vec::new();
    available.reserve(48);
    let limit = if dlc_check() { 41 } else { 36 };
    available.push(0);
    for x in 0..limit {
        if index > 35 {
            if [0, 4, 14, 17, 23, 27 ].iter().any(|&x1| x1 == x) { continue; }
        }
        if table.iter().any(|&y| x+1 == y) { continue; }
        available.push( x + 1 );

    }
    if let Some(v) = available.iter().find(|&&v| v > current) { *v } else { 0 }
}

pub fn get_update_position_dec(index: i32) -> i32 {
    let table = CUSTOM_RECRUITMENT_TABLE.lock().unwrap(); 
    let current = table[index as usize];
    let mut available: Vec<i32> = Vec::new();
    available.reserve(48);
    let limit = if dlc_check() { 41 } else { 36 };
    available.push(0);
    for x in 0..limit {
        if index > 35 {
            if [0, 4, 14, 17, 23, 27 ].iter().any(|&x1| x1 == x) { continue; }
        }
        if table.iter().any(|&y| x+1 == y) { continue; }
        available.push( x + 1 );

    }
    if let Some(v) = available.iter().filter(|&&v| v < current).max() {
        return *v;
    }
    else {
        if let Some(v) = available.iter().filter(|&&v| v <= limit).max() { *v }
        else { 0 }
    }
}


pub struct CustomRecruitmentMenuItem;
impl ConfigBasicMenuItemSwitchMethods for CustomRecruitmentMenuItem {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut().get_virtual_method_mut("PlusCall").map(|method| method.method_ptr = crecruitment_menu_plus_call as _).unwrap();
        this.get_class_mut().get_virtual_method_mut("MinusCall").map(|method| method.method_ptr =crecruitment_menu_minus_call as _).unwrap();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let index = this.index;
        let pad = get_instance::<Pad>();
        if pad.npad_state.buttons.left() && !pad.old_buttons.left() {
            CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[index as usize] = get_update_position_dec(index);
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else if pad.npad_state.buttons.right() && !pad.old_buttons.right() { 
            CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[index as usize] = get_update_position_inc(index);
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else {
            BasicMenuResult::new()
        }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[this.index as usize];
        if value == 0 {
            this.help_text = format!("{} will be randomized.", Mess::get(MPIDS[this.index as usize])).into();
        }
        else if value == this.index+1 {
            this.help_text = format!("{} will not be randomized.", Mess::get(MPIDS[this.index as usize])).into();
        }
        else {
            this.help_text = format!("{} will replace {}.",  Mess::get(MPIDS[value as usize - 1]), Mess::get(MPIDS[this.index as usize])).into();
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[this.index as usize];
        this.command_text = 
            if value == 0 { "Random".into()  }
            else {  Mess::get(MPIDS[value as usize - 1]) };
    }
}

pub fn crecruitment_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if CONFIG.lock().unwrap().random_recruitment != 3 { return BasicMenuResult::new(); }
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );
    this.menu.proc.parent.as_ref().unwrap().get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu.proc.parent.as_ref().unwrap(), method.method_info);
        }
    );
    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = crate::menus::submenu::open_anime_all_ondispose_to_dvc_main2 as _).unwrap();
    config_menu.full_menu_item_list.clear();
    let limit = if dlc_check() && CONFIG.lock().unwrap().dlc & 2 == 0 { 41 } else { 36 };
    if CONFIG.lock().unwrap().dlc & 2 != 0 {
        for x in 0..36 { 
            if CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[x] > 36 {
                CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[x] = 0;
            }
        }
        for x in 36..41 { CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[x] = x as i32 + 1;  }
    }
    for x in 0..limit {
        let mpid = MPIDS[x as usize];
        let switch = ConfigBasicMenuItem::new_switch::<CustomRecruitmentMenuItem>( Mess::get(mpid).to_string().as_str() );
        config_menu.add_item(switch);
    }
    for x in 0..limit {
        let item = &mut config_menu.full_menu_item_list[x as usize];
        item.index = x;
        CustomRecruitmentMenuItem::set_help_text(*item, None);
        CustomRecruitmentMenuItem::set_command_text(*item, None);
        item.update_text();
    }
    TitleBar::open_header("Draconic Vibe Crystal", "Custom Unit Recruitment Order Setting", "");
    BasicMenuResult::se_cursor()
}
pub fn crecruitment_menu_plus_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let index = this.index;
    if !CUSTOM_RECRUITMENT_TABLE.is_poisoned() {
        CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[index as usize] = 0;
        CustomRecruitmentMenuItem::set_help_text(this, None);
        CustomRecruitmentMenuItem::set_command_text(this, None);
        this.update_text();
        BasicMenuResult::se_cursor()
    }
    else {  BasicMenuResult::new()  }
}

pub fn crecruitment_menu_minus_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let index = this.index;
    if !CUSTOM_RECRUITMENT_TABLE.is_poisoned() {
        for x in 0..41 {
            if CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[x as usize] == index+1 {
                return BasicMenuResult::se_miss();
            }
        }
        CUSTOM_RECRUITMENT_TABLE.lock().unwrap()[index as usize] = index+1;
        CustomRecruitmentMenuItem::set_help_text(this, None);
        CustomRecruitmentMenuItem::set_command_text(this, None);
        this.update_text();
        BasicMenuResult::se_cursor()
    }
    else { BasicMenuResult::new() }
}
