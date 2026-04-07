use engage::gamedata::{god::GodGrowthData, item::ItemData, skill::SkillData, Gamedata, GodData, PersonData};
use engage::mess::Mess;
use crate::enums::EMBLEM_ASSET;
use crate::randomizer::data::{GameData, SkillsList};
use crate::randomizer::data::enemy::EnemyEmblemData;
use crate::randomizer::data::sync::get_lowest_priority;
use crate::randomizer::EMBLEM_GIDS;

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
                        // println!("'{}' added as custom emblem #{}", Mess::get(god.mid), custom_count);
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
        god_data.get_level_data().and_then(|l| { l[0].engage_skills.iter().find(|s| !s.is_hidden() ).map(|s| s.get_index() ) });
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
    pub fn reset_skill(&self) {
        if let Some(person) = PersonData::try_get_hash(self.hash) {
            self.common_skill.set_skill_array(person.get_common_skills());
            self.normal_skill.set_skill_array(person.get_normal_skills());
            self.hard_skill.set_skill_array(person.get_hard_skills());
            self.maddening_skill.set_skill_array(person.get_lunatic_skills());
            person.set_engage_skill(SkillData::try_get_hash(self.engage_atk));
        }
    }
    pub fn reset_engage_skill(&self) { self.get_person().set_engage_skill(SkillData::try_get_hash(self.engage_atk)); }
    pub fn get_person(&self) -> &'static mut PersonData { PersonData::try_get_hash_mut(self.hash).unwrap() }
}
