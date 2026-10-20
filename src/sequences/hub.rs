use std::collections::HashMap;
use engage::{
    app::{
        hubsequence::*, hubcookingsequence::*, dlcmanager::*,
        commonrewardsequence::*,
        GameIcon,
        itemdata::*, rewarddata::*, itemlistyesnodialog::*, chapterdata::*, gameuserdata::*,
        IBasicDialogMethods, IStructBase, IStructData_1Methods,
        Mess, CookData, HubUtil, PersonData, UnitPool
    },
    GameVariableManager,
    List_1Ext,
    prelude::List_1,
    system::{collections::generic::IList_1Methods, Action},
    app::Transporter
};
use super::*;
use unity::{Cast, Il2CppString, IlNull, OptionalMethod};
use crate::{
    DVCVariable, DVCVariables,
    dvc::{mess, mess::ReplacementMessData, DVC, self, RandomItemReplacementType},
    gamedata::{GameData, PersonPool},
    utils::to_opt
};

pub fn hub_sequence_desc_edit(proc: ProcInst) {
    dvc::randomizer::godbond::update_lueur_bonds();
    ReplacementMessData::randomize_hub_data(false);
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast::<HubSequence>() { 
        descs.set(6, Proc::call_2(ProcVoidMethod::from_fn(proc.into(), load_hub_script).unwrap()));
        descs.set(26, Proc::call_2(ProcVoidMethod::from_fn(proc.into(), hub_prepare_event).unwrap()));
        descs.set(32, Proc::call_2(ProcVoidMethod::new(proc.into(), hub_sequence_dlc_gift_0_event_method_info().into())));
        descs.set(33, Proc::call_2(ProcVoidMethod::new(proc.into(), hub_sequence_patch_0_gift_event_method_info().into())));
        descs.set(34, Proc::call_2(ProcVoidMethod::new(proc.into(), hub_sequence_dlc_gift_1_event_method_info().into())));
        descs.set(35, Proc::call_2(ProcVoidMethod::new(proc.into(), hub_sequence_patch_3_gift_event_method_info().into())));
        descs.set(188, Proc::call_2(ProcVoidMethod::from_fn(proc, hub_sequence_kizuna_exit).unwrap()));
        descs.set(216, Proc::call_2(ProcVoidMethod::from_fn(proc, hub_sequence_unload_script).unwrap()));
    }
}
pub fn hub_cooking_sequence_edit(proc: ProcInst) {
    if let Some(proc) = proc.try_cast::<HubCookingSequence>() {
        let descs = proc.m_descs();
        descs.set(4,  Proc::call_2(ProcVoidMethod::from_fn(proc, hub_cooking_sequence_init).unwrap()));
    }
}
extern "C" fn hub_prepare_event(hub: HubSequence, _: OptionalMethod) {
    hub.prepare_event();
    dvc::randomizer::hub::hub_item_randomization();
}
extern "C" fn load_hub_script(hub_sequence: HubSequence, _: OptionalMethod) {
    hub_sequence.load_script();
    let sequence = DVCVariables::get_sequence();
    if sequence == 5 {
        let chapter = DVCVariables::user_data().get_chapter();
        if !chapter.is_null() {
            let mess = chapter.get_mess();
            if !mess.is_null() { mess::edit_msgfile_2(mess, true); }
        }
        if DVCVariables::is_main_chapter_complete(4) {
            let dvc = DVC::get();
            dvc.god_recruit = 0;
            let playable_list = &PersonPool::get().playable;
            if playable_list.len() > 41{
                dvc.kizuna_recruit.clear();
                for i in 41..playable_list.len() {
                    if let Some(playable) = playable_list.get(i)
                        .and_then(|d| d.get_replacement(false))
                        .filter(|p| UnitPool::get_from_person(*p, false).is_null())
                    {
                        dvc.kizuna_recruit.push(playable.hash());
                    }
                    if dvc.kizuna_recruit.len() >= 4 { break; }
                }
            }
        }
    }
}
pub fn well_sequence_desc_edit(proc: ProcInst){
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast(){
        descs.set(19, Proc::call_2(ProcVoidMethod::from_fn(proc, dvc::randomizer::item::well_get_item_rng).unwrap()));
    }
}
extern "C" fn hub_sequence_kizuna_exit(hub: HubSequence, _: OptionalMethod) {
    let mode =  DVCVariable::Continuous.get_value();
    if DVCVariables::is_main_chapter_complete(4) && (mode == 2 || mode == 1 ){
        hubmenu::next_chapter_dialog(hub, Action::null());
    }
    else { hub.main_kizuna_exit(); }
}
extern "C" fn hub_sequence_unload_script(proc: HubSequence, _: OptionalMethod){
    DVC::get().reset_set_jobs();
    proc.unload_script();
    if let Some(chapter) = DVC::get().con_chapter.and_then(|v| to_opt(ChapterData::try_get_from_hash(v))) {
        let mess = chapter.get_mess();
        if !mess.is_null() { Mess::free(mess); }
        DVC::get().con_chapter = None;
    }
}
extern "C" fn hub_cooking_sequence_init(proc: HubCookingSequence, _: OptionalMethod){
    proc.init();
    if proc.m_current_cook().is_null() {
        let pid = HubUtil::get_current_cooking_pid();
        if !pid.is_null() {
            let person = PersonData::get(pid);
            PersonPool::create_missing_data(person);
            PersonPool::get().randomize_cook_data(CookData::get(pid));
        }
    }
    ReplacementMessData::init_random_data(true);
    ReplacementMessData::randomize_hub_data(true);
}
#[unity::callback]
extern "C" fn hub_sequence_dlc_gift_0_event(proc: HubSequence, _optional_method: OptionalMethod) {
    if !DLCManager::has_content(DLCManager_Content::e0()) { return; }
    let (flag, reward, message) = ("G_拠点_DLC特典アイテム0受け取り済み", "DLC購入特典0", "MID_MSG_GET_ITEM_DLC_Accessory1");
    try_gift_event(proc, flag.into(), reward.into(), message.into());
}
#[unity::callback]
extern "C" fn hub_sequence_dlc_gift_1_event(proc: HubSequence, _optional_method: OptionalMethod) {
    if !DLCManager::has_content(DLCManager_Content::e0()) { return; }
    let (flag, reward, message) = ("G_拠点_DLC特典アイテム1受け取り済み", "DLC購入特典1", "MID_MSG_GET_ITEM_DLC_Accessory2");
    try_gift_event(proc, flag.into(), reward.into(), message.into());
}
#[unity::callback]
extern "C" fn hub_sequence_patch_0_gift_event(proc: HubSequence, _optional_method: OptionalMethod) {
    let (flag, reward, message) = ("G_拠点_Patch0特典アイテム受け取り済み", "Patch0特典", "MID_MSG_GET_ITEM_Patch0");
    try_gift_event(proc, flag.into(), reward.into(), message.into());
}
#[unity::callback]
extern "C" fn hub_sequence_patch_3_gift_event(proc: HubSequence, _optional_method: OptionalMethod) {
    let (flag, reward, message) = ("G_拠点_Patch3特典アイテム受け取り済み", "Patch3特典", "MID_MSG_GET_ITEM_Patch3");
    try_gift_event(proc, flag.into(), reward.into(), message.into());
}

