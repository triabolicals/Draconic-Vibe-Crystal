pub use engage::{
    gamedata::{Gamedata, PersonData, GodData},
    random::Random,
    god::GodPool,
    gamevariable::*, 
    gameuserdata::*,
    util::get_singleton_proc_instance,
    unit::UnitPool,
    sequence::talk::*,
    mess::*,
    tmpro::*,
};
use engage::sequence::eventdemo::EventDemoSequence;
use crate::config::DVCFlags;
use crate::utils::*;
use crate::ironman::vtable_edit;
use crate::randomizer::data::RandomizedGameData;

pub const VEYRE: [&str; 7] = [
    "ヴェイル_黒_善_角折れ",
    "ヴェイル_フード", "ヴェイル_包帯", "ヴェイル_フード_顔出し", "ヴェイル_白_悪",
    "ヴェイル_黒_悪", "ヴェイル_黒_善", 
];
/*
const LUEUR: [&str; 6] = [
    "青リュール", "デモ用_神竜王リュール", "青リュール_男性", "青リュール_女性",
    "デモ用_神竜王リュール_女性", "デモ用_神竜王リュール_男性"
];
 */

const RAFALE: [&str; 2] = ["イル", "デモ用_竜石なし_ラファール"];
pub fn fill_name_array() {
    vtable_edit(
        Il2CppClass::from_name("App.Talk3D", "TalkTagName").unwrap(),
        "Initialize",
        crate::message::talk_tag_name_initialize as _ );

    vtable_edit(
        Il2CppClass::from_name("App.Talk3D", "TalkTagAnimation").unwrap(),
        "Initialize",
        talk_tag_anim as _
    );
    vtable_edit(
        Il2CppClass::from_name("App.Talk3D", "TalkTagWindow").unwrap(),
        "Initialize",
        talk_tag_window as _
    );
    vtable_edit(
        Il2CppClass::from_name("App.Talk3D", "TalkTagAddLetter").unwrap(),
        "Execute",
        crate::message::talk_tag_add_letter_execute_edit as _
    );
}

