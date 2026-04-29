use engage::{
    combat::CharacterAppearance, god::GodPool,
    gamedata::{ChapterData, Gamedata, assettable::AssetTableResult},
    gmapspotmanager::*,gamevariable::GameVariableManager,
    menu::{BasicMenuItem, BasicMenuItemAttribute},
    proc::{desc::ProcDesc, ProcInst, ProcVoidMethod},
    sequence::gmap_sequence::{GmapSequence, GmapSpotState},
    unityengine::UnityRenderer,
    util::try_get_instance,
};
use unity::{prelude::OptionalMethod, il2cpp::object::Array};
use crate::{
    assets::gmap::GmapPlayerUnit, DVCVariables,
    enums::EMBLEM_PARA,
    procs::call_proc_original_method,
    randomizer::map::dispos::PARALOGUE_LEVELS,
};
pub fn gmap_sequence_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[5] = ProcDesc::call(ProcVoidMethod::new(None, gmap_sequence_load_actor));
    if DVCVariables::EmblemRecruitment.get_value() > 0 || DVCVariables::Continuous.get_value() > 0 {
        descs[11] = ProcDesc::call(ProcVoidMethod::new(None, attach_update_gmap_spots));
    }
}
fn emblem_is_available(idx: i32) -> bool {
    DVCVariables::get_god_from_index(idx, false)
        .and_then(|data| GodPool::try_get(data, false))
        .map(|g_unit| !g_unit.is_escaping && !g_unit.darkness)
        .unwrap_or(false)
}
pub fn check_gmap_spots(emblem_index: i32) {
    match emblem_index {
        1 => { if is_spot_hidden(13) { set_spot(13, 2); } }
        2|11 => { if is_spot_hidden(14) { set_spot(14, 2); } }
        3 => { if is_spot_hidden(20) { set_spot(20, 2); } }
        5 => {
            if is_spot_hidden(14) { set_spot(14, 2); }
            if is_spot_hidden(16) { set_spot(16, 2); }
        }
        _ => { return; }
    }
}
fn emblem_paralogue_chapter_check(idx: i32) -> bool {
    let s = if idx == 6 { 11 } else { 11 };
    GameVariableManager::get_bool(format!("G_Cleared_M0{}", s))
}
fn is_spot_hidden(chapter: i32) -> bool {
    if let Some(spot) = GmapSpotManager::find_spot_cid(format!("CID_M0{}", chapter)) {
        if GameVariableManager::get_bool(format!("G_Cleared_M0{}", chapter - 1)) { false }
        else {
            let state = spot.get_spot_state();
            state != GmapSpotState::ReserveActive && state != GmapSpotState::Active
        }
    }
    else { false }
}
fn set_spot(chapter: i32, state: i32) { GmapSpotManager::set_state(format!("CID_M0{}", chapter), state); }
extern "C" fn gmap_sequence_load_actor(proc: &mut ProcInst, _optional_method: OptionalMethod) {
    call_proc_original_method(proc, "LoadActor");
    if let Some(gmap_unit) = try_get_instance::<GmapPlayerUnit>() {
        if gmap_unit.unit.person.parent.index > 1 {
            let result = AssetTableResult::get_from_unit(1, gmap_unit.unit, CharacterAppearance::get_constions(None));
            result.scale_stuff[16] = 4.8;
            gmap_unit.unit.actor.unit_model.load_async(result);
        }
    }
}
pub fn enter_chapter_build_attribute(_basic_menu_item: &BasicMenuItem, _: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCVariables::EmblemRecruitment.get_value() == 0 || DVCVariables::Continuous.get_value() > 0 { return BasicMenuItemAttribute::Enable; }
    else if let Some(spot) = GmapSequence::get_instance().map(|v| v.now_spot){
        let chapter = spot.get_chapter();
        let cid = chapter.cid.to_string();
        if let Some(num) = cid.trim_start_matches("CID_M").parse::<i32>().ok().filter(|n| *n > 10 && *n < 21) {
            let previous_complete = GameVariableManager::get_bool(format!("G_Cleared_M0{}", num-1));
            if !previous_complete { return  BasicMenuItemAttribute::Hide; }
        }
    }
    BasicMenuItemAttribute::Enable
}
extern "C" fn attach_update_gmap_spots(proc: &mut ProcInst, _optional_method: OptionalMethod){
    if DVCVariables::EmblemRecruitment.get_value() != 0 {
        for x in 0..12 {
            let recruit_idx = DVCVariables::get_dvc_emblem_index(x as i32, true);
            if recruit_idx != x && recruit_idx < 12 {
                if let Some(chapter) = ChapterData::get_mut(format!("CID_{}", EMBLEM_PARA[x])) {
                    chapter.recommended_level = PARALOGUE_LEVELS[4*recruit_idx];
                }
            }
        }
        for x in 0..12 {
            let idx = DVCVariables::get_dvc_emblem_index(x as i32, false);
            if idx == x || idx >= 12 || !emblem_paralogue_chapter_check(x as i32) { continue; }
            if let Some(replacement_spot) = GmapSpotManager::find_spot_cid(format!("CID_{}", EMBLEM_PARA[idx])) {
                if emblem_is_available(idx as i32) {
                    if replacement_spot.get_spot_state() == GmapSpotState::Hide { replacement_spot.set_spot_state(GmapSpotState::ReserveActive); }
                    check_gmap_spots(idx as i32)
                } else { replacement_spot.set_spot_state(GmapSpotState::Hide); }
            }
        }
        if GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み"){
            let edelgard_replacement = DVCVariables::get_dvc_emblem_index(12, false);
            if edelgard_replacement < 12 && emblem_paralogue_chapter_check(edelgard_replacement as i32){
                let new_cid = format!("CID_{}", EMBLEM_PARA[edelgard_replacement]);
                if let Some(replacement_spot) = GmapSpotManager::find_spot_cid(new_cid.as_str()) {
                    if replacement_spot.get_spot_state() == GmapSpotState::Hide { replacement_spot.set_spot_state(GmapSpotState::ReserveActive); }
                    check_gmap_spots(edelgard_replacement as i32);
                }
            }
            for x in 13..19 {
                let dlc_gmap_spot = format!("G_GmapSpot_{}", EMBLEM_PARA[x]);
                if GameVariableManager::get_number(dlc_gmap_spot.as_str()) == 1 { GameVariableManager::set_number(dlc_gmap_spot.as_str(), 2); }
                let dlc_replacement = DVCVariables::get_dvc_emblem_index(x as i32, false);
                if dlc_replacement < 12 && emblem_paralogue_chapter_check(dlc_replacement as i32){
                    if let Some(replacement_spot) = GmapSpotManager::find_spot_cid(format!("CID_{}", EMBLEM_PARA[dlc_replacement])) {
                        if emblem_is_available(dlc_replacement as i32) && GameVariableManager::get_bool(format!("G_Cleared_{}", EMBLEM_PARA[x])){
                            if replacement_spot.get_spot_state() == GmapSpotState::Hide {
                                replacement_spot.set_spot_state(GmapSpotState::ReserveActive);
                                check_gmap_spots(dlc_replacement as i32);
                            }
                        }
                        else { replacement_spot.set_spot_state(GmapSpotState::Hide); }
                    }
                }
            }
        }
    }    
    if DVCVariables::EmblemRecruitment.get_value() != 0 {
        call_proc_original_method(proc, "AttachSpotModels");
        [12, 13, 14, 16, 17, 20].iter().for_each(|x| {
            if let Some(spot) = GmapSpotManager::find_spot_cid(format!("CID_M0{}", x).as_str()) {
                let previous_complete = GameVariableManager::get_bool(format!("G_Cleared_M0{}", x - 1));
                if spot.get_spot_state() != GmapSpotState::Hide && !previous_complete && !spot.obj.is_null() {
                    if let Some(controller) = spot.controller.as_ref() {
                        controller.set_material(2);
                        controller.render_mesh.set_material(controller.materials[2]);
                        controller.render_mesh.set_enabled(false);
                        if !controller.effect.is_null() { controller.effect.set_active2(false); }
                    }
                }
            }
        });
    }
}