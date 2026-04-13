use std::collections::HashMap;
use engage::gamedata::{god::GodGrowthData, item::ItemData, skill::SkillData, Gamedata, GodData, PersonData};
use engage::gamedata::skill::SkillDataCategorys;
use engage::gamevariable::GameVariableManager;
use engage::mess::Mess;
use unity::prelude::Il2CppString;
use crate::DVCVariables;
use crate::enums::EMBLEM_ASSET;
use crate::randomizer::data::{GameData, SkillsList};
use crate::randomizer::data::enemy::EnemyEmblemData;
use crate::randomizer::data::sync::get_lowest_priority;
use crate::randomizer::EMBLEM_GIDS;

const EMBLEM_HASHES: [i32; 24] = [
    998659272, 1995155639, 1709540132, 462041932,
    1981217683, 1289051445, -2116658132, -1964439889,
    364445343, 1050116739, 1339715833, -1107657133,
    52812801, 1978213856, -445657450, 1429549728,
    -339120642, 1893352633, 1120993642, 374589614,
    59738509, -1526367221, 2044088482, -682425036
];
pub mod item;
pub mod engage_attacks;
pub mod sync;
pub mod aptitude;
pub mod enemy;

pub struct EmblemPool {
    pub emblem_list: Vec<i32>,
    pub emblem_data: Vec<EmblemData>,
    pub emblem_persons: Vec<EmblemPerson>,
    pub enemy_emblem: Vec<EnemyEmblemData>,
}

impl EmblemPool {
    pub fn get_dvc_emblem_data<'a>(gid: impl Into<&'a Il2CppString>) -> Option<&'static GodData> {
        GameVariableManager::try_get_string(format!("G_R_{}", gid.into()))
            .and_then(|gid| GodData::get(gid))
    }
    pub fn is_custom(god_data: &GodData) -> bool {
        let hash = god_data.parent.hash;
        !EMBLEM_HASHES.contains(&hash)
    }
    pub fn reset_all(&self, data: &GameData) {
        self.emblem_persons.iter().for_each(|p| {
            p.reset_skill(data);
            p.reset_engage_skill(data);
        });
        self.enemy_emblem.iter().for_each(|e| { e.reset(); });
        self.emblem_data.iter().for_each(|e|{
            e.reset_weapons();
            e.reset_all_skills();
            e.reset_weapon_prof();
        });
    }
    pub fn init() -> Self {
        let god_list = GodData::get_list().unwrap();
        let mut emblem_data = vec![];
        let mut emblem_list: Vec<i32> = Vec::new();
        let mut ggids: Vec<String> = Vec::new();
        let gids = EMBLEM_GIDS.iter().map(|s| s.trim_start_matches("GID_")).collect::<Vec<&str>>();
        EMBLEM_GIDS.iter().enumerate().for_each(|(x, gid)|{
            let god = GodData::get(gid).unwrap();
            if let Some(ggid) = god.grow_table.map(|s| s.to_string()) {
                emblem_list.push(god.parent.hash);
                if !ggids.contains(&ggid) {
                    ggids.push(ggid);
                    emblem_data.push(EmblemData::new(god));
                }
            }
        });
        let mut custom_count = 0;

        god_list.iter()
            .filter(|god| god.force_type == 0 && god.grow_table.is_some() && !EMBLEM_ASSET.iter().any(|asset| god.gid.str_contains(asset)))
            .for_each(|god|{
                if let Some(grow) = god.get_level_data() {
                    let ggid = god.grow_table.unwrap().to_string();
                    if grow.len() >= 20 && ggids.iter().find(|&c_ggid| *c_ggid == ggid).is_none() {
                        ggids.push(ggid);
                        custom_count += 1;
                        emblem_list.push(god.parent.hash);
                        emblem_data.push(EmblemData::new(god));
                    }
                }
            });

        let emblem_persons =
        PersonData::get_list().unwrap().iter()
            .filter(|s| s.jid.is_some_and(|jid| jid.str_contains("JID_紋章士") & gids.iter().any(|&gid| s.pid.str_contains(gid))))
            .flat_map(|person|  gids.iter().position(|&gid| person.pid.str_contains(gid)).map(|pos| EmblemPerson::new(person, pos)))
            .collect();
        Self {
            emblem_list, emblem_data,
            emblem_persons,
            enemy_emblem: god_list.iter()
                .filter(|x| !x.gid.str_contains("_M026") && ( x.gid.contains("_M0") || x.gid.contains("_E0")))
                .map(|x| EnemyEmblemData::new(x)).collect(),
        }
    }

}

