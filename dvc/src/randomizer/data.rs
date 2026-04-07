use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use engage::{
    unit::Unit,
    mess::Mess,
    gamevariable::GameVariableManager,
    random::Random,
    gamedata::{
        GamedataArray, Gamedata,
        item::ItemData, 
        person::*, GodData, JobData, PersonData, job::BattleStyles, ring::RingData, 
        skill::SkillData,
    }
};
use engage::unit::{UnitPool};
use unity::system::Il2CppString;
use crate::config::{DVCFlags, DVCVariables};
use crate::enums::{EMBLEM_ASSET, PIDS};
use crate::randomizer::data::{
    aptitude::EmblemAptitudeRandomizer, engage_attacks::EngageAttackRandomizer, items::ItemPool,
    sync::{get_lowest_priority, get_skill_level, EmblemSkillRandomizer, DARK_EMBLEM_SKILLS},
};
use crate::randomizer::{Randomizer, item::*, names::AppearanceRandomizer, *};
use crate::utils::{create_rng, get_rng};
mod skill;
mod emblem;
mod person;
mod items;
mod bondring;

pub use skill::*;
pub use person::*;
pub use emblem::*;
pub use bondring::*;
pub use items::*;
use crate::config::menu::{DVCMenuItemKind, CUSTOM_RECRUITMENT_ORDER};
use crate::randomizer::data::emblem::item::EngageItemRandomizer;

