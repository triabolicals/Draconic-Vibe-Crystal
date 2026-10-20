use engage::app::ISingletonProcInst_1Methods;
use super::*;
use engage::{
    app::{
        hubaccess::*, hubsequence::*, hubaccessdata::*,
        hubaccessmanager::*, hubdisposdata::*,
        HubUtil,
        UnitPool,
        assettable::prelude::IList_1Methods,
        cooking_menu::{UnitSelectMenu_2, UnitSelectMenu_UnitSelectMenuItem, UnitSelectMenu_UnitSelectMenuItem_SelectEventHandler},
        IHubLocatorGroupMethods,
    },
    List_1Ext,
    system::Object,
};
use crate::{dvc::talk::window::WindowReplacementType, utils::{il2str, to_opt}, dvc::DVC};

pub fn hub_method_edits() {
    if let Some(class) = Class::try_lookup("App.CookingMenu", "UnitSelectMenu.<>c__DisplayClass8_0").ok()
        .map(|v| v.raw_mut())
    {
        if let Some(method) = class.get_method_from_name("<CreateBind>b__0", 1) {
            method.method_ptr = cooking_unit_select_menu as _;
        }
    }
}

pub fn hub_item_randomization() {
    let hub_sequence = HubSequence::get_instance();
    if hub_sequence.is_null() { return; }
    let random = DVCFlags::RandomExplorationItems.get_value();
    let item_pool = &GameData::get().item_pool;
    HubSequence::get_instance().m_hub_locator_group().get_access_list().iter()
        .filter(|a| to_opt(a.m_access_data()).and_then(|a| il2str(a.aid())).is_some_and(|v| v.starts_with("IID_")))
        .for_each(|a|{
            let data = a.m_access_data();
            let mut current = unity::field_get_value_at_offset::<i32>(data, 0x3c);
            if current == 0 {
                let item = ItemData::get(data.aid());
                if !item.is_null() {
                    current = item.hash();
                    unity::field_set_value_at_offset::<i32>(data, 0x3c, current);
                }
            }
            if let Some(item) = to_opt(ItemData::try_get_from_hash(current)) {
                if random {
                    if item.get_kind().value < 14 {
                        let new_iid = item_pool.random_item(RandomItemReplacementType::Exploration, false);
                        a.set_aid(new_iid);
                        // println!("New Item Replacement: {} -> {} [{}]", engage::app::Mess::get_game_data_name(item.get_iid()), engage::app::Mess::get_game_data_name(new_iid), ItemData::get(new_iid).get_kind().value);
                    }
                }
                else { a.set_aid(item.get_iid()); }
            }
        });
    println!("Hub Item Randomization: {} [Completed]", random);
}


