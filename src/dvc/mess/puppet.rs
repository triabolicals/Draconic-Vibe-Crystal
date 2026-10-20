use engage::{
    nintendo::message_studio::lib::IBinMsgFileMethods,
    system::collections::generic::IDictionary_2Methods
};
use crate::{
    dvc::talk::window::WindowReplacementType,
    enums::{EMBLEM_GIDS, PIDS},
    gamedata::PersonPool
};
use super::*;
const DEMO_SEPARATOR: [char; 7] = ['(', ')', ',', '[', ']', '/', '\\'];

pub fn edit_all_pid_demo_mess_label(mess_label: &str) {
    let (found, int_ptr) =  Mess::s_event_cmd_data_dictionary().try_get_value(mess_label.into());
    if found && !int_ptr.is_null() {
        let ptr = int_ptr.as_ptr();
        let mut message = copy_from_u16_ptr(ptr);
        let len = message.len();
        if len == 0 { return; }
        PIDS.iter().enumerate().for_each(|(i, &x)| { find_and_splice(x, i, &mut message, TextReplacementKind::DemoPID); });
        if message.len() <= len { for i in 0..message.len() { unsafe { ptr.add(i).replace(message[i]); } } }
    }
} 

pub fn edit_demo_mess_file(mess: Il2CppString){
    let (found, file) = Mess::s_event_cmd_file_dictionary().try_get_value(mess.into());
    if found && !file.is_null() {
        #[cfg(feature = "debug")] println!("Editing: {} Puppet", mess);
        let entries = file.get_text_num();
        let mess_str = mess.to_rust_string();
        for i in 0..entries {
            let ptr = file.get_text_2(i).as_ptr();
            let mut message = copy_from_u16_ptr(ptr);
            let len = message.len();
            if len == 0 { continue; }
            PIDS.iter().enumerate().for_each(|(i, &x)| { find_and_splice(x, i, &mut message, TextReplacementKind::DemoPID); });
            if mess_str.contains("E00") {
                for i in 1..7 {
                    let chapter = format!("PID_E00{}_", i);
                    for x in 36..41 {
                        let pid = PIDS[x].replace("PID_", &chapter);
                        find_and_splice(pid.as_str(), x, &mut message, TextReplacementKind::DemoPID);
                    }
                    let il = format!("PID_{}_イル", i);
                    find_and_splice(il.as_str(), 37, &mut message, TextReplacementKind::DemoPID);
                }
            }
            find_and_splice("PID_デモ用_竜石なし_ラファール", 43, &mut message, TextReplacementKind::DemoPID);
            find_and_splice("PID_ジェーデ_兜あり", 42, &mut message, TextReplacementKind::DemoPID);
            if mess_str.contains("S0") && !mess_str.contains("S015") {
                for i in 0..12 { find_and_splice(EMBLEM_GIDS[i], i, &mut message, TextReplacementKind::DemoCustomEmblem); }
            }
            else {
                for i in 0..23 {
                    if i == 19 { continue; }
                    find_and_splice(EMBLEM_GIDS[i], i, &mut message, TextReplacementKind::DemoGID);
                }
            }
            if message.len() <= len { for i in 0..message.len() { unsafe { ptr.add(i).replace(message[i]); } } }
        }
    }
}
pub fn edit_demo_mess_for_reliance(mess: Il2CppString, pid_1: &str, pid_2: &str, person_after_1: i32, person_after_2: i32){
    let (found, file) = Mess::s_event_cmd_file_dictionary().try_get_value(mess.into());
    if found && !file.is_null() {
        let entries = file.get_text_num();
        for i in 0..entries {
            let ptr = file.get_text_2(i).as_ptr();
            let mut message = copy_from_u16_ptr(ptr);
            let len = message.len();
            if len == 0 { continue; }
            find_and_splice(pid_1, person_after_1 as usize, &mut message, TextReplacementKind::DemoPIDNoSwap);
            find_and_splice(pid_2, person_after_2 as usize, &mut message, TextReplacementKind::DemoPIDNoSwap);
            if message.len() <= len { for i in 0..message.len() { unsafe { ptr.add(i).replace(message[i]); } } }
        }
    }
}
pub fn edit_reliance_talk_character(mess: Il2CppString, person_before_1: i32, person_before_2: i32, person_after_1: i32, person_after_2: i32){
    let (found, file) = Mess::s_mess_file_dictionary().try_get_value(mess);
    if found && !file.is_null() {
        let person_pool = PersonPool::get();
        let pid_1 = &person_pool.pids[person_before_1 as usize];
        let pid_2 = &person_pool.pids[person_before_2 as usize];
        let num = file.get_text_num();
        let name_1 = Mess::get(format!("MPID_{}", pid_1.1.as_str())).to_rust_string();
        let name_2 = Mess::get(format!("MPID_{}", pid_2.1.as_str())).to_rust_string();
        for i in 0..num {
            let ptr = file.get_text_2(i).as_ptr();
            let mut str = copy_from_u16_ptr(ptr);
            let len = str.len();
            WindowReplacementType::DirectSwap.try_replace_pid(&mut str, person_before_1, false, Some(person_after_1));
            WindowReplacementType::DirectSwap.try_replace_pid(&mut str, person_before_2, false, Some(person_after_2));
            find_and_splice(name_1.as_str(), person_after_1 as usize, &mut str, TextReplacementKind::UnitNameNoSwap);
            if person_after_1 != 0 { find_and_splice("Divine One", person_after_1 as usize, &mut str, TextReplacementKind::UnitNameNoSwap); }
            find_and_splice(name_2.as_str(), person_after_2 as usize, &mut str, TextReplacementKind::UnitNameNoSwap);
            let len2 = str.len();
            if str.len() <= len { for i in 0..len2 { unsafe { ptr.add(i).replace(str[i]); } } }
        }
    }
}
pub fn remove_line_with_argument(message: &mut Vec<u16>, arg: &str) {
    let slice = arg.encode_utf16().collect::<Vec<u16>>();
    while let Some(pos) = message.windows(slice.len()).position(|w| w == slice){
        let line_break_before = message.iter().enumerate().filter(|&(i, c)| i < pos && *c == 10).map(|(i, _)| i).max().unwrap_or(0);
        if let Some(line_break_after) = message.iter().enumerate().filter(|&(i, c)| i > (pos + slice.len())  && *c == 10).map(|(i, _)| i).min(){
            message.splice(line_break_before..line_break_after, [10]);
        }
        else { message.splice(line_break_before..message.len(), [0]); }
        if message.len() < slice.len() { return; }
    };
}
pub fn find_and_splice(pid: &str, index: usize, message: &mut Vec<u16>, text_replacement_kind: TextReplacementKind) {
    let pid_slice = pid.encode_utf16().collect::<Vec<u16>>();
    if message.len() < pid_slice.len() { return; }
    let index =
        if text_replacement_kind == TextReplacementKind::DemoPID {
            match index {
                0..41 => { index }
                41|42 => { 16 }
                43 => { 37 }
                50..60 => { 32 }
                _ => { index }
            }
        }
        else if text_replacement_kind == TextReplacementKind::DemoGID {
            match index {
                20|21 => 12,
                0..20 => index,
                _ => { index }
            }
        }
        else { index };
    let name_only = TextReplacementKind::UnitNameNoSwap == text_replacement_kind;

    if name_only && index < PersonPool::get().playable.len() {
        let msg_data = MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap();
        while let Some(pos) = message.windows(pid_slice.len()).position(|w| w == pid_slice) {
            message.splice(pos..pos+pid_slice.len(), [14, text_replacement_kind.to_mess_tag(), index as u16, 0]);
            if !msg_data.gender.is_empty() && pos > 2 {
                for x in 0..3 {
                    if x == 1 { continue; }
                    if let Some(pos2) = msg_data.gender.get(x).and_then(|mess|mess.find_from(message, pos - 2)).filter(|p| p.position < pos){
                        pos2.replace_for_text(message, TextReplacementKind::GenderStringNoSwap, index as u16 | ((x as u16) << 8));
                        break;
                    }
                }
            }
            if message.len() < pid_slice.len() { return; }
        };
    }
    else {
        while let Some(pos) = message.windows(pid_slice.len() + 1)
            .position(|w|
                pid_slice.iter().zip(w.iter()).all(|(p, p2)| *p == *p2) &&
                    w.last().and_then(|w| char::from_u32(*w as u32)).is_none_or(|w| w.is_whitespace() || DEMO_SEPARATOR.contains(&w))
            )
        {
            message.splice(pos..pos+pid_slice.len(), [14, text_replacement_kind.to_mess_tag(), index as u16, 0]);
            if message.len() < pid_slice.len() { return; }
        };
    }
}