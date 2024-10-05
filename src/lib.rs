#![feature(lazy_cell, ptr_sub_ptr)]
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
pub const VERSION: &str = "2.5.0";

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
    let mut is_max_limit = false;
    for x in 0..jobs.len() {
        let cap = jobs[x].get_limit();
        if cap[1] >= 127 {
            is_max_limit = true; 
            break;
        }
    }
    if !is_max_limit { return; }
    for x in 0..persons.len() {
        let personal_limits = persons[x].get_limit();
        for y in 0..11 { personal_limits[y] = 0; } 
    }
}

extern "C" fn initalize_random_persons(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                if proc.hashcode == 1650205480 && *label == 17 {    // map ending
                    randomizer::person::m011_ivy_recruitment_check();
                    continuous::update_ignots();
                    continuous::continous_mode_post_battle_stuff(proc);
                    //deployment::engage_plus_remove_rings(); 
                    randomizer::bgm::reset_bgm();
                }
                if proc.hashcode == -988690862 && *label == 0 { // On Initial Title Screen Load
                    enums::generate_black_list();
                    randomizer::intitalize_game_data();
                    
                    //grow::get_style_interact_default_values();
                    //asset::gather_all_accesories();
                    message::replace_hub_fxn();
                    set_personal_caps();
                    randomizer::assets::ASSET_DATA.lock().unwrap().apply_bust_changes();
                }
                if proc.hashcode == -339912801 && *label == 1 {
                    CONFIG.lock().unwrap().correct_rates();
                    CONFIG.lock().unwrap().save();
                }
                if proc.hashcode == -1912552174 && *label == 19 {
                    CONFIG.lock().unwrap().correct_rates();
                    CONFIG.lock().unwrap().save();
                }
                //Reset things
                if proc.hashcode == -339912801 && *label == 2 { randomizer::reset_gamedata(); }
                // randomized stuff
                if proc.hashcode == -1118443598 && *label == 0 { 
                    continuous::do_continious_mode();
                    randomizer::randomize_stuff(); 
                    randomizer::tutorial_check();
                    autolevel::update_learn_skills();
                    continuous::update_next_chapter();
                    ironman::ironman_code_edits();  
                    randomizer::emblem::emblem_gmap_spot_adjust();
                    if engage::gamevariable::GameVariableManager::get_number("G_Random_Recruitment") != 0 { 
                        randomizer::person::change_map_dispos(); 
                    }
                }
                // New Game After Character Creation
                if proc.hashcode == -1912552174 && *label == 28 { randomizer::start_new_game();  }
                // when map starts, iron code edits activate
                if proc.hashcode == -339912801 && *label == 12 { 
                    ironman::ironman_code_edits();
                    randomizer::item::shop::randomize_hub_random_items();
                    autolevel::calculate_player_cap(); 
                    continuous::update_bonds();
                    randomizer::bgm::randomize_bgm_map(); 
                    unsafe { crate::enums::LUEUR_CHANGE = true; }
                }
                //if proc.hashcode == 1650205480
               // {  println!("Proc: {}, Hash {}, label {}", proc.name.unwrap().get_string().unwrap(), proc.hashcode, label); }
                if proc.hashcode == 1650205480 && *label == 12 {    //MapSequence TurnHuman
                    unsafe { autosave_proc_inst(proc, 5, 0, None, None); }
                }
                if proc.hashcode == -1624221522 && *label == 14 { // sortie sequence ends
                    deployment::inspectors::adjust_map_inspectors();
                }
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
    cobapi::register_system_event_handler(initalize_random_persons);
    skyline::install_hooks!( 
        randomizer::job::add_job_list_unit, 
        randomizer::try_get_index, 
        deployment::create_player_team, 
        message::script_get_string,
        randomizer::person::unit::unit_create_impl_2_hook, 
        randomizer::person::unit::create_from_dispos_hook,
        randomizer::assets::animation::asset_table_result_setup_hook,
        event::get_cmd_info_from_cmd_lines_hook,
        event::get_active_character_hook,
        message::mess_get_impl_hook, 
        event::mess_load_hook,
    ); 
    // Fixes the emblem weapons arena issue
    Patch::in_text(0x01ca9afc).nop().unwrap();
    Patch::in_text(0x01e41118).bytes(&[0x3f, 0x0d, 0x00,0x71]).unwrap();
    Patch::in_text(0x02677308).bytes(&[0x1f, 0x15, 0x00,0x71]).unwrap();
    Patch::in_text(0x01e40d7c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();
    Patch::in_text(0x01e40f0c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();
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