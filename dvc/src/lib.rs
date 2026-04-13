use cobapi::{Event, SystemEvent};
use skyline::patching::Patch;
pub use outfit_core::*;
pub use config::DeploymentConfig;

#[allow(static_mut_refs, warnings)] pub mod config;
#[allow(static_mut_refs, warnings)] pub mod randomizer;
#[allow(static_mut_refs, warnings)] pub mod utils;
#[allow(static_mut_refs)] pub mod autolevel;
#[allow(static_mut_refs)] pub mod ironman;
#[allow(static_mut_refs)] pub mod enums;
#[allow(static_mut_refs)] pub mod continuous;
#[allow(static_mut_refs)] pub mod message;
#[allow(static_mut_refs, warnings)] pub mod menus;
#[allow(static_mut_refs)] pub mod deployment;
#[allow(static_mut_refs)] pub mod talk;
#[allow(static_mut_refs)] pub mod events;
#[allow(static_mut_refs)] pub mod script;
#[allow(static_mut_refs)] pub mod misc;
#[allow(static_mut_refs, warnings)] pub mod assets;
#[allow(static_mut_refs)] mod sprite;
mod procs;

pub use config::variables::*;
use crate::utils::return_true;

pub static mut CONFIG: DeploymentConfig = DeploymentConfig::default();
pub const VERSION: &str = "2.16.0H";

extern "C" fn event_install(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                println!("Proc: {}, Label: {}", proc.name.unwrap(), *label);
                let proc_label = *label;
                match proc.hashcode {
                    events::TITLE_SEQUENCE => { events::title_loop_events(proc, proc_label); }
                    events::MAINMENU_SEQUENCE => { events::main_menu_sequence_events(proc, proc_label);}
                    events::MAIN_SEQUENCE => { events::main_sequence_events(proc, proc_label); }
                    events::MAP_SEQUENCE => { events::map_sequence_events(proc, proc_label); }
                    events::SORTIE_SEQUENCE => { events::sortie_sequence_events(proc, proc_label); }
                    events::PROC_SCENE => { events::proc_scene_event(proc, proc_label); }
                    _ => {}
                }
            }
            SystemEvent::SaveLoaded { ty, slot_id: _ } => {
                if *ty > 1 { randomizer::save_file_load(); }
            }
            SystemEvent::ProcInstBind { proc, parent} => {
                let mut proc = proc.borrow_mut();
                let hashcode = proc.hashcode;
                if let Some(parent) = parent.borrow().as_ref() {
                    println!("Parent: {} [Desc/Label: {}/{}] Create Bind: {} ({})", parent.name.unwrap(), parent.desc_index, parent.get_label(), hashcode, proc.name.unwrap());
                }
                procs::proc_bind_desc_edit(&mut proc);
            }
            _ => {}
        }
    } 
    else { println!("NO EVENT :("); }
}

#[skyline::main(name = "vibe")]
pub fn main() {
    println!("Draconic Vibe Crystal v{}", VERSION);
    unsafe { CONFIG = DeploymentConfig::new(); }
    UnitAssetMenuData::get().is_dvc = true;
    DeploymentConfig::get().save();
    cobapi::register_system_event_handler(event_install);
    cobapi::install_lua_command_registerer(randomizer::map::register_script_commands);
    cobapi::install_lua_command_registerer(script::chapter::install_script_edits);
    Patch::in_text(0x02517830).bytes(&[0xa0, 0x02, 0x80, 0x52]).unwrap();
    Patch::in_text(0x0251a9c0).bytes(&[0x01, 0x00, 0x84, 0x52]).unwrap();
    Patch::in_text(0x024cba50).bytes(&[0x01, 0x00, 0x85, 0x52]).unwrap();


    // Patch::in_text(0x1ccd53c).bytes(&[0x68, 0, 0x80, 0x52]).unwrap();   // GmapSequence Jump to 3
    return_true(0x0203e1b0);    // Forging Item Display
    skyline::install_hooks!(
        // sprite::facethumbnail_getpath_god,
        // talk::get_cmd_info_from_cmd_lines_hook,
        sprite::unit_icon_set_god_icon,
        script::script_get_string,
        randomizer::person::unit::unit_create_impl_2_hook,
        message::mess_get_impl_hook,
        sprite::try_get_sprite,
        randomizer::item::calc_reward,
        randomizer::person::unit_pool_get_from_person,
        // sprite::get_bond_face,
        // sprite::get_god_face,
        // randomizer::person_sound,
        assets::engage_attack::combat_record_post_process,
        randomizer::emblem::god_bond_holder_get,
        randomizer::item::item_refine_data_try_get,
        randomizer::skill::learn::unit_learn_job_skill_hook,
        randomizer::item::shop::item_buy_item_create_menu_item,
        randomizer::item::shop::weapon_buy_item_create_menu_item,
        randomizer::emblem::menu::ring_list_skill_menu_create_menu_items,
        randomizer::emblem::menu::skill_inheritance_menu_create_menu_item_list,

        message::mess_add_tag_to_string,
        message::talk::mess_load,
    );
    misc::code_patches();
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