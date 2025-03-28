#![feature(ptr_sub_ptr)]
use cobapi::{Event, SystemEvent};
use std::sync::{Mutex, LazyLock};
use crate::config::*;
use skyline::patching::Patch;

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
pub mod talk;
pub mod events;
pub mod script;
pub mod misc;
pub mod assets;

pub static CONFIG: LazyLock<Mutex<DeploymentConfig>> = LazyLock::new(|| DeploymentConfig::new().into() );
pub const VERSION: &str = "2.11.0";

extern "C" fn event_install(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                if CONFIG.lock().unwrap().debug { println!("Proc: {} Label: {}", proc.name.unwrap(), *label);  }
                if let Some(pos) = events::PROC_CHECK.iter().position(|&x| x == proc.hashcode) {
                    let proc_label = *label;
                    // println!("[DVC Event Type {}] Label: {}", pos, proc_label);
                    match proc.hashcode {
                        events::TITLE_SEQUENCE => { events::title_loop_events(proc, proc_label); }
                        events::MAINMENU_SEQUENCE => { events::main_menu_sequence_events(proc, proc_label);}
                        events::MAIN_SEQUENCE => { events::main_sequence_events(proc, proc_label); }
                        events::MAP_SEQUENCE => { events::map_sequence_events(proc, proc_label); }
                        events::MAP_HUMAN_SEQUENCE => { events::map_sequence_human_events(proc, proc_label); }
                        events::SORTIE_SEQUENCE => { events::sortie_sequence_events(proc, proc_label); }
                        events::PROC_SCENE => { events::proc_scene_event(proc, proc_label); }
                        _ => {}
                    }
                }
            }
            SystemEvent::SaveLoaded { ty, slot_id } => {
                println!("Save File Type: {}, Slot: {}", *ty, *slot_id);
                if *ty > 1 {  randomizer::save_file_load();  }
            }
            _ => {},
        }
    } 
    else {  println!("We received a missing event, and we don't care!"); }
}

#[skyline::main(name = "vibe")]
pub fn main() {
    let _ = std::fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    cobapi::register_system_event_handler(event_install);
    println!("Draconic Vibe Crystal v{}", VERSION);
    randomizer::intialize_added_data();
    cobapi::install_lua_command_registerer(randomizer::map::register_script_commands);
    CONFIG.lock().unwrap().save();
    assets::get_accessory_count();
    skyline::install_hooks!( 
        randomizer::job::reclass::add_job_list_unit, 
        randomizer::try_get_index, 
        deployment::create_player_team, 
        script::script_get_string,
        randomizer::item::random_well_item,
        randomizer::person::unit::unit_create_impl_2_hook, 
        assets::asset_table_result_setup_hook,
        assets::emblem::asset_table_result_god_setup,
        assets::emblem::asset_table_robin_hook,
        script::event_sequence_map_opening_hook,
        assets::transform::change_dragon2,
        assets::transform::tranformation_chain_atk,
        randomizer::emblem::arena_emblem_weapon,
        randomizer::bgm::special_bgm_hook,
        talk::get_cmd_info_from_cmd_lines_hook,
        talk::get_active_character_hook,
        message::mess_get_impl_hook, 
        randomizer::emblem::on_deserialize,
        randomizer::person::get_bond_face,  
        randomizer::person::get_thumb_face,
        randomizer::person::get_god_face,
        randomizer::person::get_god_thumb_face,
        assets::emblem::asset_table_result_get_preset_name,
        talk::talk_ptr,
        talk::calculate_str_width,
        randomizer::skill::learn::unit_learn_job_skill_hook,
        randomizer::skill::learn::class_change_job_menu_content_hook,
    ); 
    Patch::in_text(0x01bb272c).nop().unwrap(); // AssetTableResult Setup Prevents Commit
    crate::misc::code_patches();
    menus::install_vibe();
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
