use engage::{
    app::{
        procinst::*, unit::*,
        IWellSequenceMethods, Random_2,
        CommonRewardSequence,
        IStructData_1Methods,
        ItemData,
        Proc, ProcBoolMethod, ProcVoidMethod,
        UnitUtil, WellSequence,
        GameMessage, GameSound, Mess, UnitPool
    },
    system::collections::generic::IList_1Methods,
    ProcBoolMethodExt,
    ProcVoidMethodExt,
    prelude::{List_1},
};
use unity::{Array, Cast, Il2CppString, OptionalMethod};
use crate::{
    DVCFlags,
    DVCVariables,
    enums::PIDS,
    gamedata::{GameData, PersonPool},
    utils::to_opt
};

pub fn create_continuous_mode_bind(parent_proc: impl Into<ProcInst>) {
    let exp = crate::dvc::autolevel::get_exp_list(true);
    let item_list = get_items();
    let reward = CommonRewardSequence::new(exp, item_list, 0 , false, false);
    let proc = reward.into();
    let descs = [
        Proc::call_2(ProcVoidMethod::new(proc, CommonRewardSequence::load_res_method_info().into())),
        Proc::wait_while_true_2(ProcBoolMethod::new(proc, CommonRewardSequence::is_loading_res_method_info().into())),
        Proc::label(0), // Exp
        Proc::jump_true_3(ProcBoolMethod::from_fn(reward, check_cannot_gain_exp).unwrap(), 2),
        Proc::call_2(ProcVoidMethod::new(proc, CommonRewardSequence::open_reward_exp_method_info().into())),
        Proc::label(1), // Level Ups
        Proc::call_2(ProcVoidMethod::new(proc, CommonRewardSequence::check_level_up_method_info().into())),
        Proc::label(2), // Item
        Proc::call_2(ProcVoidMethod::new(proc, CommonRewardSequence::open_reward_item_method_info().into())),
        Proc::call_2(ProcVoidMethod::new(proc, CommonRewardSequence::close_method_info().into())),
        Proc::label(3), // Recruit DLC
        Proc::call_2(ProcVoidMethod::from_fn(reward, dlc_unit_recruit).unwrap()),
        Proc::jump(3),
        Proc::label(4),
        Proc::end(),
    ];
    let descs = Array::from_slice(&descs).unwrap();
    reward.create_bind(parent_proc, descs, Il2CppString::null());
}

fn get_items() -> List_1<ItemData> {
    let list: List_1::<ItemData>;
    let chapter_complete = DVCVariables::chapter_number_complete(false);
    if DVCFlags::ContinuousModeItems.get_value() && chapter_complete > 5 && GameData::get_item_pool().has_reward {
        let well_sequence = WellSequence::new();
        let level =
            if chapter_complete < 10 { 1 }
            else if chapter_complete < 14 { 2 }
            else if chapter_complete < 20 { 3 }
            else if chapter_complete < 23 { 4 }
            else { 5 };
        list = well_sequence.calc_item_exchange(level, Random_2::get_system());
    }
    else {
        list = List_1::<ItemData>::new();
        list.add(ItemData::get("IID_絆のかけら500".into()));
    }
    if chapter_complete == 6 {
        list.add(ItemData::get("IID_トライゾン".into()));
        list.add(ItemData::get("IID_ルヴァンシュ".into()));
        if DVCVariables::is_random_map() && DVCVariables::has_dlc() { list.add(ItemData::get("IID_マスタープルフ".into())); }
    }
    if chapter_complete >= 4 {
        let iron = ItemData::get("IID_てつの晶石".into());
        for _ in 0..5 { list.add(iron); }
    }
    if chapter_complete >= 6 {
        let iron = ItemData::get("IID_はがねの晶石".into());
        for _ in 0..chapter_complete/5 { list.add(iron); }
    }
    if chapter_complete >= 12 {
        let iron = ItemData::get("IID_ぎんの晶石".into());
        let amount = 1 + (chapter_complete - 12) / 4;
        for _ in 0..amount { list.add(iron); }
    }
    list
}

extern "C" fn check_cannot_gain_exp(_: CommonRewardSequence, _: OptionalMethod) -> bool { !DVCFlags::ContinuousExp.get_value() }

extern "C" fn dlc_unit_recruit(proc: CommonRewardSequence, _: OptionalMethod) {
    if DVCVariables::is_main_chapter_complete(17) && DVCVariables::has_dlc(){
        let mut recruits = vec![];
        for i in [38, 39, 40, 36, 37]{
            if let Some(replacement) = PersonPool::get_replacement(i, false, false).filter(|p| UnitPool::get_from_person(*p, false).is_null()){
                if let Some(unit) = to_opt(UnitUtil::join_unit_2(replacement)).filter(|u| u.get_force_type().value == 3) { recruits.push(unit); }
            }
            else if let Some(unit) = to_opt(UnitUtil::join_unit(PIDS[i as usize])).filter(|u| u.get_force_type().value == 3) {
                recruits.push(unit);
            }
            if recruits.len() == 3 { break; }
        }
        if !recruits.is_empty() { recruit_unit_message(proc, &recruits); }
        else { proc.jump(4); }
    }
    else { proc.jump(4); }
}
fn recruit_unit_message(proc: impl Into<ProcInst>, units: &Vec<Unit>) {
    let message = ["MID_MSG_JOIN_UNIT", "MID_MSG_JOIN_UNIT2", "MID_MSG_JOIN_UNIT3"];
    let n = units.len();
    let mut count = 0;
    for i in 0..n {
        if let Some(unit) = units.get(i) {
            Mess::set_argument_2(count as i32, unit.get_name());
            count += 1;
        }
    }
    if count > 1 {
        let message = Mess::get(message[count-1]);
        GameMessage::create_key_wait(proc, message);
        GameSound::join_unit();
    }
}