pub struct GrowthData {
    person_stats: [Vec<u8>; 10],
    job_stat: [Vec<i8>; 10],
    person_grow: Vec<(i32, [u8; 10])>,
    non_base_grow_jobs: HashSet<i32>,
    job_grow: Vec<[i8; 10]>,
    job_cap: Vec<[u8; 10]>,
    person_cap: Vec<(i32, [i8; 10])>,
}
impl GrowthData {
    pub fn new() -> GrowthData{
        let mut person_stats = [const { Vec::new() }; 10];
        let mut job_stat = [const { Vec::new() }; 10];
        let mut non_base_grow_jobs = HashSet::new();
        let person_grow =
        PersonData::get_list().unwrap().iter().filter(|x| !x.grow.is_zero() )
            .map(|x|{
                let grow = x.get_grow();
                let mut g = [0; 10];
                for x in 0..10 {
                    let v = grow[x];
                    g[x] = v;
                    let s = if v > 100 { 20 } else { v/5 };
                    person_stats[x].push(s);
                }
                (x.parent.index, g)
            }).collect();
        let person_cap =
        PersonData::get_list().unwrap().iter().filter(|x| !x.limit.is_zero() )
            .map(|x|{
                let mut g = [0; 10];
                let limit = x.get_limit();
                for x in 0..10 { g[x] = limit[x]; }
                (x.parent.index, g)
            }).collect();

        let job_grow =
        JobData::get_list().unwrap().iter().map(|x|{
            let mut g = [0; 10];
            if x.base_grow.is_zero() { non_base_grow_jobs.insert(x.parent.index); }
            let diff = x.get_diff_grow();
            for x in 0..10 {
                let v = diff[x];
                g[x] = v;
                let s = if v > 25 { 5 } else if v < -25 { -5 } else { v/5 };
                job_stat[x].push(s);
            }
            g
        }).collect();
        let job_cap = JobData::get_list().unwrap().iter().map(|x| {
            let mut g = [0; 10];
            let limit = x.get_limit();
            for x in 0..10 { g[x] = limit[x]; }
            g
        }).collect();
        Self{ person_grow, job_stat, job_grow, person_stats, non_base_grow_jobs, job_cap, person_cap }
    }
    pub fn get_personal(&self, rng: &Random, cap: &mut engage::unit::Capability) -> i32 {
        let mut total = 0;
        for i in 0..10 {
            let v =  5 * self.person_stats[i].get_random_element(rng).map(|u| *u).unwrap_or(0);
            cap[i] = v;
            total += v as i32;
        }
        total
    }
    pub fn set_job_diff(&self, rng: &Random, cap: &mut engage::unit::CapabilitySbyte){
        for i in 0..10 {
            let v =  5 * self.job_stat[i].get_random_element(rng).map(|u| *u).unwrap_or(0);
            cap[i] = v;
        }
    }
    pub fn personal_caps(&self) {
        if DVCFlags::PersonalCaps.get_value() {
            self.person_cap.iter().for_each(|x|{
                if let Some(person) = PersonData::try_index_get(x.0) {
                    let rng = create_rng(person.parent.hash, 2);
                    let limit = person.get_limit();
                    for x in 0..8 {
                        let v = rng.get_value(11) - 5;
                        limit[x] = v as i8;
                    }
                }
            });
        }
        else {
            self.person_cap.iter().for_each(|xx|{
                if let Some(person) = PersonData::try_index_get(xx.0) {
                    let limit = person.get_limit();
                    for x in 0..10 { limit[x] = xx.1[x]; }
                }
            });
        }
    }
    pub fn reset(&self, flag: i32) {
        if flag & 1 != 0 {
            let person_list = PersonData::get_list_mut().unwrap();
            self.person_grow.iter().for_each(|x|{
                let grow = person_list[x.0 as usize].get_grow();
                for i in 0..10 { grow[i] = x.1[i]; }
            })
        }
        if flag & 2 != 0 {
            JobData::get_list_mut().unwrap().iter().zip(self.job_grow.iter()).for_each(|(j,x)| {
                let diff = j.get_diff_grow();
                for i in 0..10{ diff[i] = x[i]; }
            })
        }
    }
    pub fn set_job_caps(&self, enable: bool) {
        if enable {
            JobData::get_list_mut().unwrap().iter().for_each(|job|{
                let base = job.get_base();
                let cap = job.get_limit();
                for x in 0..10 { cap[x] = base[x] + 120; }
                cap[10] = 99;
            });
        }
        else {
            JobData::get_list_mut().unwrap().iter_mut().zip(self.job_cap.iter()).for_each(|(j,x)| {
                j.get_limit().iter_mut().zip(x.iter()).for_each(|(j,v)| { *j = *v; });
            });
        }
    }
}
pub struct GameData {
    pub bond_ring: Vec<BondRingData>,
    pub playables: Vec<PlayableCharacter>,
    pub enemy: Vec<EnemyCharacter>,
    pub growth_data: GrowthData,
    pub job_style_attr: Vec<(i32, i32)>,
    pub interactions: Vec<i32>,
    pub units: HashMap<i32, i32>,
    pub unit_name: HashMap<String, i32>,
    pub skill_pool: SkillPool,
    pub item_pool: ItemPool,
    pub emblem_pool: EmblemPool,
}
impl GameData {
    pub fn init() -> Self {
        let bond_ring = RingData::get_list().unwrap().iter().map(|r|BondRingData::from_data(r)).collect();
        let mut playables: Vec<_> = PIDS.iter().enumerate().map(|(i, &pid)| PlayableCharacter::new(PersonData::get(pid).unwrap(), i as i32)).collect();
        let mut playable_count = playables.len() as i32;
        let person_list =  PersonData::get_list().unwrap();
        let count = person_list.iter().filter(|p| p.get_sp() > 0 ).count();
        person_list.iter().filter(|p| !p.pid.str_contains("_竜化") && !p.get_grow().is_zero() ).for_each(|p| {
            if p.get_asset_force() == 0 && p.get_sp() > 0 && !playables.iter().any(|h| h.hash == p.parent.hash ) && count < 150 {
                println!("Playable Character: {} #{}", Mess::get_name(p.pid), p.parent.hash);
                playables.push(PlayableCharacter::new(p, playable_count) );
                playable_count += 1;
            }
        });
        for x in [0, 4, 14, 17, 23, 27] { unsafe { CUSTOM_RECRUITMENT_ORDER[x as usize] = x; } }
        unsafe { CUSTOM_RECRUITMENT_ORDER[41] = playable_count as u8; }
        let job_style_attr = JobData::get_list().unwrap().iter().map(|j|{ (BattleStyles::get_style(j.style_name) as i32 , j.attrs) }).collect();
        let mut unit_name: HashMap<String, i32> = HashMap::new();
        let mut units: HashMap<i32, i32> = HashMap::new();
            include_str!("person/persons.txt").lines().enumerate().for_each(|(i, l)|{
                l.split_ascii_whitespace().for_each(|s|{
                   if !s.contains("_") {
                       person_list.iter()
                           .filter(|p| p.pid.str_contains(s) && !p.pid.str_contains("M024") && !p.pid.str_contains("_竜化") && p.bmap_size == 1)
                           .for_each(|p|{
                               if let Some(name) = p.name.as_ref() { unit_name.insert(name.to_string(), i as i32); }
                               units.insert(p.parent.hash, i as i32);
                           });
                   }
                   else if let Some(person) = PersonData::get(s).or_else(|| PersonData::get(format!("PID_{}", s))) {
                       if let Some(name) = person.name.as_ref() { unit_name.insert(name.to_string(), i as i32); }
                       units.insert(person.parent.hash, i as i32);
                   }
                });
            });

        let enemy =
            units.iter().filter(|(h, i)| **i < 41 && !playables.iter().any(|x| x.hash == **h))
                .flat_map(|x|
                    PersonData::try_get_hash(*x.0).filter(|v| v.asset_force != 0 || *x.1 > 35).zip(Some(x.1)))
                .map(|p| { EnemyCharacter::new(p.0, *p.1) })
                .collect();
        for x in 36..41 {
            let hash = PersonData::get(PIDS[x]).unwrap().parent.hash;
            units.insert(hash, x as i32);
        }
        Self {
            job_style_attr,
            growth_data: GrowthData::new(),
            item_pool: ItemPool::init(),
            skill_pool: SkillPool::init(),
            emblem_pool: EmblemPool::init(),
            interactions: InteractData::get_list().unwrap().iter().map(|data| data.flag.value).collect(),
            playables, enemy, bond_ring, units, unit_name,
        }
    }
    pub fn get() -> &'static Self { RANDOMIZER_DATA.get_or_init(||Self::init()) }
    pub fn get_item_pool() -> &'static ItemPool { &Self::get().item_pool }
    pub fn get_random_skill(diff: i32, rng: &Random) -> &'static SkillData { Self::get().skill_pool.get_random_skill(diff, rng) }
    pub fn get_random_skill_job(diff: i32, rng: &Random, unit: &Unit) -> Option<&'static SkillData> {
        Self::get().skill_pool.get_random_skill_job(diff, rng, unit)
    }
    pub fn get_random_skill_dispos(diff: i32, rng: &Random) -> &'static SkillData { RANDOMIZER_DATA.get().unwrap().skill_pool.get_random_skill_dispos(diff, rng) }
    pub fn get_random_item(ty: i32, with_rare: bool) -> &'static Il2CppString { RANDOMIZER_DATA.get().unwrap().item_pool.random_item(ty, with_rare) }
    pub fn get_weapon_db() -> &'static crate::randomizer::item::data::WeaponDatabase { &Self::get().item_pool.weapon_db }
    pub fn update_personals(&self){
        let personal = DVCFlags::PersonalSkills.get_value();
        self.playables.iter().for_each(|x| { x.update_personal_skill(personal); });
        self.enemy.iter().filter(|e| e.playable_slot.is_some_and(|x| x < 41) )
            .for_each(|e| {
                e.update_person();
                let enemy_person = e.get_person_mut();
                if let Some(person) = e.playable_slot.and_then(|s|Self::get_randomized_person(s)){
                    if let Some(personal) = person.common_skills.iter().find(|s| !s.is_hidden()) {
                        if let Some(skill) = enemy_person.get_common_skills().list.iter_mut().find(|x| !x.is_hidden() ) {
                            skill.set_index(personal.get_index());
                            // println!("{} -> enemy {}: Personal {}", Mess::get(MPIDS[e.playable_slot.unwrap()]), Mess::get_name(e.get_person_mut().pid), personal.get_skill().unwrap().name.unwrap());
                        }
                    }
                }
            });
    }
    pub fn get_playable_emblem_hashes() -> Vec<i32> {
        Self::get().emblem_pool.emblem_list.iter().enumerate().filter(|x| x.0 < 20 || x.0 >= 24).map(|x| *x.1).collect()
    }
    pub fn get_playable_god_list() -> Vec<&'static GodData> {
        GameData::get_playable_emblem_hashes().iter().flat_map(|x| GodData::try_get_hash(*x)).collect()
    }
    pub fn reset_interaction(&self) {
        InteractData::get_list_mut().unwrap().iter_mut().zip(self.interactions.iter()).for_each(|(interaction, data)|{ interaction.flag.value = *data; });
    }
    pub fn reset_battle_styles(&self) {
        JobData::get_list_mut().unwrap().iter_mut().zip(self.job_style_attr.iter()).for_each(|(job, data)|{ job.style = data.0; });
    }
    pub fn reset_job_diff(&self) { self.growth_data.reset(2); }

    pub fn update_bond_ring(&self) { randomize_bond_ring_skills(); }
    pub fn get_randomized_person(index: usize) -> Option<&'static PersonData> {
        Self::get().playables.get(index).and_then(|p|{
            let key = format!("G_R_{}", p.get_person_data().pid);
            if GameVariableManager::exist(&key) { PersonData::get(GameVariableManager::get_string(&key)) }
            else { Some(p.get_person_data()) }
        })
    }
}

