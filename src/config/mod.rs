use serde_big_array::BigArray;
use serde::{Deserialize, Serialize};
use engage::{
    app::{IGameVariable, IRandom_2Methods, IStructData_1Methods, JobData},
    system::collections::generic::IHashSet_1Methods
};
use crate::{*, utils::clamp_value};
pub use crate::{CONFIG, config::variable::DVCVariable};
pub use engage::{GameVariableManager, app::gameuserdata::*};
use engage::app::{Gender, IBitField32, IJobDataMethods};

pub mod flags;
pub mod variable;

// mod menu;
pub use flags::*;
pub use variable::*;
use crate::gamedata::{EmblemPool, PersonPool};
use crate::menus::RecruitmentOrder;

#[derive(Serialize, Deserialize)]
pub struct DVCConfig {
    pub seed: u32,
    pub debug: bool,
    pub randomized: bool,
    // Misc Settings
    pub continuous: i32,
    // Recruitment Settings
    pub random_recruitment: i32,
    pub emblem_mode: i32,
    pub dlc: i32,
    // Emblem Settings
    pub emblem_weap_prof_mode: i32,
    pub sync_skill_mode: i32,
    pub engage_skill_mode: i32,
    pub emblem_inheritance_mode: i32,
    pub engrave_settings: i32,

    // Skill Settings
    pub random_skill: i32,
    pub learn_skill: i32,
    pub bond_ring_skill_s_rate: i32,
    pub bond_ring_skill_a_rate: i32,
    pub bond_ring_skill_b_rate: i32,
    pub bond_ring_skill_c_rate: i32,
    // Class Settings
    pub random_job: i32,
    pub random_reclass: i32,
    pub random_battle_styles: i32,
    pub single_class: i32,
    pub class_m: i32,
    pub class_f: i32,
    //Item Settings
    pub random_gift_items: i32,
    pub exploration_items: i32,
    pub random_inventory: i32,
    pub interaction_type: i32,

    // Growths
    pub random_grow: i32,
    pub enemy_off: i8,
    // Assets
    pub random_class_outfits: i32,
    pub generic_mode: i32,
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
    pub terrain: i32,
    pub fow: i32,

    pub misc_option_1 : f32,
    pub misc_option_2 : f32,
    pub flags: i64,
    // Custom Recruitment
    #[serde(with = "BigArray")]
    pub unit: [u8; 41],
    #[serde(with = "BigArray")]
    pub custom_unit: [u8; 41],
    #[serde(with = "BigArray")]
    pub emblem: [u8; 41],
}