#[skyline::hook(offset=0x216a830)]
pub fn hub_access_data_confirm_content(this: HubAccessData, manager: HubAccessManager, optional_method: OptionalMethod) {
    let data = this.get_dispos_data();
    let sequence =  DVCVariables::get_sequence();
    let kind =  data.get_content_type().value;
    call_original!(this, manager, optional_method);
    if unity::field_get_value_at_offset::<bool>(this, 0x2d) { return; }
    let dvc = DVC::get();
    let chapter_index = DVCVariables::get_chapter_index();
    if chapter_index > 32 && chapter_index < 45 && this.get_is_god() && sequence == 5 {
        if let Some(god) = EmblemPool::get_replacement_god(GodData::get(this.aid()), false).filter(|v| EmblemPool::is_custom(*v))
        {
            this.set_aid(god.get_gid());
            this.set_is_god(true);
            return;
        }
    }
    if kind == 3 {
        if sequence == 5 {
            if dvc.god_recruit != 0 {
                let god = GodData::try_get_from_hash(dvc.god_recruit);
                this.set_aid(god.get_gid());
                call_original!(this, manager, optional_method);
                this.set_is_god(true);
                this.set_is_unit(false);
                this.set_is_reliance(false);
                this.set_is_person(false);
                unity::field_set_value_at_offset::<bool>(this, 0x2e, true);
                dvc.god_recruit = 0;
                return;
            }
            else if !dvc.kizuna_recruit.is_empty() {
                if let Some(person) = dvc.kizuna_recruit.get_remove(Random_2::get_hub())
                    .map(|v| PersonData::try_get_from_hash(v))
                {
                    this.set_aid(person.get_pid());
                    call_original!(this, manager, optional_method);
                    this.set_is_person(true);
                    this.set_is_unit(false);
                    this.set_is_reliance(false);
                    this.set_is_god(false);
                    unity::field_set_value_at_offset::<bool>(this, 0x2e, true);
                    return;
                }
            }
        }
        if let Some(person) = il2str(this.try_get_pid()).and_then(|v| to_opt(PersonData::get(v.into()))) {
            if PersonPool::is_playable(person) {
                let units: Vec<_> =
                    UnitPool::s_units().iter().filter(|p| !p.get_person().is_null() && !p.m_force().is_null())
                        .filter(|&p| !dvc.kizuna_person_hashes.contains(&p.get_person().hash()) && p.get_force_type().value < 4 && !p.is_hero())
                        .map(|p| (p.get_pid().to_rust_string(), p.get_person().hash()))
                        .collect();
                if let Some((pid, hash)) = units.get_random_element(Random_2::get_hub()) {
                    this.set_aid(pid.as_str().into());
                    unity::field_set_value_at_offset::<bool>(this, 0x2d, true);
                    dvc.kizuna_person_hashes.push(*hash);
                } else {
                    this.set_is_person(false);
                    this.set_is_unit(false);
                    this.set_is_reliance(false);
                    this.set_is_god(false);
                    data.set_content_type(HubDisposData_Type::material());
                    this.set_aid("".into());
                    this.set_item_count(0);
                    this.m_messages().clear();
                    unity::field_set_value_at_offset::<bool>(this, 0x2d, true);
                    return;
                }
            }
        }
        call_original!(this, manager, optional_method);
        return;
    }
    if sequence == 5 {
        if kind == 2 {
            call_original!(this, manager, optional_method);
            if let Some(replacement) = il2str(this.try_get_pid())
                .and_then(|p| to_opt(PersonData::get(p.into())))
                .and_then(|p| PersonPool::get_replacement_2(p, false))
            {
                this.set_aid(replacement.get_pid());
                dvc.kizuna_person_hashes.push(replacement.hash());
                if let Some(idx) = PersonPool::get_replacement_index_3(replacement, true).filter(|&v| v < 41) {
                    this.m_messages().iter().for_each(|m| { mess::hub::edit_label_pid(m.to_rust_string().as_str(), WindowReplacementType::HubAccessSwap, idx); });
                    mess::hub::edit_label_pid(this.get_mid().to_rust_string().as_str(), WindowReplacementType::HubAccessSwap, idx);
                }
            }
            unity::field_set_value_at_offset::<bool>(this, 0x2d, true);
        }
        else { call_original!(this, manager, optional_method); }
    }
    else if sequence == 4 {
        if il2str(data.get_aid()).is_some_and(|c| c == "GID_シグルド") && data.get_content_type() == HubDisposData_Type::priority_character() {
            if let Some(replacement) = EmblemPool::get_replacement_god_3(1, false).map(|g| g.get_gid()) {
                unity::field_set_value_at_offset::<bool>(this, 0x2d, true);
                this.set_aid(replacement);
            }
        }
        call_original!(this, manager, optional_method);
    }
}
pub fn cooking_unit_select_menu(this: IlInstance, unit: Unit, _: OptionalMethod){
    if unit.is_hero() { return; }
    if PersonPool::get_replacement_index_2(unit.get_pid(), true).is_some_and(|v| v == 0) { return; }
    if HubUtil::get_current_cooking_pid() == unit.get_pid() { return; }
    let select = UnitSelectMenu_UnitSelectMenuItem_SelectEventHandler::new(Object::null(), UnitSelectMenu_2::on_select_menu_item_method_info().into());
    let item = UnitSelectMenu_UnitSelectMenuItem::new(unit, select);
    let list = unity::field_get_value_at_offset::<List_1::<UnitSelectMenu_UnitSelectMenuItem>>(this, 0x10);
    list.add(item);
}
#[skyline::from_offset(0x1b55a00)]
fn cooking_menu_unit(this: u64, unit: Unit, optional_method: OptionalMethod);