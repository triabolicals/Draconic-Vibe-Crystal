use engage::{
    combat::CharacterAppearance, gamedata::assettable::AssetTableResult,
    unit::{UnitPool, Unit}, gameuserdata::GameUserData, sequence::gmap_sequence::GmapSequence,
    util::try_get_instance,
};
use unity::prelude::*;
use crate::DVCVariables;

#[unity::class("App", "GmapPlayerUnit")]
pub struct GmapPlayerUnit {
    pub parent: [u8; 0x10],
    pub unit: &'static mut Unit,
}
impl GmapPlayerUnit {
    pub fn load_actor() {
        if GameUserData::get_sequence() == 6 {
            if let Some(gmap_unit) = try_get_instance::<GmapPlayerUnit>() {
                if gmap_unit.unit.person.parent.index > 1 {
                    let result = AssetTableResult::get_from_unit(1, gmap_unit.unit, CharacterAppearance::get_constions(None));
                    result.scale_stuff[16] = 4.8;
                    gmap_unit.unit.actor.unit_model.load_async(result);
                }
            }
        }
    }
    pub fn set_unit_from_unit_pool(&self) { unsafe { gmap_player_unit_get_from_unit_pool(self, None) } }
    pub fn on_create(&mut self, _optional_method: OptionalMethod) {
        self.set_unit_from_unit_pool();
        if let Some(hero) = UnitPool::get_from_pid(DVCVariables::get_dvc_person(0, false), false) {
            self.unit.copy_from(hero);
            self.unit.clear_god_unit_from_copy();
            self.unit.put_off_all_item();
        }
    }
    pub fn apply_changes() {}
}

#[unity::from_offset("App", "GmapPlayerUnit", "LoadActor")]
fn gmap_player_unit_load_actor(this: &GmapPlayerUnit, method_info: OptionalMethod);
#[unity::from_offset("App", "GmapSequence", "LoadActor")]
fn gmap_sequence_load_actor(this: &GmapSequence, method_info: OptionalMethod);

#[unity::from_offset("App", "GmapPlayerUnit", "SetUnitFromUnitPool")]
fn gmap_player_unit_get_from_unit_pool(this: &GmapPlayerUnit, method_info: OptionalMethod);