fn try_gift_event(hub: HubSequence, flagname: Il2CppString, reward: Il2CppString, message_id: Il2CppString) {
    if !DVCVariables::is_main_chapter_complete(4) || !GameData::get_item_pool().has_reward { return; }
    let mode = DVCVariable::Continuous.get_value();
    let sequence = DVCVariables::get_sequence();
    if ((mode == 1 || mode == 2) && sequence == 5) || sequence == 4 {
        if !GameVariableManager::is_exist(flagname) || GameVariableManager::get_number(flagname) == 0 {
            GameVariableManager::make_entry(flagname, 1);
            let mode = DVCVariable::RandomGifts.get_value();
            if mode == 0 { hub.gift_get(reward, message_id); }
            else { random_gifts(hub, reward, message_id, mode == 2); }
            GameVariableManager::set_number(flagname, 1);
        }
    }
}

fn random_gifts(proc: impl Into<ProcInst>, reward_id: Il2CppString, message_id: Il2CppString, with_rare: bool) {
    let list = RewardData::calc_rewards_2(reward_id);
    if list.is_null() || list.count() == 0 { return; }
    let (list, monies, bond) = adjust_reward_item(list, with_rare);
    let user = DVCVariables::user_data();
    user.set_gold(monies + user.get_gold());
    user.set_piece_of_bond(user.m_piece_of_bond() + bond);
    let param_list = List_1::<ItemListYesNoDialog_ItemParam>::new();
    let mut discard = false;
    list.into_iter().map(|v| (ItemData::try_get_from_hash(v.0), v.1))
        .for_each(|(item, count)| {
            let dialog_item = ItemListYesNoDialog_ItemParam::new();
            dialog_item.set_sprite(GameIcon::try_get_item_2(item));
            dialog_item.set_num(count);
            unsafe { dialog_item.set_name(item_data_get_name(item, None)); }
            if item.is_inventory() { item.add_inventory(count); }
            else {
                for _ in 0..count {
                    if Transporter::get_empty_index() < 0 {
                        discard = true;
                        break;
                    }
                    else { Transporter::add_2(item); }
                }
            }
            param_list.add(dialog_item);
        });
    if bond > 0 {
        let dialog_item = ItemListYesNoDialog_ItemParam::new();
        dialog_item.set_name(Mess::get("MID_PieceOfBonds"));
        dialog_item.set_num(bond);
        param_list.add(dialog_item);
    }
    let message = Mess::get(message_id);
    let money_param =
        if monies < 1 { ItemListYesNoDialog_MoneyParam::null() }
        else {
            let item = ItemListYesNoDialog_MoneyParam::new();
            item.set_num(monies);
            item
        };
    let proc = CommonRewardSequence::try_create_bind_discard_message(proc, discard);
    let dialog = ItemListYesNoDialog::create_bind(proc, message, param_list, money_param);
    dialog.set_background_blur_off();
}

