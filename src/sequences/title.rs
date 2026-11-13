use engage::app::{ProcInst, Stream_2};
use outfit_core::UnitAssetMenuData;
use skyline::install_hooks;
use skyline::patching::Patch;
use crate::dvc::PersonPool;
use super::*;
use crate::utils::override_vtable;

pub fn title_sequence_event_jump(label: i32) {
    UnitAssetMenuData::get().is_loaded = false;
    UnitAssetMenuData::get().data.clear();
    if label == 0 {
        DVC::get().enabled = false;
        if !UnitAssetMenuData::get().init {
            outfit_core::install_outfit_plugin(true);
            Patch::in_text(0x01bb272c).nop().unwrap();
            install_hooks!(
                asset::asset_table_result_setup_hook,
                asset::emblem::asset_table_result_god_setup,
                asset::asset_table_setup_person,
                asset::color::combat_character_play_facial,
                dvc::mess::unit_get_name,
            );
            override_vtable("App", "GameUserData", "OnDeserialize", game_user_data_on_deserialize as _);
        }
    }
}

fn game_user_data_on_deserialize(this: GameUserData, stream: Stream_2, version: i32, _: OptionalMethod){
    unsafe { GameUserData::on_deserialize(this, stream, version) };
    outfit_core::deserialize_outfit_data(stream, version);
    PersonPool::setup_reliance_data();
}