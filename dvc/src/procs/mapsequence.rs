use engage::{
    eventsequence::EventSequence,
    force::{Force, ForceType},
    gameuserdata::GameUserData,
    proc::{ProcInst, ProcVoidMethod, desc::ProcDesc},
    sequence::mapsequence::MapSequence,
};
use crate::{
    config::DVCFlags, procs, randomizer::{
        self,
        map::shuffle::shuffle_deployment,
        person::unit
    }, DVCVariables,
    message::{TextSwapper, MESSAGE_SWAPPER},
    deployment::fulldeploy,
    script::adjust_person_map_inspectors,
    utils::remove_equip_emblems
};
use std::sync::RwLock;
use unity::{il2cpp::object::Array, prelude::OptionalMethod};

pub fn map_sequence_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[4] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_setup_chapter));
    descs[19] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_dispos_event));
    descs[21] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_dispos_unit));
    descs[45] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_map_opening));
    descs[63] = ProcDesc::call(ProcVoidMethod::new(None, map_start));
    descs[0xCF] = ProcDesc::call(ProcVoidMethod::new(None, procs::hubsequence::hub_sequence_unload_script));
}

extern "C" fn map_sequence_setup_chapter(map_sequence: &mut MapSequence, _optional_method: OptionalMethod) {
    map_sequence.setup_chapter();
    let swap = MESSAGE_SWAPPER.get_or_init(||RwLock::new(TextSwapper::init()));
    if let Ok(mut lock) = swap.try_write() { lock.get_chapter_talk(); }
}

pub extern "C" fn map_sequence_dispos_event(this: &mut MapSequence, _method_info: OptionalMethod) {
    this.dispos_event();
    if DVCVariables::UnitDeployment.get_value() == 4 && !DVCFlags::RandomDeploySpot.get_value() {
        fulldeploy::load_extra_deployment_slots();
    }
    if DVCFlags::RandomDeploySpot.get_value() { shuffle_deployment(); }
    randomizer::terrain::fow::map_start_fow();
    randomizer::map::dispos::change_map_dispos();
    if !this.is_resume {
        randomizer::terrain::randomized_emblem_power_spots();
        let emblem = DVCVariables::EmblemDeployment.get_value();
        if emblem == 1 || emblem == 2 { remove_equip_emblems(); }
    }
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit| { unit::enemy_check_soar(unit); });
}

extern "C" fn map_sequence_dispos_unit(proc: &mut MapSequence, _method_info: OptionalMethod) {
    proc.dispos_unit();
    if !proc.is_resume && GameUserData::get_sequence() != 3 { crate::deployment::deployment_modes(); }
    if DVCVariables::is_main_chapter_complete(3) { randomizer::map::tilabolical(); }
}

extern "C" fn map_sequence_map_opening(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    EventSequence::map_opening(proc);
    adjust_person_map_inspectors();
}
extern "C" fn map_start(_proc: &mut ProcInst, _optional_method: OptionalMethod) {
    crate::script::post_sortie_script_adjustment();
}