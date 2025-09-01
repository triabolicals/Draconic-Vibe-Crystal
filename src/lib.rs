#![feature(ptr_sub_ptr)]
use cobapi::{Event, SystemEvent};
use std::sync::{LazyLock, Mutex};
use engage::proc::desc::ProcDesc;
use engage::proc::ProcVoidFunction;
use crate::config::*;
use skyline::patching::Patch;
use crate::script::map_opening_proc_edit;
#[allow(static_mut_refs)]pub mod config;
#[allow(static_mut_refs)] pub mod randomizer;
#[allow(static_mut_refs)] pub mod utils;
#[allow(static_mut_refs)] pub mod autolevel;
#[allow(static_mut_refs)] pub mod ironman;
#[allow(static_mut_refs)] pub mod enums;
#[allow(static_mut_refs)] pub mod continuous;
#[allow(static_mut_refs)] pub mod message;
#[allow(static_mut_refs)] pub mod menus;
#[allow(static_mut_refs)] pub mod deployment;
#[allow(static_mut_refs)] pub mod talk;
#[allow(static_mut_refs)] pub mod events;
#[allow(static_mut_refs)] pub mod script;
#[allow(static_mut_refs)] pub mod misc;
#[allow(static_mut_refs)] pub mod assets;
#[allow(static_mut_refs)] mod sprite;

pub static CONFIG: LazyLock<Mutex<DeploymentConfig>> = LazyLock::new(|| DeploymentConfig::new().into() );
pub const VERSION: &str = "2.13.5";
extern "C" fn event_install(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                /*
                if CONFIG.lock().unwrap().debug {
                    println!("Proc: {} Label: {}", proc.name.unwrap(), *label);
                }
                 */
                map_opening_proc_edit();
                if events::PROC_CHECK.iter().any(|&x| x == proc.hashcode) {
                    let proc_label = *label;
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
            SystemEvent::ProcInstBind { proc, parent: _} => {
                let mut proc = proc.borrow_mut();
                let hashcode = proc.hashcode;
                // println!("Create DVC Proc Bind: {}, {}", hashcode, proc.name.unwrap());
                match hashcode {
                    engage::proc::WELL_SEQUENCE => {
                        (*proc.descs.get_mut())[19] = ProcDesc::call(
                            ProcVoidFunction::new(
                                None,
                                crate::randomizer::item::well::well_get_item_rng)
                        );
                    }
                    engage::proc::MAP_UNIT_COMMAND_MENU => {
                        crate::assets::accmenu::map_unit_command_accessory(&mut proc);
                    }
                    engage::proc::MAP_SYSTEM_MENU => {
                        crate::menus::ingame::map_system_add_dvc_add(&mut proc);
                    }
                    engage::proc::MAP_SEQUENCE_BATTLE => {
                        proc.descs.get_mut()[60] =
                            ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::map_sequence_battle_pre_bgm));
                    }
                    engage::proc::MAP_SEQUENCE_BATTLE_ACTION => {
                        proc.descs.get_mut()[1] =
                            ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::map_sequence_battle_action_pre_bgm));
                    }
                    engage::proc::COMBAT_COMBAT_SEQUENCE => {
                        proc.descs.get_mut()[4] =
                            ProcDesc::call(ProcVoidFunction::new(None, randomizer::bgm::combat_sequence_pre_bgm));
                    }
                    _ => {}
                }
            }
            _ => {},
        }
    } 
    else { }
}
#[skyline::main(name = "vibe")]
pub fn main() {
    let _ = std::fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    cobapi::register_system_event_handler(event_install);
    println!("Draconic Vibe Crystal v{}", VERSION);
    cobapi::install_lua_command_registerer(randomizer::map::register_script_commands);
    CONFIG.lock().unwrap().save();
    assets::get_accessory_count();
    skyline::install_hooks!(
        sprite::facethumbnail_getpath_god,
        sprite::unit_icon_set_god_icon,
        randomizer::job::reclass::add_job_list_unit,
        deployment::create_player_team, 
        script::script_get_string,
        randomizer::person::unit::unit_create_impl_2_hook, 
        assets::asset_table_result_setup_hook,
        assets::emblem::asset_table_result_god_setup,
        assets::emblem::asset_table_robin_hook,
        sprite::try_get_sprite,
        assets::transform::change_dragon2,
        assets::transform::transformation_chain_atk,
        randomizer::emblem::arena_emblem_weapon,
        talk::get_cmd_info_from_cmd_lines_hook,
        talk::get_active_character_hook,
        message::mess_get_impl_hook, 
        randomizer::emblem::on_deserialize,
        sprite::get_bond_face,
        sprite::get_god_face,
        randomizer::person_sound,
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