use std::sync::RwLock;
use engage::eventsequence::EventSequence;
use engage::force::{Force, ForceType};
use engage::gamedata::dispos::DisposData;
use engage::gamedata::GamedataArray;
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::god::GodPool;
use engage::map::situation::MapSituation;
use engage::proc::desc::ProcDesc;
use engage::proc::{ProcInst, ProcVoidMethod};
use engage::sequence::mapsequence::MapSequence;
use engage::util::{get_instance};
use unity::il2cpp::object::Array;
use unity::prelude::OptionalMethod;
use crate::deployment::{fulldeploy, get_emblem_paralogue_level};
use crate::{procs, randomizer, DVCVariables, DeploymentConfig};
use crate::config::DVCFlags;
use crate::message::{TextSwapper, MESSAGE_SWAPPER};
use crate::randomizer::data::GameData;
use crate::randomizer::map::shuffle::shuffle_deployment;
use crate::script::adjust_person_map_inspectors;

pub fn map_sequence_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[4] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_setup_chapter));
    descs[19] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_dispos_event));
    descs[21] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_dispos_unit));
    descs[45] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_map_opening));
    descs[0xCF] = ProcDesc::call(ProcVoidMethod::new(None, procs::hubsequence::hub_sequence_unload_script));
    if !GameVariableManager::get_bool("G_Cleared_M001") && DeploymentConfig::get().debug {
        GameData::get_playable_god_list().iter().for_each(|g|{
            if let Some(g_unit) = GodPool::create(g) { g_unit.set_escape(false); }
        });
    }
}
/*
pub fn map_sequence_mind_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[61] = ProcDesc::call(ProcVoidMethod::new(None, map_sequence_mind_done_action_visit));
}
 */

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
    randomizer::terrain::randomized_emblem_power_spots();
    if DVCVariables::EmblemRecruitment.get_value() != 0 { get_emblem_paralogue_level(); }
    randomizer::terrain::fow::map_start_fow();
    randomizer::map::dispos::change_map_dispos();
    
    if !this.is_resume {
        let count = DisposData::try_get_mut("Player").map(|v| v.len()).unwrap_or(10) as i32;
        let mut levels = vec![];
        let vander_unit = DVCVariables::get_dvc_person_data(1, false).map(|v| v.parent.hash).unwrap_or(0);
        if let Some((player, absent)) = Force::get(ForceType::Player).zip(Force::get(ForceType::Absent)) {
            player.iter().chain(absent.iter()).for_each(|unit| {
                if unit.person.parent.hash != vander_unit { levels.push(unit.level as i32 + unit.internal_level as i32); }
            });
            let mut total = 0;
            let mut current_count = 0;
            while let Some(value) = levels.iter().max(){
                total += value;
                if let Some(pos) = levels.iter().position(|x| x == value) { levels.swap_remove(pos); }
                if current_count == count { break; }
                else { current_count += 1; }
            }
            if current_count == 0 { return; }
            let avg_level = (total / current_count) + GameUserData::get_difficulty(false);
            get_instance::<MapSituation>().average_level = avg_level;
        }
    }
}

extern "C" fn map_sequence_dispos_unit(proc: &mut MapSequence, _method_info: OptionalMethod) {
    proc.dispos_unit();
    if !proc.is_resume && GameUserData::get_sequence() != 3 { crate::deployment::deployment_modes(); }
}

extern "C" fn map_sequence_map_opening(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    EventSequence::map_opening(proc);
    adjust_person_map_inspectors();
    // if DeploymentConfig::get().debug { GameVariableManager::set_bool("勝利", true); }
}