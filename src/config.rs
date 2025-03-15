use serde::{Deserialize, Serialize};
use unity::prelude::*;
use super::VERSION;
use engage::{
    gameuserdata::GameUserData, gamevariable::*,
    gamedata::*,
};
use crate::{enums::*, utils};

/// Structure that contains and manages DVC-Related GameVariables
pub struct DVCVariables {}

impl DVCVariables {
    pub const SEED: &str = "G_Random_Seed";

    pub const LUEUR_NAME: &str = "G_Lueur_Name";
    pub const LUEUR_GENDER: &str = "G_Lueur_Gender2";
    pub const LUEUR_RANDOM_JOB_KEY: &str =  "G_Lueur_Random";
    pub const MISERCODE_TYPE: &str = "G_Misercode_Type";
    pub const LIBERATION_TYPE: &str = "G_Liberation_Type";

    pub const GENERIC_APPEARANCE_KEY: &str =  "G_GenericMode";
    pub const ENEMY_OUTFIT_KEY: &str =  "G_EnemyOutfits";
    pub const PLAYER_OUTFIT_KEY: &str =  "G_PlayerOutfit";
    pub const ASSETS: &str = "G_RandAsset";
    pub const BGM_KEY: &str =  "G_RandomBGM";

    pub const DEPLOYMENT_KEY: &str =  "G_DeploymentMode";
    pub const EMBLEM_DEPLOYMENT_KEY: &str =  "G_EmblemDeployMode";

    pub const HUB_ITEM_KEY: &str =  "G_HubItem";
    pub const ITEM_GAUGE_KEY: &str =  "G_ItemGauge";
    pub const SHOP_KEY: &str =  "G_Random_Shop_Items";
    pub const ITEM_KEY: &str =  "G_Random_Item";
    pub const ITEM_DROP_GAUGE_KEY: &str =  "G_ItemDropGauge";
    pub const PLAYER_INVENTORY: &str = "G_PRW";
    pub const SKILL_KEY: &str =  "G_Random_Skills";
    pub const ENEMY_SKILL_GUAGE_KEY: &str =  "G_EnemySkillGauge";

    pub const CUSTOM_JOB_KEY: &str =  "G_CJobs";
    pub const JOB_LEARN_SKILL_KEY: &str =  "G_LearnMode";
    pub const ENEMY_JOB_GAUGE_KEY: &str =  "G_EnemyJobGauge";
    pub const RECLASS_KEY: &str =  "G_RandomCC";
    pub const JOB_KEY: &str =  "G_Random_Job";

//Emblem Key
    pub const ENGAGE_P_KEY: &str =  "G_EngagePlus";
    pub const EMBLEM_SYNC_KEY: &str =  "G_Random_God_Sync";
    pub const EMBLEM_SKILL_CHAOS_KEY: &str =  "G_ChaosMode";
    pub const EMBLEM_SKILL_KEY: &str =  "G_Random_God_Mode";
    pub const EMBLEM_ITEM_KEY: &str =  "G_Random_Engage_Weps";
    pub const EMBLEM_NAME_KEY: &str =  "G_Random_Names";
    pub const WEAPON_PROF_KEY: &str =  "G_EmblemWepProf";
    pub const SP_KEY: &str =  "G_SPCost";

    pub const DVC_AUTOLEVEL_KEY: &str =  "G_DVC_Autolevel";
    pub const AUTOLEVEL_BENCH_KEY: &str =  "G_AutoBench";
    pub const REVIVAL_STONE_GAUGE_KEY: &str =  "G_EnemyRevivalStone";
    pub const ENEMY_EMBLEM_KEY: &str =  "G_EnemyEmblemGauge";

    pub const TERRAIN: &str = "G_RandomEnergy";

    pub const STYLES_KEY: &str =  "G_BattleStyles";
    pub const ENGRAVE_KEY: &str =  "G_EngraveSetting";
    pub const INTERACT_KEY: &str =  "G_InteractSetting";

    pub const RECRUITMENT_KEY: &str =  "G_Random_Recruitment";
    pub const EMBLEM_RECRUITMENT_KEY: &str =  "G_Emblem_Mode";
    pub const CUSTOM_UNIT_RECRUIT_DISABLE: &str =  "G_CUD";
    
