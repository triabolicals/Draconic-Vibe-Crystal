use std::collections::{HashMap};
use engage::gamedata::ring::RingData;
use engage::gamedata::skill::SkillArray;
use engage::random::Random;
use crate::config::DVCVariables;
use crate::enums::EIRIKA_TWIN_SKILLS;
use crate::randomizer::data::GameData;
use crate::randomizer::{DVCFlags, Randomizer};
use crate::randomizer::blacklist::DVCBlackLists;
use crate::utils::{create_rng, get_rng};
use super::*;

const EIRIKA_HASH: [i32; 12] = [ 1166279381, 1203307432, 244739392, 446418448, 933063973, -1323396701, 	1137740356, -1874837901, 919405771, -213541829, -1311625676, 1981791378];
const GAMBITS_HASH: [i32; 6] = [ 1238512915, 924387794, -47761637, -1370924721, -1158568192, -201508301, ];
const NONE_SID: i32 = 359194254;
const FRIENDLY_RIVARLRY: i32 = 1238512915;
const NIGHT_DAY: i32 = 924387794;
const BOOK_OF_WORLDS: i32 = 106021179;

pub const DARK_EMBLEM_SKILLS: [(i32, i32); 5] = [
    (40103191, 976162037), (870225249, 557493185), (1467048018, 659396462), (1876651662, -1416849344),
    (-1255795212, 459105266)

];

#[derive(Clone, PartialEq)]
pub struct SkillGroup {
    pub group: i32,
    pub indexes: Vec<i32>,
    pub emblem_index: Option<usize>,
    pub can_inherit: bool,
    pub is_sync: bool,
}

impl SkillGroup {
    pub fn new(skill: &SkillData) -> Self {
        let mut indexes = vec![skill.parent.index];
        if skill.parent.hash == BOOK_OF_WORLDS || skill.group == 0 || skill.high_skill.is_none() {
            Self { group: 0, indexes, can_inherit: skill.inheritance_cost != 0, emblem_index: None, is_sync: false }
        }
        else {
            let mut current = skill;
            while let Some(s) = current.high_skill {
                indexes.push(s.parent.index);
                current = s;
            }
            /*
            if skill.group > 0 {
                if let Some(name) = skill.name.map(|m| Mess::get(m)) {
                    println!("#{} {}: Group: {}, #{}", skill.parent.index, name, skill.group, indexes.len());
                }
            }
             */
            Self { group: skill.group, indexes, can_inherit: skill.inheritance_cost != 0, emblem_index: None, is_sync: false }
        }

    }
    pub fn new_emblem(skill: &SkillData, emblem: usize, is_sync: bool) -> Self {
        let mut new = Self::new(skill);
        new.emblem_index = Some(emblem);
        new.is_sync = is_sync;
        new
    }
    pub fn from_index(index_low: i32, can_inherit: bool, group: i32) -> Self {
        let mut indexes = vec![index_low];
        Self { indexes, can_inherit, group, emblem_index: None, is_sync: false }
    }
    pub fn new_from_index(index_low: i32, index_high: i32, can_inherit: bool, group: i32, emblem: usize, is_sync: bool) -> Self {
        let mut indexes = vec![index_low, index_high];
        Self { indexes, can_inherit, group, emblem_index: Some(emblem), is_sync }
    }
    pub fn has_skill(&self, index: i32) -> bool { self.indexes.iter().any(|x| *x == index) }
}

pub struct EmblemSkillPool {
    pub engage_skill: Vec<i32>,
    pub groups: Vec<Vec<SkillGroup>>,
    pub syncs: Vec<Vec<SkillGroup>>,
    pub extra_syncs: Vec<i32>,
    pub inherit_only: Vec<SkillGroup>,
}

