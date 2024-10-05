use il2cpp::method;
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
    if str_contains(mid, "MID_HUB") { return true; }
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
                list_pid.push(x);
            }
        }
        for x in list_pid {
            let pid = GameVariableManager::get_string(&format!("G_R_{}", PIDS[x]));
            cmd = replace_strs_il2str(cmd, PIDS[x], pid);
        }
    }
    if GameVariableManager::get_number("G_Emblem_Mode") != 0 {
        let mut list_pid: Vec<usize> = Vec::new();
        for x in 0..EMBLEM_GIDS.len() {
            if str_contains(cmd, EMBLEM_GIDS[x]) {
                list_pid.push(x);
            }
        }

        for x in list_pid {
            let pid = GameVariableManager::get_string(&format!("G_R_{}", EMBLEM_GIDS[x]));
            cmd = replace_strs_il2str(cmd, EMBLEM_GIDS[x], pid);
        }
    }

    //println!("Mess: {}: CMD: {}", this.mess_file_name.get_string().unwrap(), cmd_line.get_string().unwrap());
    return call_original!(this, cmd, method_info);
}

#[skyline::hook(offset=0x020c5370)]
pub fn get_active_character_hook(this: &mut TalkSequence, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(this, method_info);
    if is_character_specific() { return result; }
    let str1 = result.get_string().unwrap();
    if GameVariableManager::exist(&format!("G_R_PID_{}", str1)) {
        let pid = GameVariableManager::get_string(&format!("G_R_PID_{}", str1));
        return il2_str_substring(pid, 4);
    }

    if GameVariableManager::exist(&format!("G_R_GID_{}", str1)) && !is_emblem_paralogue()  {
        let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", str1));
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

#[skyline::hook(offset=0x025d7f90)]
pub fn mess_load_hook(filename: &Il2CppString, method_info: OptionalMethod) -> *const u8 {
    println!("Mess Label: {}", filename.get_string().unwrap());
    let mut result = call_original!(filename, method_info);
    let mut string = unsafe { to_string(filename, result, None) };
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        let mut list_pid: Vec<usize> = Vec::new();
        for x in 1..41 {
            let mpid = Mess::get(MPIDS[x]);
            if str_contains2(string, mpid) {
                list_pid.push(x);
            }
        }
        for x in list_pid {
            let pid = GameVariableManager::get_string(&format!("G_R_{}", PIDS[x]));
            let mpid_2 = Mess::get_name(pid);
            let mpid_1 = Mess::get_name(PIDS[x]);
            string = replace_strs_il2str(string, mpid_1, mpid_2);
        }
        println!("new string: {}", string.get_string().unwrap());
        result = string.get_string().unwrap().as_str().as_ptr();
    }
    result

}