impl DVCConfig {
    pub fn get() -> &'static mut Self { unsafe { &mut CONFIG } }
    pub fn new() -> Self {
        #[cfg(feature = "debug")] println!("Creating dvc.toml");
        if let Ok(content) = std::fs::read_to_string("sd:/engage/config/dvc.toml") {
            toml::from_str(&content).ok().unwrap_or_else(|| DVCConfig::default() )
        }
        else {
            let config = DVCConfig::default();
            config.save();
               config
        }
    }
    pub const fn default() -> Self {
        let config = DVCConfig {
            randomized: true,
            body_scale: 0,
            unit: [255; 41],
            custom_unit: [255; 41],
            emblem: [255; 41],
            seed: 0,
            random_enemy_job_rate: 50,
            random_enemy_skill_rate: 50,
            revival_stone_rate: 0,
            enemy_emblem_rate: 0,
            bond_ring_skill_s_rate: 75,
            bond_ring_skill_a_rate: 30,
            bond_ring_skill_b_rate: 10,
            bond_ring_skill_c_rate: 5,
            engrave_settings: 0,
            exploration_items: 0,
            enemy_drop_rate: 10,
            deployment_type: 0,
            emblem_deployment: 0,
            emblem_mode: 0,
            continuous: 0,
            fow: 0,
            enemy_off: 0,
            terrain: 0,
            random_inventory: 0,
            random_recruitment: 0,
            random_job: 0,
            random_skill: 0,
            random_grow: 0,
            emblem_inheritance_mode: 0,
            emblem_weap_prof_mode: 0,
            random_gift_items: 0,
            interaction_type: 0,
            random_battle_styles: 0,
            generic_mode: 0,
            random_reclass: 0,
            learn_skill: 0,
            debug: false,
            dlc: 0,
            emblem_appearance: 0,
            random_class_outfits: 0,
            single_class: 0,
            class_m: 0,
            sync_skill_mode: 0,
            misc_option_1 : 0.0,
            misc_option_2 : 1.0,
            engage_skill_mode: 0,
            flags: 0,
            class_f: 0,
        };
        config
    }
    pub fn adjust_custom_recruitment(&mut self){
        [0, 4, 14, 17, 23, 27].into_iter().for_each(|x| { self.reset_recruitment_position(x, RecruitmentOrder::UnitCustom); });
        if !DVCVariables::has_dlc() {
            [36, 37, 38, 39, 40].into_iter().for_each(|x| {
                self.reset_recruitment_position(x, RecruitmentOrder::Unit);
                self.reset_recruitment_position(x, RecruitmentOrder::UnitCustom);
            });
        }
    }
    pub fn reset_recruitment_position(&mut self, index: i32, order: RecruitmentOrder) {
        let x = index as u8;
        match order {
            RecruitmentOrder::Unit => {
                self.unit.iter_mut().filter(|c| **c == x).for_each(|c| *c = 255);
                self.unit[index as usize] = x;
            }
            RecruitmentOrder::Emblem => {
                self.emblem.iter_mut().filter(|c| **c == x).for_each(|c| *c = 255);
                self.emblem[index as usize] = x;
            }
            RecruitmentOrder::UnitCustom => {
                self.custom_unit.iter_mut().filter(|c| **c == x).for_each(|c| *c = 255);
                self.custom_unit[index as usize] = x;
            }
        }
    }
    pub fn set_recruitment_random(&mut self, index: i32, order: RecruitmentOrder) {
        match order {
            RecruitmentOrder::Unit => { self.unit[index as usize] = 255; }
            RecruitmentOrder::Emblem => { self.emblem[index as usize] = 255; }
            RecruitmentOrder::UnitCustom => { self.custom_unit[index as usize] = 255; }
        }
    }
    pub fn get_next_recruitment(&mut self, index: i32, increase: bool, order: RecruitmentOrder) -> u8 {
        let (limit, set) =
            match order {
                RecruitmentOrder::Unit => (41, &mut self.unit),
                RecruitmentOrder::Emblem => (EmblemPool::get().recruitment_list.len(),  &mut self.emblem),
                RecruitmentOrder::UnitCustom => (PersonPool::get().playable.len(), &mut self.custom_unit),
            };
        let mut avail = (0..limit).collect::<Vec<usize>>();
        if order == RecruitmentOrder::Emblem {
            if let Some(lueur) = avail.iter().position(|v| *v == 19) { avail.remove(lueur); }
        }
        set.iter_mut().for_each(|&mut x|{ if let Some(pos) = avail.iter().position(|&a| a == (x as usize)){ avail.remove(pos); } });
        let current = set[index as usize] as usize;
        let new =
            if increase {
                avail.iter()
                    .filter(|&&x| x > current).min()
                    .or_else(|| avail.iter().min())
                    .map(|v| *v as u8)
            }
            else {
                avail.iter()
                    .filter(|&&x| x < current).max()
                    .or_else(|| avail.iter().max())
                    .map(|v| *v as u8)
            }.unwrap_or(255);
        set[index as usize] = new;
        new
    }
    pub fn set_next_custom_recruitment(&mut self, index: i32, increase: bool) -> u8 {
        let person_count = PersonPool::get().playable.len();
        let current = self.custom_unit[index as usize] as usize;
        let mut avail = (0..person_count).collect::<Vec<usize>>();
        self.custom_unit.iter().filter(|&&x| (x as usize) < person_count)
            .for_each(|&x|{
                if let Some(pos) = avail.iter().position(|&a| a == (x as usize)){ avail.remove(pos); }
            });
        let new =
        if increase {
            avail.iter()
                .filter(|&&x| x > current).min()
                .or_else(|| avail.iter().min())
                .map(|v| *v as u8)
        }
        else {
            avail.iter()
                .filter(|&&x| x < current).max()
                .or_else(|| avail.iter().max())
                .map(|v| *v as u8)
        }.unwrap_or(255);
        self.custom_unit[index as usize] = new;
        new
    }
    /// Output is Person Index, New Person Index
    pub fn get_custom_recruitment(&self, is_emblem: bool) -> Vec<(i32, i32)> {
        let mut output: Vec<(i32, i32)> = Vec::new();
        let table: Vec<u8> =
            if is_emblem { self.emblem.iter().map(|u| if *u >= 19 && *u < 50 { *u - 1 } else { *u }).collect() }
            else if self.random_recruitment == 4 { self.custom_unit.iter().map(|u| *u).collect() }
            else { self.unit.iter().map(|u| *u).collect() };
        
        let mut available = vec![];
        let count;
        if is_emblem {
            count = EmblemPool::get().emblem_data.len() as u8;
            available = (0..12).collect();
            if DVCVariables::has_dlc() { available.extend(12..19); }    // Add DLC Emblems
            if count > 20 { available.extend(20..count); }    // Custom Emblems
        }
        else {
            count = if self.random_recruitment == 4 { PersonPool::get().playable.len() as u8 } else { 41 };
            available = (0..count).collect();
            if !DVCVariables::has_dlc() { available.retain(|&r| r < 36 && r > 40) }
        }
        let limit = if is_emblem { if DVCVariables::has_dlc() { 19 } else { 12 } } else if DVCVariables::has_dlc() { 41 } else { 36 };
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
        let rng = DVCVariables::get_rng();
        pool.iter().for_each(|&xi|{
            if available.len() > 0 {
                let index = rng.get_value_2( available.len() as i32) as usize;
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
        let count = EmblemPool::get().emblem_data.len() as u8;
        for x in 0..19 {
            let idx = self.emblem[x];
            if idx == 19 || idx >= count { self.emblem[x] = 50; }
        }
    }
    pub fn get_bond_ring_rates(&self) -> [i32; 4] {
        let mut rate: [i32; 4] = [self.bond_ring_skill_s_rate, self.bond_ring_skill_a_rate, self.bond_ring_skill_b_rate, self.bond_ring_skill_c_rate];
        let v = DVCVariable::BondRingSkillRate.get_value();
        if v != 0 {
            for x in 0..4 { rate[x] = (v >> (8*x)) & 255; }
        }
        else {
            let mut value = 0;
            for x in 0..4 { value += rate[x] << x*8; }
            DVCVariable::BondRingSkillRate.create_variable(value);
        }
        rate
    }
    pub fn set_bond_ring_rate(&mut self, index: usize, value: i32) {
        let value = clamp_value(value, 0, 100);
        match index {
            0 => { self.bond_ring_skill_s_rate = value },
            1 => { self.bond_ring_skill_a_rate = value },
            2 => { self.bond_ring_skill_b_rate = value },
            3 => { self.bond_ring_skill_c_rate = value },
            _ => {}
        }
    }
    pub fn save(&self) {
        toml::to_string(&self).unwrap();
        let out_toml = toml::to_string(&self).unwrap(); // toml::to_string_pretty(&self).unwrap();
        std::fs::write("sd:/engage/config/dvc.toml", out_toml).unwrap();
    }
    pub fn is_valid_single_class(hash: i32, gender: Gender) -> bool {
        if hash == 10000 && gender.value == 0 { DVCVariables::has_dlc() }
        else {
            let job = JobData::try_get_from_hash(hash);
            if job.is_null() || (job.is_low() && job.has_high_jobs()) { false }
            else {
                let flag = job.get_flag().m_value();
                if gender.value == 0 { flag & 23 == 3 }
                else if gender.value == 1 { flag & 7 == 3 }
                else { flag & 19 == 3 }
            }
        }
    }
    pub fn create_game_variables(&mut self, set_value: bool) {
        GameVariableManager::make_entry_norewind(DVCFlags::FlagSet1, 0);
        GameVariableManager::make_entry_norewind(DVCFlags::FlagSet2, 0);
        if set_value {
            DVCFlags::set_value_from_config(self.randomized);
            self.get_bond_ring_rates();
            if self.random_job != 2 { self.single_class = 0; }
            if self.random_job == 2 {
                if self.single_class == 1000 || !JobData::try_get_from_hash(self.single_class).is_null() {
                    DVCVariable::ClassMode.set_value(2);
                    self.random_job = 0;
                }
            }
            else if self.random_job == 5 {
                if !Self::is_valid_single_class(self.class_m, Gender::male()) { self.class_m = 0; }
                if !Self::is_valid_single_class(self.class_f, Gender::female()) { self.class_f = 0; }
                if self.class_m == 0 && self.class_f == 0 {
                    DVCVariable::ClassMode.set_value(0);
                    self.random_job = 0;
                }
                DVCVariable::GenderClassM.init_var(self.class_m, true);
                DVCVariable::GenderClassF.init_var(self.class_f, true);

            }
            for x in 0..5 {    // ShopItems
                GameVariableManager::make_entry_norewind(format!("G_DVC_I{}", x).as_str(), -1);
                GameVariableManager::make_entry_norewind(format!("G_DVC_W{}", x).as_str(), -1);
            }
            for x in 0..30 {
                if let Some(v) = DVCVariable::from(x) {
                    let value = if set_value { self.get_value(v) } else { 0 };
                    let key = v.get_key();
                    GameVariableManager::make_entry_norewind(key, value);
                }
            }
            for x in 50..60 {
                if let Some(v) = DVCVariable::from(x) { GameVariableManager::make_entry_norewind(v.get_key(), 0); }
            }
            let mut v = 0;
            for x in 0..5 {
                if let Some(var) = DVCVariable::from(30+x).map(|s| self.get_value(s)){
                    let v2 = clamp_value(var, 0, 100) / 10;
                    v |= v2 << x*5;
                }
            }
            GameVariableManager::make_entry_norewind(DVCVariable::EnemySkillGauge.get_key(), v);
        }
        else {
            let hash_set = DVCVariables::user_data().m_variable().m_rewindable();
            let (mut v1, mut v2) = DVCFlags::get_i32();
            for x in 6..50 {
                if DVCFlags::from(x).is_none() && x != 27{
                    if x < 32 {
                        v1 |= 1 << x;
                        v1 ^= 1 << x;
                    }
                    else {
                        v2 |= 1 << (x % 32);
                        v2 ^= 1 << (x % 32);
                    }
                }
            }
            DVCFlags::set_i32(Some(v1), Some(v2));
            for x in 0..60 {
                if let Some(v) = DVCVariable::from(x) {
                    let key = v.get_key();
                    if !GameVariableManager::is_exist(key) { GameVariableManager::make_entry_norewind(key, 0); }
                    else if hash_set.contains(key.into()) { hash_set.remove(key.into()); }
                }
            }
            let mode = DVCVariable::ClassMode.get_value();
            if mode == 2 {
                if !Self::is_valid_single_class(DVCVariable::SingleJob.get_setting_value(), Gender::none()) {
                    DVCVariable::ClassMode.set_value(0);
                    DVCVariable::SingleJob.set_value(0);
                }
            }
            else if mode == 5 {
                if !Self::is_valid_single_class(DVCVariable::GenderClassM.get_setting_value(), Gender::male()) {
                    DVCVariable::GenderClassM.set_value(0);
                }
                if !Self::is_valid_single_class(DVCVariable::GenderClassF.get_setting_value(), Gender::female()) {
                    DVCVariable::GenderClassF.set_value(0);
                }
                if DVCVariable::GenderClassF.get_setting_value() == 0 && DVCVariable::GenderClassM.get_setting_value() == 0 {
                    DVCVariable::ClassMode.set_value(0);
                }
            }
            for x in 0..5 {    // ShopItems
                let key1 = format!("G_DVC_I{}", x);
                let key2 = format!("G_DVC_W{}", x);
                if !GameVariableManager::is_exist(key1.as_str()) { GameVariableManager::make_entry_norewind(key1.as_str(), -1); }
                if !GameVariableManager::is_exist(key2.as_str()) { GameVariableManager::make_entry_norewind(key2.as_str(), -1); }
            }
            [DVCFlags::FlagSet1, DVCFlags::FlagSet2, DVCVariable::Seed.get_key()].into_iter().for_each(|key|{
                if hash_set.contains(key.into()) { hash_set.remove(key.into()); }
            });
        }
    }
    pub fn set_value(&mut self, dvc_key: DVCVariable, value: i32){
        match dvc_key {
            DVCVariable::Seed => { self.seed = value as u32; }
            DVCVariable::EmblemWepProf => { self.emblem_weap_prof_mode = value; },
            DVCVariable::EmblemSyncSkill => { self.sync_skill_mode = value; },
            DVCVariable::EmblemEngageSkill => { self.engage_skill_mode = value; },
            DVCVariable::EmblemInherit => { self.emblem_inheritance_mode = value; },
            DVCVariable::ClassMode => { self.random_job = value; },
            DVCVariable::UnitInventory => { self.random_inventory = value; },
            DVCVariable::PersonalGrowthMode => { self.random_grow = value; },
            DVCVariable::Reclassing => { self.random_reclass = value },
            DVCVariable::SingleJob => { self.single_class = value; },
            DVCVariable::BattleStyles => { self.random_battle_styles = value; },
            DVCVariable::InteractSetting => { self.interaction_type = value; },
            DVCVariable::GenericAppearance=> { self.generic_mode = value;}
            DVCVariable::EnemySkillGauge => { self.random_enemy_skill_rate = clamp_value(value, 0, 100); },
            DVCVariable::EnemyJobGauge => { self.random_enemy_job_rate = clamp_value(value, 0, 100); },
            DVCVariable::EnemyRevivalStone=> { self.revival_stone_rate = clamp_value(value, 0, 100); },
            DVCVariable::EnemyEmblemGauge => { self.enemy_emblem_rate = clamp_value(value, 0, 100); },
            DVCVariable::EnemyItemDropGauge => { self.enemy_drop_rate = clamp_value(value, 0, 100); },
            DVCVariable::UnitDeployment=> { self.deployment_type = value; },
            DVCVariable::EmblemDeployment => { self.emblem_deployment = value; },
            DVCVariable::EmblemRecruitment => { self.emblem_mode = value; },
            DVCVariable::UnitRecruitment => { self.random_recruitment = value; },
            DVCVariable::ExplorationItem => { self.exploration_items = value; },
            DVCVariable::RandomGifts => { self.random_gift_items = value; },
            DVCVariable::RandomJobOutfit => { self.random_class_outfits = value; },
            DVCVariable::BondSkillS => { self.bond_ring_skill_s_rate = clamp_value(value, 0, 100); }
            DVCVariable::BondSkillA => { self.bond_ring_skill_a_rate = clamp_value(value, 0, 100); },
            DVCVariable::BondSkillB => { self.bond_ring_skill_b_rate = clamp_value(value, 0, 100); },
            DVCVariable::BondSkillC  => { self.bond_ring_skill_c_rate = clamp_value(value, 0, 100); },
            DVCVariable::Continuous => { self.continuous = value; },
            DVCVariable::EmblemAppearance => { self.emblem_appearance = value; },
            DVCVariable::FogOfWar => { self.fow = value; },
            DVCVariable::EngraveLevel => { self.engrave_settings = value; },    //Setting
            DVCVariable::TerrainEffect => { self.terrain = value; },
            DVCVariable::JobLearn => { self.learn_skill = value; },
            DVCVariable::BodyScaling => { self.body_scale = value; },
            DVCVariable::GenderClassM => { self.class_m = value; }
            DVCVariable::GenderClassF => { self.class_f = value; },
            DVCVariable::EnemyCapabilityOffset => { self.enemy_off = value as i8; },
            _ => {}
        }
    }
    pub fn get_value(&self, dvc_key: DVCVariable) -> i32 {
        match dvc_key {
            DVCVariable::Seed => self.seed as i32,
            DVCVariable::EmblemWepProf => self.emblem_weap_prof_mode,
            DVCVariable::EmblemSyncSkill => self.sync_skill_mode,
            DVCVariable::EmblemEngageSkill => { self.engage_skill_mode },
            DVCVariable::EmblemInherit => { self.emblem_inheritance_mode },
            DVCVariable::ClassMode => { self.random_job },
            DVCVariable::UnitInventory => { self.random_inventory },
            DVCVariable::PersonalGrowthMode => { self.random_grow },
            DVCVariable::Reclassing => { self.random_reclass },
            DVCVariable::SingleJob => { self.single_class },
            DVCVariable::BattleStyles => { self.random_battle_styles },
            DVCVariable::InteractSetting => { self.interaction_type },
            DVCVariable::GenericAppearance=> { self.generic_mode}
            DVCVariable::EnemySkillGauge => { self.random_enemy_skill_rate},
            DVCVariable::EnemyJobGauge => { self.random_enemy_job_rate},
            DVCVariable::EnemyRevivalStone=> { self.revival_stone_rate},
            DVCVariable::EnemyEmblemGauge => { self.enemy_emblem_rate},
            DVCVariable::EnemyItemDropGauge => { self.enemy_drop_rate},
            DVCVariable::UnitDeployment=> { self.deployment_type },
            DVCVariable::EmblemDeployment => { self.emblem_deployment },
            DVCVariable::EmblemRecruitment => { self.emblem_mode },
            DVCVariable::UnitRecruitment => { self.random_recruitment },
            DVCVariable::ExplorationItem => { self.exploration_items },
            DVCVariable::RandomGifts => { self.random_gift_items },
            DVCVariable::RandomJobOutfit => { self.random_class_outfits },
            DVCVariable::BondSkillS => { self.bond_ring_skill_s_rate},
            DVCVariable::BondSkillA => { self.bond_ring_skill_a_rate},
            DVCVariable::BondSkillB => { self.bond_ring_skill_b_rate},
            DVCVariable::BondSkillC  => { self.bond_ring_skill_c_rate},
            DVCVariable::Continuous => { self.continuous },
            DVCVariable::EmblemAppearance => { self.emblem_appearance },
            DVCVariable::FogOfWar => { self.fow },
            DVCVariable::EngraveLevel => { self.engrave_settings },
            DVCVariable::TerrainEffect => { self.terrain },
            DVCVariable::JobLearn => { self.learn_skill },
            DVCVariable::BodyScaling => { self.body_scale }
            DVCVariable::GenderClassM => { self.class_m }
            DVCVariable::GenderClassF => { self.class_f }
            DVCVariable::EnemyCapabilityOffset => self.enemy_off as i32,
            _ => { 0 }
        }
    }
    pub fn get_flag(&self, flag: DVCFlags) -> bool {
        let idx = flag as i32;
        self.flags & (1 << idx) != 0
    }
}