pub struct RandomizedGameData {
    pub engage_atks: EngageAttackRandomizer,
    pub engage_weapons: EngageItemRandomizer,
    pub engage_skills: EmblemSkillRandomizer,
    pub emblem_aptitude_randomizer: EmblemAptitudeRandomizer,
    pub person_appearance: AppearanceRandomizer,
    pub evolve: Vec<i32>,
    pub refine: Vec<i32>,
}

impl RandomizedGameData {
    pub fn get_read() -> RwLockReadGuard<'static, Self> { RANDOMIZED_DATA.get().unwrap().read().unwrap() }
    pub fn get_write() -> RwLockWriteGuard<'static, Self> { RANDOMIZED_DATA.get().unwrap().write().unwrap() }
    pub fn new(emblem: usize, playables: usize) -> Self {
        Self {
            refine: vec![-1; ItemData::get_count() as usize],
            person_appearance: AppearanceRandomizer::init(),
            engage_atks: EngageAttackRandomizer::new(emblem),
            evolve: vec![],
            engage_weapons: EngageItemRandomizer::init(),
            engage_skills: EmblemSkillRandomizer::init( playables),
            emblem_aptitude_randomizer: EmblemAptitudeRandomizer::new(emblem),
        }
    }
    pub fn randomize(&mut self, data: &GameData) {
        let mut refine_set = data.item_pool.refine_iid.clone();
        refine_set.extend(data.item_pool.refine_iid.iter());
        self.refine.iter_mut().for_each(|v|{ *v = 0; });

        let rng = get_rng();
        ItemData::get_list().unwrap()
            .iter()
            .enumerate()
            .filter(|(i, x)| x.is_weapon() && x.flag.value & 128 == 0 )
            .for_each(|(i,x)|{ if let Some(v) = refine_set.get_remove(rng) { self.refine[i] = v; } });

        self.person_appearance.randomize(false);
        data.skill_pool.randomize(data);
        self.evolve.clear();
        let rng = get_rng();
        ItemEvolveData::get_list().unwrap().iter()
            .for_each(|list| {
                list.iter().for_each(|item|{
                    let price = 3*item.price as i32 + 250;
                    loop {
                        let item = data.item_pool.random_item_data(rng);
                        if item.price > price || item.flag.value & 2 != 0 { continue; }
                        match item.use_type {
                            1|5|6|8|9|11|7|23|24 => {
                                self.evolve.push(item.parent.hash);
                                break;
                            },
                            _ => { continue; },
                        }
                    }
                });
            });
        self.engage_atks.randomize(data);
        self.engage_skills.randomize(data);
        self.engage_weapons.randomize(data, &self.engage_atks);
        self.emblem_aptitude_randomizer.randomize();
    }
    pub fn update_evolve_items(&self, data: &GameData) {
        if DVCFlags::EvolveItems.get_value() {
            ItemEvolveData::get_list_mut().unwrap().iter_mut()
                .flat_map(|v| v.iter_mut())
                .zip(self.evolve.iter().flat_map(|&h| ItemData::try_get_hash(h)))
                .for_each(|(data, item)|{ data.iid = item.iid; });
        }
        else {
            ItemEvolveData::get_list_mut().unwrap().iter_mut()
                .flat_map(|v| v.iter_mut())
                .zip(data.item_pool.evolve_data.iter().flat_map(|&h| ItemData::try_get_hash(h)))
                .for_each(|(data, item)|{ data.iid = item.iid; });
        }
    }
    pub fn update_engage_atk_items(&self, data: &GameData) {
        self.engage_atks.commit(data);
        self.engage_weapons.commit(data);
    }
    pub fn commit(&self, data: &GameData) {
        let engrave_setting = DVCVariables::EngraveLevel.get_value();
        data.growth_data.set_job_caps(DVCFlags::MaxStatCaps.get_value());
        data.growth_data.personal_caps();
        crate::randomizer::emblem::engrave::random_engrave_by_setting(engrave_setting, true);
        styles::randomize_job_styles();
        interact::change_interaction_data(DVCVariables::InteractSetting.get_value(), true);
        self.update_evolve_items(data);
        self.engage_skills.commit_stats(data);
        self.engage_skills.commit(data);
        self.update_engage_atk_items(data);
        self.update_enemy_emblem(data);
        self.emblem_aptitude_randomizer.commit(data);
        data.update_personals();
        data.update_bond_ring();
        DVCFlags::Initialized.set_value(true);
    }
    pub fn update_enemy_emblem(&self, data: &GameData) {
        if !DVCFlags::Initialized.get_value() && DVCVariables::is_changed_recruitment_order(true) {
            data.emblem_pool.enemy_emblem.iter().for_each(|enemy|{
                if enemy.emblem_index != 12 && enemy.emblem_index != 13 {
                    let enemy_god = enemy.emblem_data.get_god_mut();
                    if let Some(source_god) = enemy.get_replacement_source() {
                        let randomized_index = DVCVariables::get_dvc_emblem_index(enemy.emblem_index as i32, false);
                        if randomized_index < 20 {
                            enemy_god.link_gid = source_god.link_gid;
                            enemy_god.engage_attack_link = source_god.engage_attack_link;
                            enemy_god.ascii_name = source_god.ascii_name;
                            enemy_god.mid = source_god.mid;
                            enemy_god.nickname = source_god.nickname;
                            enemy_god.sound_id = source_god.sound_id;
                            let m002 = enemy_god.gid.to_string().contains("M002");
                            if m002 || (randomized_index == 8 || randomized_index == 10 || randomized_index == 11) || randomized_index > 18 { enemy_god.asset_id = source_god.asset_id; }
                            else if enemy.emblem_index < 19 { enemy_god.asset_id = format!("敵{}", EMBLEM_ASSET[randomized_index]).into(); }
                            enemy_god.face_icon_name = source_god.face_icon_name;
                            enemy_god.face_icon_name_darkness = source_god.face_icon_name_darkness;
                            enemy_god.ascii_name = source_god.ascii_name;
                            enemy_god.unit_icon_id = source_god.unit_icon_id;
                        }
                    }
                }
            });
        }
        else { data.emblem_pool.enemy_emblem.iter().for_each(|e| { e.reset(); }); }

        data.emblem_pool.enemy_emblem.iter()
            .filter(|x| x.emblem_index < 19 )
            .for_each(|enemy| {
                enemy.emblem_data.reset_weapons();
                enemy.update_engage_atk();
                enemy.emblem_data.reset_all_skills();
                enemy.emblem_data.reset_weapons();
                let enemy_god = enemy.emblem_data.get_god_mut();
                let randomized_index = if enemy.emblem_index >= 12 { enemy.emblem_index } else { DVCVariables::get_dvc_emblem_index(enemy.emblem_index as i32, false) };
                let syncs_old = &data.emblem_pool.emblem_data[enemy.emblem_index].syncs;
                let syncs_new = &data.emblem_pool.emblem_data[if enemy.emblem_index == 12 { 12 } else { randomized_index} ].syncs;
                let weapons = &data.emblem_pool.emblem_data[randomized_index].level_data[0].style_items;
                let new_engage = data.emblem_pool.emblem_data[randomized_index].get_engage_skill().map(|s| s.parent.index).unwrap_or(0);
                if let Some(level_data) = enemy_god.get_level_data() {
                    level_data.iter_mut().for_each(|level|{
                        level.synchro_skills.list.iter_mut().for_each(|sk|{
                            let skill = sk.get_skill().unwrap();
                            let level = get_skill_level(skill);
                            let hash = DARK_EMBLEM_SKILLS.iter().find(|s| s.0 == skill.parent.hash).map(|s| s.1).or_else(|| Some(skill.parent.hash)).unwrap();
                            let lowest = get_lowest_priority(SkillData::try_get_hash_mut(hash).unwrap());
                            if let Some(s) = syncs_old.iter().zip(syncs_new.iter()).find(|(old, new)| **old == lowest.parent.hash) {
                                let new_skill = SkillData::try_get_hash_mut(*s.1).unwrap();
                                let mut new_index = new_skill.parent.index;
                                for _ in 0..level {
                                    if let Some(higher) = SkillData::try_index_get(new_index).and_then(|s| s.high_skill) {
                                        new_index = higher.parent.index;
                                    }
                                }
                                sk.set_index(self.engage_skills.get_sync_replacement_index(new_index));
                            }
                        });
                        level.engaged_skills.list
                            .iter_mut()
                            .for_each(|sk| {
                                let skill = sk.get_skill().unwrap();
                                let level = get_skill_level(skill);
                                let hash = DARK_EMBLEM_SKILLS.iter().find(|s| s.0 == skill.parent.hash).map(|s| s.1).or_else(|| Some(skill.parent.hash)).unwrap();
                                let lowest = get_lowest_priority(SkillData::try_get_hash_mut(hash).unwrap());
                                if let Some(s) = syncs_old.iter().zip(syncs_new.iter()).find(|(old, new)| **old == lowest.parent.hash) {
                                    let new_skill = SkillData::try_get_hash_mut(*s.1).unwrap();
                                    let mut new_index = new_skill.parent.index;
                                    for _ in 0..level {
                                        if let Some(higher) = SkillData::try_index_get(new_index).and_then(|s| s.high_skill) {
                                            new_index = higher.parent.index;
                                        }
                                    }
                                    sk.set_index(self.engage_skills.get_sync_replacement_index(new_index));
                                } 
                            });
                        if let Some(s) = level.engage_skills.list.iter_mut().find(|skill| !skill.is_hidden()){
                            if DVCVariables::EmblemEngageSkill.get_value() != 0 { s.set_index(self.engage_skills.get_engage_skill_index(new_engage)); }
                            else { s.set_index(new_engage); }
                        }
                        if DVCFlags::EngageWeapons.get_value() {
                            level.style_items.iter_mut().flat_map(|x| x.iter_mut())
                                .for_each(|x1| {
                                    let hash = x1.parent.hash;
                                    *x1 = self.engage_weapons.get_replacement(hash);
                                });
                        }
                        else if DVCVariables::is_changed_recruitment_order(true) && randomized_index != enemy.emblem_index {
                            let weap = ["IID_ベレト_ルーン", "IID_ベレト_ヴァジュラ", "IID_ベレト_天帝の覇剣", "IID_チキ_つめ", "IID_チキ_しっぽ", "IID_チキ_ブレス"];
                            level.style_items.iter_mut().enumerate().for_each(|(style_index, style_items)|{
                                style_items.iter_mut().enumerate().for_each(|(item_pos, item)| {
                                    if randomized_index == 9 { if let Some(v) = ItemData::get_mut(weap[item_pos]) { *item = v; } }
                                    else if randomized_index == 13 { if let Some(v) = ItemData::get_mut(weap[3+item_pos]) { *item = v; } }
                                    else if let Some(v) = ItemData::try_get_hash_mut(weapons[style_index * 3 + item_pos]) { *item = v; }
                               });
                            })
                        }
                    });
                }
            });
    }
    pub fn a_call_menu_action(setting: DVCMenuItemKind) {
        let data = GameData::get();
        match setting {
            DVCMenuItemKind::Variable(variables) => {
                let value = variables.get_value();
                match variables {
                    DVCVariables::EmblemWepProf => { Self::get_read().emblem_aptitude_randomizer.commit(data); }
                    DVCVariables::EmblemSyncSkill|DVCVariables::EmblemEngageSkill => { RandomizedGameData::get_read().engage_skills.commit(data); }
                    DVCVariables::JobLearnMode => { crate::randomizer::skill::learn::update_learn_skills(true); }
                    DVCVariables::BattleStyles => { styles::randomize_job_styles(); }
                    DVCVariables::InteractSetting => { interact::change_interaction_data(value, false);  }
                    DVCVariables::PersonalGrowthMode => { grow::random_grow(); }
                    DVCVariables::EngraveLevel => { crate::randomizer::emblem::engrave::random_engrave_by_setting(value, false); }
                    _ => { println!("No Action for Variable #{}: {}", variables as i32, variables.get_key()); }
                }
            }
            DVCMenuItemKind::Flag(flag) => {
                let v = flag.get_value();
                match flag {
                    DVCFlags::EngageWeapons|DVCFlags::EngageAttacks => {
                        let random_data = Self::get_read();
                        random_data.update_engage_atk_items(data);
                        random_data.update_enemy_emblem(data);
                    }
                    DVCFlags::RandomClassGrowth => { grow::random_grow(); }
                    // DVCFlags::CustomSkillEnemy => {}
                    DVCFlags::RandomClassAttrs => { styles::randomize_job_attrs(); }
                    DVCFlags::RingStats|DVCFlags::BondRing => { data.update_bond_ring(); }
                    DVCFlags::EquipLearnSkills => {
                        if v {
                            for x in 1..250 {
                                if let Some(unit) = UnitPool::get(x).filter(|x| x.force.is_some_and(|x| x.force_type < 5) && x.person.asset_force == 0 && x.learned_job_skill.is_some()) {
                                    if let Some(skill) = unit.learned_job_skill.as_ref() { unit.add_to_equip_skill_pool(skill); }
                                }
                            }
                        }
                    }
                    DVCFlags::BGM => { if GameUserData::get_sequence() == 3 { bgm::change_bgm(); } }
                    DVCFlags::EvolveItems => { Self::get_read().update_evolve_items(data); }
                    DVCFlags::EmblemStats => { Self::get_read().engage_skills.commit_stats(data); }
                    DVCFlags::AdaptiveGrowths => { grow::randomize_person_grow(); }
                    DVCFlags::PersonalSkills => { data.update_personals(); }
                    // DVCFlags::RefineItem => {}
                    DVCFlags::MaxStatCaps => {
                        GameData::get().growth_data.set_job_caps(DVCFlags::MaxStatCaps.get_value());
                    }
                    DVCFlags::PersonalCaps => { data.growth_data.personal_caps(); }
                    _ => { println!("Flag #{} was not affected", flag as i32); }
                }
            }
            DVCMenuItemKind::Order(_) => {

            }
            DVCMenuItemKind::SingleJob => {
                if DVCVariables::get_single_class(false, false).is_some() && DVCFlags::SingleJobEnabled.get_value() {
                    for x in 1..250 {
                        if let Some(unit) = UnitPool::get(x).filter(|x| x.force.is_some_and(|f| f.force_type < 5) && x.person.asset_force == 0) {
                            crate::randomizer::job::unit_change_to_random_class(unit, false);
                            // println!("{} Changed into {}", unit.get_name(), Mess::get_name(unit.job.jid));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}