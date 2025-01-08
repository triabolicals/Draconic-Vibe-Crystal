#![feature(ptr_sub_ptr)]

use cobapi::{Event, SystemEvent};
use std::sync::{Mutex, LazyLock};
use skyline::patching::Patch;
use engage::gamedata::*;
pub mod config;
pub mod randomizer;
pub mod utils;
pub mod autolevel;
pub mod ironman;
pub mod enums;
pub mod continuous;
pub mod message;
pub mod menus;
pub mod deployment;
pub mod event;

use crate::config::DeploymentConfig;
use unity::prelude::OptionalMethod;
use engage::proc::ProcInst;
pub static CONFIG: LazyLock<Mutex<DeploymentConfig>> = LazyLock::new(|| DeploymentConfig::new().into() );
pub const VERSION: &str = "2.9.8";
#[skyline::from_offset(0x02285890)]
pub fn autosave_proc_inst(this: &ProcInst, kind: i32, index: i32, stuff: Option<&ProcInst>, method_info: OptionalMethod);

fn disable_support_restriction() {
    let replace = &[0x1f, 0x25, 0x00, 0x71];
    Patch::in_text(0x0209950C).bytes(replace).unwrap();
    Patch::in_text(0x020994E0).bytes(replace).unwrap();
    Patch::in_text(0x02099538).bytes(replace).unwrap();
    Patch::in_text(0x01a2a7c0).bytes(&[0xe1,0x0e,0x80,0x12]).unwrap();
    Patch::in_text(0x01a2a7c4).bytes(&[0x02,0x0f,0x80,0x52]).unwrap();
    Patch::in_text(0x01fdea34).bytes(&[0x01,0x04,0x80, 0x52]).unwrap();
}

pub fn set_personal_caps(){
    let persons = PersonData::get_list_mut().expect("triabolical is 'None'");
    let jobs = JobData::get_list_mut().expect("triabolical2 is 'None'");
    if CONFIG.lock().unwrap().max_stat_caps {
        jobs.iter().for_each(|job|{
            let base = job.get_base();
            let cap = job.get_limit();
            for x in 0..10 { cap[x] = base[x] + 125; }
            cap[10] = 99;
        });
        persons.iter_mut().for_each(|person|{ let limits = person.get_limit(); for y in 0..11 { limits[y] = 0; }});
    }
}
extern "C" fn initalize_random_persons(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                // println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                if proc.hashcode == -988690862 && *label == 0 { // On Initial Title Screen Load
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    randomizer::intitalize_game_data();
                    enums::generate_black_list();
                    set_personal_caps();
                    message::replace_hub_fxn();
                    randomizer::assets::ASSET_DATA.lock().unwrap().apply_bust_changes();
                }
                if proc.hashcode == -1912552174 && *label == 19  {
                    CONFIG.lock().unwrap().correct_rates();
                    CONFIG.lock().unwrap().save();
                }
                // randomized stuff
                if proc.hashcode == -1118443598 && *label == 0 { // ProcScene
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    randomizer::assets::install_dvc_outfit();
                    continuous::do_continious_mode();
                    continuous::update_next_chapter();
                    randomizer::randomize_stuff();
                    randomizer::tutorial_check();
                    autolevel::update_learn_skills();
                    ironman::ironman_code_edits();
                    randomizer::emblem::emblem_gmap_spot_adjust();
                    deployment::fulldeploy::adjust_miasma_tiles();
                    if engage::gamevariable::GameVariableManager::get_number("G_Random_Recruitment") != 0 { 
                        randomizer::person::change_map_dispos(); 
                        deployment::inspectors::replace_lueur_chapter22();
                    }
                }
                // New Game After Character Creation
                if proc.hashcode == -1912552174 && *label == 28 {
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    randomizer::start_new_game();  
                }  // MainMenuSequence
                if proc.hashcode == -339912801 {    // MainSequence
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    match *label {
                        1 => {
                            CONFIG.lock().unwrap().correct_rates();
                            CONFIG.lock().unwrap().save();
                        }
                        12 => { // Map Loading
                            continuous::update_next_chapter();  // For Chapter 11/22 Continue Flag 
                            ironman::ironman_code_edits();
                            randomizer::item::shop::randomize_hub_random_items();
                            autolevel::calculate_player_cap(); 
                            continuous::update_bonds();
                            randomizer::bgm::randomize_bgm_map(); 
                            continuous::continous_rand_emblem_adjustment();
                            unsafe { crate::enums::LUEUR_CHANGE = true; }
                        }
                        16 => {
                            continuous::do_continious_mode();
                            continuous::update_next_chapter();
                        },
                        2 => {
                            randomizer::reset_gamedata(); 
                            set_personal_caps();
                        }   //Reset gamedata
                        _ => {}
                    }
                }
               // if ( proc.hashcode == 1811932770 && *label == 7 ) || (proc.hashcode == 570083351 && *label == 30)  { continuous::update_next_chapter(); }
                if proc.hashcode == 1650205480 {    // MapSequence
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    match *label {
                        12 => {
                            deployment::fulldeploy::power_spot();
                            randomizer::emblem::player_emblem_check();
                            unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
                        }
                        17 => { // Map Ending
                            randomizer::emblem::post_map_emblem_adjustment();
                            randomizer::person::m011_ivy_recruitment_check();
                            continuous::update_ignots();
                            continuous::continous_mode_post_battle_stuff(proc);
                            autolevel::autolevel_party();
                            deployment::fulldeploy::adjust_miasma_tiles();
                            randomizer::bgm::reset_bgm()
                        }
                        4|10 => { randomizer::in_map_randomize(); }
                        _ => {},
                    }
                }
                if proc.hashcode == -1624221522 && *label == 14 { // sortie sequence ends
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                    deployment::inspectors::adjust_map_inspectors();
                    randomizer::emblem::pre_map_emblem_adjustment();
                }
                if proc.hashcode == 1525873615 && *label == 3 {
                    println!("Proc: {}, Label: {}, Code: {}", proc.name.unwrap(), label, proc.hashcode);
                     randomizer::emblem::player_emblem_check(); }
                // iron man auto-saving
                /*
                if proc.hashcode == -813168385 && *label == 3 {    // UnitGrowSeuence
                    unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
                }
                if proc.hashcode == 575009008 && *label == 26 {    // MapSequenceMind
                    unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
                }
                */
            }
            _ => {},
        }
    } 
    else {  println!("We received a missing event, and we don't care!"); }
}

