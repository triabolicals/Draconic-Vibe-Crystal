use engage::combat::CharacterAppearance;
use engage::gamedata::assettable::AssetTableResult;
use engage::proc::desc::ProcDesc;
use engage::proc::{ProcInst, ProcVoidMethod};
use engage::util::try_get_instance;
use unity::il2cpp::object::Array;
use unity::prelude::OptionalMethod;
use crate::assets::gmap::GmapPlayerUnit;
use crate::procs::call_proc_original_method;
use crate::randomizer::emblem;

pub fn gmap_sequence_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[5] = ProcDesc::call(ProcVoidMethod::new(None, gmap_sequence_load_actor)); 
}
extern "C" fn gmap_sequence_load_actor(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    call_proc_original_method(proc, "LoadActor");
    emblem::emblem_gmap_spot_adjust();
    if let Some(gmap_unit) = try_get_instance::<GmapPlayerUnit>() {
        if gmap_unit.unit.person.parent.index > 1 {
            let result = AssetTableResult::get_from_unit(1, gmap_unit.unit, CharacterAppearance::get_constions(None));
            result.scale_stuff[16] = 4.8;
            gmap_unit.unit.actor.unit_model.load_async(result);
        }
    }
}