fn is_character_specific() -> bool {
    let mid = Talk::get_playing_mid();
    let chapter = GameUserData::get_chapter().cid.to_string();
    (mid.contains("MID_TK_") && chapter == "CID_M022") ||
    chapter == "CID_M026" ||
    mid.contains( "MID_KR_") ||
    mid.contains( "MID_GR_") ||
    mid.contains( "MID_DIE") ||
    mid.contains( "MID_RELIANCE") ||
    mid.contains( "MID_LVUP") ||
    (mid.contains( "MID_HUB") && ( !mid.contains( "MID_HUB_DLC") && !mid.contains( "MID_HUB_Mascot")))
}
fn is_emblem_paralogue() -> bool {
    let chapter = GameUserData::get_chapter().cid.to_string();
    (
        chapter.contains("CID_S0") &&
            !chapter.contains("CID_S015") &&
            !chapter.contains("CID_S001") &&
            !chapter.contains("CID_S002")
    )
    || chapter.contains("CID_G00")
}
/*
#[skyline::hook(offset=0x024d3c50)]
pub fn get_cmd_info_from_cmd_lines_hook(this: &EventDemoSequence, mut cmd: &mut Il2CppString, method_info: OptionalMethod) -> u64 {
    if this.demo_name.str_contains("MID_RELIANCE") || this.demo_name.str_contains("MID_GR") { return call_original!(this, cmd, method_info); }
    if DVCVariables::UnitRecruitment.get_value() != 0 {
        let mut list_pid: Vec<usize> = Vec::new();
        if let Some(pos) = VEYRE.iter().position(|&x| cmd.str_contains(format!("PID_{}", x).as_str())) { list_pid.push(50+pos) }
        if cmd.str_contains("デモ用_竜石なし_ラファール"){ list_pid.push(43); }
        for x in 0..41 {
            if cmd.str_contains(PIDS[x]) {
                println!("Replacing {}", x);
                if PIDS[x] == "PID_ジェーデ" {
                    if cmd.str_contains("PID_ジェーデ_兜あり") {  //Check for Jade Helmet
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
                43 => { // Cutscene Rafale
                    let pid = GameVariableManager::get_string("G_R_PID_ラファール");
                    cmd = replace_strs_il2str(cmd, "PID_デモ用_竜石なし_ラファール", pid);
                }
                50..57 => { //Alternative Veyle
                    let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
                    if pid.to_string() != PIDS[32] { cmd = replace_strs_il2str(cmd, format!("PID_{}", VEYRE[x-50]), pid); }
                }
                /*
                60..66 => { // Other Alears
                    let pid = GameVariableManager::get_string("G_R_PID_リュール");
                    if pid.to_string() != PIDS[0] { cmd = replace_strs_il2str(cmd, format!("PID_{}", LUEUR[x-60]), pid); }
                }
                */
                _ => {
                    let pid = GameVariableManager::get_string(&format!("G_R_{}", PIDS[x]));
                    cmd = replace_strs_il2str(cmd, PIDS[x], pid);
                }
            }
        }
    }
    if DVCVariables::EmblemRecruitment.get_value() != 0 && !is_emblem_paralogue() {
        let mut list_pid2: Vec<usize> = Vec::new();
        for x in 0..19 { if cmd.str_contains(EMBLEM_GIDS[x]) { list_pid2.push(x); } }
        if cmd.str_contains("GID_ディミトリ") { list_pid2.push(20); }
        if cmd.str_contains("GID_クロード") { list_pid2.push(21); }
        for x in list_pid2 {
            if x == 20 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.to_string() != "GID_エーデルガルト" {  cmd = replace_strs_il2str(cmd, "GID_ディミトリ", gid); }
            }
            else if x == 21 {
                let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
                if gid.to_string() != "GID_エーデルガルト" { cmd = replace_strs_il2str(cmd, "GID_クロード", gid); }
            }
            else {
                let pid = GameVariableManager::get_string(&format!("G_R_{}", EMBLEM_GIDS[x]));
                cmd = replace_strs_il2str(cmd, EMBLEM_GIDS[x], pid);
            }
        }
    }
    call_original!(this, cmd, method_info)
}
*/
#[skyline::hook(offset=0x20c5440)]
pub fn get_active_character_hook(this: &mut TalkPtr, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(this, method_info);
    let talk_pid = GameVariableManager::get_number("TalkPID");
    if talk_pid != 0 {
        if let Some(p) = PersonData::try_get_hash(talk_pid) {
            return il2_str_substring(p.pid, 4);
        }
    }
    if is_character_specific() { return result; }
   //  if get_singleton_proc_instance::<EventDemoSequence>().is_none() && (sequence  == 4 || sequence == 5)  { return result; }
    let str1 = result.to_string();
    if GameVariableManager::exist(&format!("G_R_PID_{}", str1)) {
        let pid = GameVariableManager::get_string(&format!("G_R_PID_{}", str1));
        return il2_str_substring(pid, 4);
    }
    if result.contains("ジェーデ_兜あり") {
        if GameVariableManager::exist("G_R_PID_ジェーデ") {
            let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
            return il2_str_substring(pid, 4);
        }
    }
    if VEYRE.iter().any(|v| result.contains(v)) {
        let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
        return il2_str_substring(pid, 4);
    }
    if RAFALE.iter().any(|v| str1 == *v) {
        let pid = GameVariableManager::get_string("G_R_PID_ラファール");
        return il2_str_substring(pid, 4);
    }
    if GameVariableManager::exist(&format!("G_R_GID_{}", str1)) && !is_emblem_paralogue()  {
        let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", str1));
        if let Some(god) = GodPool::try_get_gid(gid, false) {
            return il2_str_substring(god.data.gid, 4);
        }
        else { return il2_str_substring(gid, 4); }
    }
    if result.contains("ディミトリ") || result.contains("クロード") && GameVariableManager::exist("G_R_GID_エーデルガルト") {
        let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
        return il2_str_substring(gid, 4);
    }
    if result.contains("M000_マルス") {
        if GameVariableManager::exist("G_R_GID_マルス") && !is_emblem_paralogue()  {
            let gid = GameVariableManager::get_string("G_R_GID_マルス");
            return il2_str_substring(gid, 4);
        }
    }
    result
}

