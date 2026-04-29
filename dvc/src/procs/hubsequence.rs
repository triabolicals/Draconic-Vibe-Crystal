use std::sync::RwLock;
use engage::{
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
    proc::{ProcInst, ProcVoidMethod, desc::ProcDesc},
    sequence::hub::HubSequence,
    
};
use unity::{prelude::OptionalMethod, il2cpp::object::Array};
use crate::{
    message::{TextSwapper, MESSAGE_SWAPPER},
    procs::call_proc_original_method,
    randomizer, DVCVariables,
    randomizer::item,
    utils::dlc_check,
};
fn is_continuous_gift(key: &str) -> bool {
    !GameVariableManager::get_bool(key) && 
    DVCVariables::is_continuous() && GameUserData::get_sequence() == 5 && DVCVariables::is_main_chapter_complete(4)
}
pub fn hub_sequence_desc_edit(descs: &mut Array<&'static mut ProcDesc>) {
    randomizer::emblem::update_lueur_bonds();
    if GameUserData::get_sequence() == 5 { randomizer::person::hub::change_kizuna_dispos(); }
    // Edit Message Scripts after loading
    descs[6] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_load_script));
    // Randomized Exploration Items here
    descs[28] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_map_opening_event));
    
    // Gift Events for Continuious Mode here
    descs[32] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_dlc_gift_0_event));
    descs[33] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_patch_0_gift_event));
    descs[34] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_dlc_gift_1_event));
    descs[35] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_patch_3_gift_event));
    
    // Clear edited message scripts when it unloads
    descs[216] = ProcDesc::call(ProcVoidMethod::new(None, hub_sequence_unload_script));
}


pub extern "C" fn hub_sequence_load_script(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    call_proc_original_method(proc, "LoadScript");
    if GameUserData::get_sequence() != 4 {
        let swap = MESSAGE_SWAPPER.get_or_init(||RwLock::new(TextSwapper::init()));
        if let Ok(mut lock) = swap.try_write() { lock.get_chapter_talk(); }
    }
}

pub extern "C" fn hub_sequence_unload_script(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    call_proc_original_method(proc,"UnloadScript");
    let swap = MESSAGE_SWAPPER.get_or_init(||RwLock::new(TextSwapper::init()));
    if let Ok(mut lock) = swap.try_write() { lock.current_talk_lines.clear(); }
}

pub extern "C" fn hub_sequence_map_opening_event(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    if GameUserData::get_sequence() == 4 || GameUserData::get_sequence() == 5 {
        call_proc_original_method(proc,"MapOpeningEvent");
        item::hub::hub_item_randomized();
    }
}

pub extern "C" fn hub_menu_sequence_next_map_bind(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    let chapter = GameUserData::get_chapter();
    let current_flags = chapter.flag;
    chapter.flag = 0;
    call_proc_original_method(proc, "OpenDialogNext");
    chapter.flag = current_flags;
}

extern "C" fn hub_sequence_dlc_gift_0_event(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    let flag = "G_拠点_DLC特典アイテム0受け取り済み";
    let reward = "DLC購入特典0";
    let message = "MID_MSG_GET_ITEM_DLC_Accessory1";
    if is_continuous_gift(flag) && dlc_check() {
        if try_hub_gift_get(flag, reward, message) { return; }
    }
    call_proc_original_method(proc,"DlcGift0Event");
}
extern "C" fn hub_sequence_dlc_gift_1_event(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    let flag = "G_拠点_DLC特典アイテム1受け取り済み";
    let reward = "DLC購入特典1";
    let message = "MID_MSG_GET_ITEM_DLC_Accessory2";
    if is_continuous_gift(flag) && dlc_check() {
        if try_hub_gift_get(flag, reward, message) { return; }
    }
    call_proc_original_method(proc,"DlcGift1Event");
}

extern "C" fn hub_sequence_patch_0_gift_event(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    let flag = "G_拠点_Patch0特典アイテム受け取り済み";
    let reward = "Patch0特典";
    let message = "MID_MSG_GET_ITEM_Patch0";
    if is_continuous_gift(flag) && dlc_check() {
        if try_hub_gift_get(flag, reward, message) { return; }
    }
    call_proc_original_method(proc,"Patch0GiftEvent");
}

extern "C" fn hub_sequence_patch_3_gift_event(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    let flag = "G_拠点_Patch3特典アイテム受け取り済み";
    let reward = "Patch3特典";
    let message = "MID_MSG_GET_ITEM_Patch3";
    if is_continuous_gift(flag) && dlc_check() { if try_hub_gift_get(flag, reward, message) { return; } }
    call_proc_original_method(proc,"Patch3GiftEvent");
}


fn try_hub_gift_get(flag: &str, reward_id: &str, message_id: &str) -> bool {
    if let Some(v) = HubSequence::get_instance(){
        if !GameVariableManager::exist(flag) { GameVariableManager::make_entry(flag, 1); }
        else if GameVariableManager::get_bool(flag) { return false; }
        else { GameVariableManager::set_bool(flag, true); }
        v.gift_get(reward_id, message_id);
        true
    }
    else { false }
}
