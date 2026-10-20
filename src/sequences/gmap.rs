use super::*;
use engage::{
    unity_engine::{
        gameobject::*,
        IRendererMethods, IComponentMethods, IObject_2Methods, ITransformMethods
    },
    List_1Ext,
    app::{
        BasicMenuItem_Attribute,
        IGmapCameraMethods,
        ResourceManager_2,
        gmapsequence::*, gmapspot::*, gmapspotmanager::*,
        chapterdata::*, gmapspotcontroller::*,
        godunit::*, godpool::*,
        IStructData_1Methods,
        assettable::prelude::IList_1Methods,
        gmap::gmappathcollection::*,
    },
};
use engage::app::Talk;
use crate::{
    DVCVariable, gamedata::*,
    utils::to_opt, get_chapter_from_idx,
    dvc::autolevel::get_difficulty_adjusted_average_level,
    dvc::randomizer::map::dispos::PARALOGUE_LEVELS,
    utils::override_vtable
};
use crate::dvc::mess;

pub const EMBLEM_PARA: [&str; 19] = ["S014", "S009", "S013", "S011", "S012", "S010", "S003", "S004", "S005", "S006", "S007", "S008", "G007", "G001", "G002", "G003", "G004", "G005", "G006"];
const PARALOGUE_UNLOCK: [i32; 14] = [5, 6, 11, 12, 13, 14, 15, 16, 17, 17, 19, 19, 20, 22];
pub fn vtable_gmap_enter_chapter_item() {
    override_vtable("App", "GmapMenuSequence.GmapMenu.EnterChapterItem", "BuildAttribute", enter_chapter_build_attribute as _);
    if let Some(talk) = Class::lookup("App", "GmapSequence.EnterChapterSequence").raw_mut().get_method_from_name("Talk", 0) {
        talk.method_ptr = gmap_enter_chapter_sequence_talk as _;
    }
}

pub fn gmap_enter_chapter_sequence_talk(this: ProcInst, _: OptionalMethod) {
    Mess::load("M010");
    mess::hub::replace_for_recruitment_demo_name("MID_GMAP1", true);
    Talk::start_bind(this, "MID_GMAP1", false);
}
pub fn enter_chapter_build_attribute(_: u64, _: OptionalMethod) -> BasicMenuItem_Attribute {
    let con_mode = DVCVariable::Continuous.get_value();
    if DVCConfig::get().debug || DVCVariable::EmblemRecruitment.get_value() == 0 || (con_mode > 0 && con_mode < 3) {
        BasicMenuItem_Attribute::enable()
    }
    else if to_opt(GmapSequence::get_instance()).and_then(|n| to_opt(n.m_now_spot())).is_none_or(|s| is_spot_active(s)) { BasicMenuItem_Attribute::enable() }
    else { BasicMenuItem_Attribute::hide() }
}
fn is_spot_active(spot: GmapSpot) -> bool {
    let con_mode = DVCVariable::Continuous.get_value();
    let open_mode = con_mode > 2;
    let chapter = spot.get_chapter();
    let cid = chapter.get_cid().to_rust_string();
    if let Some(num) = cid.trim_start_matches("CID_M").parse::<i32>().ok() {
        (open_mode && ((DVCVariables::is_main_chapter_complete(11) && num > 11) || (num < 11))) || DVCVariables::is_main_chapter_complete(num - 1)
        || DVCVariables::is_main_chapter_complete(num)
    }
    else { true }
}

pub fn gmap_desc_edit(proc: ProcInst) {
    let descs = proc.m_descs();
    if let Some(gmap) = proc.try_cast::<GmapSequence>() {
        descs.set(11, Proc::call_2(ProcVoidMethod::from_fn(gmap, attach_update_gmap_spots).unwrap()));
        descs.set(30, Proc::call_2(ProcVoidMethod::from_fn(gmap, check_path_appear).unwrap()));
    }
}

