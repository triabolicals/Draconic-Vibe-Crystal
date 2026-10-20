#![feature(string_replace_in_place)]

use skyline::patching::Patch;
pub use engage::{app::{IMess, IMessMethods}, prelude::*};
pub use unity::{system::string::*, prelude::*};
pub use config::{*, DVCVariable, DVCVariables, DVCFlags};

mod config;
mod utils;
mod gamedata;
mod dvc;
mod enums;
mod patches;
mod sequences;
pub mod menus;

pub static mut CONFIG: DVCConfig = DVCConfig::default();
pub const VERSION: &str = "2.17.1.4";
pub const VARIABLE_VERSION: i32 = 10;

#[skyline::main(name = "vibe")]
pub fn main() {
    unsafe { CONFIG = DVCConfig::new(); }
    DVCConfig::get().save();
    Patch::in_text(0x02517830).bytes(&[0xa0, 0x02, 0x80, 0x52]).unwrap();
    Patch::in_text(0x0251a9c0).bytes(&[0x01, 0x00, 0x84, 0x52]).unwrap();
    Patch::in_text(0x024cba50).bytes(&[0x01, 0x00, 0x85, 0x52]).unwrap();
    #[cfg(feature = "debug")]{
        Patch::in_text(0x0228151c).bytes(&[0x8A, 0x0C, 0x80, 0x52]).unwrap();
        Patch::in_text(0x02281fb8).bytes(&[0x88, 0x0C, 0x80, 0x52]).unwrap();
    }
    dvc::DVC::install();
    skyline::install_hooks!(
        dvc::script::script_get_string,
        dvc::mess::god_unit_get_name,
        dvc::randomizer::shop::item_buy_item_create_menu_item,
        dvc::randomizer::shop::weapon_buy_item_create_menu_item,
        dvc::randomizer::inherit::skill_inheritance_menu_create_menu_item_list,
        dvc::randomizer::godbond::god_bond_holder_get,
        dvc::asset::sprite::unit_icon_set_god_icon,
        dvc::asset::sprite::unit_icon_set_icon,
        dvc::asset::color::modify_colors,
        dvc::asset::engage_attack::combat_record_post_process,
        dvc::asset::appearance_create_from_result_2,
        dvc::talk::talk_tag_parser_parse,
        dvc::mess::add_tag_string,
        dvc::randomizer::hub::hub_access_data_confirm_content,
        dvc::randomizer::inherit::ring_list_skill_menu_create_menu_items,
        dvc::randomizer::reclass::class_change_job_menu_set_unit_info_all,
    );
    println!("Draconic Vibe Crystal v{}", VERSION);
    patches::code_patches();
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