impl EmblemSkillPool {
    pub fn init() -> Self {
        let mut groups = vec![Vec::new(); 6];
        let mut syncs = vec![Vec::new(); 6];
        let mut engage_skill = vec![];
        let mut inherit_only: Vec<SkillGroup> = vec![];
        for x in 0..6 {
            if let Some(s) = SkillData::try_get_hash(EIRIKA_HASH[x]).zip(SkillData::try_get_hash(EIRIKA_HASH[x+6])){
                syncs[1].push(SkillGroup::new_from_index(s.0.parent.index, s.1.parent.index, s.0.inheritance_cost != 0, s.0.group, 11, true));
                groups[1].push(SkillGroup::new_from_index(s.0.parent.index, s.1.parent.index, s.0.inheritance_cost != 0, s.1.group, 11, true));
            }
        }
        let mut extra_syncs = Vec::new();
        for x in 3..6 { if let Some(s) = SkillData::try_get_hash(GAMBITS_HASH[x]) { extra_syncs.push(s.parent.index); } }

        RingData::get_list().unwrap().iter()
            .flat_map(|x| x.get_equip_skills().iter() )
            .filter(|x| x.get_skill().is_some_and(|x| x.flag & 1 == 0))
            .for_each(|s| {
                let index = (s.value as i32) & 0xFFF;
                if !extra_syncs.contains(&index) { extra_syncs.push(index); }
            });
        let mut count = 0;
        GodData::get_list().unwrap().iter().filter(|x| x.force_type == 0)
            .enumerate()
            .flat_map(|x| x.1.get_level_data().zip(Some(x.0)))
            .for_each(|(x, i)|{
                x[0].engaged_skills.iter()
                    .flat_map(|x| x.get_skill())
                    .filter(|x| x.flag & 1 == 0 && !EIRIKA_HASH.iter().any(|&h| h == x.parent.hash) && !GAMBITS_HASH.iter().any(|&h| h == x.parent.hash))
                    .map(|skill| get_lowest_priority(skill))
                    .for_each(|x| {
                        let diff = if x.parent.hash == BOOK_OF_WORLDS { 0 } else { get_priority_count(x) };
                        if let Some(s_group) = syncs.get_mut(diff as usize).filter(|s| !s.iter().any(|v| v.has_skill(x.parent.index))){
                            s_group.push(SkillGroup::new_emblem(x, i, false));
                        }
                        if let Some(s_group) = groups.get_mut(diff as usize).filter(|s| !s.iter().any(|v| v.has_skill(x.parent.index))){
                            s_group.push(SkillGroup::new_emblem(x, i, false));
                        }
                    });
                x[0].synchro_skills.iter()
                    .flat_map(|x| x.get_skill())
                    .filter(|x| x.flag & 1 == 0 && !EIRIKA_HASH.iter().any(|&h| h == x.parent.hash) && !GAMBITS_HASH.iter().any(|&h| h == x.parent.hash))
                    .map(|skill| get_lowest_priority(skill))
                    .for_each(|x| {
                        let diff = if x.parent.hash == BOOK_OF_WORLDS { 0 } else { get_priority_count(x) };
                        if let Some(s_group) = syncs.get_mut(diff as usize).filter(|s| !s.iter().any(|v| v.has_skill(x.parent.index))){
                            s_group.push(SkillGroup::new_emblem(x, i, true));
                        }
                        if let Some(s_group) = groups.get_mut(diff as usize).filter(|s| !s.iter().any(|v| v.has_skill(x.parent.index))){
                            s_group.push(SkillGroup::new_emblem(x, i, true));
                        }
                    });
                if let Some(z) = x[0].engage_skills.iter()
                    .flat_map(|x| x.get_skill())
                    .find(|x| x.flag & 1 == 0).map(|skill| skill.parent.index) { engage_skill.push(z); }
                count+= 1;
            });
        GodData::get_list().unwrap().iter()
            .filter(|x| x.force_type == 0 && x.main_data.parent.hash == x.parent.hash)
            .filter_map(|g| GodGrowthData::try_get_from_god_data(g))
            .flat_map(|g| g.iter())
            .flat_map(|grow| grow.get_inheritance_skills())
            .flat_map(|x| x.iter())
            .flat_map(|sid| SkillData::get(*sid))
            .for_each(|inherit_skill|{
                if inherit_skill.inheritance_cost > 0 && inherit_only.iter().all(|x| !x.has_skill(inherit_skill.parent.index)) {
                    inherit_only.push(SkillGroup::new(inherit_skill));
                }
            });

        let black_list = DVCBlackLists::get_read();
        SkillData::get_list().unwrap().iter()
            .filter(|s|
                !GAMBITS_HASH.iter().any(|&h| h == s.parent.hash) && !EIRIKA_HASH.iter().any(|&h| h == s.parent.hash) &&
                    s.low_skill.is_none() && s.priority < 50 && s.flag & 959 == 0 && black_list.skill.allowed_index(s.parent.hash)
                    && s.help.is_some_and(|m| Mess::get(m).to_string().len() > 2 ) && s.name.is_some_and(|m| Mess::get(m).to_string().len() > 2 )
                    && !s.sid.str_contains("E00") && !s.is_style_skill()
            )
            .for_each(|x| {
                let diff = if x.parent.hash == BOOK_OF_WORLDS { 0 } else { get_priority_count(x) };
                if let Some(s_group) = groups.get_mut(diff as usize).filter(|s| !s.iter().any(|v| v.has_skill(x.parent.index))){
                    let s = SkillGroup::new(x);
                    s_group.push(s);
                }
            });
        Self { syncs, groups, extra_syncs, engage_skill, inherit_only }
    }
    pub fn get_skill_pool(&self, is_sync: bool, diff: usize) -> EmblemSkillRandomizerPool {
        let pool = if is_sync { &self.syncs[diff] } else { &self.groups[diff] };
        let mut inherit: Vec<SkillGroup> = self.inherit_only.iter().filter(|l| l.indexes.len() == (diff+1)).map(|x| x.clone()).collect();
        let mut non_inherit: Vec<_> = pool.iter().filter(|x| !x.can_inherit).map(|x| x.clone()).collect::<Vec<_>>();
        //inherit.extend(self.inherit_only.iter().filter(|p| p.indexes.len() == diff).map(|x| x.clone()));

        if diff == 0 {
            self.extra_syncs.iter().for_each(|x| {
                let s = SkillGroup::from_index(*x, false, 1000);
                non_inherit.push(s);
            });
        }
        EmblemSkillRandomizerPool { inherit, non_inherit, }
    }
    pub fn get_chaos_inherit(&self) -> Vec<SkillGroup> {
        let mut s: Vec<SkillGroup> = self.groups.iter().flat_map(|v| v.iter()).map(|x| x.clone()).collect();
        s.extend(self.extra_syncs.iter().map(|x| SkillGroup::from_index(*x, false, 1000)));
        s.extend(self.engage_skill.iter().map(|x| SkillGroup::from_index(*x, false, 1000)));
        s
    }
}

