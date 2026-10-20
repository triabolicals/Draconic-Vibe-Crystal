use engage::{
    app::{FaceThumbnail, ITelopInstanceMethods, Proc, ProcInst, ProcVoidMethod, ResourceManager_2, ResourceObject_Callback, RingListSkillMenu, TelopInstance, TelopManager, TelopManager_ProcBondLevelUp, Ut},
    system::Action_1,
    unity_engine::{GameObject, Sprite}
};
use unity::OptionalMethod;
use crate::dvc::asset::facethumb::{get_bond_face, get_god_face};
use crate::utils::to_opt;
use super::*;

pub fn telop_bond_level_desc_edit(this: ProcInst) -> bool {
    let descs = this.m_descs();
    if let Some(proc) = this.try_cast() {
        descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_level_up_load_face).unwrap()));
        descs.set(3, Proc::call_2(ProcVoidMethod::from_fn(proc, telop_bond_level_up_play).unwrap()));
        descs.set(5, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_level_up_release_face).unwrap()));
        true
    }
    else { false }
}

extern "C" fn proc_bond_level_up_load_face(this: TelopManager_ProcBondLevelUp, _optional_method: OptionalMethod) {
    if let Some(unit) = to_opt(this.m_unit()) { ResourceManager_2::load_global_async::<Sprite>(get_bond_face(unit), Action_1::<Sprite>::null()); }
    if let Some(god) = to_opt(this.m_god_data()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
}

extern "C" fn proc_bond_level_up_release_face(this: TelopManager_ProcBondLevelUp, _optional_method: OptionalMethod) {
    if let Some(unit) = to_opt(this.m_unit()) { ResourceManager_2::release_global(get_bond_face(unit)); }
    if let Some(god) = to_opt(this.m_god_data()) { ResourceManager_2::release_global(get_god_face(god)); }
}

extern "C" fn telop_bond_level_up_play(this: TelopManager_ProcBondLevelUp, _info: OptionalMethod) {
    let god = this.m_god();
    let mut god_data = god.m_data();
    if !god_data.is_null() && !god.get_parent().is_null() {
        let parent = god.get_parent();
        if parent.is_engaging_2() { god_data = god_data.get_main_data(); }
    }
    this.set_m_menu_item_list(RingListSkillMenu::create_god_growth_menu_item(god_data, RingListSkillMenu::null(), this.m_level(), this.m_next_level(), 0,false));
    let resource_object = TelopManager::create_bind(this, "Telop/LevelUp/Prefabs/BondLevelUpRoot");
    let call_back_1 = ResourceObject_Callback::new(this.into(), telop_level_up_play_0_method_info().into());
    resource_object.set_start_callback(call_back_1);
    let call_back_2 = ResourceObject_Callback::new(this.into(), telop_level_up_play_1_method_info().into());
    resource_object.set_tick_callback(call_back_2);
    resource_object.set_endless(true);
}


#[unity::callback]
fn telop_level_up_play_0(proc: TelopManager_ProcBondLevelUp, go: GameObject, _: OptionalMethod) {
    if let Some(root) = to_opt(Ut::find_child_component_2::<TelopInstance>(go, "BondLevelUp_Chara")) { proc.set_m_root_telop(root); }
    if let Some(level_job) = to_opt(Ut::find_child_component_2::<TelopInstance>(go, "BondLevelUp_Text")) { proc.set_m_level_telop(level_job); }
    if let Some(go_skill) = to_opt(Ut::find_child_game_object_2(go, "BondLevelUp_Skill")) { proc.set_m_skill_obj(go_skill); }
    if let Some(go_skill) = to_opt(Ut::find_child_game_object_2(go, "Effect")) { proc.set_m_effect_obj(go_skill); }
    if !proc.m_root_telop().is_null() { proc.m_root_telop().set_key_lock(); }
    engage::app::Mess::set_argument_2(0, proc.m_unit().get_name());
    engage::app::Mess::set_argument_2(1, proc.m_god().get_name());
    let mess = engage::app::Mess::get("MID_SYS_BondCombi");
    TelopManager::try_set_text(go, "BondLevelUp_Text/Text/SubTitleRoot/SubTitle", mess);
    TelopManager::try_set_text_3(go, "BondLevelUp_Text/Text/Lv/LvBefore/LvValue", proc.m_level());
    TelopManager::try_set_text_3(go, "BondLevelUp_Text/Text/Lv/LvAfter/LvValue", proc.m_next_level());
    let unit_path = get_bond_face(proc.m_unit());
    let god_path = get_god_face(proc.m_god().m_data());
    let sola = PersonData::get("PID_ソラ".into());
    let unit_sprite = ResourceManager_2::get_asset::<Sprite>(unit_path);
    if !unit_sprite.is_null() { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/Unit/Face", unit_sprite); }
    else {
        let thumbnail = FaceThumbnail::get_2(proc.m_unit().get_person());
        if !thumbnail.is_null() { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/Unit/Face", thumbnail); }
        else { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/Unit/Face", FaceThumbnail::get_2(sola)); }
    }
    let god_sprite = ResourceManager_2::get_asset::<Sprite>(god_path);
    if !god_sprite.is_null() { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/God/Face", god_sprite); }
    else {
        let thumbnail = FaceThumbnail::get_3(proc.m_god().m_data());
        if !thumbnail.is_null() { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/God/Face", thumbnail); }
        else { TelopManager::try_set_image_sprite(go, "BondLevelUp_Chara/God/Face", FaceThumbnail::get_2(sola)); }
    }
}
#[unity::callback]
fn telop_level_up_play_1(proc: TelopManager_ProcBondLevelUp, go: GameObject, _: OptionalMethod) { unsafe { play_telop_1_original(proc, go, None) }; }

#[skyline::from_offset(0x2202e70)] fn play_telop_0_original(this: TelopManager_ProcBondLevelUp, go: GameObject, method_info: OptionalMethod);
#[skyline::from_offset(0x2203250)] fn play_telop_1_original(this: TelopManager_ProcBondLevelUp, go: GameObject, method_info: OptionalMethod);