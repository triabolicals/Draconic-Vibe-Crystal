use std::sync::{OnceLock, RwLock};
use engage::{
    app::{goddata::*, IPersonDataMethods, IStructBase, IStructData_1Methods, Mess, PersonData},
    GameVariableManager
};
use unity::{Array, Cast, Il2CppString, IntPtr, OptionalMethod};
use crate::{
    dvc::asset::AppearanceManager, dvc::talk::tag::TextReplacementKind,
    DVCFlags::GodNames, gamedata::EmblemPool, DVCVariables, gamedata::PersonPool
};

#[allow(unused)] mod swapdata;
pub mod chapter;
pub mod puppet;
pub mod hub;

pub use chapter::*;
pub use swapdata::{MessageList, *};

pub static MESS_REPLACEMENT_DATA: OnceLock<RwLock<MessageList>> = OnceLock::new();

#[skyline::hook(offset=0x1f29c20)]
pub fn unit_get_name(this: PersonData, optional_method: OptionalMethod) -> Il2CppString {
    if this.get_name().is_null() || DVCVariables::is_main_menu() { call_original!(this, optional_method) }
    else if let Some(name) = AppearanceManager::get_appearance_name(this, None) { name }
    else { call_original!(this, optional_method) }
}

#[skyline::hook(offset=0x233f090)]
pub fn god_unit_get_name(data: GodData, optional_method: OptionalMethod) -> Il2CppString {
    if data.is_null() { call_original!(data, optional_method) }
    else {
        let mut hash = data.hash();
        if let Some(enemy) = EmblemPool::get_enemy_replacement(data){ hash = enemy.hash(); }
        if GodNames.get_value() {
            if let Some(app_idx) = EmblemPool::get_appearance_index(hash).filter(|&v| v < 24){
                return AppearanceManager::get_emblem_appearance_name(app_idx);
            }
        }
        if hash != data.hash() {
            let god = GodData::try_get_from_hash(hash);
            call_original!(god, optional_method)
        }
        else { call_original!(data, optional_method) }
    }
}

#[unity::hook("App", "Mess", "AddTagString")]
pub fn add_tag_string(tag_group: i32, tag_id: u16, param: Array<u8>, method_info: OptionalMethod){
    match tag_group {
        0..12 => { call_original!(tag_group, tag_id, param, method_info); }
        12 => {
            let name =
                if tag_group == 0 { GameVariableManager::get_string(DVCVariables::LUEUR_NAME) }
                else {
                    let mpid = PersonPool::get().playable[tag_id as usize].get_person_data().get_name();
                    let int_ptr = Mess::get_int_ptr(mpid);
                    get_from_int_ptr(int_ptr).into()
                };
            Mess::set_argument_2(0, name);
        }
        13 => {
            let emblem = EmblemPool::get().emblem_data[tag_id as usize].get_data().get_mid();
            let int_ptr = Mess::get_int_ptr(emblem);
            Mess::set_argument_2(0, get_from_int_ptr(int_ptr));
        }
        14 => { Mess::set_argument(0, tag_id as i32); }
        15..50 => {
            let mut current_mess = Mess::s_mess().to_rust_string();
            let kind = TextReplacementKind::from_mess_tag(tag_group as u16);
            if kind != TextReplacementKind::Pass {
                let arg1 = tag_id & 255;
                let arg2 = (tag_id >> 8) & 255;
                let line_break = if !param.is_null() { Some(param.get(0) as u16) } else { None };
                current_mess += kind.get_text(arg1 as i32, arg2 as i32, line_break).as_str();
                Mess::set_s_mess(current_mess.into());
            }
        }
        50..100 => {
            let mut current_mess = Mess::s_mess().to_rust_string();
            let character = tag_id >> 8;
            let mid = ReplacementMessData::get_random_mid(character as i32, tag_group - 50);
            current_mess += Mess::get(mid).to_rust_string().as_str();
            Mess::set_s_mess(current_mess.into());
        }
        100..150 => {
            let mut current_mess = Mess::s_mess().to_rust_string();
            current_mess += Mess::get(format!("MID_DVC_{}", tag_group - 100)).to_rust_string().as_str();
            Mess::set_s_mess(current_mess.into());
        }
        _ => {}
    }
}

pub fn get_from_int_ptr(int_ptr: IntPtr) -> String {
    let ptr: *mut u16 = int_ptr.as_ptr();
    let mut out = vec![];
    let mut count = 0;
    unsafe {
        loop {
            let c = *ptr.add(count);
            count += 1;
            if c == 14 {
                count += 2;
                let command_lenth = *ptr.add(count);
                count += 1;
                if command_lenth > 0 {
                    let commad_size = command_lenth >> 1;
                    for _ in 0..commad_size { count += 1; }
                }
            }
            else if c == 0 { return String::from_utf16(&out).unwrap(); }
            else { out.push(c); }
        }
    }
}
pub fn copy_from_u16_ptr(ptr: *mut u16) -> Vec<u16> {
    let mut out = vec![];
    let mut count = 0;
    unsafe {
        loop {
            let c = *ptr.add(count);
            out.push(c);
            count += 1;
            if c == 14 {
                let tag = *ptr.add(count);  count += 1;
                out.push(tag);
                let tag_group = *ptr.add(count);    count += 1;
                out.push(tag_group);
                let command_lenth = *ptr.add(count);count += 1;
                out.push(command_lenth );
                if command_lenth > 0 {
                    let commad_size = command_lenth >> 1;
                    for _ in 0..commad_size {
                        let c = *ptr.add(count);    count += 1;
                        out.push(c);
                    }
                }
            }
            else if c == 0 { return out; }
        }
    }
}