pub struct EmblemSkillRandomizer {
    pub random_skill: HashMap<i32, i32>,
    pub chaos_skill: HashMap<i32, i32>,
    pub chaos_inherit: HashMap<i32, i32>,
    pub chaos_engage: HashMap<i32, i32>,
    pub random_engage: HashMap<i32, i32>,
    pub random_inherit: HashMap<i32, i32>,
    pub unit_inherit: Vec<HashMap<i32, i32>>,
    pub random_gambits: [i32; 3],
    pub chaos_gambits: [i32; 3],
}

pub struct EmblemSkillRandomizerPool {
    inherit: Vec<SkillGroup>,
    non_inherit: Vec<SkillGroup>,
}
impl EmblemSkillRandomizerPool {
    pub fn get_random2(&mut self, rng: &Random) -> Option<SkillGroup> {
        if self.inherit.len() > 0 {
            if self.non_inherit.len() > 0 && rng.get_value(2) == 1 { self.non_inherit.get_remove(rng) }
            else { self.inherit.get_remove(rng) }
        }
        else { self.non_inherit.get_remove(rng) }
    }
    pub fn get_random(&mut self, rng: &Random, is_inherit: bool, exclude_extra: bool) -> Option<SkillGroup> {
        let mut pool = if is_inherit { &mut self.inherit } else { &mut self.non_inherit };
        let pool_len = pool.len() as i32;
        if !is_inherit && exclude_extra && pool_len > 5 {
            let mut counter = 0;
            while counter < 10 {
                let selection = rng.get_value( pool_len ) as usize;
                if pool[selection].group != 1000 { return Some(pool.swap_remove(selection)); }
                else { counter += 1; }
            }
            None
        }
        else if pool_len > 1 {
            let selection = rng.get_value( pool_len ) as usize;
            Some(pool.swap_remove(selection))
        }
        else { None }
    }
}
impl EmblemSkillRandomizer {
    pub fn init(playables: usize) -> Self {
        Self {
            random_gambits: [-1; 3],
            chaos_gambits: [-1; 3],
            random_skill: HashMap::new(),
            random_inherit: HashMap::new(),
            chaos_skill: HashMap::new(),
            chaos_inherit: HashMap::new(),
            chaos_engage: HashMap::new(),
            random_engage: HashMap::new(),
            unit_inherit: vec![HashMap::new(); playables],
        }
    }
    pub fn reset(&mut self) {}
    pub fn randomize(&mut self, game_data: &GameData) {
        println!("Randomizing Emblem Skills...");
        let rng = get_rng();
        let pool = &game_data.skill_pool.emblem_skill;
        self.random_skill.clear();
        self.chaos_skill.clear();
        self.chaos_inherit.clear();
        self.random_engage.clear();
        self.chaos_engage.clear();
        self.random_inherit.clear();
        let mut has_extra = [false; 200];
        let skill_data = SkillData::get_list_mut().unwrap();
        pool.groups.iter().enumerate().for_each(|(diff, g)| {
            let increase = ((9000 - 2500) / (50 * (diff + 1))) as i32;
            g.iter().for_each(|x| {
                let mut sp = 100 + 50 * rng.get_value(49);
                x.indexes.iter().for_each(|&x| {
                    skill_data[x as usize].pad4 = sp;
                    sp += increase * rng.get_value(increase);
                });
            })
        });
        let emblem_count = game_data.emblem_pool.emblem_data.len();
        let chaos_pool = pool.get_chaos_inherit();

        // Chaos Inherit
        let inherit_skills: Vec<i32> = pool.inherit_only.iter().flat_map(|v| v.indexes.iter()).map(|v| *v).collect();
        let mut chaos_random_pool: Vec<i32> = chaos_pool.iter().flat_map(|s| s.indexes.last()).map(|x| *x).collect();
        if chaos_random_pool.len() < inherit_skills.len() {
            chaos_random_pool.extend(chaos_pool.iter().filter(|s| s.indexes.len() > 2)
                .flat_map(|s| s.indexes.first()).map(|x| *x));
        }
        self.chaos_inherit = inherit_skills.iter().zip(chaos_random_pool.iter()).map(|(x, y)| (*x, *y)).collect();
        chaos_random_pool.shuffle(rng, emblem_count as i32);

        // Random Inherit
        for diff in 0..6 {
            let mut avail_sync = pool.get_skill_pool(true, diff);
            pool.inherit_only.iter().filter(|&x| x.indexes.len() == (diff+1))
                .for_each(|inherit_only|{
                    if let Some(sync) = avail_sync.inherit.get_remove(rng) {
                        self.random_inherit.extend(inherit_only.indexes.iter().zip(sync.indexes.iter()));
                    }
                });
            if let Some(skill_groups) = pool.syncs.get(diff) {
                skill_groups.iter().filter(|group| group.can_inherit)
                    .for_each(|original_skill_group| {
                        if let Some(inherit) = avail_sync.inherit.get_remove(rng) {
                            self.random_inherit.extend(original_skill_group.indexes.iter().zip(inherit.indexes.iter()));
                        }
                    });
            }
        }
        pool.syncs.iter().enumerate().for_each(|(diff, skill_groups)| {
            let mut avail_sync = pool.get_skill_pool(true, diff);
            let mut avail_chaos = pool.get_skill_pool(false, diff);
            skill_groups.iter().for_each(|original_skill_group|{
                let extra1 = original_skill_group.emblem_index.map(|x1| has_extra[x1]).unwrap_or(false);
                let extra2 = original_skill_group.emblem_index.map(|x1| has_extra[x1+50]).unwrap_or(false);
                // Sync to Random Sync [Inherit must match]
                if let Some(sync_group) = avail_sync.get_random(rng, original_skill_group.can_inherit, extra1) {
                    if sync_group.group == 1000 { if let Some(x1) = original_skill_group.emblem_index { has_extra[x1] = true; } }
                    self.random_skill.extend(original_skill_group.indexes.iter().zip(sync_group.indexes.iter()));
                }
                // Skill to Chaos Skill
                if let Some(chaos_group) = avail_chaos.get_random(rng, original_skill_group.can_inherit, extra2) {
                    if chaos_group.group == 1000 { if let Some(x1) = original_skill_group.emblem_index { has_extra[x1+50] = true; } }
                    self.chaos_skill.extend(original_skill_group.indexes.iter().zip(chaos_group.indexes.iter()));
                }
            });
            if diff == 0 {
                for x in 0..3 {
                    self.random_gambits[x] = avail_sync.get_random2(rng).map(|x| x.indexes[0]).unwrap_or(-1);
                    self.chaos_gambits[x] = avail_chaos.get_random2(rng).map(|x| x.indexes[0]).unwrap_or(-1);
                }
                pool.engage_skill.iter().enumerate().for_each(|(i, x)| {
                    if let Some(g) = avail_chaos.get_random(rng, false, has_extra[i + 50])
                        .or_else(|| avail_chaos.get_random(rng, true, has_extra[i + 50])) 
                    {
                        self.chaos_engage.insert(*x, g.indexes[0]);
                    }
                });
            }
        });
        let mut engage_pool: Vec<_> = pool.engage_skill.iter().filter(|x| **x > 0).map(|x| *x).collect();
        pool.engage_skill.iter().enumerate().filter(|x| *x.1 > 0).for_each(|(i, x)| {
            if engage_pool.len() > 1 {
                let selection = rng.get_value(engage_pool.len() as i32) as usize;
                let index = engage_pool.swap_remove(selection);
                self.random_engage.insert(*x, index);
            } else { self.random_engage.insert(*x, engage_pool[0]); }
        });
        DARK_EMBLEM_SKILLS.iter().for_each(|x| {
            if let Some(x) = SkillData::try_get_hash(x.0).zip(SkillData::try_get_hash(x.1))
                .map(|s| (s.0.parent.index, s.1.parent.index))
            {
                if let Some(s) = self.random_engage.get(&x.1) { self.random_engage.insert(x.0, *s); }
                if let Some(s) = self.chaos_engage.get(&x.1) { self.chaos_engage.insert(x.0, *s); }
                if let Some(s) = self.random_skill.get(&x.1) { self.random_skill.insert(x.0, *s); }
                if let Some(s) = self.chaos_skill.get(&x.1) { self.chaos_skill.insert(x.0, *s); }
            }
        });
        if self.unit_inherit.len() < game_data.playables.len() { self.unit_inherit.resize(game_data.playables.len(), HashMap::new()); }
        chaos_random_pool = chaos_pool.iter().flat_map(|s| s.indexes.last()).map(|x| *x).collect();
        game_data.playables.iter().map(|p| p.hash).zip(self.unit_inherit.iter_mut())
            .for_each(|(hash, mut inherit_list)|{
                chaos_random_pool.shuffle(create_rng(hash, 2), 5);
                *inherit_list = inherit_skills.iter().zip(chaos_random_pool.iter()).map(|(x, y)| (*x, *y)).collect();
            });
        // if let Ok(_) = self.print() { println!("Saved Inherit List"); }
    }
    pub fn get_unit_inherit(&self, skill: &SkillData, playable_index: i32) -> Option<&'static mut SkillData> {
        let i = if playable_index as usize >= self.unit_inherit.len() { 0 } else { playable_index as usize };
        self.unit_inherit[i].get(&skill.parent.index).and_then(|index| SkillData::try_index_get_mut(*index))
    }
    pub fn get_inherit(&self, skill: &SkillData) -> Option<&'static mut SkillData> {
        /*
        if let Some(name) = skill.name.map(|v| Mess::get(v)) {
            println!("Getting Inherit Replacement for Skill: #{} {}", skill.parent.index, name);
        }
         */
        match DVCVariables::EmblemInherit.get_value() {
            0 => Some(self.get_sync_replacement_index(skill.parent.index)),
            1 => self.random_inherit.get(&skill.parent.index).or_else(|| self.random_skill.get(&skill.parent.index)).cloned(),
            2 => self.chaos_inherit.get(&skill.parent.index).or_else(|| self.chaos_skill.get(&skill.parent.index)).cloned(),
            _ => None
        }.and_then(|index| SkillData::try_index_get_mut(index))
    }
    pub fn get_sync_replacement_index(&self, index: i32) -> i32 {
        match DVCVariables::EmblemSyncSkill.get_value() {
            1 => { self.random_skill.get(&index).map(|v| *v).unwrap_or(index) }
            2 => { self.chaos_skill.get(&index).map(|v| *v).unwrap_or(index) }
            _ => index
        }
    }
    pub fn get_sync_replacement_skill(&self, index: i32) -> Option<&'static SkillData> {
        SkillData::try_index_get(self.get_sync_replacement_index(index))
    }
    pub fn get_engage_skill_index(&self, index: i32) -> i32 {
        match DVCVariables::EmblemEngageSkill.get_value() {
            1 => { self.random_engage.get(&index).map(|v| *v).unwrap_or(index) }
            2 => { self.chaos_engage.get(&index).map(|v| *v).unwrap_or(index) }
            _ => index
        }
    }
    pub fn get_replacement_engage_enemy(&self, index: i32) -> Option<&'static SkillData> {
        SkillData::try_index_get(self.get_engage_skill_index(index))
            .and_then(|skill|
                DARK_EMBLEM_SKILLS.iter().find(|s| s.1 == skill.parent.hash)
                    .and_then(|x| SkillData::try_get_hash(x.0))
                    .or_else(|| Some(skill))
            )
    }
    pub fn get_replacement_skill_enemy(&self, index: i32) -> Option<&'static SkillData> {
        self.get_sync_replacement_skill(index)
            .and_then(|skill|
                DARK_EMBLEM_SKILLS.iter().find(|s| s.1 == skill.parent.hash)
                    .and_then(|x| SkillData::try_get_hash(x.0))
                    .or_else(|| Some(skill))
            )
    }
    pub fn change_skill_array(&self, array: &mut SkillArray) {
        array.list.iter_mut().for_each(|s| {
            let x = (s.value & 0xFFF) as i32;
            let new_x = self.get_sync_replacement_index(x);
            let new_x = if x != new_x { new_x } else { self.get_engage_skill_index(x) };
            s.set_index(new_x);
        })
    }
    pub fn commit_stats(&self, data: &GameData) {
        let stat = DVCFlags::EmblemStats.get_value();
        if stat {
            let rng = get_rng();
            data.emblem_pool.emblem_data.iter()
                .flat_map(|g| g.get_god().get_level_data())
                .flat_map(|l| l.iter_mut())
                .for_each(|g| {
                    let stat_boosts = get_stats_boosts(rng);
                    g.synchro_skills.list.iter_mut().filter(|s| data.skill_pool.is_emblem_stat_boost(s)).zip(stat_boosts.iter()).for_each(|(s, &i)| {
                        let level = s.get_skill().map(|s| if s.priority == 0 { 0 } else { s.priority - 1 } as usize).unwrap_or(0);
                        s.set_index(data.skill_pool.emblem_stat_boost[i * 10 + level]);
                    });
                    g.engaged_skills.list.iter_mut().filter(|s| data.skill_pool.is_emblem_stat_boost(s)).zip(stat_boosts.iter()).for_each(|(s, &i)| {
                        let level = s.get_skill().map(|s| if s.priority == 0 { 0 } else { s.priority - 1 } as usize).unwrap_or(0);
                        s.set_index(data.skill_pool.emblem_stat_boost[i * 10 + level]);
                    });
                });
        }
        else if DVCFlags::Initialized.get_value() { data.emblem_pool.emblem_data.iter().for_each(|e|{ e.reset_stats(data); }); }
    }
    pub fn commit(&self, data: &GameData) {
        let sync = DVCVariables::EmblemSyncSkill.get_value();
        let engage = DVCVariables::EmblemEngageSkill.get_value();
        data.emblem_pool.emblem_data.iter().for_each(|e|{ e.reset_all_skills(); });
        data.emblem_pool.emblem_persons.iter().for_each(|e|{ e.reset_skill(); });
        data.skill_pool.reset_sp_cost();
        SkillData::try_get_hash_mut(GAMBITS_HASH[2]).unwrap().change_skills.iter_mut().enumerate()
            .for_each(|(i, s)|{ *s = SkillData::try_get_hash_mut(GAMBITS_HASH[3+i]).unwrap(); });

        if engage != 0 {  // Engage Skill
            data.emblem_pool.emblem_data.iter()
                .flat_map(|g| g.get_god().get_level_data())
                .flat_map(|l| l.iter_mut())
                .for_each(|g| {
                    g.engage_skills.list.iter_mut().for_each(|s| { s.set_index(self.get_engage_skill_index(s.get_index())); });
                });
        }
        if sync != 0 {  // Sync
            data.emblem_pool.emblem_data.iter()
                .flat_map(|g| g.get_god().get_level_data())
                .flat_map(|l| l.iter_mut())
                .for_each(|g| {
                    g.synchro_skills.list.iter_mut().for_each(|s| { s.set_index(self.get_sync_replacement_index(s.get_index())); });
                    g.engaged_skills.list.iter_mut().for_each(|s| { s.set_index(self.get_sync_replacement_index(s.get_index())); });
                });

            // Gambit
            let gambit_list = if sync == 1 { &self.random_gambits } else { &self.chaos_gambits };
            if let Some(gambit) = SkillData::get_mut("SID_計略") {
                gambit.change_skills.iter_mut().zip(gambit_list.iter().flat_map(|&h| SkillData::try_index_get_mut(h)))
                    .for_each(|(s, new_s)|{ *s = new_s; });
            }
        }
        else {
            if let Some(gambit) = SkillData::get_mut("SID_計略") {
                let original_gambits = ["SID_計略_猛火計", "SID_計略_聖盾の備え", "SID_計略_毒矢"];
                gambit.change_skills.iter_mut().zip(original_gambits.iter().flat_map(|&h| SkillData::get_mut(h)))
                    .for_each(|(s, new_s)|{ *s = new_s; });
            }
        }
        if sync != 0 || engage != 0 {
            data.emblem_pool.emblem_persons.iter().map(|e| e.get_person() ).for_each(|p| {
                self.change_skill_array(p.get_common_skills());
                self.change_skill_array(p.get_normal_skills());
                self.change_skill_array(p.get_hard_skills());
                self.change_skill_array(p.get_lunatic_skills());
            });
        }
    }
    /*
    pub fn print(&self) -> Result<bool, std::io::Error> {
        let mut file = File::options().create(true).write(true).truncate(true).open("sd:/Draconic Vibe Crystal/emblem_skill.txt")?;
        writeln!(&mut file, "Draconic Vibe Crystal: {} Emblem Skill Randomization", VERSION)?;
        let skill_count = SkillData::get_count();
        let mut skill_list = vec![-1; SkillData::get_count() as usize];
        for skill_set in [
            ("Random Skill List", &self.random_skill),
            ("Chaos Skill List", &self.chaos_skill),
            ("Random Inherit List", &self.random_inherit),
            ("Chaos Inherit List", &self.chaos_inherit),
        ]
        {
            skill_list.iter_mut().for_each(|x| *x = -1);
            skill_set.1.iter().for_each(|s| if *s.0 < skill_count { skill_list[*s.0 as usize] = *s.1; });
            writeln!(&mut file, "\n{} [{} Swaps]", skill_set.0, skill_set.1.len())?;
            skill_list.iter().enumerate().filter(|x| *x.1 > 0)
                .for_each(|(i,s)|{
                    if let Some((s1, s2)) = SkillData::try_index_get(i as i32).zip(SkillData::try_index_get(*s)){
                        let name1 = s1.name.map(|v| Mess::get(v)).filter(|n| n.to_string().len() > 2 )
                            .map(|v| format!("{}: {}", s1.parent.index, v))
                            .unwrap_or(format!("#{}: {}", s1.parent.index, s1.sid));
                        let name2 = s2.name.map(|v| Mess::get(v)).filter(|n| n.to_string().len() > 2 )
                            .map(|v| format!("{}: {}", s2.parent.index, v))
                            .unwrap_or(format!("#{}: {}", s2.parent.index, s1.sid));
                        writeln!(&mut file, "{} to {}", name1, name2).unwrap();
                    }
                });
        }
        Ok(true)
    }
    */
}