pub struct EmblemData {
    pub hash: i32,
    pub engage_atk: i32,
    pub link_engage_atk: i32,
    pub engage_skill_index: Option<i32>,
    pub engrave_stats: [i8; 6],
    pub link_gid: Option<String>,
    pub level_data: Vec<EmblemLevelData>,
    pub growth_apt: Vec<i32>,
    pub is_enemy: bool,
    pub syncs: Vec<i32>,
    pub flag: i32,
}

#[derive(Default, Clone)]
pub struct EngageAtk {
    pub engage_atk: i32,
    pub linked_engage_atk: i32,
    pub linked_emblem: i32,
}

pub struct EmblemLevelData {
    pub engage_skills: SkillsList,
    pub engaged_skills: SkillsList,
    pub sync_skills: SkillsList,
    pub apt: i32,
    pub style_items: Vec<i32>,
}

pub struct EmblemPerson {
    pub emblem_index: usize,
    pub hash: i32,
    pub engage_atk: i32,
    pub common_skill: SkillsList,
    pub normal_skill: SkillsList,
    pub hard_skill: SkillsList,
    pub maddening_skill: SkillsList,
}

impl EmblemData {
    pub fn new(god_data: &GodData) -> Self {
        let engage_atk = god_data.engage_attack.and_then(|sid| SkillData::get(sid)).map(|skill| skill.parent.hash).unwrap_or(0);
        let link_engage_atk = god_data.engage_attack_link.and_then(|sid| SkillData::get(sid)).map(|skill| skill.parent.hash).unwrap_or(0);
        let mut engrave_stats = [0; 6];
        for x in 0..6 { engrave_stats[x] = god_data.get_engrave_value(x as i32); }
        let level_data = god_data.get_level_data().map(|level|{
            level.iter().map(|level|{
                EmblemLevelData {
                    engaged_skills: SkillsList::from_skill_array(level.engaged_skills),
                    sync_skills:  SkillsList::from_skill_array(level.synchro_skills),
                    engage_skills: SkillsList::from_skill_array(level.engage_skills),
                    apt: level.aptitude.value,
                    style_items: level.style_items.iter().flat_map(|i| i.iter()).map(|item| item.parent.hash).collect(),
                }
            }).collect()
        }).unwrap_or_default();

        let growth_apt = GodGrowthData::try_get_from_god_data(god_data)
            .map(|l| l.iter().map(|g| g.aptitude.value ).collect()).unwrap_or_default();

        let syncs =
        god_data.get_level_data().map(|l| {
            l[0].synchro_skills.iter()
                .filter_map(|s| s.get_skill().filter(|s| s.flag & 1 == 0))
                .map(|s| get_lowest_priority(s))
                .map(|lowest| lowest.parent.hash)
                .collect()
        }).unwrap_or_default();
        let engage_skill_index =
        god_data.get_level_data()
            .and_then(|l| { l[0].engage_skills.iter()
                .find(|s| !s.is_hidden() ).map(|s| s.get_index() ) });
        Self {
            is_enemy: god_data.force_type == 1, growth_apt, syncs,
            link_gid: god_data.link_gid.map(|gid| gid.to_string()),
            hash: god_data.main_data.parent.hash, engage_skill_index,
            engage_atk, link_engage_atk, engrave_stats, level_data,
            flag: god_data.flag.value,
        }
    }
    pub fn get_god(&self) -> &'static GodData { GodData::try_get_hash(self.hash).unwrap() }
    pub fn get_god_mut(&self) -> &'static mut GodData { GodData::try_get_hash_mut(self.hash).unwrap() }
    pub fn get_engage_skill(&self) -> Option<&'static SkillData> { self.engage_skill_index.and_then(|i|SkillData::try_index_get(i)) }
    pub fn reset_weapons(&self) {
        self.get_god().get_level_data().unwrap().iter_mut().zip(self.level_data.iter()).for_each(|(level_data, growth_data)| {
            level_data.style_items.iter_mut().flat_map(|s| s.iter_mut())
                .zip(growth_data.style_items.iter())
                .for_each(|(item, hash)|{ *item = ItemData::try_get_hash_mut(*hash).unwrap(); });
            });
        self.get_god_mut().flag.value = self.flag;
    }
    pub fn reset_stats(&self, gdata: &GameData) {
        self.get_god().get_level_data().unwrap().iter_mut().zip(self.level_data.iter()).for_each(|(level_data, data)| {
            level_data.synchro_skills.list.iter_mut().zip(data.sync_skills.list.iter()).filter(|(s, d)| gdata.skill_pool.is_emblem_stat_boost(s))
                .for_each(|(s, d)| { s.set_index(d.index); });
            level_data.engaged_skills.list.iter_mut().zip(data.engage_skills.list.iter()).filter(|(s, d)| gdata.skill_pool.is_emblem_stat_boost(s))
                .for_each(|(s, d)| { s.set_index(d.index); });
        });
    }
    pub fn reset_all_skills(&self) {
        self.get_god().get_level_data().unwrap().iter_mut().zip(self.level_data.iter()).for_each(|(level_data, data)| {
            data.sync_skills.set_skill_array(level_data.synchro_skills);
            data.engaged_skills.set_skill_array(level_data.engaged_skills);
            data.engage_skills.set_skill_array(level_data.engage_skills);
        });
    }
    pub fn reset_non_stats(&self, gdata: &GameData) {
        self.get_god().get_level_data().unwrap().iter_mut().zip(self.level_data.iter()).for_each(|(level_data, data)| {
            level_data.synchro_skills.list.iter_mut().zip(data.sync_skills.list.iter()).filter(|(s, d)| !gdata.skill_pool.is_emblem_stat_boost(s))
                .for_each(|(s, d)| { s.set_index(d.index); });
            level_data.engaged_skills.list.iter_mut().zip(data.engage_skills.list.iter()).filter(|(s, d)| !gdata.skill_pool.is_emblem_stat_boost(s))
                .for_each(|(s, d)| { s.set_index(d.index); });
        });
    }
    pub fn reset_weapon_prof(&self) {
        let god = self.get_god();
        god.get_level_data().unwrap().iter_mut().zip(self.level_data.iter())
            .for_each(|(level_data, data)| { level_data.aptitude.value = data.apt; });
        if let Some(growth) = GodGrowthData::try_get_from_god_data(god){
            growth.iter_mut().zip(self.growth_apt.iter())
                .for_each(|(grow, apt)|{ grow.aptitude.value = *apt; });
        }
    }
}

