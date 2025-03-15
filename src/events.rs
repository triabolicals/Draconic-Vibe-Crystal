use unity::prelude::*;
use engage::{
    gameuserdata::GameUserData,
    proc::ProcInst, sequence::{mainmenusequence::MainMenuSequenceLabel, mainsequence::MainSequenceLabel, mapsequence::{human::MapSequenceHumanLabel, MapSequenceLabel}},
};
use crate::{config::DVCVariables, CONFIG};

pub const TITLE_SEQUENCE: i32 = -988690862;
pub const MAIN_SEQUENCE: i32 = -339912801;
pub const MAINMENU_SEQUENCE: i32 = -1912552174;
pub const MAP_SEQUENCE: i32 = 1650205480;
pub const SORTIE_SEQUENCE: i32 = -1624221522;
pub const MAP_HUMAN_SEQUENCE: i32 = 1525873615;
pub const PROC_SCENE: i32 = -1118443598;
pub const PROC_CHECK: [i32; 7] = [TITLE_SEQUENCE, MAINMENU_SEQUENCE, MAIN_SEQUENCE, MAP_SEQUENCE, MAP_HUMAN_SEQUENCE, PROC_SCENE, SORTIE_SEQUENCE];

pub fn map_sequence_events(proc: &ProcInst, label: i32) {
    let sequence_label: MapSequenceLabel = label.into();
    match sequence_label {
        MapSequenceLabel::ResumeSortie | MapSequenceLabel::ResumeMap => { 
            crate::randomizer::terrain::fow::resume_fow();
            crate::randomizer::in_map_randomize();
            crate::randomizer::map::tilabolical();
        },
        MapSequenceLabel::TurnHuman => {
            crate::randomizer::terrain::terrain_spots();   // Random TerrainTiles
            crate::randomizer::emblem::player_emblem_check(); 
            unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
        },
        MapSequenceLabel::TurnBranch => {
            if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().accessory {
                crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.accessory = true).unwrap();
            }
        }
        MapSequenceLabel::TurnEnd => { crate::randomizer::terrain::fow::rando_fow();  }
        MapSequenceLabel::Complete => {
            crate::randomizer::person::hub::change_kizuna_dispos();
            crate::randomizer::emblem::post_map_emblem_adjustment();
            crate::randomizer::person::m011_ivy_recruitment_check();
            crate::randomizer::person::lueur_recruitment_check();
            crate::continuous::postchapter::update_ignots();
            crate::continuous::continous_mode_post_battle_stuff(proc);
            crate::autolevel::autolevel_party();
            crate::randomizer::map::remove_map_effects();
            crate::randomizer::bgm::reset_bgm();
            crate::randomizer::emblem::enemy::adjust_enemy_edelgard_post_chapter();
            crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_complete() ).unwrap();
        },
        MapSequenceLabel::RestartLoad => { crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.inspectors_set = false).unwrap(); }
        _ => {}
    }
}

pub fn main_menu_sequence_events(proc: &ProcInst, label: i32) {
    let sequence_label: MainMenuSequenceLabel = label.into();
    match sequence_label {
        MainMenuSequenceLabel::PlayerGenderSelect => {
            crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock|lock.reset() ).unwrap();
            if CONFIG.lock().unwrap().randomized { crate::menus::dvc_ng_menu_create_bind(proc); }
        },
        MainMenuSequenceLabel::ToStartGame => {
            CONFIG.lock().unwrap().save();
            crate::randomizer::start_new_game(); 
        },
        MainMenuSequenceLabel::Option => {
            CONFIG.lock().unwrap().correct_rates();
            CONFIG.lock().unwrap().save();
        },
        _ => {}
    }
}

pub fn main_sequence_events(_proc: &ProcInst, label: i32) {
    let sequence_label: MainSequenceLabel = label.into();
    match sequence_label {
        MainSequenceLabel::TitleLoop | MainSequenceLabel::GameOver => { crate::randomizer::reset_gamedata(); },

        MainSequenceLabel::Gmap | MainSequenceLabel::Kizuna | MainSequenceLabel::NextChapter => { // Switching Lueur with replacement
            crate::randomizer::person::change_lueur_for_recruitment(false);
            if sequence_label == MainSequenceLabel::Kizuna { crate::randomizer::person::hub::change_kizuna_dispos(); }
        },

        MainSequenceLabel::Hub => {
            crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.kizuna_replacements = false).unwrap(); 
        },

        MainSequenceLabel::Map => {
            if GameUserData::get_sequence() > 3 && !GameUserData::is_evil_map() { GameUserData::get_status().value &= !8192; }
            crate::continuous::update_next_chapter();  // For Chapter 11/22 Continue Flag 
            crate::ironman::ironman_code_edits();
            crate::autolevel::calculate_player_cap(); 
            crate::continuous::postchapter::update_bonds();
            crate::randomizer::bgm::randomize_bgm_map(); 
            crate::continuous::random::continous_rand_emblem_adjustment();
            crate::randomizer::emblem::enemy::adjust_enemy_edelgard_chapter();
            crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.kizuna_replacements = false).unwrap(); 
        },

        MainSequenceLabel::AfterChapterSave => {
            crate::randomizer::person::change_lueur_for_recruitment(false);
            crate::continuous::do_continious_mode();
            crate::continuous::update_next_chapter();
        },
        _ => {}
    }
}

pub fn title_loop_events(_proc: &ProcInst, label: i32) {
    match label {
        0 => {
            crate::enums::generate_black_list();
            crate::randomizer::intitalize_game_data();
            crate::message::initialize_mess_hashs();
            if let Some(asset_data) = crate::randomizer::assets::data::ASSET_DATA.get()  { asset_data.apply_bust_changes(); }
            crate::menus::menu_calls_install();
            crate::ironman::map_save_menu_edits();
        }
        _ => {}
    }
}

pub fn proc_scene_event(_proc: &ProcInst, label: i32) {
    if label == 0 {
        crate::randomizer::assets::install_dvc_outfit();
        crate::misc::set_personal_caps();
        crate::randomizer::randomize_stuff();
        crate::randomizer::tutorial_check();
        crate::randomizer::skill::learn::update_learn_skills(false);
        crate::continuous::do_continious_mode();
        crate::continuous::update_next_chapter();
        crate::ironman::ironman_code_edits();
        crate::randomizer::emblem::emblem_gmap_spot_adjust();
        crate::randomizer::terrain::adjust_miasma_tiles();
        crate::deployment::lueur_status_check();
        if engage::gamevariable::GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 { 
            crate::randomizer::person::change_map_dispos(); 
            crate::script::replace_lueur_chapter22();
        }
        if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { crate::script::adjust_person_map_inspectors();  }
    }
}

pub fn sortie_sequence_events(_proc: &ProcInst, label: i32) {
    if label == 14 {
        crate::script::post_sortie_script_adjustment();
        crate::randomizer::map::tilabolical();
        crate::randomizer::emblem::pre_map_emblem_adjustment();
    }
}

pub fn map_sequence_human_events(proc: &ProcInst, label: i32) {
    crate::ironman::map_save_proc_edit(proc);
    let sequence_label: MapSequenceHumanLabel = label.into();
    if sequence_label == MapSequenceHumanLabel::PickCursorResume { crate::randomizer::emblem::player_emblem_check(); }
}

#[skyline::from_offset(0x02285890)]
pub fn autosave_proc_inst(this: &ProcInst, kind: i32, index: i32, stuff: Option<&ProcInst>, method_info: OptionalMethod);