    pub const GROWTH_KEY: &str =  "G_Random_Grow_Mode";
    pub const PERSONAL_GROWTH_KEY: &str =  "G_PGMode";

    pub const CONTINIOUS: &str = "G_Continuous";
    pub const FOW: &str = "G_FOW";

    pub const IRONMAN: &str = "G_Ironman";
    pub const PLAYER_AVERAGE_CAP: &str = "G_Player_Rating_Average";
    pub const EMBLEM_PARALOGUE_LEVEL: &str = "G_Paralogue_Level";
    pub const TILE: &str = "G_Tile";

    pub fn get_seed() -> i32 {  GameVariableManager::get_number(Self::SEED) }
    pub fn is_main_menu() -> bool { GameUserData::get_sequence() == 0 }
    pub fn random_enabled() -> bool { Self::get_seed() != 0 }

    pub fn get_dvc_person(pid_index: i32, reverse: bool) -> &'static Il2CppString {
        let key = 
            if reverse {format!("G_R_{}", PIDS[pid_index as usize]) }
            else { format!("G_R2_{}", PIDS[pid_index as usize]) };
        if GameVariableManager::exist(key.as_str()) { GameVariableManager::get_string(key.as_str()) }
        else { PIDS[pid_index as usize].into() }
    }

    pub fn get_dvc_emblem(index: i32, reverse: bool) -> &'static Il2CppString {
        let key = 
            if reverse {format!("G_R_{}", EMBLEM_GIDS[index as usize]) }
            else { format!("G_R2_{}", EMBLEM_GIDS[index as usize]) };
        if GameVariableManager::exist(key.as_str()) { GameVariableManager::get_string(key.as_str()) }
        else { EMBLEM_GIDS[index as usize].into() }
    }

    /// Non-Custom Emblems Only
    pub fn set_emblem_recruitment(emblem_index: i32, replace_emblem_index: i32) {
        if emblem_index > 18 || replace_emblem_index > 18 { return; }
        GameVariableManager::set_string(&format!("G_R_{}",EMBLEM_GIDS[emblem_index as usize]), EMBLEM_GIDS[replace_emblem_index as usize]);
        GameVariableManager::set_string(&format!("G_R2_{}",EMBLEM_GIDS[replace_emblem_index as usize]), EMBLEM_GIDS[emblem_index as usize]);
    }

    pub fn set_person_recruitment(pid_index: i32, replace_pid_index: i32) {
        if pid_index > 40 || replace_pid_index > 40 { return; }
        GameVariableManager::set_string(&format!("G_R_{}",PIDS[pid_index as usize]), PIDS[replace_pid_index as usize]);
        GameVariableManager::set_string(&format!("G_R2_{}",PIDS[replace_pid_index as usize]), PIDS[pid_index as usize]);
    }

    pub fn get_god_from_index(index: i32, randomized: bool) -> Option<&'static GodData> {
        if let Some(list) =  crate::randomizer::emblem::EMBLEM_LIST.get(){
            if index as usize >=  list.len() { return None; }
            let hash = list[index as usize];
            if GameVariableManager::get_number(Self::EMBLEM_RECRUITMENT_KEY) == 0 || !randomized { return GodData::try_get_hash(hash);  }
            else { 
                let key = format!("G_R_{}", GodData::try_get_hash(hash).unwrap().gid);
                return GodData::get(GameVariableManager::get_string(key));
            }
        }
        return None;
    }

    pub fn create_recruitment_variables() {
        if !GameVariableManager::get_bool("G_Random_Emblem_Set") {
            for i in 0..19 { 
                GameVariableManager::make_entry_str(&format!("G_R_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[i as usize]);
                GameVariableManager::make_entry_str(&format!("G_R2_{}",EMBLEM_GIDS[i as usize]), EMBLEM_GIDS[i as usize]);
            }
        }
        if !GameVariableManager::get_bool("G_Random_Person_Set") {
            for i in 0..41 { 
                GameVariableManager::make_entry_str(&format!("G_R_{}",PIDS[i as usize]), PIDS[i as usize]);
                GameVariableManager::make_entry_str(&format!("G_R2_{}",PIDS[i as usize]), PIDS[i as usize]);
            }
        }
    }
    pub fn is_random_map() -> bool { GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3 }

    pub fn changed_setting_text(var1: &str, var2: &str) -> String {
        if Self::is_main_menu() { "" }
        else {
            if GameVariableManager::get_number(var1) != GameVariableManager::get_number(var2) { "*" }
            else {  "" }
        }.to_string()
    }

    pub fn is_main_chapter_complete(main_index: i32) -> bool {
        GameVariableManager::get_bool(if main_index < 10 { format!("G_Cleared_M00{}", main_index) } else { format!("G_Cleared_M0{}", main_index) })
    }
}



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
    pub player_inventory: bool,
    pub random_recruitment: i32,
    pub random_job: i32,
    pub random_skill: bool,
    pub random_skill_cost: i32,
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
    pub generic_mode: i32,
    pub custom_units: bool,
    pub random_reclass: bool,
    pub apply_rando_post_new_game: bool,
    pub auto_adjust_asset_table: bool,
    pub enable_tradables_item: bool, 
    pub custom_jobs: bool,
    pub learn_skill: i32,
    pub debug: bool,
    pub dlc: i32,
    pub terrain: i32,
    pub fow: i32,
    pub assets: i32,
    pub player_growth: i32,
    pub max_stat_caps: bool,
    pub custom_unit_recruitment_disable: bool,
    pub tile: bool,
    pub misc_option_1 : f32,
    pub misc_option_2 : f32,
}