#[skyline::main(name = "vibe")]
pub fn main() {
    let _ = std::fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    randomizer::intialize_added_data();
    cobapi::register_system_event_handler(initalize_random_persons);
    CONFIG.lock().unwrap().save();
    skyline::install_hooks!( 
        randomizer::job::add_job_list_unit, 
        randomizer::try_get_index, 
        deployment::create_player_team, 
        message::script_get_string,
        //deseralized_save,
        randomizer::item::random_well_item,
        randomizer::person::unit::unit_create_impl_2_hook, 
        randomizer::person::unit::create_from_dispos_hook,
        randomizer::assets::animation::asset_table_result_setup_hook,
        randomizer::assets::emblem::asset_table_result_god_setup,
        randomizer::assets::emblem::asset_table_robin_hook,
        randomizer::assets::transform::asset_table_result_setup_person_hook,
        randomizer::assets::transform::change_dragon,
        randomizer::emblem::arena_emblem_weapon,
        randomizer::bgm::special_bgm_hook,
        randomizer::assets::transform::combat_character_game_status_import,
        event::get_cmd_info_from_cmd_lines_hook,
        event::get_active_character_hook,
        message::mess_get_impl_hook, 
        randomizer::person::get_unit_ascii_name,
        randomizer::person::get_bond_face,  
        randomizer::person::get_thumb_face,
        randomizer::person::get_god_face,
        randomizer::person::get_god_thumb_face,
        randomizer::assets::emblem::asset_table_result_get_preset_name,
        //get_file_count,
        event::talk_ptr,
        event::calculate_str_width,
    ); 
    Patch::in_text(0x01bb24a8).nop().unwrap();
    Patch::in_text(0x01e41118).bytes(&[0x3f, 0x0d, 0x00,0x71]).unwrap();
    Patch::in_text(0x02677308).bytes(&[0x1f, 0x15, 0x00,0x71]).unwrap();
    Patch::in_text(0x01e40d7c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();
    Patch::in_text(0x01e40f0c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();

    // FX-Related
    Patch::in_text(0x01c79694).nop().unwrap();
    Patch::in_text(0x01c79740).nop().unwrap();
    Patch::in_text(0x01c796f0).nop().unwrap();

    // Patch::in_text(0x0228151c).bytes(&[0xEA, 0x01, 0x80, 0x52]).unwrap();
    // Patch::in_text(0x02281fb8).bytes(&[0xe8, 0x01, 0x80, 0x52]).unwrap();

    menus::install_vibe();
    disable_support_restriction();
    remove_skill_equip_restrictions();
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };
        let err_msg = format!(
            "Draconic Vibe Crystal {} has panicked at '{}' with the following message:\n{}\0",
            VERSION,
            location,
            msg
        );
        skyline::error::show_error(
            5,
            "Draconic Vibe Crystal has panicked! Please open the details and send a screenshot to triabolical, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
}




fn remove_skill_equip_restrictions() {
    Patch::in_text(0x01a379b4).bytes(&[0x09, 0x00, 0x00, 0x14]).unwrap();
    crate::utils::return_true(0x02490780);
    let offsets = [0x01a36588, 0x01a38b68, 0x01a38144, 0x01a35fa4, 0x01a391e8, 0x024a63fc, 0x01a36f34, 0x01a35ec8];
    for x in offsets { Patch::in_text(x).nop().unwrap(); }
}