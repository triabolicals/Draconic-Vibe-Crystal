use unity::prelude::*;
use engage::{
    gameuserdata::GameUserData, proc::ProcInst,
    sequence::{
        mainmenusequence::MainMenuSequenceLabel,
        mainsequence::MainSequenceLabel,
        mapsequence::MapSequenceLabel
    }
};
use num_traits::cast::FromPrimitive;
use engage::gamevariable::GameVariableManager;

use outfit_core::{install_outfit_plugin, UnitAssetMenuData};
use skyline::patching::Patch;
use crate::{config::DVCVariables, randomizer, DeploymentConfig};
pub const TITLE_SEQUENCE: i32 = -988690862;
pub const MAIN_SEQUENCE: i32 = -339912801;
pub const MAINMENU_SEQUENCE: i32 = -1912552174;
pub const MAP_SEQUENCE: i32 = 1650205480;
pub const SORTIE_SEQUENCE: i32 = -1624221522;
pub const MAP_HUMAN_SEQUENCE: i32 = 1525873615;
pub const PROC_SCENE: i32 = -1118443598;
pub const PROC_CHECK: [i32; 8] = [570083351, TITLE_SEQUENCE, MAINMENU_SEQUENCE, MAIN_SEQUENCE, MAP_SEQUENCE, MAP_HUMAN_SEQUENCE, PROC_SCENE, SORTIE_SEQUENCE];

pub fn map_sequence_events(proc: &ProcInst, label: i32) {
    let sequence_label: MapSequenceLabel = MapSequenceLabel::from_i32(label).unwrap_or(MapSequenceLabel::End);
    match sequence_label {
        MapSequenceLabel::ResumeSortie | MapSequenceLabel::ResumeMap => { 
            randomizer::terrain::fow::resume_fow();
            randomizer::in_map_randomize();
            randomizer::map::tilabolical();
            if DVCVariables::UnitRecruitment.get_value() != 0 { crate::script::replace_lueur_chapter22(); }
        },
        MapSequenceLabel::TurnHuman => {
            randomizer::terrain::terrain_spots();   // Random TerrainTiles
            unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
            if DVCVariables::UnitRecruitment.get_value()  != 0 {
                crate::script::replace_lueur_chapter22();
            }
        },
        MapSequenceLabel::TurnEnd => { randomizer::terrain::fow::rando_fow();  }
        MapSequenceLabel::Complete => {
            GameVariableManager::set_number(DVCVariables::TILE_RNG, 0);
            randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_complete() ).unwrap();
            if GameUserData::get_chapter().cid.str_contains("M002") { randomizer::item::change_liberation_type(); }
            randomizer::person::hub::change_kizuna_dispos();
            randomizer::person::m011_ivy_recruitment_check();
            randomizer::person::lueur_recruitment_check();
            crate::continuous::postchapter::update_ignots();
            crate::continuous::continous_mode_post_battle_stuff(proc);
            crate::autolevel::autolevel_party();
            randomizer::map::remove_map_effects();
            randomizer::bgm::reset_bgm();
            crate::continuous::update_next_chapter();
            randomizer::item::shop::update_added_shop_items(true);
        },
        _ => {}
    }
}
pub fn main_menu_sequence_events(proc: &ProcInst, label: i32) {
    let sequence_label: MainMenuSequenceLabel = MainMenuSequenceLabel::from_i32(label).unwrap_or(MainMenuSequenceLabel::None);
    match sequence_label {
        MainMenuSequenceLabel::PlayerGenderSelect => { 
            randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock|lock.reset() ).unwrap();
            if DeploymentConfig::get().randomized {
                crate::config::menu::dvc_ng_menu_create_bind(proc);
            }
        },
        MainMenuSequenceLabel::ToStartGame => {
            DeploymentConfig::get().save();
            randomizer::start_new_game();
        },
        MainMenuSequenceLabel::Option => {
            DeploymentConfig::get().correct_rates();
            DeploymentConfig::get().save();
        },
        _ => {}
    }
}

pub fn main_sequence_events(_proc: &ProcInst, label: i32) {
    let sequence_label: MainSequenceLabel = MainSequenceLabel::from_i32(label).unwrap();
    match sequence_label {
        MainSequenceLabel::TitleLoop | MainSequenceLabel::GameOver => { randomizer::reset_gamedata(); },
        MainSequenceLabel::Gmap | MainSequenceLabel::Kizuna | MainSequenceLabel::NextChapter => { // Switching Lueur with replacement 
            randomizer::person::change_lueur_for_recruitment(false);
        },
        MainSequenceLabel::Hub => { 
            randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.kizuna_replacements = false).unwrap(); 
        },
        MainSequenceLabel::Map => {},
        MainSequenceLabel::AfterChapterSave => {
            randomizer::person::change_lueur_for_recruitment(false);
            crate::continuous::do_continious_mode();
            crate::continuous::update_next_chapter();
        },
        _ => {}
    }
}

pub fn title_loop_events(_proc: &ProcInst, label: i32) {
    match label {
        0 => {
            if !UnitAssetMenuData::get().init {
                install_outfit_plugin(true);
                Patch::in_text(0x01bb272c).nop().unwrap(); // AssetTableResult Setup Prevents Commit
                skyline::install_hooks!(
                    crate::assets::asset_table_result_setup_hook,
                    crate::assets::emblem::asset_table_result_god_setup,
                    crate::assets::emblem::asset_table_robin_hook,
                );
            }
            crate::ironman::map_save_menu_edits();
        }
        _ => { UnitAssetMenuData::get().is_loaded = false; }
    }
}

pub fn proc_scene_event(_proc: &ProcInst, label: i32) {
    if label == 0 {
        // randomizer::randomize_stuff();
        randomizer::tutorial_check();
        randomizer::skill::learn::update_learn_skills(false);
        crate::continuous::do_continious_mode();
        crate::continuous::update_next_chapter();
        crate::ironman::ironman_code_edits();
        randomizer::terrain::adjust_miasma_tiles();
        randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.map_complete() ).unwrap();
        if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { crate::script::adjust_person_map_inspectors(); }
        randomizer::item::shop::update_added_shop_items(false);
        crate::menus::menu_calls_install();
        randomizer::job::adjust_missing_weapon_mask();
        randomizer::item::adjust_non_unit_items_inventory();
        /*
        if crate::DeploymentConfig::get().debug {
            Force::get(ForceType::Player).unwrap().iter().for_each(|u|{
                let move_stat = u.base_capability[10] as i32;
                if move_stat < 20 { u.set_base_capability(10,move_stat + 20); }
                u.set_hp(u.get_capability(0, true));
            });
        }
         */
    }
}

pub fn sortie_sequence_events(_proc: &ProcInst, label: i32) {
    if label == 14 {
        crate::script::post_sortie_script_adjustment();
        randomizer::map::tilabolical();
    }
}

#[skyline::from_offset(0x02285890)]
pub fn autosave_proc_inst(this: &ProcInst, kind: i32, index: i32, stuff: Option<&ProcInst>, method_info: OptionalMethod);
