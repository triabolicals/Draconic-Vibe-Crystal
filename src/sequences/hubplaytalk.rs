use super::*;
use engage::{
    system::Action,
    app::{
        IBasicMenuItemMethods, IUnitMethods, IStructData_1Methods, IHubDisposDataMethods,
        hubaccess::*, hubaccessdata::*, hubplaytalk::*, hubplaytalkafter::*,
        goddata::*, godunit::*,
        GameMessage, GameSound, GodPool, Mess, PersonData, UnitUtil,
        basicmenu::*,
        assettable::prelude::IList_1Methods
    }
};
use crate::dvc::autolevel::try_autoscale_unit;
use crate::menus::UnitRecruitMenuItem;

pub fn hub_play_talk_descs_edit(proc: ProcInst) {
    if DVCVariables::get_sequence() == 5 {
        if let Some(talk) = proc.try_cast::<HubPlayTalk>() {
            let descs = talk.m_descs();
            descs.set(7, Proc::call_2(ProcVoidMethod::from_fn(talk.into(), hub_play_talk_exec_talk_after).unwrap()));
        }
    }
}
pub fn hub_play_talk_after_descs_edit(proc: ProcInst) {
    if let Some(hub_play_talk_after) = proc.try_cast::<HubPlayTalkAfter>() {
        proc.m_descs().set(5, Proc::call_2(ProcVoidMethod::from_fn(hub_play_talk_after, hub_play_talk_after_create_select).unwrap()))
    }
}
extern "C" fn hub_play_talk_after_create_select(this: HubPlayTalkAfter, _: OptionalMethod) {
    this.create_bind_select_menu();
    let access = this.get_access();
    if access.get_dispos_data().get_script_name().is_null() {
        if !this.get_child().is_null() {
            if let Some(menu) = this.get_child().try_cast::<HubPlayTalkAfter_SelectListMenu>(){
                unsafe {
                    let item = UnitRecruitMenuItem::instantiate().unwrap();
                    IBasicMenuItemMethods::ctor(item);
                    menu.m_full_menu_item_list().add(item.cast());
                }
                menu.set_m_show_row_num(menu.m_full_menu_item_list().get_count());
                menu.rebuild();
            }
        }
    }
}
extern "C" fn hub_play_talk_exec_talk_after(this: HubPlayTalk, _: OptionalMethod) {
    let access = this.get_access().m_access_data();
    if unity::field_get_value_at_offset::<bool>(access, 0x2e){
        if access.get_is_god() {
            let action = Action::new(this.into(), god_recruit_yes_method_info().into());
            menus::dialog::create_dialog_confirm(this, "Recruit this emblem?", action);
            return;
        }
        else if access.get_is_person() {
            let action = Action::new(this.into(), unit_recruit_yes_method_info().into());
            menus::dialog::create_dialog_confirm(this, "Recruit this character?", action);
            return;
        }
    }
    this.exec_talk_after();
}
#[unity::callback]
fn unit_recruit_yes(talk: HubPlayTalk, _: OptionalMethod) {
    let pid = talk.get_access().m_access_data().aid();
    let person = PersonData::get(pid);
    let unit = UnitUtil::join_unit_2(person);
    try_autoscale_unit(unit);
    dvc::unit::item::adjust_unit_items(unit);
    Mess::set_argument_2(0, unit.get_name());
    let message = Mess::get("MID_MSG_JOIN_UNIT");
    GameMessage::create_key_wait(talk, message);
    unity::field_set_value_at_offset::<bool>(talk.get_access().m_access_data(), 0x2e, false);
    GameSound::join_unit();
}

#[unity::callback]
fn god_recruit_yes(talk: HubPlayTalk, _: OptionalMethod) {
    let god_unit = GodPool::create(GodData::get(talk.get_access().m_access_data().aid()));
    Mess::set_argument_2(0, Mess::get(god_unit.m_data().get_mid()));
    let name = Mess::get("MPID_God_Prefix");
    Mess::set_argument_2(0, name);
    let message = Mess::get("MID_MSG_JOIN_UNIT");
    unity::field_set_value_at_offset::<bool>(talk.get_access().m_access_data(), 0x2e, false);
    GameMessage::create_key_wait(talk, message);
    GameSound::join_unit();
}