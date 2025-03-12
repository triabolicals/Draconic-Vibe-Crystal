use super::*;
use std::sync::Mutex;
use engage::{
    menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult}, 
    pad::Pad, 
    titlebar::TitleBar, 
    util::get_instance
};
pub static CUSTOM_EMBLEM_TABLE: Mutex<[i32; 19]> = Mutex::new([0; 19]);

pub fn get_update_position_inc(index: i32) -> i32 {
    let table = CUSTOM_EMBLEM_TABLE.lock().unwrap(); 
    let current = table[index as usize];
    let mut available: Vec<i32> = Vec::new();
    available.reserve(24);
    let limit = if dlc_check() { 19} else { 12 };
    available.push(0);
    for x in 0..limit {
        if table.iter().any(|&y| x+1 == y) { continue; }
        available.push( x + 1 );
    }
    if let Some(v) = available.iter().find(|&&v| v > current) { *v } else { 0 }
}

pub fn get_update_position_dec(index: i32) -> i32 {
    let table = CUSTOM_EMBLEM_TABLE.lock().unwrap(); 
    let current = table[index as usize];
    let mut available: Vec<i32> = Vec::new();
    available.reserve(24);
    let limit = if dlc_check() { 19} else { 12 };
    available.push(0);
    for x in 0..limit {
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


pub struct CustomEmblemRecruitmentMenuItem;
impl ConfigBasicMenuItemSwitchMethods for CustomEmblemRecruitmentMenuItem {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.get_class_mut().get_virtual_method_mut("PlusCall").map(|method| method.method_ptr =  cemblem_recruitment_menu_plus_call as _).unwrap();
        this.get_class_mut().get_virtual_method_mut("MinusCall").map(|method| method.method_ptr = cemblem_recruitment_menu_minus_call as _).unwrap();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let index = this.index;
        let pad = get_instance::<Pad>();
        if pad.npad_state.buttons.left() && !pad.old_buttons.left() {
            CUSTOM_EMBLEM_TABLE.lock().unwrap()[index as usize] = get_update_position_dec(index);
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else if pad.npad_state.buttons.right() && !pad.old_buttons.right() { 
            CUSTOM_EMBLEM_TABLE.lock().unwrap()[index as usize] = get_update_position_inc(index);
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        }
        else {
            Self::set_help_text(this, None);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::new()
        }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = CUSTOM_EMBLEM_TABLE.lock().unwrap()[this.index as usize];
        let name = GodData::get( EMBLEM_GIDS[ this.index as usize ]).unwrap().mid;
        if value == 0 {
            this.help_text = format!("{} will be randomized.", Mess::get( name )).into();
        }
        else if value == this.index+1 {
            this.help_text = format!("{} will not be randomized.", Mess::get(name)).into();
        }
        else {
            let name2 = GodData::get( EMBLEM_GIDS[ value as usize - 1 ]).unwrap().mid;
            this.help_text = format!("{} will replace {}.",  Mess::get(name2), Mess::get(name)).into();
        }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = CUSTOM_EMBLEM_TABLE.lock().unwrap()[this.index as usize];

        this.command_text = 
            if value == 0 { "Random".into()  }
            else {  
                let name = GodData::get( EMBLEM_GIDS[ value as usize - 1 ]).unwrap().mid;
                Mess::get(name) 
            };
    }
}

pub fn cemblem_recruitment_menu_a_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if CONFIG.lock().unwrap().emblem_mode != 3 { return BasicMenuResult::new(); }
    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        }
    );

    ConfigMenu::create_bind(this.menu);
    let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
    config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = crate::menus::submenu::open_anime_all_ondispose_to_dvc_main2 as _).unwrap();
    config_menu.full_menu_item_list.clear();
    let limit = if dlc_check() && CONFIG.lock().unwrap().dlc & 1 == 0 { 19 } else { 12 };
    for x in 0..limit {
        let name = GodData::get( EMBLEM_GIDS[ x as usize ]).unwrap().mid;
        let switch = ConfigBasicMenuItem::new_switch::<CustomEmblemRecruitmentMenuItem>( Mess::get(name).to_string().as_str() );
        config_menu.add_item(switch);
    }
    if CONFIG.lock().unwrap().dlc & 1 != 0 {
        for x in 0..12 { 
            if CUSTOM_EMBLEM_TABLE.lock().unwrap()[x] > 12 {
                CUSTOM_EMBLEM_TABLE.lock().unwrap()[x] = 0;
            }
        }
        for x in 12..19 { CUSTOM_EMBLEM_TABLE.lock().unwrap()[x] = x as i32 + 1;  }
    }
    for x in 0..limit {
        let item = &mut config_menu.full_menu_item_list[x as usize];
        item.index = x;
        CustomEmblemRecruitmentMenuItem::set_help_text(*item, None);
        CustomEmblemRecruitmentMenuItem::set_command_text(*item, None);
        item.update_text();
    }
    TitleBar::open_header("Draconic Vibe Crystal", "Custom Emblem Recruitment Order Setting", "");
    BasicMenuResult::se_cursor()

}
pub fn  cemblem_recruitment_menu_plus_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let index = this.index;
    if !CUSTOM_EMBLEM_TABLE.is_poisoned() {
        CUSTOM_EMBLEM_TABLE.lock().unwrap()[index as usize] = 0;
        CustomEmblemRecruitmentMenuItem::set_help_text(this, None);
        CustomEmblemRecruitmentMenuItem::set_command_text(this, None);
        this.update_text();
        BasicMenuResult::se_cursor()
    }
    else {  BasicMenuResult::new()  }
}

pub fn  cemblem_recruitment_menu_minus_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let index = this.index;
    if !CUSTOM_EMBLEM_TABLE.is_poisoned() {
        for x in 0..19 {
            if CUSTOM_EMBLEM_TABLE.lock().unwrap()[x as usize] == index+1 {
                return BasicMenuResult::se_miss();
            }
        }
        CUSTOM_EMBLEM_TABLE.lock().unwrap()[index as usize] = index+1;
        CustomEmblemRecruitmentMenuItem::set_help_text(this, None);
        CustomEmblemRecruitmentMenuItem::set_command_text(this, None);
        this.update_text();
        BasicMenuResult::se_cursor()
    }
    else {
        BasicMenuResult::new()
    }
}