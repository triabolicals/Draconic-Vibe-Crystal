use engage::gamedata::{Gamedata, GodData, JobData, PersonData};
use engage::gamedata::god::GodGrowthData;
use engage::gamedata::skill::{SkillArray, SkillArrayEntity, SkillData, SkillDataCategorys};
use engage::unit::Unit;
use engage::gamevariable::GameVariableManager;
use engage::random::Random;
use num_traits::FromPrimitive;
use crate::config::{DVCFlags, DVCVariables};
use crate::randomizer::{get_dvc_black_list_read, Randomizer};
use crate::randomizer::blacklist::{DVCBlackLists};
use crate::randomizer::data::GameData;
use crate::randomizer::data::sync::EmblemSkillPool;
use crate::randomizer::person::unit;
use crate::randomizer::skill::ORIGINAL_HASHS;

pub struct SkillPool {
    pub emblem_stat_boost: [i32; 110],
    pub engage_skill: Vec<i32>,
    pub engage_attacks: Vec<(i32, i32)>,    // Skill Hash and Weapon Prohibit
    pub sp_cost: Vec<(i32, u16)>,
    pub pool: Vec<i32>, // General Skill Pool
    pub non_upgrades: Vec<i32>, // Personal + Jobs
    pub emblem_skill: EmblemSkillPool,
    pub job_restrictions: SkillRestriction,
}
pub struct SkillWeaponRestrictions {
    pub hash: i32,
    pub mask: i32,
}

pub struct SkillRestriction {
    pub list: Vec<SkillWeaponRestrictions>,
}
impl SkillRestriction {
    pub fn is_valid_for_weapon_mask(&self, skill: &SkillData, job_mask: i32) -> bool {
        !self.list.iter().any(|restriction| restriction.hash == skill.parent.hash && restriction.mask & job_mask == 0)
    }
    pub fn init() -> Self {
        let mut list: Vec<SkillWeaponRestrictions>  = Vec::new();
        SkillData::get_list().unwrap().iter().for_each(|skill|{
            let weapon_restrict = skill.weapon_prohibit.value;
            if weapon_restrict != 0 && skill.flag & 63 == 0 {
                let hash1 =  skill.parent.hash;
                if !list.iter().any(|s| s.hash == hash1) { list.push( SkillWeaponRestrictions { hash: hash1, mask: 1024 - weapon_restrict }); }
            }
        });
        include_str!("restrict.txt").lines()
            .into_iter()
            .for_each(|line|{
                let new_line: Vec<_> = line.split_whitespace().collect();
                if new_line.len() >= 2 {
                    if let Some((skill, mask)) = SkillData::get(new_line[0]).zip(new_line[1].parse::<i32>().ok()) {
                        list.push( SkillWeaponRestrictions { hash: skill.parent.hash, mask});
                    }
                }
            });
        // println!("{} skills in the restrict skills list.", list.len());
        SkillRestriction{ list }
    }
}



pub struct SkillsList { pub list: Vec<SkillArrayElement>, }

impl SkillsList {
    pub fn new() -> Self { SkillsList { list: vec![] } }
    pub fn from_skill_array(array: &SkillArray) -> Self {
        Self { list: array.iter().map(|e| SkillArrayElement::from(e) ).collect() }
    }
    pub fn set_skill_array(&self, array: &SkillArray) {
        array.clear();
        self.list.iter().for_each(|e| {
            let cat = SkillDataCategorys::from_i32(e.category).unwrap();
            if let Some(s) = SkillData::try_index_get(e.index) { array.add_skill(s, cat, 0); }
        });
    }
}
#[derive(Clone)]
pub struct SkillArrayElement {
    pub index: i32,
    pub category: i32,
}

impl From<&SkillArrayEntity> for SkillArrayElement {
    fn from(entity: &SkillArrayEntity) -> Self { Self { index: (entity.value & 0xFFF) as i32, category: (entity.value >> 28) as i32, } }
}

