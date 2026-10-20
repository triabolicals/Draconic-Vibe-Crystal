use engage::app::{ResourceManager_2, TelopManager_ProcBondEngageQuartette};
use engage::system::Action_1;
use engage::unity_engine::Sprite;
use unity::OptionalMethod;
use crate::dvc::asset::facethumb::get_god_face;
use crate::utils::to_opt;
use super::*;

pub fn telop_engage_quartette_edit(this: ProcInst) {
    let descs = this.m_descs();
    if let Some(proc) = this.try_cast(){
        descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_engage_quart_load_face).unwrap()));
        descs.set(3, Proc::call_2(ProcVoidMethod::from_fn(proc, telop_quart_play).unwrap()));
        descs.set(5, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_engage_quart_release_face).unwrap()));
    }
}

extern "C" fn proc_bond_engage_quart_load_face(this: TelopManager_ProcBondEngageQuartette, _optional_method: OptionalMethod) {
    if let Some(god) = to_opt(this.m_1st_main_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
    if let Some(god) = to_opt(this.m_2nd_main_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
    if let Some(god) = to_opt(this.m_3rd_main_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
    if let Some(god) = to_opt(this.m_sub_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
}
extern "C" fn proc_bond_engage_quart_release_face(this: TelopManager_ProcBondEngageQuartette, _optional_method: OptionalMethod) {
    if let Some(god) = to_opt(this.m_1st_main_god()) { ResourceManager_2::release_global(get_god_face(god)); }
    if let Some(god) = to_opt(this.m_2nd_main_god()) { ResourceManager_2::release_global(get_god_face(god)); }
    if let Some(god) = to_opt(this.m_3rd_main_god()) { ResourceManager_2::release_global(get_god_face(god)); }
    if let Some(god) = to_opt(this.m_sub_god()) { ResourceManager_2::release_global(get_god_face(god)); }
}
extern "C" fn telop_quart_play(this: TelopManager_ProcBondEngageQuartette, _info: OptionalMethod) {
    let resource_object = TelopManager::create_bind(this, "Telop/BondEngage/Prefabs/BondEngageQuartette");
    let call_back_1 = ResourceObject_Callback::new(this.into(), telop_quart_play_0_method_info().into());
    resource_object.set_start_callback(call_back_1);
    let call_back_2 = ResourceObject_Callback::new(this.into(), telop_quart_play_1_method_info().into());
    resource_object.set_tick_callback(call_back_2);
    resource_object.set_endless(true);
}

#[unity::callback]
fn telop_quart_play_0(this: TelopManager_ProcBondEngageQuartette, go: GameObject, _: OptionalMethod) {
    unsafe { play_quart_0_original(this, go, None) };
    for x in 0..4 {
        if let Some(god) = to_opt(unity::field_get_value_at_offset::<GodData>(this, 0x88 + x*8)) {
            let path = get_god_face(god);
            let obj_name = format!("FaceGrp/Face{}/Image{}", x, x);
            let sprite = ResourceManager_2::get_asset::<Sprite>(path);
            if !sprite.is_null() { TelopManager::try_set_image_sprite(go, obj_name.as_str(), sprite); }
            else { TelopManager::try_set_image_sprite(go, obj_name.as_str(), FaceThumbnail::get_3(god)); }
        }
    }
}
#[unity::callback]
fn telop_quart_play_1(this: TelopManager_ProcBondEngageQuartette, go: GameObject, _: OptionalMethod) {
    TelopManager_ProcBondEngageBase::execute_audio_playback_sequence(this.into(), this.m_1st_main_god(), this.m_sub_god(), go);
}
#[skyline::from_offset(0x22023a0)] fn play_quart_0_original(this: TelopManager_ProcBondEngageQuartette, go: GameObject, method_info: OptionalMethod);