pub fn get_lowest_priority(skill: &'static SkillData) -> &'static SkillData {
    if let Some(pos) = EIRIKA_HASH.iter().position(|s| *s == skill.parent.hash) {
        if pos >= 6 { return SkillData::get( EIRIKA_TWIN_SKILLS[ pos - 6 ]).unwrap(); }
        else { return skill; }
    }
    if skill.low_skill.is_none() { skill }
    else {
        let mut current = skill;
        while let Some(s) = current.low_skill { current = s ; }
        current
    }
}
pub fn get_highest_priority(skill: &'static SkillData) -> &'static SkillData {
    if let Some(pos) = EIRIKA_HASH.iter().position(|s| *s == skill.parent.hash) {
        if pos < 6 { return SkillData::get( EIRIKA_TWIN_SKILLS[ 6 + pos ]).unwrap(); }
        else { return skill; }
    }
    if skill.high_skill.is_none() { skill }
    else {
        let mut current = skill;
        while let Some(s) = current.high_skill.filter(|s| s.name.is_some()) { current = s ; }
        current
    }
}
fn get_priority_count(skill: &'static SkillData) -> i32 {
    let mut count = 0;
    let mut current = get_lowest_priority(skill);
    if current.high_skill.is_none() { return 0;}
    while let Some(s) = current.high_skill.filter(|x| x.name.is_some() && x.help.is_some()){
        count += 1;
        current = s ;
    }
    count
}

pub fn get_skill_level(index: &'static SkillData) -> i32 {
    let mut count = 0;
    let mut lowest = get_lowest_priority(index);
    if lowest.parent.index == index.parent.index { return 0; }
    while let Some(s) = lowest.high_skill.filter(|x| x.name.is_some() && x.help.is_some()){
        if s.parent.index > index.parent.index { break; }
        count += 1;
        lowest = s;
    }
    count
}

pub fn get_stats_boosts(rng: &Random) -> [usize; 5] {
    let mut v: Vec<usize> = (0..11).collect();
    v.remove(9);    // remove sight
    let mut out: [usize; 5] = [0; 5];
    for x in 0..5 { out[x] = v.get_remove(rng).unwrap(); }
    out
}