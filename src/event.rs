use unity::prelude::*;
pub use engage::{
    random::Random,
    gamevariable::*, 
    gameuserdata::*,
    mess::*,
    tmpro::*,
};
use unity::il2cpp::object::Array;
use crate::enums::*;
use crate::utils::*;

const VEYRE: [&str; 7] = ["PID_ヴェイル_フード", "PID_ヴェイル_包帯", "PID_ヴェイル_フード_顔出し", "PID_ヴェイル_白_悪", "PID_ヴェイル_黒_悪", "PID_ヴェイル_黒_善", "PID_ヴェイル_黒_善_角折れ" ];

#[unity::class("App", "EventDemoSequence")]
pub struct EventDemoSequence {
    proc: [u8; 0x78],
    pub demo_name: &'static Il2CppString,
    pub mess_file_name: &'static Il2CppString,
}
#[unity::class("App", "Talk3D.TalkSequence")]
pub struct TalkSequence {
    proc: u64,
    pub active_pid: &'static Il2CppString,
}

#[skyline::from_offset(0x03782000)]
pub fn to_string(this: &Il2CppString, value: *const u8, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[skyline::from_offset(0x03780660)]
pub fn to_char_array(this: &Il2CppString, method_info: OptionalMethod) -> &'static Array<u16>;

#[skyline::from_offset(0x020b8e10)]
fn get_current_mid(method_info: OptionalMethod) -> &'static Il2CppString;

fn is_character_specific() -> bool {
    let mid = unsafe { get_current_mid(None) };
    if str_contains(mid, "MID_KR_") { return true; }
    if str_contains(mid, "MID_GR_") { return true; }
    if str_contains(mid, "MID_DIE") { return true; }
    if str_contains(mid, "MID_RELIANCE") { return true; }
    if str_contains(mid, "MID_LVUP") { return true; }
    if str_contains(mid, "MID_HUB") { 
        if str_contains(mid, "MID_HUB_DLC") { return false; }
        if str_contains(mid, "MID_HUB_Mascot") { return false; }    
        return true; 
    }
    return false; 
}
fn is_emblem_paralogue() -> bool {
    let chapter = GameUserData::get_chapter().cid;
    if str_contains(chapter, "CID_S015") || str_contains(chapter, "CID_S001") || str_contains(chapter, "CID_S002") {
        return false
    }
    return str_contains(chapter, "CID_S0") || str_contains(chapter, "CID_G00");
}


#[skyline::hook(offset=0x024d3c50)]
pub fn get_cmd_info_from_cmd_lines_hook(this: &EventDemoSequence, mut cmd: &mut Il2CppString, method_info: OptionalMethod) -> u64 {
    //println!("MID for CMD: {} / {}", this.demo_name.get_string().unwrap(), unsafe { get_current_mid(None) }.get_string().unwrap());
    if str_contains(this.demo_name, "MID_RELIANCE") || str_contains(this.demo_name, "MID_GR") { return call_original!(this, cmd, method_info); }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        let mut list_pid: Vec<usize> = Vec::new();
        for x in 0..41 {
            if str_contains(cmd, PIDS[x]) {
                if PIDS[x] == "PID_ヴェイル" {  //Check for other Veyles
                    if VEYRE.iter().find(|&x| str_contains(cmd, x)).is_none() { list_pid.push(x);  }
                    else if str_contains(cmd, "PID_ヴェイル_包帯") { list_pid.push(43); }
                }
                else if PIDS[x] == "PID_ジェーデ" {
                    if str_contains(cmd, "PID_ジェーデ_兜あり") {  //Check for Jade Helmet
                        list_pid.push(42); 
                    }
                    else { list_pid.push(x); }
                }
                else { list_pid.push(x); }
            }
        }
        for x in list_pid {
            match x {
                42 => { //Alternative Jade
                    let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
                    cmd = replace_strs_il2str(cmd, "PID_ジェーデ_兜あり", pid);
                }
                43 => { //Alternative Veyle
                    let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
                    cmd = replace_strs_il2str(cmd, "PID_ヴェイル_包帯", pid);
                }
                _ => {
                    let pid = GameVariableManager::get_string(&format!("G_R_{}", PIDS[x]));
                    cmd = replace_strs_il2str(cmd, PIDS[x], pid);
                }
            }
        }
    }
    if GameVariableManager::get_number("G_Emblem_Mode") != 0 && !is_emblem_paralogue() {
        let mut list_pid2: Vec<usize> = Vec::new();
        for x in 0..EMBLEM_GIDS.len() {
            if str_contains(cmd, EMBLEM_GIDS[x]) { list_pid2.push(x); }
        }
        if str_contains(cmd, "GID_ディミトリ") { list_pid2.push(20); }
        if str_contains(cmd, "GID_クロード") { list_pid2.push(21); }
        for x in list_pid2 {
            if x == 20 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.get_string().unwrap() != "GID_エーデルガルト" {  cmd = replace_strs_il2str(cmd, "GID_ディミトリ", gid); }
            }
            else if x == 21 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.get_string().unwrap() != "GID_エーデルガルト" { cmd = replace_strs_il2str(cmd, "GID_クロード", gid); }
            }
            else {
                let pid = GameVariableManager::get_string(&format!("G_R_{}", EMBLEM_GIDS[x]));
                cmd = replace_strs_il2str(cmd, EMBLEM_GIDS[x], pid);
            }
        }
    }
    return call_original!(this, cmd, method_info);
}

#[skyline::hook(offset=0x020c5370)]
pub fn get_active_character_hook(this: &mut TalkSequence, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(this, method_info);
    if is_character_specific() { return result; }
    let str1 = result.get_string().unwrap();
    if str_contains(result, "ジェーデ_兜あり") {
        if GameVariableManager::exist("G_R_PID_ジェーデ") {
            let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
            return il2_str_substring(pid, 4);
        }
    }
    if str_contains(result, "ヴェイル_包帯") {
        if GameVariableManager::exist("G_R_PID_ヴェイル") {
            let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
            return il2_str_substring(pid, 4);
        }
    }
    if GameVariableManager::exist(&format!("G_R_PID_{}", str1)) {
        let pid = GameVariableManager::get_string(&format!("G_R_PID_{}", str1));
        return il2_str_substring(pid, 4);
    }

    if GameVariableManager::exist(&format!("G_R_GID_{}", str1)) && !is_emblem_paralogue()  {
        let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", str1));
        return il2_str_substring(gid, 4);
    }
    if ( str_contains(result, "ディミトリ") || str_contains(result, "クロード") ) && GameVariableManager::exist("G_R_GID_エーデルガルト") {
        let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
        return il2_str_substring(gid, 4);
    }
    if str_contains(result, "M000_マルス") {
        if GameVariableManager::exist("G_R_GID_マルス") && !is_emblem_paralogue()  {
            let gid = GameVariableManager::get_string("G_R_GID_マルス");
            return il2_str_substring(gid, 4);
        }
    }
    return result; 
}