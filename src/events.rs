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
            else { crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.kizuna_replacements = false).unwrap(); }
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
        crate::autolevel::update_learn_skills(false);
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

pub fn map_sequence_human_events(_proc: &ProcInst, label: i32) {
    let sequence_label: MapSequenceHumanLabel = label.into();
    if sequence_label == MapSequenceHumanLabel::PickCursorResume { crate::randomizer::emblem::player_emblem_check(); }
}

#[skyline::from_offset(0x02285890)]
pub fn autosave_proc_inst(this: &ProcInst, kind: i32, index: i32, stuff: Option<&ProcInst>, method_info: OptionalMethod);


/*
extern "C" fn event_install(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                if let Some(pos) = PROC_CHECK.iter().position(|&x| x == proc.hashcode) {
                    println!("[DVCEventCheck] Proc: {}, label: {}", proc.name.unwrap(), *label);
                    match (pos, *label) {
                        (0, 0) => {
                            println!("TitleScreen");
                            enums::generate_black_list();
                            randomizer::intitalize_game_data();
                            message::initialize_mess_hashs();
                            if let Some(asset_data) = randomizer::assets::data::ASSET_DATA.get()  { asset_data.apply_bust_changes(); }
                            menus::menu_calls_install();
                        }
                        (1, 19) => {
                            CONFIG.lock().unwrap().correct_rates();
                            CONFIG.lock().unwrap().save();
                        }
                        (1, 6) => {
                            let _ = randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock|lock.reset() );
                            if CONFIG.lock().unwrap().randomized { menus::dvc_ng_menu_create_bind(proc); }
                        }
                        (1, 28) => {
                            CONFIG.lock().unwrap().save();
                            randomizer::start_new_game(); 
                        }
                        (5, 0) => {
                            randomizer::assets::install_dvc_outfit();
                            set_personal_caps();
                            randomizer::randomize_stuff();
                            randomizer::tutorial_check();
                            autolevel::update_learn_skills(false);
                            continuous::do_continious_mode();
                            continuous::update_next_chapter();
                            ironman::ironman_code_edits();
                            randomizer::emblem::emblem_gmap_spot_adjust();
                            randomizer::terrain::adjust_miasma_tiles();
                            deployment::lueur_status_check();
                            if engage::gamevariable::GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 { 
                                randomizer::person::change_map_dispos(); 
                                deployment::inspectors::replace_lueur_chapter22();
                            }
                            if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { script::adjust_person_map_inspectors();  }
                        }
                        (2, 1) => {
                            CONFIG.lock().unwrap().correct_rates();
                            CONFIG.lock().unwrap().save();
                        }
                        (2, 12) => {
                            if GameUserData::get_sequence() > 3 && !GameUserData::is_evil_map() { GameUserData::get_status().value &= !8192; }
                            continuous::update_next_chapter();  // For Chapter 11/22 Continue Flag 
                            ironman::ironman_code_edits();
                            autolevel::calculate_player_cap(); 
                            continuous::postchapter::update_bonds();
                            randomizer::bgm::randomize_bgm_map(); 
                            continuous::random::continous_rand_emblem_adjustment();

                            randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock|{
                                lock.kizuna_replacements = false;
                            }).unwrap();
                            randomizer::emblem::enemy::adjust_enemy_edelgard_chapter();
                        }
                        (2, 16) => {
                            randomizer::person::change_lueur_for_recruitment(false);
                            continuous::do_continious_mode();
                            continuous::update_next_chapter();
                        }
                        (2, 2) | (2, 14) => { randomizer::reset_gamedata(); }
                        (2, 6|7|11) => { // Switching Lueur with replacement
                            randomizer::person::change_lueur_for_recruitment(false);
                            if *label == 7 { randomizer::person::hub::change_kizuna_dispos(); }
                        }
                        (3, 12) => {
                            randomizer::terrain::terrain_spots();   // Random TerrainTiles
                            randomizer::emblem::player_emblem_check(); 
                            unsafe { autosave_proc_inst(proc, 5, 0, None, None); }  //AutoSave
                        }
                        (3, 16) => {  randomizer::terrain::fow::rando_fow();  }
                        (3, 17) => {
                            randomizer::person::hub::change_kizuna_dispos();
                            randomizer::emblem::post_map_emblem_adjustment();
                            randomizer::person::m011_ivy_recruitment_check();
                            randomizer::person::lueur_recruitment_check();
                            continuous::postchapter::update_ignots();
                            continuous::continous_mode_post_battle_stuff(proc);
                            autolevel::autolevel_party();
                            randomizer::map::remove_map_effects();
                            randomizer::bgm::reset_bgm();
                            randomizer::emblem::enemy::adjust_enemy_edelgard_post_chapter();
                            crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_complete() ).unwrap();
                        }
                        (3, 4)|(3, 3) => {  // MapSequence Resume
                            randomizer::terrain::fow::resume_fow();
                            randomizer::in_map_randomize();
                            randomizer::map::tilabolical();
                        }
                        (3, 10) =>  { 
                            if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().accessory {
                                let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.accessory = true);
                            }
                        }
                        (3, 21) => {
                            let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.inspectors_set = false);
                        }
                        (6, 14) => {
                            deployment::inspectors::adjust_map_inspectors();
                            randomizer::map::tilabolical();
                            randomizer::emblem::pre_map_emblem_adjustment();
                        }
                        (4, 3) => { randomizer::emblem::player_emblem_check(); }
                        _ => {}
                    }
                }
            }
            SystemEvent::SaveLoaded { ty, slot_id } => {
                println!("Save File Type: {}, Slot Id: {}", *ty, *slot_id);
                if *ty > 1 { 
                    // randomizer::emblem::emblem_bond_check();
                    randomizer::save_file_load(); 
                }
            }
            _ => {},
        }
    } 
    else {  println!("We received a missing event, and we don't care!"); }
}
*/