fn talk_tag_window(this: &mut TalkTagWindow, ptr: &TalkPtr, _optional_method: OptionalMethod) {
    this.initialize_(ptr);
    if this.tag_id < 8 {
        if let Some(pid) = this.pid { this.pid = Some(get_pid_replacement(pid)); }
        if DVCFlags::RandomBossesNPCs.get_value() {
            if let Some(pidx) = this.pid.as_ref() {
                let pidx = pidx.to_string().into();
                let pid = format!("PID_{}", pidx);
                if let Some(person) = PersonData::get(pid.as_str()).filter(|x| x.asset_force != 0)
                    .and_then(|p| RandomizedGameData::get_read().person_appearance.get_person_appearance(p).map(|p| p.mpid.clone()))
                {
                    this.replacement_name = Mess::get(person.as_str());
                    if let Some(talk) = TalkSequence::get_instance() {
                        talk.add_replace_talker_name(pidx,  Mess::get(person.as_str()));
                    }
                }
            }
        }
    }
}
fn talk_tag_anim(this: &mut TalkTagAnimation, ptr: &TalkPtr, _optional_method: OptionalMethod) {
    this.initialize_(ptr);
    if let Some(pid) = this.pid { this.pid = Some(get_pid_replacement(pid)); }
}

fn get_pid_replacement(result: &'static Il2CppString) -> &'static Il2CppString {
    let talk_pid = GameVariableManager::get_number("TalkPID");
    if talk_pid != 0 {
        if let Some(p) = PersonData::try_get_hash(talk_pid) {
            return il2_str_substring(p.pid, 4);
        }
    }
    let sequence = GameUserData::get_sequence();
    if get_singleton_proc_instance::<EventDemoSequence>().is_none() && (sequence  == 4 || sequence == 5)  { return result; }
    if is_character_specific() { return result; }

    let str1 = result.to_string();
    if GameVariableManager::exist(&format!("G_R_PID_{}", str1)) {
        let pid = GameVariableManager::get_string(&format!("G_R_PID_{}", str1));
        return il2_str_substring(pid, 4);
    }
    if result.contains("ジェーデ_兜あり") {
        if GameVariableManager::exist("G_R_PID_ジェーデ") {
            let pid = GameVariableManager::get_string("G_R_PID_ジェーデ");
            return il2_str_substring(pid, 4);
        }
    }
    if VEYRE.iter().any(|v| result.contains(v)) {
        let pid = GameVariableManager::get_string("G_R_PID_ヴェイル");
        return il2_str_substring(pid, 4);
    }
    if RAFALE.iter().any(|v| str1 == *v) {
        let pid = GameVariableManager::get_string("G_R_PID_ラファール");
        return il2_str_substring(pid, 4);
    }
    if GameVariableManager::exist(&format!("G_R_GID_{}", str1)) && !is_emblem_paralogue()  {
        let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", str1));
        if let Some(god) = GodPool::try_get_gid(gid, false) {
            return il2_str_substring(god.data.gid, 4);
        }
        else { return il2_str_substring(gid, 4); }
    }
    if (result.contains("ディミトリ") || result.contains("クロード")) && GameVariableManager::exist("G_R_GID_エーデルガルト") {
        let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト");
        return il2_str_substring(gid, 4);
    }
    if result.contains("マルス") {
        if GameVariableManager::exist("G_R_GID_マルス") && !is_emblem_paralogue()  {
            let gid = GameVariableManager::get_string("G_R_GID_マルス");
            return il2_str_substring(gid, 4);
        }
    }
    result
}