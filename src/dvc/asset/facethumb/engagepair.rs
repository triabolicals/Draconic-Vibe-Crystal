use engage::app::{Proc, ProcInst, ProcVoidMethod, TelopManager_ProcBondEngagePair};
use engage::unity_engine::GameObject;
use unity::OptionalMethod;
use super::*;

pub fn telop_engage_pair_desc_edit(this: ProcInst) -> bool {
    let descs = this.m_descs();
    if let Some(proc) = this.try_cast() {
        descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_engage_pair_load_face).unwrap()));
        descs.set(3, Proc::call_2(ProcVoidMethod::from_fn(proc, telop_pair_play).unwrap()));
        descs.set(5, Proc::call_2(ProcVoidMethod::from_fn(proc, proc_bond_engage_pair_release_face).unwrap()));
        true
    }
    else { false }
}
extern "C" fn proc_bond_engage_pair_load_face(this: TelopManager_ProcBondEngagePair, _optional_method: OptionalMethod) {
    if let Some(god) = to_opt(this.m_main_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
    if let Some(god) = to_opt(this.m_sub_god()) { ResourceManager_2::load_global_async::<Sprite>(get_god_face(god), Action_1::<Sprite>::null()); }
}
extern "C" fn proc_bond_engage_pair_release_face(this: TelopManager_ProcBondEngagePair, _optional_method: OptionalMethod) {
    if let Some(god) = to_opt(this.m_main_god()) { ResourceManager_2::release_global(get_god_face(god)); }
    if let Some(god) = to_opt(this.m_sub_god()) { ResourceManager_2::release_global(get_god_face(god)); }
}
extern "C" fn telop_pair_play(this: TelopManager_ProcBondEngagePair, _info: OptionalMethod) {
    let resource_object = TelopManager::create_bind(this, "Telop/BondEngage/Prefabs/BondEngagePair");
    let call_back_1 = ResourceObject_Callback::new(this.into(), telop_pair_play_0_method_info().into());
    resource_object.set_start_callback(call_back_1);
    let call_back_2 = ResourceObject_Callback::new(this.into(), telop_pair_play_1_method_info().into());
    resource_object.set_tick_callback(call_back_2);
    resource_object.set_endless(true);
}

#[unity::callback]
fn telop_pair_play_1(this: TelopManager_ProcBondEngagePair, go: GameObject, _: OptionalMethod) {
    TelopManager_ProcBondEngageBase::execute_audio_playback_sequence(this.into(), this.m_main_god(), this.m_sub_god(), go);
}

#[unity::callback]
fn telop_pair_play_0(this: TelopManager_ProcBondEngagePair, go: GameObject, _: OptionalMethod) {
    for i in 0..2 {
        if let Some(god) = to_opt(unity::field_get_value_at_offset::<GodData>(this, 0x88 + i*8)) {
            let path = get_god_face(god);
            let obj_name = format!("FaceGrp/Face{}/Image{}", i, i);
            let sprite = ResourceManager_2::get_asset::<Sprite>(path);
            if !sprite.is_null() { TelopManager::try_set_image_sprite(go, obj_name.as_str(), sprite); }
            else { TelopManager::try_set_image_sprite(go, obj_name.as_str(), FaceThumbnail::get_3(god)); }
        }
    }

}


#[skyline::from_offset(0x2201970)] fn play_pair_0_original(this: TelopManager_ProcBondEngagePair, go: GameObject, method_info: OptionalMethod);