fn deactivate_inactive_spot_objects() {
    let manager = GmapSpotManager::get_instance();
    manager.m_spot_array().iter().for_each(|spot| {
        if !is_spot_active(spot) {
            if let Some(controller) = to_opt(spot.m_controller()) {
                controller.set_material(GmapSpotController_Type::cleared());
                controller.m_mesh_renderer().set_material(controller.m_materials().get(2));
                controller.m_mesh_renderer().set_enabled(false);
                controller.m_stone_mesh_renderer().set_enabled(false);
                if !controller.m_effect().is_null() { controller.m_effect().set_active(false); }
            }
        }
    });
}
extern "C" fn check_path_appear(gmap_sequence: GmapSequence, _: OptionalMethod) {
    deactivate_inactive_spot_objects();
    if let Some((start, appear)) = get_open_spot() {
        if let Some(obj) = get_path_object(gmap_sequence.m_path_collection(), start, appear) {
            gmap_sequence.set_m_changing_path(obj);
        }
        appear.set_spot_state(GmapSpot_State::reserve_active());
        start.get_position();
        gmap_sequence.m_gmap_camera().set_position(start.get_position(), false);
        let obj = gmap_sequence.m_effect_parent();
        let path = gmap_sequence.appear_line_effect_path();
        gmap_sequence.set_m_appear_line_effect(ResourceManager_2::instantiate(path, obj));
        gmap_sequence.set_m_changing_spot(appear);
        gmap_sequence.m_appeared_spot_list().add(appear);
        return;
    }
    gmap_sequence.check_appear_path();
}
fn get_connecting_chapter_spot(chapter_index: i32) -> Option<i32> {
    match chapter_index {
        5..16|17..20|62..67 => { Some(chapter_index -1) }
        16 => Some(14),
        20 => Some(10),
        21 => Some(19),
        23 => Some(21),
        24 => Some(21),
        25 => Some(24),
        31 => Some(4),
        32 => Some(6),
        33 => Some(8),
        34 => Some(5),
        35 => Some(9),
        36 => Some(11),
        37 => Some(6),
        38 => Some(14),
        39 => Some(12),
        40 => Some(16),
        41 => Some(20),
        42 => Some(5),
        43 => Some(13),
        44 => Some(7),
        45 => Some(22),
        51 => Some(32),
        52..55 => Some(51),
        55|56 => Some(52),
        _ => None,
    }
}
fn get_gmap_from_chapter_index(manager: GmapSpotManager, chapter_index: i32) -> Option<GmapSpot> {
    let cid = format!("CID_{}", get_chapter_from_idx(if chapter_index == 22 { 21 } else { chapter_index }));
    to_opt(manager.find_spot_by_cid(cid))
}
fn get_next_start_appear_spots(manager: GmapSpotManager, chapter_index: i32) -> Option<(GmapSpot, GmapSpot)> {
    if let Some(appear_spot) = get_gmap_from_chapter_index(manager, chapter_index).filter(|s| s.get_spot_state().value < 3){
        get_connecting_chapter_spot(chapter_index)
            .and_then(|connecting_spot| {
                if let Some(start_spot) = get_gmap_from_chapter_index(manager, connecting_spot) {
                    if start_spot.get_spot_state().value == 3 { Some((start_spot, appear_spot)) }
                    else { get_next_start_appear_spots(manager, connecting_spot) }
                }
                else { None }
            })
    }
    else { None }
}
fn get_path_object(path: GmapPathCollection, start: GmapSpot, appear: GmapSpot) -> Option<GameObject> {
    let root = path.m_root().get_transform();
    let start = start.get_gmap_spot_id().to_rust_string();
    let appear = appear.get_gmap_spot_id().to_rust_string();
    let children = root.get_child_count();
    for i in 0..children {
        let child = root.get_child(i);
        let name = child.get_name().to_rust_string();
        if name.contains(&start) && name.contains(&appear) {
            return Some(child.get_game_object());
        }
    }
    None
}
fn get_open_spot() -> Option<(GmapSpot, GmapSpot)> {
    if !DVCVariables::is_main_chapter_complete(4) { return None; }
    if DVCVariables::is_main_chapter_complete(22) { GameVariableManager::set_number("G_GmapSpot_M021M022", 3); }
    let mode = DVCVariables::user_data().get_map_mode().value;
    let manager = GmapSpotManager::get_instance();
    if mode == 0 {
        let (start, end) =
            if DVCVariable::Continuous.get_value() > 2 {
                if DVCVariables::is_main_chapter_complete(22) { (23, 26) }
                else if DVCVariables::is_main_chapter_complete(11) { (12, 22) }
                else { (5, 11) }
            }
            else {
                let mut chapter_index = 4;
                for i in 4..26 {
                    if !DVCVariables::is_main_chapter_complete(i) {
                        chapter_index = i;
                        break;
                    }
                }
                (5, chapter_index)
            };

        for i in start..end {
            if i == 22 || i >= 26 { continue; }
            if let Some(spot) = to_opt(manager.find_spot_by_cid(format!("CID_M{:03}", i))) {
                if spot.get_spot_state().value < 3 {
                    spot.set_spot_state(GmapSpot_State::reserve_active());
                    let next = get_next_start_appear_spots(manager, i);
                    if next.is_some() { return next; }
                }
            }
        }
        for i in 0..2 {
            if DVCVariables::is_main_chapter_complete(PARALOGUE_UNLOCK[i as usize]) {
                let spot = manager.find_spot_by_cid(format!("CID_S{:03}", i + 1));
                if spot.get_spot_state().value < 3 {
                    spot.set_spot_state(GmapSpot_State::reserve_active());
                    let next = get_next_start_appear_spots(manager, 31 + i);
                    if next.is_some() { return next; }
                }
            }
        }
        if DVCVariable::EmblemRecruitment.get_value() != 0 {
            for i in 3..15 {
                let spot = manager.find_spot_by_cid(format!("CID_S{:03}", i));
                if spot.get_spot_state().value == 2 {
                    let next = get_next_start_appear_spots(manager, 30 + i);
                    if next.is_some() { return next; }
                }
            }
        }
    }
    if GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み"){
        for i in 1..7 {
            if let Some(spot) = get_gmap_from_chapter_index(manager, 50 + i) {
                if spot.get_spot_state().value < 3 { spot.set_spot_state(GmapSpot_State::reserve_active()); }
                if (i == 1 && mode == 0) || (i > 1 && mode == 1){
                    let next = get_next_start_appear_spots(manager, 50 + i);
                    if next.is_some() { return next; }
                }
            }
        }
    }
    None
}
pub fn get_available_ring_paralogue() -> Vec<(ChapterData, u8, bool)> {
    let mut out = vec![];
    if DVCVariables::is_main_chapter_complete(11) {
        for i in 0..12 {
            let level = PARALOGUE_LEVELS[4 * i as usize];
            let chapter = ChapterData::get(format!("CID_{}", EMBLEM_PARA[i as usize]).into());
            let mut entry = (chapter, 255, GameVariableManager::get_bool(chapter.get_cleared_flag_name()) );
            if let Some(god) = EmblemPool::get_replacement_god_unit(i, false).filter(|v| EmblemPool::is_custom(v.m_data())) {
                if !god.get_is_darkness() && !god.get_is_escaping() { entry.1 = level; }
            }
            else if let Some(god) = to_opt(GodPool::try_get(format!("GID_{}", EmblemPool::GIDS[i as usize]), false)){
                if !god.get_is_darkness() && !god.get_is_escaping(){
                    if let Some(index) = EmblemPool::get_replacement_index_2(i, true){
                        if index < 12 { entry.1 = PARALOGUE_LEVELS[4 * index as usize]; }
                        else if index < 19 {
                            let avg_level = get_difficulty_adjusted_average_level() as u8;
                            entry.1 =
                                if avg_level < level { if avg_level < 20 { 20 } else { avg_level } }
                                else { level };
                        }
                    }
                }
            }
            out.push(entry);
        }
    }
    out
}
extern "C" fn attach_update_gmap_spots(proc: GmapSequence, _: OptionalMethod) {
    GameVariableManager::set_number("G_GmapSpot_M004", 3);
    let adjusted_emblem_recruitment = DVCVariable::EmblemRecruitment.get_value() != 0;
    let manager = GmapSpotManager::get_instance();
    if EmblemPool::get_replacement_god_unit(12, false).is_some() && DVCVariables::is_main_chapter_complete(6) {
        GameVariableManager::set_bool("G_拠点_神竜導入イベント再生済み", true);
    }
    if GameVariableManager::get_bool("G_拠点_神竜導入イベント再生済み")  {
        if adjusted_emblem_recruitment {
            for i in 1..7 {
                let key = format!("G_GmapSpot_G00{}", i);
                if GameVariableManager::get_number(key.as_str()) < 3 { GameVariableManager::set_number(key.as_str(), 2); }
            }
        }
    }
    if adjusted_emblem_recruitment && DVCVariables::is_main_chapter_complete(11) {
        get_available_ring_paralogue().into_iter()
            .for_each(|(chapter, level, cleared)|{
                if level == 255 { chapter.set_gmap_spot_state(GmapSpot_State::hide()); }
                else {
                    if chapter.get_gmap_spot_state().value < 3 {
                        chapter.set_gmap_spot_state(GmapSpot_State{value: if cleared { 3 } else { 2 } });
                    }
                    chapter.set_recommended_level(level);
                }
            });
    }
    let mode = DVCVariable::Continuous.get_value();
    if mode == 3 || mode == 4 {
        let (start, end) = 
            if DVCVariables::is_main_chapter_complete(22) { (23, 26) } 
            else if DVCVariables::is_main_chapter_complete(11) { (12, 22) } 
            else { (6, 11) };
        
        for i in start..end {
            if let Some(spot) = get_gmap_from_chapter_index(manager, i){
                if spot.get_spot_state().value < 3 { spot.set_spot_state(GmapSpot_State::reserve_hide()); }
            }
        }
    }
    proc.attach_spot_models();
    deactivate_inactive_spot_objects();
}
