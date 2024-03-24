#![feature(lazy_cell, ptr_sub_ptr)]
use cobapi::{Event, SystemEvent};
use std::sync::{Mutex, LazyLock};
use serde::{Deserialize, Serialize};
use engage::gameuserdata::*;
use engage::gamevariable::GameVariableManager;

pub mod deploy;
pub mod emblem;
pub mod item;
pub mod person;
pub mod random;
pub mod ironman;
pub mod skill;
pub mod grow;
pub const VERSION: &str = "1.0.2";

#[derive(Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    deployment_type: i32,
    emblem_deployment: i32,
    iron_man: bool,
    emblem_mode: i32,
    seed: u32,
    random_recruitment: bool,
    random_job: bool,
    random_skill: bool,
    random_grow: i32,
    random_god_mode: i32,
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
                println!("Triabolical Config: Config file could not be parsed, a default config file has been created.");
                let config = DeploymentConfig::default();
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
            deployment_type: 0,
            emblem_deployment: 0,
            iron_man: false,
            emblem_mode: 0,
            seed: 0,
            random_recruitment: false,
            random_job: false,
            random_skill: false,
            random_grow: 0,
            random_god_mode: 0,
        };
        config
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
            SystemEvent::LanguageChanged => {
                skill::create_skill_pool();
                unsafe {
                    for i in 0..41 { 
                        person::RAND_PERSONS[i as usize] = i; 
                        person::RAND_PERSONS[41 + i as usize] = i; 
                    }
                }
                
            },
            SystemEvent::ProcInstJump {proc, label } => {
                if proc.name.is_some() {
                   println!("Proc: {}, Hash {}, label {}", proc.name.unwrap().get_string().unwrap(), proc.hashcode, label);
                }
                //Reset things
                if proc.hashcode == -339912801 && *label == 2 { random::reset_gamedata(); }
                // randomized stuff
                if proc.hashcode == -1118443598 && *label == 0 { 
                    random::randomize_stuff(); 
                    ironman::ironman_code_edits();
                }
                // Game start and classic mode with iron man = true
                if proc.hashcode == -1912552174 && *label == 28 {
                    random::start_new_game(); 
                    if GameUserData::get_game_mode() == GameMode::Classic && CONFIG.lock().unwrap().iron_man { 
                        GameVariableManager::make_entry("G_Ironman", 1);
                    }
                }
                // when map starts, iron code edits activate
                if proc.hashcode == -339912801 && *label == 12 { ironman::ironman_code_edits(); }

                if proc.hashcode == -881910643 {    //TalkSequence
                    return;
                }
            }
            _ => {},
        }
    } 
    else {  println!("We received a missing event, and we don't care!"); }
}


#[skyline::main(name = "deployment")]
pub fn main() {
    //Deployment
    cobapi::register_system_event_handler(initalize_random_persons);
    //skyline::install_hooks!( person::talk_hook, person::cmd_info_ctor_hook);
    skyline::install_hooks!( ironman::set_tip_text, ironman::game_mode_bind, ironman::game_over_hook, ironman::set_last_save_data_info);
    skyline::install_hooks!( person:: mess_get_impl_hook, random::try_get_index, deploy::create_player_team, random::script_get_string, person::unit_create_impl_2_hook, person::create_from_dispos_hook); 
    random::install_vibe();
    /*
    deploy::install_deployment();
    emblem::install_rng_emblems();
    person::install_rng_person();
    item::install_rnd_jobs();
    skill::install_skill_rnd();
    grow::install_rng_grow();
    */
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
            "Plugin has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );
        skyline::error::show_error(
            4,
            "Randomizer/Deployment Plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
}