pub use engage::{
    gamedata::{Gamedata, PersonData, GodData},
    random::Random, god::GodPool,
    gamevariable::*, gameuserdata::*,
    util::get_singleton_proc_instance, unit::UnitPool,
    sequence::{talk::*, eventdemo::EventDemoSequence}, 
    mess::*, tmpro::*,
};
use crate::{
    DVCVariables,
    config::DVCFlags,
    enums::{EMBLEM_ASSET, EMBLEM_GIDS},
    utils::*, ironman::vtable_edit,
    randomizer::data::{EmblemPool, RandomizedGameData},
};
pub const VEYRE: [&str; 8] = [
    "ヴェイル_フード_顔出し", // Hooded but with Face
    "ヴェイル_黒_善_角折れ", // Broken Helm
    "ヴェイル_フード", // Hooded no face
    "ヴェイル_包帯",  // Normal
    "ヴェイル_白_悪", // white evil
    "ヴェイル_黒_悪", // black evil
    "ヴェイル_黒_善", // black good
    "M021_ヴェイル",    // Chapter 21 black evil
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
fn talk_tag_window(this: &mut TalkTagWindow, ptr: &TalkPtr, _optional_method: OptionalMethod) {
    this.initialize_(ptr);
    if this.tag_id < 8 {
        if let Some(pid) = this.pid {
            let pid_str = pid.to_string();
            if VEYRE.iter().position(|v| pid_str.ends_with(*v)).filter(|x| *x != 2).is_some() {
                if let Some(person) = DVCVariables::get_dvc_person_data(32, false) {
                    this.replacement_name = person.get_name();
                    if let Some(talk) = TalkSequence::get_instance() { talk.add_replace_talker_name(pid, person.get_name()); }
                    return;
                }
            }
            else { this.pid = Some(get_pid_replacement(pid)); }
        }
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
        return GameVariableManager::get_string(&format!("G_R_PID_{}", str1)).to_string().trim_start_matches("PID_").into();
    }
    if result.contains("ジェーデ_兜あり") {
        if GameVariableManager::exist("G_R_PID_ジェーデ") {
            return GameVariableManager::get_string("G_R_PID_ジェーデ").to_string().trim_start_matches("PID_").into();
        }
    }
    if RAFALE.iter().any(|v| str1 == *v) {
        return GameVariableManager::get_string("G_R_PID_ラファール").to_string().trim_start_matches("PID_").into();
    }
    if let Some(emblem) = EMBLEM_ASSET.iter().position(|v| result.contains(v)) {
        if is_emblem_paralogue() && emblem < 12 {
            if let Some(god) = EmblemPool::get_dvc_emblem_data(EMBLEM_GIDS[emblem])
                .filter(|g| EmblemPool::is_custom(g))
            {
                return god.gid.to_string().trim_start_matches("GID_").into();
            }
        }
        else if !is_emblem_paralogue() {
            if let Some(god_unit) = DVCVariables::get_current_god(emblem as i32) {
                return god_unit.gid.to_string().trim_start_matches("GID_").into();
            }
            else if let Some(god) = DVCVariables::get_god_from_index(emblem as i32, true){
                return god.gid.to_string().trim_start_matches("GID_").into();
            }
        }
    }
    result
}