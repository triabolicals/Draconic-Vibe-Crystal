use serde::{Deserialize, Serialize};
use engage::{gamevariable::*, gamedata::{accessory::AccessoryData, *}};
use crate::{
    *,
    utils::{clamp_value, dlc_check, get_rng},
    config::menu::CUSTOM_RECRUITMENT_ORDER,
    randomizer::data::GameData,
};
pub use crate::{CONFIG, config::variables::DVCVariables};

pub mod flags;
pub mod variables;
pub mod menu;
pub use flags::*;

#[derive(Default, Serialize, Deserialize)]
pub struct DVCConfig {
    pub randomized: bool,
    pub seed: u32,
    pub debug: bool,
    // Misc Settings
    pub continuous: i32,
    pub continuous_dlc: bool,
    pub continuous_items: bool,
    pub ironman: bool,
    // Recruitment Settings
    pub random_recruitment: i32,
    pub emblem_mode: i32,
    pub dlc: i32,
    pub recruitment_option: i32,
    // Emblem Settings
    pub random_engage_attacks: bool,
    pub random_engage_weapon: bool,
    pub emblem_weap_prof_mode: i32,
    pub emblem_stats: bool,
    pub sync_skill_mode: i32,
    pub engage_skill_mode: i32,
    pub emblem_inheritance_mode: i32,
    pub engrave_settings: i32,

    // Skill Settings
    pub personal_skills: bool,
    pub random_skill: i32,
    pub learn_skill: i32,
    pub random_skill_cost: bool,
    pub enemy_custom_skill: bool,
    pub equip_learn_skill: bool,
    pub bond_ring_skill: bool,
    pub bond_ring_skill_s_rate: i32,
    pub bond_ring_skill_a_rate: i32,
    pub bond_ring_skill_b_rate: i32,
    pub bond_ring_skill_c_rate: i32,
    pub bond_ring_stat: bool,
    // Class Settings
    pub random_job: i32,
    pub custom_jobs: bool,
    pub random_reclass: i32,
    pub random_battle_styles: i32,
    pub random_job_attrs: bool,
    pub single_class: i32,
    //Item Settings
    pub random_item: bool,
    pub random_gift_items: i32,
    pub exploration_items: i32,
    pub random_shop_items: bool,
    pub random_inventory: i32,
    pub interaction_type: i32,
    pub random_evolve_items: bool,
    pub random_refine: bool,
    // Growths
    pub random_grow: i32,
    pub class_growth: bool,
    pub adaptive_growth: bool,
    pub autolevel: bool,
    pub post_chapter_scaling: bool,
    pub personal_caps: bool,
    // Assets
    pub random_class_outfits: i32,
    pub weapon_asset: bool,
    pub unit_info_asset: bool,
    pub generic_mode: i32,
    pub bosses: bool,
    pub random_names: bool,
    pub emblem_appearance: i32,
    pub body_scale: i32,
    // Enemy
    pub random_enemy_job_rate: i32,
    pub random_enemy_skill_rate: i32,
    pub enemy_drop_rate: i32,
    pub revival_stone_rate: i32,
    pub enemy_emblem_rate: i32,

    // Map
    pub deployment_type: i32,
    pub emblem_deployment: i32,
    pub random_deploy_spots: bool,
    pub tile: bool,
    pub random_map_bgm: bool,
    pub terrain: i32,
    pub fow: i32,

    pub max_stat_caps: bool,
    pub misc_option_1 : f32,
    pub misc_option_2 : f32,
    pub cutscene_options: i32,
    // Custom Recruitment
    pub unit1: [u8; 32],
    pub unit2: [u8; 10],
    pub emblem: [u8; 25],
}