impl DeploymentConfig {
    
    pub fn new() -> Self {
        println!("Opening triabolical.toml");
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
            fow: 0,
            terrain: 0,
            player_inventory: false,
            random_recruitment: 0,
            random_job: 0,
            random_skill: false,
            random_skill_cost: 0,
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
            generic_mode: 0,
            random_names: false,
            random_reclass: false,
            apply_rando_post_new_game: false,
            auto_adjust_asset_table: false,
            custom_units: false,
            custom_jobs: false,
            learn_skill: 0,
            enable_tradables_item: false,
            debug: false,
            dlc: 0,
            player_growth: 0,
            assets: 0,
            custom_unit_recruitment_disable: false,
            tile: false,
            max_stat_caps: false,
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
        println!("Config Saved");
    }
    pub fn create_game_variables(&self, include_non_change: bool) {
        GameVariableManager::make_entry(DVCVariables::HUB_ITEM_KEY, self.exploration_items); 
        GameVariableManager::make_entry(DVCVariables::ENGAGE_P_KEY, self.engage_link as i32); 
        GameVariableManager::make_entry(DVCVariables::ENEMY_SKILL_GUAGE_KEY, self.random_enemy_skill_rate); 
        GameVariableManager::make_entry(DVCVariables::ENEMY_JOB_GAUGE_KEY, self.random_enemy_job_rate); 
        GameVariableManager::make_entry(DVCVariables::ENEMY_EMBLEM_KEY, self.enemy_emblem_rate); 
        GameVariableManager::make_entry(DVCVariables::DEPLOYMENT_KEY, self.deployment_type); 
        GameVariableManager::make_entry(DVCVariables::EMBLEM_DEPLOYMENT_KEY, self.emblem_deployment); 
        GameVariableManager::make_entry(DVCVariables::DVC_AUTOLEVEL_KEY, self.autolevel as i32); 
        GameVariableManager::make_entry(DVCVariables::BGM_KEY, self.random_map_bgm as i32 ); 
        GameVariableManager::make_entry(DVCVariables::REVIVAL_STONE_GAUGE_KEY, self.revival_stone_rate); 
        GameVariableManager::make_entry(DVCVariables::ITEM_GAUGE_KEY, self.replaced_item_price); 
        GameVariableManager::make_entry(DVCVariables::STYLES_KEY, self.random_battle_styles as i32);
        GameVariableManager::make_entry(DVCVariables::ENGRAVE_KEY, self.engrave_settings as i32); 
        GameVariableManager::make_entry(DVCVariables::INTERACT_KEY, self.interaction_type as i32);
        GameVariableManager::make_entry(DVCVariables::ITEM_DROP_GAUGE_KEY, self.enemy_drop_rate as i32);
        GameVariableManager::make_entry(DVCVariables::GENERIC_APPEARANCE_KEY, self.generic_mode as i32); 
        GameVariableManager::make_entry(DVCVariables::ENEMY_OUTFIT_KEY, 0);
        GameVariableManager::make_entry(DVCVariables::PLAYER_OUTFIT_KEY, 0);
        GameVariableManager::make_entry(DVCVariables::AUTOLEVEL_BENCH_KEY, 0);
        GameVariableManager::make_entry(DVCVariables::ASSETS, self.assets);
        GameVariableManager::make_entry(DVCVariables::CUSTOM_JOB_KEY, self.custom_jobs as i32);
        GameVariableManager::make_entry(DVCVariables::MISERCODE_TYPE, 0);
        GameVariableManager::make_entry(DVCVariables::LIBERATION_TYPE, 0);
        GameVariableManager::make_entry_norewind(DVCVariables::PLAYER_AVERAGE_CAP, 0);
        GameVariableManager::make_entry(DVCVariables::FOW, self.fow);
        GameVariableManager::make_entry(DVCVariables::TERRAIN, self.terrain);
        GameVariableManager::make_entry(DVCVariables::JOB_LEARN_SKILL_KEY, if GameVariableManager::exist(DVCVariables::SKILL_KEY) { 0 } else { self.learn_skill } );
        if crate::randomizer::person::PLAYABLE.get().unwrap().len() > 94 || self.custom_unit_recruitment_disable {
            GameVariableManager::make_entry(DVCVariables::CUSTOM_UNIT_RECRUIT_DISABLE, 1);
        }
        else {
            GameVariableManager::make_entry(DVCVariables::CUSTOM_UNIT_RECRUIT_DISABLE, 0);
        }
        GameVariableManager::make_entry_norewind(DVCVariables::PLAYER_INVENTORY, if self.random_item == 0 { 0 } else { self.player_inventory as i32} );
        GameVariableManager::make_entry_norewind(DVCVariables::PERSONAL_GROWTH_KEY, if self.random_grow & 1 != 0 { self.player_growth as i32 } else { 0 } );
        GameVariableManager::make_entry_norewind(DVCVariables::TILE, self.tile as i32);
        if include_non_change {
            GameVariableManager::make_entry(DVCVariables::WEAPON_PROF_KEY, self.emblem_weap_prof_mode as i32); 
            GameVariableManager::make_entry(DVCVariables::SHOP_KEY,  self.random_shop_items as i32 );
            GameVariableManager::make_entry(DVCVariables::EMBLEM_RECRUITMENT_KEY, self.emblem_mode as i32);
            GameVariableManager::make_entry(DVCVariables::RECRUITMENT_KEY, self.random_recruitment as i32);
            GameVariableManager::make_entry(DVCVariables::JOB_KEY, self.random_job as i32);
            GameVariableManager::make_entry(DVCVariables::LUEUR_RANDOM_JOB_KEY, 0);
            GameVariableManager::make_entry(DVCVariables::SKILL_KEY, self.random_skill as i32);
            GameVariableManager::make_entry(DVCVariables::GROWTH_KEY, self.random_grow as i32);
            GameVariableManager::make_entry(DVCVariables::EMBLEM_SKILL_KEY,  self.random_god_mode as i32);
            GameVariableManager::make_entry(DVCVariables::ITEM_KEY,  self.random_item as i32);
            GameVariableManager::make_entry(DVCVariables::EMBLEM_SYNC_KEY, self.random_god_sync_mode as i32);
            GameVariableManager::make_entry(DVCVariables::EMBLEM_SKILL_CHAOS_KEY, self.emblem_skill_chaos as i32);
            GameVariableManager::make_entry(DVCVariables::EMBLEM_ITEM_KEY, self.random_engage_weapon as i32);
            GameVariableManager::make_entry(DVCVariables::EMBLEM_NAME_KEY, self.random_names as i32);
            GameVariableManager::make_entry(DVCVariables::RECLASS_KEY, self.random_reclass as i32 );
            GameVariableManager::make_entry(DVCVariables::SP_KEY, self.random_skill_cost as i32);
            DVCVariables::create_recruitment_variables();
        }
    }
    pub fn create_game_variables_after_new_game(&self) {
        if !self.apply_rando_post_new_game { return; }
        GameVariableManager::make_entry(DVCVariables::SEED, 0);
        if self.randomized && GameVariableManager::get_number(DVCVariables::SEED) == 0 {
            if self.seed == 0 {  GameVariableManager::set_number(DVCVariables::SEED, utils::get_random_number_for_seed() as i32); }
            else {  GameVariableManager::set_number(DVCVariables::SEED, self.seed as i32); }
        }
        self.create_game_variables(true);   // Make sure variables exists

        if GameVariableManager::get_number(DVCVariables::SKILL_KEY) == 0 { GameVariableManager::set_number(DVCVariables::SKILL_KEY , self.random_skill as i32); }
        if GameVariableManager::get_number(DVCVariables::WEAPON_PROF_KEY) == 0 { GameVariableManager::set_number(DVCVariables::WEAPON_PROF_KEY, self.emblem_weap_prof_mode as i32);  }
        if GameVariableManager::get_number(DVCVariables::SHOP_KEY) == 0 {  GameVariableManager::set_number(DVCVariables::SHOP_KEY,  self.random_shop_items as i32 );  }
        if GameVariableManager::get_number(DVCVariables::JOB_KEY) == 0 { GameVariableManager::set_number(DVCVariables::JOB_KEY, self.random_job as i32);  }
        if GameVariableManager::get_number(DVCVariables::GROWTH_KEY) == 0{  GameVariableManager::set_number(DVCVariables::GROWTH_KEY, self.random_grow as i32); }
        if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_KEY) == 0 { GameVariableManager::set_number(DVCVariables::EMBLEM_SKILL_KEY,  self.random_god_mode as i32); }
        if GameVariableManager::get_number(DVCVariables::ITEM_KEY) == 0 { GameVariableManager::set_number(DVCVariables::ITEM_KEY,  self.random_item as i32); }
        if GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) == 0 {  GameVariableManager::set_number(DVCVariables::EMBLEM_SYNC_KEY , self.random_god_sync_mode as i32); }
        if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) == 0 { GameVariableManager::set_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY, self.emblem_skill_chaos as i32); }
        if GameVariableManager::get_number(DVCVariables::EMBLEM_ITEM_KEY) == 0 { GameVariableManager::set_number(DVCVariables::EMBLEM_ITEM_KEY, self.random_engage_weapon as i32); }
        if !GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) { GameVariableManager::set_number(DVCVariables::EMBLEM_NAME_KEY, self.random_names as i32); }
        if !GameVariableManager::get_bool(DVCVariables::RECLASS_KEY) { GameVariableManager::set_bool(DVCVariables::RECLASS_KEY, self.random_reclass); }
        if GameVariableManager::get_number(DVCVariables::SP_KEY) == 0 { GameVariableManager::set_number(DVCVariables::SP_KEY, self.random_skill_cost); }
        if !GameVariableManager::get_bool(DVCVariables::CUSTOM_JOB_KEY) { GameVariableManager::set_bool(DVCVariables::CUSTOM_JOB_KEY, self.custom_jobs); }
    }


    pub fn get_custom_jobs(&self) -> bool {
        if DVCVariables::is_main_menu() { self.custom_jobs } else { GameVariableManager::get_bool(DVCVariables::CUSTOM_JOB_KEY) }
    }
    pub fn set_custom_jobs(&mut self, value: bool) {
        if DVCVariables::is_main_menu() { self.custom_jobs = value; }
        else { GameVariableManager::set_bool(DVCVariables::CUSTOM_JOB_KEY, value); }
    }
    pub fn get_random_cc(&self) -> bool {
        if DVCVariables::is_main_menu() { self.random_reclass} else { GameVariableManager::get_bool(DVCVariables::RECLASS_KEY) }
    }
    pub fn set_random_cc(&mut self, value: bool) {
        if DVCVariables::is_main_menu() { self.random_reclass = value; }
        else { GameVariableManager::set_bool(DVCVariables::RECLASS_KEY, value); }
    }

}