use serde::{Deserialize, Serialize};
use super::VERSION;

#[derive(Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub randomized: bool,
    pub draconic_vibe_version: String,
    pub seed: u32,
    pub random_enemy_job_rate: i32,
    pub random_enemy_skill_rate: i32,
    pub revival_stone_rate: i32,
    pub enemy_emblem_rate: i32,
    pub random_map_bgm: bool,
    pub bond_ring_skill_s_rate: i32,
    pub bond_ring_skill_a_rate: i32,
    pub bond_ring_skill_b_rate: i32,
    pub bond_ring_skill_c_rate: i32,
    pub engrave_settings: i32,
    pub engrave_lower_score: i32,
    pub engrave_upper_score: i32,
    pub engage_link: bool,
    pub exploration_items: i32,
    pub replaced_item_price: i32,
    pub enemy_drop_rate: i32,
    pub autolevel: bool,
    pub iron_man: bool,
    pub deployment_type: i32,
    pub emblem_deployment: i32,
    pub emblem_mode: i32,
    pub continuous: i32,
    pub random_recruitment: i32,
    pub random_job: i32,
    pub random_skill: bool,
    pub random_item: i32,
    pub random_grow: i32,
    pub random_god_mode: i32,
    pub random_god_sync_mode: i32,
    pub emblem_skill_chaos: i32,
    pub emblem_weap_prof_mode: i32,
    pub random_engage_weapon: bool,
    pub random_gift_items: i32,
    pub interaction_type: i32,
    pub random_shop_items: bool,
    pub random_battle_styles: i32,
    pub change_unit_offset: bool,
    pub random_names: bool,
    pub custom_units: bool,
    pub apply_rando_post_new_game: bool,
    pub auto_adjust_asset_table: bool,
    pub misc_option_1 : f32,
    pub misc_option_2 : f32,
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
            enemy_drop_rate: 10,
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
            emblem_weap_prof_mode: 0,
            emblem_skill_chaos: 0,
            random_engage_weapon: false,
            random_gift_items: 0,
            interaction_type: 0,
            random_shop_items: false,
            random_battle_styles: 0,
            change_unit_offset: true,
            random_names: false,
            apply_rando_post_new_game: false,
            auto_adjust_asset_table: false,
            custom_units: false,
            misc_option_1 : 0.0,
            misc_option_2 : 1.0,
        };
        config
    }

    pub fn correct_rates(&mut self) {
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

    pub fn get_bond_ring_rates(&self) -> [i32; 4] { 
        return [self.bond_ring_skill_s_rate, self.bond_ring_skill_a_rate, self.bond_ring_skill_b_rate, self.bond_ring_skill_c_rate ];
    }

    pub fn save(&self) {
        let out_toml = toml::to_string_pretty(&self).unwrap();
        std::fs::write("sd:/engage/config/triabolical.toml", out_toml).expect("should be able to write to write default configuration");
    }
}