impl DVCConfig {
    pub fn get() -> &'static mut Self { unsafe { &mut CONFIG } }
    pub fn new() -> Self {
        println!("Opening dvc.toml");
        let config_content = std::fs::read_to_string("sd:/engage/config/dvc.toml");
        // If the file is read to a string or there is no failure, parse into the config struct.
        if config_content.is_ok() {
            let content = config_content.unwrap();
            let config = toml::from_str(&content);
            if config.is_ok() {
                println!("DVC Config file was parsed with no issues.");
                let config = config.unwrap();
                config
            } else {
                // This is mostly intended to create a new file if more items are added to the struct
                println!("DVC Config: Config file could not be parsed or new settings are added.\nNew default config file has been created.");
                let config = DVCConfig::default();
                // config.save();
                config
            }
        }
        else {
            // If the file could not be read to a string then create a new file with default values.
            println!("DVC Config: The config file was either missing or unable to be read, creating new toml.");
            let config = DVCConfig::default();
            // config.save();
            config
        }
    }
    pub const fn default() -> Self {
        let config = DVCConfig {
            body_scale: 0,
            unit1: [50; 32],
            unit2: [50; 10],
            emblem: [50; 25],
            randomized: true,
            seed: 0,
            personal_caps: false,
            random_enemy_job_rate: 50,
            random_enemy_skill_rate: 50,
            revival_stone_rate: 0,
            enemy_emblem_rate: 0, 
            random_map_bgm: false,
            bond_ring_skill_s_rate: 75,
            bond_ring_skill_a_rate: 30,
            bond_ring_skill_b_rate: 10,
            bond_ring_skill_c_rate: 5,
            engrave_settings: 0,
            autolevel: false,
            random_evolve_items: false,
            exploration_items: 0,
            enemy_drop_rate: 10,
            deployment_type: 0,
            emblem_deployment: 0,
            emblem_mode: 0,
            continuous: 0,
            continuous_dlc: false,
            fow: 0,
            terrain: 0,
            random_inventory: 0,
            random_recruitment: 0,
            random_job: 0,
            random_skill: 0,
            random_skill_cost: false,
            random_item: false,
            random_grow: 0,
            class_growth: false,
            random_engage_attacks: false,
            emblem_inheritance_mode: 0,
            emblem_weap_prof_mode: 0,
            random_engage_weapon: false,
            random_gift_items: 0,
            interaction_type: 0,
            random_job_attrs: false,
            random_shop_items: false,
            random_battle_styles: 0,
            generic_mode: 0,
            random_names: false,
            random_reclass: 0,
            custom_jobs: false,
            learn_skill: 0,
            enemy_custom_skill: false,
            debug: false,
            dlc: 0,
            bosses: false,
            tile: false,
            equip_learn_skill: false,
            max_stat_caps: false,
            bond_ring_skill: false,
            emblem_appearance: 0,
            random_class_outfits: 0,
            weapon_asset: false,
            single_class: 0,
            random_deploy_spots: false,
            emblem_stats: false,
            sync_skill_mode: 0,
            post_chapter_scaling: false,
            misc_option_1 : 0.0,
            misc_option_2 : 1.0,
            ironman: false,
            recruitment_option: 0,
            engage_skill_mode: 0,
            adaptive_growth: false,
            unit_info_asset: false,
            personal_skills: false,
            continuous_items: false,
            random_refine: false,
            bond_ring_stat: false,
            cutscene_options: 0,
        };
        config
    }
    pub fn set_custom_index(&mut self, index: i32, value: u8, emblem: bool) {
        if emblem { self.emblem[index as usize] = value; }
        else {
            if index < 32 { self.unit1[index as usize] = value; }
            else { self.unit2[index as usize - 32] = value; };
        }
    }
    pub fn get_next_unit(&mut self, index: i32, increase: bool) -> u8 {
        let current = if index < 32 { self.unit1[index as usize] } else { self.unit2[index as usize - 32] };
        let range = if dlc_check() { 41 } else { 36 };
        let i = index as u8;
        let used: Vec<u8> = self.unit1.iter().chain(self.unit2.iter()).map(|u| *u).collect();
        let new =
        if increase {
            let start = if current >= range { i } else if current < range { current  } else { 0 };
            let available: Vec<u8> = (start..range).collect();
            if let Some(pos) = available.iter().find(|x| !used.contains(x) ) { *pos }
            else {
                let available: Vec<u8> = (0..start+1).collect();
                available.iter().find(|x| !used.contains(x)).map(|x| *x).unwrap_or(current)
            }
        }
        else {
            let start = if current >= range { i } else if current == 0 { range} else { current };
            let available: Vec<u8> = (0..start).rev().collect();
            if let Some(pos) = available.iter().find(|x| !used.contains(x)) { *pos }
            else {
                let available: Vec<u8> = (start..range).rev().collect();
                available.iter().find(|x| !used.contains(x)).map(|x| *x).unwrap_or(current)
            }
        };
        if index < 32 { self.unit1[index as usize] = new; } else { self.unit2[index as usize - 32] = new; }
        new
    }
    pub fn get_next_emblem(&mut self, index: i32, increase: bool) -> u8 {
        let count = GameData::get_playable_emblem_hashes().len() as u8;
        let mut available: Vec<u8> = (0..12).collect();
        if dlc_check() { available.extend(12..19); }    // Add DLC Emblems
        if count > 20 { available.extend(20..count as u8); }    // Custom Emblems
        self.emblem.iter_mut().enumerate().for_each(|(emblem_idx, i)|{
            if emblem_idx >= 12 && emblem_idx < 19 && !dlc_check() { *i = 50; }
            else if *i < count {
                if !available.contains(&i) { *i = 50 }
                else { available.retain(|&r| r != *i); }
            }
        });
        let current = self.emblem[index as usize];
        if increase {
            if current == 50 { self.emblem[index as usize] = available.iter().map(|x| *x).min().unwrap_or(50); }
            else { self.emblem[index as usize] = available.iter().filter(|i| **i > current).map(|i| *i).min().unwrap_or(50); }
        }
        else {
            if current == 50 { self.emblem[index as usize] = available.iter().map(|x| *x).max().unwrap_or(50); }
            else { self.emblem[index as usize] = available.iter().filter(|i| **i < current).map(|i| *i).max().unwrap_or(50); }
        };
        self.emblem[index as usize]
    }
    /// Output is Person Index, New Person Index
    pub fn get_custom_recruitment(&self, is_emblem: bool) -> Vec<(i32, i32)> {
        let mut output: Vec<(i32, i32)> = Vec::new();
        let table: Vec<u8> =
            if is_emblem { self.emblem.iter().map(|u| *u).collect() }
            else if self.random_recruitment == 4 { unsafe { CUSTOM_RECRUITMENT_ORDER.iter().cloned().collect() } }
            else { self.unit1.iter().chain(self.unit2.iter()).map(|u| *u).collect() };
        let mut available = vec![];
        let count;
        if is_emblem {
            count = GameData::get_playable_emblem_hashes().len() as u8;
            available = (0..12).collect();
            if dlc_check() { available.extend(12..19); }    // Add DLC Emblems
            if count > 20 { available.extend(20..count); }    // Custom Emblems
        }
        else {
            count = if self.random_recruitment == 4 { GameData::get().playables.len() as u8 } else { 41 };
            available = (0..count).collect();
            if !dlc_check() { available.retain(|&r| r < 36 && r > 40) }
        }
        let limit = utils::get_total_unit_emblems(is_emblem) as u8;
        let mut pool: Vec<u8> = Vec::new();
        for x in 0..limit {
            let value = table[x as usize];
            if value < count {
                if let Some(pos) = available.iter().position(|&y| value == y) {
                    let v = available.remove(pos);
                    output.push( (x as i32, v as i32) );
                }
                else { pool.push(x); }
            }
            else { pool.push(x); }
        }
        let rng = get_rng();
        pool.iter().for_each(|&xi|{
            if available.len() > 0 {
                let index = rng.get_value( available.len() as i32) as usize;
                let xj = available[index];
                output.push( (xi as i32, xj as i32) );
                available.remove(index);
            }
        });
        output
    }
    pub fn correct_rates(&mut self) {
        self.random_enemy_skill_rate = clamp_value(self.random_enemy_skill_rate, 0, 100);
        self.random_enemy_job_rate = clamp_value(self.random_enemy_job_rate, 0, 100);
        self.revival_stone_rate = clamp_value(self.revival_stone_rate, 0, 100);
        self.bond_ring_skill_s_rate = clamp_value(self.bond_ring_skill_s_rate, 0, 100);
        self.bond_ring_skill_a_rate = clamp_value(self.bond_ring_skill_a_rate, 0, 100);
        self.bond_ring_skill_b_rate = clamp_value(self.bond_ring_skill_b_rate, 0, 100);
        self.bond_ring_skill_c_rate = clamp_value(self.bond_ring_skill_c_rate, 0, 100);
        let count = GameData::get_playable_emblem_hashes().len() as u8;
        for x in 0..19 {
            let idx = self.emblem[x];
            if idx == 19 || idx >= count { self.emblem[x] = 50; }
        }
    }
    pub fn get_bond_ring_rates(&self) -> [i32; 4] {
        let mut rate: [i32; 4] = [self.bond_ring_skill_s_rate, self.bond_ring_skill_a_rate, self.bond_ring_skill_b_rate, self.bond_ring_skill_c_rate];
        let v = DVCVariables::BondRingSkillRate.get_value();
        if v != 0 {
            for x in 0..4 { rate[x] = (v >> (8*x)) & 255; }
        }
        else {
            let mut value = 0;
            for x in 0..4 { value += rate[x] << x*8; }
            DVCVariables::BondRingSkillRate.create_variable(value);
        }
        rate
    }
    pub fn set_bond_ring_rate(&mut self, index: usize, value: i32) {
        let value = clamp_value(value, 0, 100);
        match index {
            2 => { self.bond_ring_skill_b_rate = value },
            1 => { self.bond_ring_skill_a_rate = value },
            0 => { self.bond_ring_skill_s_rate = value },
            _ => { self.bond_ring_skill_c_rate = value },
        }
    }
    pub fn save(&self) {
        toml::to_string(&self).unwrap();
        let out_toml = toml::to_string(&self).unwrap(); // toml::to_string_pretty(&self).unwrap();
        std::fs::write("sd:/engage/config/dvc.toml", out_toml)
            .expect("should be able to write to write default configuration");
    }
    pub fn create_game_variables(&mut self, set_value: bool) {
        GameVariableManager::make_entry_norewind(FLAGNAME, 0);
        GameVariableManager::make_entry_norewind(FLAGNAME2, 0);

        if set_value { DVCFlags::set_value_from_config(); }
        GameVariableManager::make_entry_norewind(DVCVariables::MISERCODE_TYPE, 0);
        GameVariableManager::make_entry_norewind(DVCVariables::LIBERATION_TYPE, 0);
        GameVariableManager::make_entry_norewind(DVCVariables::PLAYER_AVERAGE_CAP, 0);
        self.get_bond_ring_rates();
        for x in 0..10 {    // ShopItems
            GameVariableManager::make_entry_norewind(format!("G_DVC_I{}", x).as_str(), 0);
            GameVariableManager::make_entry_norewind(format!("G_DVC_W{}", x).as_str(), 0);
        }
        if self.random_job != 2 { self.single_class = 0; }
        for x in 0..30 {
            if x == 26  { continue; }
            if let Some(v) = DVCVariables::from(x) {
                let value = if set_value { self.get_value(v) } else { 0 };
                let key = v.get_key();
                GameVariableManager::make_entry_norewind(key, value);
                println!("{} Set to {}", key, value);
            }
        }
        if set_value {
            let mut v = 0;
            for x in 0..5 {
                if let Some(var) = DVCVariables::from(30+x).map(|s| self.get_value(s)){
                    let v2 = clamp_value(var, 0, 100) / 10;
                    v |= v2 << x*5;
                }
            }
            GameVariableManager::make_entry_norewind(DVCVariables::EnemySkillGauge.get_key(), v);
            if self.random_job == 2 {
                if self.single_class == 1 || JobData::try_get_hash(self.single_class).is_some() {
                    DVCFlags::SingleJobEnabled.set_value(true);
                    DVCVariables::ClassMode.set_value(2);
                }
            }
        }
    }
    pub fn set_value(&mut self, dvc_key: DVCVariables, value: i32){
        match dvc_key {
            DVCVariables::Seed => { self.seed = value as u32; }
            DVCVariables::EmblemWepProf => { self.emblem_weap_prof_mode = value; },
            DVCVariables::EmblemSyncSkill => { self.sync_skill_mode = value; },
            DVCVariables::EmblemEngageSkill => { self.engage_skill_mode = value; },
            DVCVariables::EmblemInherit => { self.emblem_inheritance_mode = value; },
            DVCVariables::ClassMode => { self.random_job = value; },
            DVCVariables::UnitInventory => { self.random_inventory = value; },
            DVCVariables::PersonalGrowthMode => { self.random_grow = value; },
            DVCVariables::Reclassing => { self.random_reclass = value },
            DVCVariables::SingleJob => { self.single_class = value; },
            DVCVariables::BattleStyles => { self.random_battle_styles = value; },
            DVCVariables::InteractSetting => { self.interaction_type = value; },
            DVCVariables::GenericAppearance=> { self.generic_mode = value;}
            DVCVariables::EnemySkillGauge => { self.random_enemy_skill_rate = clamp_value(value, 0, 100); },
            DVCVariables::EnemyJobGauge => { self.random_enemy_job_rate = clamp_value(value, 0, 100); },
            DVCVariables::EnemyRevivalStone=> { self.revival_stone_rate = clamp_value(value, 0, 100); },
            DVCVariables::EnemyEmblemGauge => { self.enemy_emblem_rate = clamp_value(value, 0, 100); },
            DVCVariables::EnemyItemDropGauge => { self.enemy_drop_rate = clamp_value(value, 0, 100); },
            DVCVariables::UnitDeployment=> { self.deployment_type = value; },
            DVCVariables::EmblemDeployment => { self.emblem_deployment = value; },
            DVCVariables::EmblemRecruitment => { self.emblem_mode = value; },
            DVCVariables::UnitRecruitment => { self.random_recruitment = value; },
            DVCVariables::ExplorationItem => { self.exploration_items = value; },
            DVCVariables::RandomGifts => { self.random_gift_items = value; },
            DVCVariables::RandomJobOutfit => { self.random_class_outfits = value; },
            DVCVariables::BondSkillS => { self.bond_ring_skill_s_rate = clamp_value(value, 0, 100); }
            DVCVariables::BondSkillA => { self.bond_ring_skill_a_rate = clamp_value(value, 0, 100); },
            DVCVariables::BondSkillB => { self.bond_ring_skill_b_rate = clamp_value(value, 0, 100); },
            DVCVariables::BondSkillC  => { self.bond_ring_skill_c_rate = clamp_value(value, 0, 100); },
            DVCVariables::Continuous => { self.continuous = value; },
            DVCVariables::EmblemAppearance => { self.emblem_appearance = value; },
            DVCVariables::FogOfWar => { self.fow = value; },
            DVCVariables::EngraveLevel => { self.engrave_settings = value; },    //Setting
            DVCVariables::TerrainEffect => { self.terrain = value; },
            DVCVariables::JobLearnMode => { self.learn_skill = value; },
            DVCVariables::BodyScaling => { self.body_scale = value; },
            _ => {}
            //_ => { println!("Config: Cannot set value from key '{}'", dvc_key); }
        }
    }
    pub fn get_value(&self, dvc_key: DVCVariables) -> i32 {
        match dvc_key {
            DVCVariables::Seed => { self.seed as i32 },
            DVCVariables::EmblemWepProf => { self.emblem_weap_prof_mode },
            DVCVariables::EmblemSyncSkill => { self.sync_skill_mode },
            DVCVariables::EmblemEngageSkill => { self.engage_skill_mode },
            DVCVariables::EmblemInherit => { self.emblem_inheritance_mode },
            DVCVariables::ClassMode => { self.random_job },
            DVCVariables::UnitInventory => { self.random_inventory },
            DVCVariables::PersonalGrowthMode => { self.random_grow },
            DVCVariables::Reclassing => { self.random_reclass },
            DVCVariables::SingleJob => { self.single_class },
            DVCVariables::BattleStyles => { self.random_battle_styles },
            DVCVariables::InteractSetting => { self.interaction_type },
            DVCVariables::GenericAppearance=> { self.generic_mode}
            DVCVariables::EnemySkillGauge => { self.random_enemy_skill_rate},
            DVCVariables::EnemyJobGauge => { self.random_enemy_job_rate},
            DVCVariables::EnemyRevivalStone=> { self.revival_stone_rate},
            DVCVariables::EnemyEmblemGauge => { self.enemy_emblem_rate},
            DVCVariables::EnemyItemDropGauge => { self.enemy_drop_rate},
            DVCVariables::UnitDeployment=> { self.deployment_type },
            DVCVariables::EmblemDeployment => { self.emblem_deployment },
            DVCVariables::EmblemRecruitment => { self.emblem_mode },
            DVCVariables::UnitRecruitment => { self.random_recruitment },
            DVCVariables::ExplorationItem => { self.exploration_items },
            DVCVariables::RandomGifts => { self.random_gift_items },
            DVCVariables::RandomJobOutfit => { self.random_class_outfits },
            DVCVariables::BondSkillS => { self.bond_ring_skill_s_rate},
            DVCVariables::BondSkillA => { self.bond_ring_skill_a_rate},
            DVCVariables::BondSkillB => { self.bond_ring_skill_b_rate},
            DVCVariables::BondSkillC  => { self.bond_ring_skill_c_rate},
            DVCVariables::Continuous => { self.continuous },
            DVCVariables::EmblemAppearance => { self.emblem_appearance },
            DVCVariables::FogOfWar => { self.fow },
            DVCVariables::EngraveLevel => { self.engrave_settings },    //Setting
            DVCVariables::TerrainEffect => { self.terrain },
            DVCVariables::JobLearnMode => { self.learn_skill },
            DVCVariables::BodyScaling => { self.body_scale }
            _ => { 0 }
        }
    }
}
pub fn migrate_to_v3() {
    println!("Migrating Save to Version 3");
    let gauges = ["G_EnemyRevivalStone", "G_EnemySkillGauge", "G_EnemyJobGauge", "G_EnemyEmblemGauge", "G_ItemDropGauge"];
    GameVariableManager::make_entry_norewind(DVCVariables::EnemyItemDropGauge.get_key(), 0);
    for x in 0..5 {
        let v = GameVariableManager::get_number(gauges[x as usize]);
        println!("{}: {}", gauges[x as usize], v);
        let v2 = clamp_value(v, 0, 100);
        if let Some(v) = DVCVariables::from(30+x) { v.set_value(v2); }
        GameVariableManager::remove(gauges[x as usize]);
    }
    GameVariableManager::set_number("G_DVC_Version", 3);
}
pub fn migrate_to_v5() {
    GameVariableManager::make_entry_norewind(FLAGNAME2, 0);
    let mut s = GameVariableManager::get_number(FLAGNAME) & !(1 << 20);
    let random_class = s & (1 << 27) != 0;
    GameVariableManager::set_number(FLAGNAME, s);
    if GameVariableManager::get_number("G_Random_Item") == 1 {
        s |= 1 << 20;
        GameVariableManager::remove("G_Random_Item");
    }
    let class_mode =
    if DVCVariables::get_single_class(false, false).is_some() {
        s |= 1 << 27;
        2
    }
    else if random_class { 1 } else { 0 };
    GameVariableManager::make_entry_norewind(DVCVariables::ClassMode.get_key(), class_mode);
    GameVariableManager::set_number(FLAGNAME, s);
    GameVariableManager::set_number("G_DVC_Version", 5);
}
pub fn remove_old_keys() {
    let mut count = 0;
    GameVariableManager::find_starts_with("G_L_JID").iter().for_each(|i|{
        let i = i.to_string();
        let jid = i.trim_start_matches("G_L_");
        if JobData::get(jid).is_none() {
            count += 1;
            GameVariableManager::remove(jid);
        }
    });
    if count > 0 { println!("Removed #{} learn jobs", count); }
    count = 0;
    GameVariableManager::find_starts_with("G_所持_AID").iter().for_each(|i|{
        let i = i.to_string();
        let jid = i.trim_start_matches("G_所持_");
        if AccessoryData::get(jid).is_none() {
            count += 1;
            GameVariableManager::remove(jid);
        }
    });
    if count > 0 { println!("Removed #{} learn accessories", count); }
}