impl EmblemPerson {
    pub fn new(person: &PersonData, index: usize) -> Self {
        Self {
            emblem_index: index,
            hash: person.parent.hash,
            normal_skill: SkillsList::from_skill_array(person.get_normal_skills()),
            common_skill: SkillsList::from_skill_array(person.get_common_skills()),
            hard_skill: SkillsList::from_skill_array(person.get_hard_skills()),
            maddening_skill: SkillsList::from_skill_array(person.get_lunatic_skills()),
            engage_atk: person.get_engage_skill().map(|x| x.parent.hash).unwrap_or(0),
        }
    }
    pub fn reset_skill(&self, game_data: &GameData){
        if let Some(person) = PersonData::try_get_hash_mut(self.hash) {
            if self.is_custom() && self.is_paralogue(){
                if let Some(god) = DVCVariables::get_god_from_index(self.emblem_index as i32, true) {
                    if let Some(data) = game_data.emblem_pool.emblem_data.iter().find(|d| d.hash == god.parent.hash){
                        if let Some(level_data) = data.level_data.last() {
                            let commons = person.get_common_skills();
                            let normals = person.get_normal_skills();
                            let lunatic = person.get_lunatic_skills();
                            let hard = person.get_hard_skills();
                            commons.clear();
                            normals.clear();
                            hard.clear();
                            lunatic.clear();
                            level_data.engage_skills.list.iter()
                                .chain(level_data.engage_skills.list.iter())
                                .chain(level_data.sync_skills.list.iter())
                                .flat_map(|v| SkillData::try_index_get(v.index))
                                .for_each(|skill|{
                                    commons.add_skill(skill, SkillDataCategorys::Private, 0);
                                    normals.add_skill(skill, SkillDataCategorys::Private, 0);
                                    lunatic.add_skill(skill, SkillDataCategorys::Private, 0);
                                    hard.add_skill(skill, SkillDataCategorys::Private, 0);
                                });
                            normals.add_sid("SID_命中回避－２０", SkillDataCategorys::Private, 0);
                            lunatic.add_sid("SID_ブレイク無効", SkillDataCategorys::Private, 0);
                        }
                        person.set_engage_skill(SkillData::try_get_hash(data.engage_atk));
                    }
                    person.aid = Some(god.gid);
                    let gid = god.gid.to_string();
                    if let Some(summon) = PersonData::get_list().unwrap().iter().find(|p| p.summon_god.is_some_and(|god| god.str_contains(gid.as_str()))){
                        set_emblem_person(person, summon);
                    }
                    else {
                        let arena_pid = god.gid.to_string().replace("GID_", "PID_闘技場_");
                        if let Some(arena_person) = PersonData::get(arena_pid.as_str()) { set_emblem_person(person, arena_person); }
                    }
                }
            }
            else {
                self.common_skill.set_skill_array(person.get_common_skills());
                self.normal_skill.set_skill_array(person.get_normal_skills());
                self.hard_skill.set_skill_array(person.get_hard_skills());
                self.maddening_skill.set_skill_array(person.get_lunatic_skills());
                person.set_engage_skill(SkillData::try_get_hash(self.engage_atk));
            }
        }
    }
    pub fn reset_engage_skill(&self, data: &GameData) {
        if self.is_custom() && self.is_paralogue(){
            if let Some(god) = DVCVariables::get_god_from_index(self.emblem_index as i32, true) {
                if let Some(data) = data.emblem_pool.emblem_data.iter().find(|d| d.hash == god.parent.hash) {
                    self.get_person().set_engage_skill(SkillData::try_get_hash(data.engage_atk));
                }
            }
        }
        else { self.get_person().set_engage_skill(SkillData::try_get_hash(self.engage_atk)); }
    }
    pub fn get_person(&self) -> &'static mut PersonData { PersonData::try_get_hash_mut(self.hash).unwrap() }
    pub fn is_custom(&self) -> bool {
        DVCVariables::get_god_from_index(self.emblem_index as i32, true).filter(|god| !EMBLEM_HASHES.contains(&god.parent.hash)).is_some()
    }
    pub fn is_paralogue(&self) -> bool {
        PersonData::try_get_hash(self.hash).is_some_and(|p| p.pid.str_contains("PID_S0"))
    }
}
fn set_emblem_person(emblem_person: &mut PersonData, other: &'static PersonData) {
    emblem_person.unit_icon_id = other.unit_icon_id;
    if other.help.is_some() { emblem_person.help = other.help; }
    emblem_person.gender = other.gender;
    emblem_person.name = other.name;
    if other.jid.is_some() { emblem_person.jid = other.jid; }
    if let Some(ascii_name) = other.get_ascii_name() { emblem_person.set_ascii_name(ascii_name); }
    if other.items.is_some() { emblem_person.items = other.items; }
}