impl SkillPool {
    pub fn init() -> Self {
       // println!("Initializing Skill Pool");
        // fix_priority_data();
        let mut emblem_stat_boost: [i32; 110] = [-1; 110];
        SkillData::get_list().unwrap().iter()
            .filter(|skill|
                skill.enhance_value.iter().filter(|x| **x > 0).count() == 1 &&
                    skill.flag & 1 == 1 && skill.parent.index > 60 &&
                    skill.condition.is_none_or(|c| c.is_null()))
            .for_each(|skill|{
                let enhance = &skill.enhance_value;
                for x in 0..11 {
                    if x == 9 { continue; }
                    if enhance[x] > 0 {
                        let level = if skill.priority > 0 && skill.priority < 11 { skill.priority - 1 } else { 0 } as usize;
                        let index = x*10 + level;
                        if emblem_stat_boost[index] == -1 { emblem_stat_boost[index] = skill.parent.index; }
                    }
                }
            });
        for x in 0..11 {
            if x == 9 { continue; }
            let mut last_non_zero = 0;
            for y in 0..10 {
                if emblem_stat_boost[x*10 + y] > 0 { last_non_zero = emblem_stat_boost[x*10 + y]; }
                else { emblem_stat_boost[x*10 + y] = last_non_zero; }
            }
        }
        let mut engage_attacks = vec![];
        GodData::get_list().unwrap().iter()
            .filter(|s| s.engage_attack.is_some() || s.engage_attack_link.is_some() )
            .for_each(|g|{
                if let Some(atk) = g.engage_attack.and_then(|sid| SkillData::get(sid) ) {
                    let value = (atk.parent.hash, atk.weapon_prohibit.value);
                    if !engage_attacks.contains(&value) { engage_attacks.push(value); }
                }
                if let Some(atk) = g.engage_attack_link.and_then(|sid| SkillData::get(sid) )  {
                    let value = (atk.parent.hash, atk.weapon_prohibit.value);
                    if !engage_attacks.contains(&value) { engage_attacks.push(value); }
                }
            });
        let blacklist = get_dvc_black_list_read();
        let mut non_upgrades = vec![];
        let mut pool = vec![];
        let mut help = String::new();
        SkillData::get_list().unwrap()
            .iter()
            .filter(|s| s.help.is_some_and(|h| h.to_string().len() > 2) && !blacklist.ignore_skill(s) && s.low_skill.is_none() )
            .for_each(|skill|{
                let help2 = skill.help.unwrap().to_string();
                if help2 != help  {
                    help = help2.clone();
                    if skill.root_command_skill.is_none() {
                        if !skill.can_override_skill() { non_upgrades.push(skill.parent.hash); }
                        pool.push(skill.parent.hash);
                        // println!("Added Skill #{}", skill.parent.index);
                    }
                }
            });
        println!("Skill Pool: {}", pool.len());
        println!("Non-Upgrades: {}", non_upgrades.len());
        Self {
            job_restrictions: SkillRestriction::init(),
            emblem_skill: EmblemSkillPool::init(),
            engage_skill: GodGrowthData::get_level_lists().entries.iter()
                .filter(|x| x.key.is_some() )
                .filter_map(|x| x.value[0].engage_skills.iter().find(|x| x.get_skill().is_some_and(|s| s.flag & 1 == 0)))
                .map(|x| (x.value as i32) & 0xFFF)
                .collect(),
            sp_cost: SkillData::get_list().unwrap().iter().filter(|x| x.inheritance_cost > 0 ).map(|x| (x.parent.hash, x.inheritance_cost)).collect(),
            engage_attacks, emblem_stat_boost, pool, non_upgrades,
        }
    }
    pub fn is_emblem_stat_boost(&self, skill_entity: &SkillArrayEntity) -> bool { self.is_emblem_stat( (skill_entity.value & 0xFFF) as i32) }
    pub fn is_emblem_stat(&self, index: i32) -> bool { self.emblem_stat_boost.iter().any(|x| *x == index && index > 0 ) }
    pub fn reset_sp_cost(&self) {
        self.sp_cost.iter()
            .map(|(hash, sp)| (SkillData::try_get_hash_mut(*hash).unwrap(), *sp))
            .for_each(|(skill, sp)| { skill.inheritance_cost = sp; });
    }
    pub fn randomize(&self, data: &GameData) {
        // if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().skill_randomized { return; }
        let personal_bl = DVCBlackLists::get_read();
        // crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.skill_randomized = true).unwrap();
        let rng = Random::new(2 * DVCVariables::get_seed() as u32);

        let mut skill_pool: Vec<_> = self.non_upgrades.iter().filter(|x| !personal_bl.personal_skill.indexes.contains(*x)).map(|x| *x).collect();
        let mut need_personal = vec![];
        data.playables.iter()
            .for_each(|p| {
                let person = PersonData::try_get_hash(p.hash).unwrap();
                let personal_key = format!("G_P_{}", person.pid);
                if GameVariableManager::exist(&personal_key) {
                    let hash = GameVariableManager::get_number(&personal_key);
                    if let Some(pos) = skill_pool.iter().position(|x| *x == hash) { skill_pool.swap_remove(pos); } else { need_personal.push(person); }
                }
                else {
                    GameVariableManager::make_entry_norewind(&format!("G_P_{}", person.pid), 0);
                    need_personal.push(person);
                }
            });
        need_personal.iter().for_each(|p| {
            if let Some(skill_hash) = skill_pool.get_remove(rng) {
                GameVariableManager::set_number(&format!("G_P_{}", p.pid), skill_hash);
                // println!("{} Personal Skill: {}", Mess::get_name(p.pid), Mess::get( SkillData::try_get_hash(skill_hash).unwrap().name.unwrap()));
            }
        });
        let mut jobs = Vec::new();
        JobData::get_list().unwrap().iter()
            .filter(|job| { (job.flag.value & 3 != 0 || job.learn_skill.is_some()) && (job.is_high() || (job.is_low() && job.max_level == 40)) })
            .for_each(|job| {
                let mut weapon_mask = 0;
                for x in 1..9 { if job.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }
                let job_key = format!("G_L_{}", job.jid);
                let hash = GameVariableManager::get_number(job_key.as_str());
                if let Some(pos) = skill_pool.iter().position(|&i| i == hash) { skill_pool.swap_remove(pos); } else {
                    GameVariableManager::make_entry_norewind(job_key.as_str(), 0);
                    jobs.push(job);
                }
            });
        jobs.iter().for_each(|j|{
            let len = skill_pool.len();
            let mut weapon_mask = 0;
            for x in 1..9 { if j.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }
            if len > 0 {
                let mut count = 0;
                let mut new_skill_hash = 0;
                loop {  // Keep randomizing if possible to prevent non-tome/non-staff jobs from getting tome/staff exclusive skills
                    new_skill_hash = skill_pool[rng.get_value(len as i32) as usize];
                    if let Some(skill) = SkillData::try_get_hash(new_skill_hash) {
                        if (skill.efficacy_ignore == 0 && data.skill_pool.job_restrictions.is_valid_for_weapon_mask(skill, weapon_mask)) || count >= 100 { break; }
                        count += 1;
                    }
                    else { break; }
                }
                GameVariableManager::set_number(format!("G_L_{}", j.jid), new_skill_hash);
            }
        });
    }
    pub fn get_random_skill(&self, difficulty: i32, rng: &Random) -> &'static SkillData {
        loop {
            if let Some(skill) = self.pool.get_random_element(rng).and_then(|s| SkillData::try_get_hash_mut(*s).filter(|v| is_custom_allow(v))){
                let chapter_completed = crate::continuous::get_story_chapters_completed() / 5;
                let mut current_skill = skill.parent.index;
                for _ in 0..chapter_completed {
                    if let Some(higher) = SkillData::try_index_get(current_skill) { current_skill = higher.parent.index; }
                }
                if let Some(skill) = SkillData::try_index_get(current_skill).filter(|s| is_custom_allow(s)) {
                    return skill;
                }
            }
        }
    }
    pub fn get_random_skill_job(&self, difficulty: i32, rng: &Random, unit: &Unit) -> Option<&'static SkillData> {
        let job_mask = unit.selected_weapon_mask.value | unit.job.get_weapon_mask2().value;
        let mut count = 0;
        while count < 50 {
            let skill = self.get_random_skill(difficulty, rng);
            if self.job_restrictions.is_valid_for_weapon_mask(skill, job_mask) && !unit::has_skill(unit, skill) && is_custom_allow(skill) {
                return Some(skill);
            }
            count += 1;
        }
        None
    }
    pub fn get_random_skill_dispos(&self, difficulty: i32, rng: &Random) -> &'static SkillData {
        loop {
            let skill = self.get_random_skill(difficulty, rng);
            if !self.job_restrictions.list.iter().any(|x| x.hash == skill.parent.hash) && is_custom_allow(skill) { return skill; }
        }
    }
}
fn is_custom_allow(skill: &SkillData) -> bool {
    if DVCFlags::CustomSkillEnemy.get_value() { true }
    else{
        let mut lowest = skill.parent.hash;
        let mut search = skill;
        while let Some(low) = search.low_skill.as_ref() {
            lowest = low.parent.hash;
            search = low;
        }
        ORIGINAL_HASHS.contains(&lowest)
    }
}