fn adjust_reward_item(list: List_1::<ItemData>, rare: bool) -> (HashMap<i32, i32>, i32, i32) {
    let pool = GameData::get_item_pool();
    let gold_total: i32 = list.iter().filter(|i| i.get_kind() == ItemData_Kinds::gold()).map(|i| i.get_price()).sum();
    let mut current_gold = 0;
    let mut bond = 0;
    let mut item_list = HashMap::new();
    list.iter().for_each(|item| {
        let item_kind = item.get_kind().value;
        if (item_kind < 14 || item_kind > 16) && item.get_use_type().value != 34 {
            let weapon_level = item.m_weapon_level().value;
            loop {
                let item = ItemData::get(pool.random_item(RandomItemReplacementType::Reward, rare));
                let new_level = item.m_weapon_level().value;
                let price = item.get_price();
                let hash = item.hash();
                if item.get_kind() == ItemData_Kinds::gold() && ((current_gold + price) <= 2 * gold_total) {
                    current_gold += price;
                    break;
                }
                else if item.get_kind() == ItemData_Kinds::piece_of_bond(){
                    bond += price;
                    break;
                }
                else if ((weapon_level == 0 && new_level < 3) || (new_level == 0 || new_level <= weapon_level)) && price < 5000 && item.get_equip_condition().is_null() {
                    if let Some(count) = item_list.get_mut(&hash) { *count += 1; }
                    else { item_list.insert(hash, 1); }
                    break;
                }
            }
        }
        else {
            let hash = item.hash();
            if let Some(count) = item_list.get_mut(&hash) { *count += 1; }
            else { item_list.insert(hash, 1); }
        }
    });
    (item_list, current_gold, bond)
}

pub extern "C" fn muscle_exit(this: ProcInst, optional_method: OptionalMethod){ DVC::get().hero_appearance = None; }
pub extern "C" fn fishing_exit(this: ProcInst, optional_method: OptionalMethod) {
    DVC::get().hero_appearance = None;
    println!("FishingSequence Exit");
    let hub_sequence = HubSequence::get_instance();
    if !hub_sequence.is_null() {
        println!("Disable Fishing Group");
        hub_sequence.disable_fishing_group();
    }
}
pub extern "C" fn dragon_ride_exit(this: ProcInst, optional_method: OptionalMethod) {
    DVC::get().hero_appearance = None;
    unsafe { dragon_ride_exit_original(this, optional_method); }
}

#[skyline::from_offset(0x27ab200)]
fn item_data_get_name(this: ItemData, optional_method: OptionalMethod) -> Il2CppString;
#[skyline::from_offset(0x2ad4570)]
fn dragon_ride_exit_original(this: ProcInst, optional_method: OptionalMethod);
