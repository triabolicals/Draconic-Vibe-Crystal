#![feature(lazy_cell, ptr_sub_ptr)]
use cobapi::{Event, SystemEvent};
use std::sync::{Mutex, LazyLock};
use serde::{Deserialize, Serialize};
use skyline::patching::Patch;
use engage::gamedata::*;
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
pub mod enums;
pub mod continuous;
pub mod message;

pub const VERSION: &str = "Pre-2.0.0";

#[derive(Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    randomized: bool,
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
    engrave_settings: i32,
    engrave_lower_score: i32,
    engrave_upper_score: i32,
    engage_link: bool,
    exploration_items: i32,
    replaced_item_price: i32,
    autolevel: bool,
    iron_man: bool,
    deployment_type: i32,
    emblem_deployment: i32,
    emblem_mode: i32,
    continuous: i32,
    random_recruitment: i32,
    random_job: i32,
    random_skill: bool,
    random_item: i32,
    random_grow: i32,
    random_god_mode: i32,
    random_god_sync_mode: i32,
    emblem_skill_chaos: i32,
    random_engage_weapon: bool,
    random_gift_items: i32,
    random_shop_items: bool,
    random_battle_styles: bool,
}

fn disable_support_restriction() {
    let replace = &[0x1f, 0x25, 0x00, 0x71];
    Patch::in_text(0x0209950C).bytes(replace).unwrap();
    Patch::in_text(0x020994E0).bytes(replace).unwrap();
    Patch::in_text(0x02099538).bytes(replace).unwrap();
    Patch::in_text(0x01a2a7c0).bytes(&[0xe1,0x0e,0x80,0x12]).unwrap();
    Patch::in_text(0x01a2a7c4).bytes(&[0x02,0x0f,0x80,0x52]).unwrap();
    Patch::in_text(0x01fdea34).bytes(&[0x01,0x04,0x80, 0x52]).unwrap();
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
            randomized: true,
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
            engrave_settings: 0,
            engrave_lower_score: -10,
            engrave_upper_score: 30,
            engage_link: false,
            autolevel: false,
            exploration_items: 0,
            replaced_item_price: 75,
            iron_man: false,
            deployment_type: 0,
            emblem_deployment: 0,
            emblem_mode: 0,
            continuous: 0,
            random_recruitment: 0,
            random_job: 0,
            random_skill: false,
            random_item: 0,
            random_grow: 0,
            random_god_mode: 0,
            random_god_sync_mode: 0,
            emblem_skill_chaos: 0,
            random_engage_weapon: false,
            random_gift_items: 0,
            random_shop_items: false,
            random_battle_styles: false,
        };
        config
    }
    fn correct_rates(&mut self) {
        self.draconic_vibe_version = VERSION.to_string();
            self.random_enemy_skill_rate = crate::utils::clamp_value(self.random_enemy_skill_rate, 0, 100);
            self.random_enemy_job_rate = crate::utils::clamp_value(self.random_enemy_job_rate, 0, 100);
            self.replaced_item_price = crate::utils::clamp_value(self.replaced_item_price, 0, 100);
            self.revival_stone_rate = crate::utils::clamp_value(self.revival_stone_rate, 0, 100);
            self.bond_ring_skill_s_rate = crate::utils::clamp_value(self.bond_ring_skill_s_rate, 0, 100);
            self.bond_ring_skill_a_rate = crate::utils::clamp_value(self.bond_ring_skill_a_rate, 0, 100);
            self.bond_ring_skill_b_rate = crate::utils::clamp_value(self.bond_ring_skill_b_rate, 0, 100);
            self.bond_ring_skill_c_rate = crate::utils::clamp_value(self.bond_ring_skill_c_rate, 0, 100);
    }
    pub fn get_engrave_limits(&mut self) -> (i32, i32, bool) {
        // auto correct 
        let a = crate::utils::clamp_value(self.engrave_lower_score, -100, 100);
        let b = crate::utils::clamp_value(self.engrave_upper_score, -100, 100);
        self.engrave_lower_score = a;
        self.engrave_upper_score = b;
        if a == b {
            self.save();
            return (0, 0, false);
        }
        else if a < b {
            if b-a < 30 {  
                self.engrave_lower_score = crate::utils::clamp_value(b-30, -100, 100);
                self.engrave_upper_score = b;
            }
        }
        else {
            if a-b < 30 {
                self.engrave_lower_score = crate::utils::clamp_value(a-30, -100, 100);
                self.engrave_upper_score = a;
            }
            else {
                self.engrave_lower_score = b;
                self.engrave_upper_score = a;
            }
        }
        self.save();
        println!("Engage Lower {}, Higher {}", self.engrave_lower_score, self.engrave_upper_score);
        return (self.engrave_lower_score, self.engrave_upper_score, true);
    }   
    pub fn get_bond_ring_rates(&self) -> [i32; 4] { return [self.bond_ring_skill_s_rate, self.bond_ring_skill_a_rate, self.bond_ring_skill_b_rate, self.bond_ring_skill_c_rate ]; }
    pub fn save(&self) {
        let out_toml = toml::to_string_pretty(&self).unwrap();
        std::fs::write("sd:/engage/config/triabolical.toml", out_toml).expect("should be able to write to write default configuration");
    }
}
pub static CONFIG: LazyLock<Mutex<DeploymentConfig>> = LazyLock::new(|| DeploymentConfig::new().into() );

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
                if proc.name.is_some() {  println!("Proc: {}, Hash {}, label {}", proc.name.unwrap().get_string().unwrap(), proc.hashcode, label); }
                if proc.hashcode == 1650205480 && *label == 17 {    // map ending
                    person::m011_ivy_recruitment_check();
                    continuous::continous_mode_post_battle_stuff(proc); 
                    continuous::update_ignots();
                    deploy::unit_status();
                }
                if proc.hashcode == -988690862 && *label == 0 { // On Initial Title Screen Load
                    message::replace_hub_fxn();
                    enums::generate_black_list();
                    emblem::get_recommended_paralogue_levels();
                    asset::add_animation_unique_classes();
                    item::create_item_pool();
                    person::get_playable_list();
                    bgm::get_bgm_pool();
                    skill::create_skill_pool();
                    emblem::get_engrave_stats();
                    emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().intialize_list();
                    set_personal_caps();
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
                    continuous::do_continious_mode();
                    random::randomize_stuff(); 
                    autolevel::update_learn_skills();
                    continuous::update_next_chapter();
                    set_personal_caps();
                    ironman::ironman_code_edits();
                    deploy::unit_status();
                }
                if proc.hashcode == -1912552174 && *label == 28 { random::start_new_game();  }
                // when map starts, iron code edits activate
                if proc.hashcode == -339912801 && *label == 12 { 
                    ironman::ironman_code_edits();
                    shop::randomize_hub_random_items();
                    autolevel::calculate_player_cap(); 
                    continuous::update_bonds();
                    unsafe { crate::enums::LUEUR_CHANGE = true; }
                }
                if proc.hashcode == -1624221522 && *label == 14 { // sortie sequence ends
                    bgm::randomize_bgm_map(); 
                    deploy::adjust_map_inspectors();
                }
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
        asset::add_job_list_unit, 
        random::try_get_index, 
        deploy::create_player_team, 
        message::script_get_string,
        person::unit_create_impl_2_hook, 
        person::create_from_dispos_hook,
        asset::asset_table_result_setup_hook,
        message::mess_get_impl_hook, 
    ); 
    // Fixes the emblem weapons arena issue
    Patch::in_text(0x01ca9afc).nop().unwrap();
    random::install_vibe();
    disable_support_restriction();
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