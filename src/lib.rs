#![feature(lazy_cell, ptr_sub_ptr)]
use unity::prelude::*;
use cobapi::{Event, SystemEvent};
use std::sync::{Mutex, LazyLock};
use serde::{Deserialize, Serialize};

pub mod deploy;
pub mod emblem;
pub mod item;
pub mod person;
pub mod random;
pub mod skill;
pub mod grow;
pub mod utils;
pub mod bgm;
pub mod autolevel;
pub mod asset;
pub mod shop;
pub mod ironman;

pub const VERSION: &str = "1.8.3";

#[derive(Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    add_new_settings: bool,
    draconic_vibe_version: String,
    seed: u32,
    random_enemy_job_rate: i32,
    random_enemy_skill_rate: i32,
    revival_stone_rate: i32,
    enemy_emblem_rate: i32,
    random_map_bgm: bool,
    bond_ring_skill_s_rate: i32,
    bond_ring_skill_a_rate: i32,
    bond_ring_skill_b_rate: i32,
    bond_ring_skill_c_rate: i32,
    engage_link: bool,
    autolevel: bool,
    iron_man: bool,
    deployment_type: i32,
    emblem_deployment: i32,
    emblem_mode: i32,
    random_recruitment: bool,
    random_job: i32,
    random_skill: bool,
    random_item: i32,
    random_grow: i32,
    random_god_mode: i32,
    random_god_sync_mode: i32,
    random_engage_weapon: bool,
    random_gift_items: i32,
    random_shop_items: bool,
}

impl DeploymentConfig {
    pub fn new() -> Self {
        let config_content = std::fs::read_to_string("sd:/engage/config/triabolical.toml");
        // If the file is read to a string or there is no failure, parse into the config struct.
        if config_content.is_ok() {
            let content = config_content.unwrap();
            let config = toml::from_str(&content);
            if config.is_ok() {
                println!("Triabolical Config file was parsed with no issues.");
                let config = config.unwrap();
                config
            } else {
                // This is mostly intended to create a new file if more items are added to the struct
                println!("Triabolical Config: Config file could not be parsed or new settings are added.\nNew default config file has been created.");
                let config = DeploymentConfig::default();
                config.save();
                config
            }
        } else {
            // If the file could not be read to a string then create a new file with default values.
            println!("Triabolical Config: The config file was either missing or unable to be read, creating new toml.");
            let config = DeploymentConfig::default();
            config.save();
            config
        }
    }
    pub fn default() -> Self {
        let config = DeploymentConfig  {
            add_new_settings: false,
            draconic_vibe_version: VERSION.to_string(),
            seed: 0,
            random_enemy_job_rate: 50,
            random_enemy_skill_rate: 50,
            revival_stone_rate: 0,
            enemy_emblem_rate: 0,
            random_map_bgm: false,
            bond_ring_skill_s_rate: 100,
            bond_ring_skill_a_rate: 25,
            bond_ring_skill_b_rate: 10,
            bond_ring_skill_c_rate: 5,
            engage_link: false,
            autolevel: false,
            iron_man: false,
            deployment_type: 0,
            emblem_deployment: 0,
            emblem_mode: 0,
            random_recruitment: false,
            random_job: 0,
            random_skill: false,
            random_item: 0,
            random_grow: 0,
            random_god_mode: 0,
            random_god_sync_mode: 0,
            random_engage_weapon: false,
            random_gift_items: 0,
            random_shop_items: false,
        };
        config
    }
    fn correct_rates(&mut self) {
        self.draconic_vibe_version = VERSION.to_string();
        unsafe {
            self.random_enemy_skill_rate = clamp(self.random_enemy_skill_rate, 0, 100, None);
            self.random_enemy_job_rate = clamp(self.random_enemy_job_rate, 0, 100, None);
            self.revival_stone_rate = clamp(self.revival_stone_rate, 0, 500, None);
            self.bond_ring_skill_s_rate = clamp(self.bond_ring_skill_s_rate, 0, 100, None);
            self.bond_ring_skill_a_rate = clamp(self.bond_ring_skill_a_rate, 0, 100, None);
            self.bond_ring_skill_b_rate = clamp(self.bond_ring_skill_b_rate, 0, 100, None);
            self.bond_ring_skill_c_rate = clamp(self.bond_ring_skill_c_rate, 0, 100, None);
        }
    }
    pub fn get_bond_ring_rates(&self) -> [i32; 4] {
        return [self.bond_ring_skill_s_rate, self.bond_ring_skill_a_rate, self.bond_ring_skill_b_rate, self.bond_ring_skill_c_rate ];
    }
    pub fn save(&self) {
        let out_toml = toml::to_string_pretty(&self).unwrap();
        std::fs::write("sd:/engage/config/triabolical.toml", out_toml).expect("should be able to write to write default configuration");
    }
}
pub static CONFIG: LazyLock<Mutex<DeploymentConfig>> = LazyLock::new(|| DeploymentConfig::new().into() );

extern "C" fn initalize_random_persons(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstJump {proc, label } => {
                if proc.name.is_some() { 
                    println!("Proc: {}, Hash {}, label {}", proc.name.unwrap().get_string().unwrap(), proc.hashcode, label);
                }
                if proc.hashcode == -988690862 && *label == 0 {
                    //asset::get_job_assets();
                    utils::dlc_check();
                    person::get_playable_list();
                    bgm::get_bgm_pool();
                    skill::create_skill_pool();
                    asset::unlock_royal_classes();
                    item::ENGAGE_ITEMS.lock().unwrap().intialize_list();
                    unsafe {
                        for i in 0..41 { 
                            person::RAND_PERSONS[i as usize] = i; 
                            person::RAND_PERSONS[41 + i as usize] = i; 
                        }
                    }
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
                if proc.hashcode == -339912801 && *label == 2 { random::reset_gamedata(); }
                // randomized stuff
                if proc.hashcode == -1118443598 && *label == 0 { 
                    random::randomize_stuff(); 
                }
                if proc.hashcode == -1912552174 && *label == 28 {
                    random::start_new_game(); 
                }
                // when map starts, iron code edits activate
                if proc.hashcode == -339912801 && *label == 12 { 
                    ironman::ironman_code_edits();
                    autolevel::calculate_player_cap(); 
                }
                if proc.hashcode == -1624221522 && *label == 14 { bgm::randomize_bgm_map(); }
            }
            _ => {},
        }
    } 
    else {  println!("We received a missing event, and we don't care!"); }
}

#[skyline::main(name = "deployment")]
pub fn main() {
    let _ = std::fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    //Deployment
    cobapi::register_system_event_handler(initalize_random_persons);
    //skyline::install_hooks!( person::talk_hook, person::cmd_info_ctor_hook);
    //skyline::install_hooks!( ironman::game_mode_bind, ironman::game_over_hook, ironman::set_last_save_data_info);
    skyline::install_hooks!( asset::add_job_list_unit, person:: mess_get_impl_hook, random::try_get_index, deploy::create_player_team, random::script_get_string, person::unit_create_impl_2_hook, person::create_from_dispos_hook); 
    random::install_vibe();

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
            "Oh no! Plugin has panicked at '{}' with the following message:\n{}\0",
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

#[skyline::from_offset(0x032dfb20)]
pub fn clamp(value: i32, min: i32, max: i32, method_info: OptionalMethod) -> i32;