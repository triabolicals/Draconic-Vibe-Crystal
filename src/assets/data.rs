use std::collections::HashSet;
use std::sync::OnceLock;
use accessory::BustData;
use concat_string::concat_string;
use god::{EngageAtkAsset, GodAssets};
use item::WeaponAssets;
use engage::{gamedata::{*, skill::*, unit::*, assettable::*}, random::Random, gamedata::item::ItemData, gamevariable::GameVariableManager};
use crate::{DVCVariables, randomizer::emblem::EMBLEM_LIST};
use job::{Mount, *};
use search::*;
// use animation::*;
use super::{result_commit_scaling, ConditionFlags, EMBLEM_ASSET, accessory::{change_accessory, clear_accessory_at_locator}};

pub static SEARCH_LIST: OnceLock<AssetData> = OnceLock::new();

pub mod item;
pub mod job;
pub mod accessory;
pub mod search;
pub mod god;

pub fn initialize_anim_data() {}

pub struct AssetData {
    pub bid: HashSet<i32>,
    pub m0: HashSet<i32>,
    pub f0: HashSet<i32>,
    pub m_body: HashSet<i32>,
    pub f_body: HashSet<i32>,
    pub aoc_m: Vec<String>,
    pub aoc_f: Vec<String>,
    pub job: Vec<JobAssetSets>,
    pub god: Vec<GodAssets>,
    pub engage: Vec<SearchData>,
    pub engage_atks: Vec<EngageAtkAsset>,
    pub male_index: i32,
    pub female_index: i32,
    pub weapon_conditions: [i32; 12],
    pub items: WeaponAssets,
    pub bust: BustData,
}

impl AssetData {
    pub fn new() -> Self { 
        Self{ 
            aoc_m: Vec::new(),
            aoc_f: Vec::new(),
            bid: HashSet::new(),
            bust: BustData::new(),
            items: WeaponAssets::new(),
            m0: HashSet::new(),
            m_body: HashSet::new(),
            f0: HashSet::new(),
            f_body: HashSet::new(),
            job: Vec::new(), engage: Vec::new(), god: Vec::new(), engage_atks: Vec::new(),
            male_index: AssetTableStaticFields::get_condition_index("男装"),
            female_index: AssetTableStaticFields::get_condition_index("女装"),
            weapon_conditions: [
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(0)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(1)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(2)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(3)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(4)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(5)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(6)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(7)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(8)),
                AssetTableStaticFields::get_condition_index(ItemData::get_kind_name(9)),
                AssetTableStaticFields::get_condition_index("弾丸"),
                AssetTableStaticFields::get_condition_index("竜石"),
            ]
        }
    }
    pub fn add_person(&mut self, mpid: &str, gender:Gender) {
        if gender != Gender::None {
            let gender_con = if gender == Gender::Female { "女装" } else { "男装" };
            let _ = search_by_2_keys(0, mpid, gender_con).map(|entry| if gender == Gender::Male { self.m0.insert(entry.parent.index) } else {  self.f0.insert(entry.parent.index) } );
            let _ = search_by_2_keys(2, mpid, gender_con).map(|entry| if gender == Gender::Male { self.m_body.insert(entry.parent.index) } else {  self.f_body.insert(entry.parent.index) });
        }
    }
    pub fn add_person_data(&mut self, person: &PersonData) {
        let gender = person.get_gender();
        if gender == 0 || gender > 2 || person.get_bmap_size() > 1 { return;}
        if let Some(name) = person.get_name() {
            let cross_dressing = person.get_flag().value & 32 != 0;
            let is_male = (gender == 1 && !cross_dressing) || (gender == 2 && cross_dressing);

            if let Some(aid) = person.aid {
                let _ = search_by_key(0, aid, None).map(|entry| if is_male { self.m0.insert(entry.parent.index) } else { self.f0.insert(entry.parent.index) });
                let _ = search_by_key_with_dress(2, aid).map(|entry| if is_male { self.m_body.insert(entry.parent.index) } else { self.f_body.insert(entry.parent.index) });
            }
            if let Some(bid) = person.belong {
                let _ = search_by_key(0, bid, None).map(|entry| self.bid.insert(entry.parent.index) );
            }
            if let Some(entry) = search_by_key_with_dress(0, name) { 
                if is_male { self.m0.insert(entry.parent.index); } else { self.f0.insert(entry.parent.index); }    }
            if let Some(entry) = search_by_key_with_dress(0, person.pid) { if is_male { self.m0.insert(entry.parent.index); } else { self.f0.insert(entry.parent.index); } }


            if let Some(entry) = search_by_key_with_dress(2, name) { if is_male { self.m_body.insert(entry.parent.index); } else { self.f_body.insert(entry.parent.index); } }
            if let Some(entry) = search_by_key_with_dress(2, person.pid) { if is_male { self.m_body.insert(entry.parent.index); } else { self.f_body.insert(entry.parent.index); } }
        }
    }
    pub fn add_god(&mut self, god: &GodData, index: i32) {
        let hash = god.parent.hash;
        let eid = god.gid.to_string().replace("GID", "EID");
        for mode in 1..3 {
            if let Some(entry) = asset_table_search(mode, &vec![eid.as_str(), "女装"]) { self.engage.push( SearchData::new(SearchType::Engage, hash, Gender::Female, index, entry) );  }
            if let Some(entry) = asset_table_search(mode, &vec![eid.as_str(), "男装"]) { self.engage.push( SearchData::new(SearchType::Engage, hash, Gender::Male, index , entry) );  }
        }
        if let Some(engage_atk) = god.engage_attack {
            let engage_atk_str = engage_atk.to_string();
            let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| engage_atk_str.contains(god)) { pos }
                else if engage_atk_str.contains("三級長エンゲージ技＋") { 20 }
                else if engage_atk_str.contains("三級長エンゲージ") { 12 }
                else { 50 } as i32;
            let emblem_asset_index = AssetTableStaticFields::get_condition_index(god.asset_id);
            if emblem_asset_index < 1 { return; }
            let engage_atk_index = AssetTableStaticFields::get_condition_index(if emblem_index == 20 { "協力エンゲージ技" } else { "エンゲージ技" });
            let mut engage_atk_data = EngageAtkAsset::new(god, emblem_index, emblem_asset_index, engage_atk_index);
            if let Some(skill) = god.engage_attack.and_then(|sid| SkillData::get(sid)) {
                skill.style_skills.iter().for_each(|s|{
                    if engage_atk_data.engage_atk_hashes.iter().find(|x| **x == s.parent.hash).is_none() { engage_atk_data.engage_atk_hashes.push(s.parent.hash);}
                });
            }
            if let Some(skill) = god.engage_attack_link.and_then(|sid| SkillData::get(sid)) {
                skill.style_skills.iter().for_each(|s|{
                    if engage_atk_data.engage_atk_hashes.iter().find(|x| **x == s.parent.hash).is_none() { engage_atk_data.engage_atk_hashes.push(s.parent.hash);}
                });
            }
            self.engage_atks.push(engage_atk_data);
        }
        let _ = search_by_key_with_dress(2, god.gid).map(|entry| if god.female == 1 { self.f_body.insert(entry.parent.index) } else { self.m_body.insert(entry.parent.index) });
        for mode in 1..3 {
            self.god.push(GodAssets::new(god, mode, index));
            if god.change_data.len() > 1 {
                for x in 1..god.change_data.len() {
                    let sub_index = index * 10 + (x as i32);
                    self.god.push(GodAssets::new(god.change_data[x], mode, sub_index));
                }
            }
        }

    }
    pub fn add_job(&mut self, job: &JobData) {
        if job.parent.index == 0 { return; }
        let hash = job.parent.hash;
        let canon = job.weapon_levels[9] > 1 && job.mask_skills.find_sid("SID_弾丸装備").is_some();
        let dragonstone = job.weapon_levels[9] > 1 && job.mask_skills.find_sid("SID_弾丸装備").is_none();
        for mode in 1..3 { 
            let mut job_data = JobAssetSets{ 
                job_hash: hash, 
                mode: mode, 
                mount: Mount::None, 
                entries: Vec::new(), 
                unique: false, 
                cannon: canon,
                dragon_stone: dragonstone,
                gender_flag: 0,
            };
            // let job_name = engage::mess::Mess::get_name(job.jid);
            // if job_data.dragon_stone { println!("Class: {} can use DragonStones", engage::mess::Mess::get_name(job.jid)); }
            if get_job_entries(&mut job_data, mode, job.jid) { 
                job_data.unique |= job.parent.index < 20;
                self.job.push(job_data); 
            }
            else {
                PersonData::get_list().unwrap().iter()
                    .filter(|person| person.parent.index > 0 && person.get_job().is_some_and(|sjob| sjob.parent.hash == hash))
                    .for_each(|person|{
                        let mut start_index = 0;
                        while let Some(entry) = search_by_key(mode, person.pid, Some(start_index)) {
                            if entry.dress_model.is_some_and(|b| b.to_string().contains("M_c")) || entry.dress_model.is_some_and(|b| b.to_string().contains("M_c")) {
                                job_data.unique = true;
                                job_data.gender_flag |= 1;
                                job_data.entries.push(entry.parent.index);

                            }
                            else if entry.dress_model.is_some_and(|b| b.to_string().contains("F_c")) || entry.dress_model.is_some_and(|b| b.to_string().contains("F_c")) {
                                job_data.unique = true;
                                job_data.gender_flag |= 2;
                                job_data.entries.push(entry.parent.index);
                            }
                            if entry.body_anim.is_some() {
                                job_data.unique = true;
                                job_data.entries.push(entry.parent.index);
                            }
                            start_index = entry.parent.index;
                        }
                    }
                );
                if job_data.entries.len() > 0 { 
                    job_data.unique |= job.parent.index < 20;
                    self.job.push(job_data);
                }
            }
        }
    }
    pub fn replace_with_god(&self, result: &mut AssetTableResult, mode: i32, god_index: i32, is_darkness: bool) {
        let _ = self.god.iter().find(|data| mode == data.mode && god_index == data.index )
            .map(|data|{
                if let Some(entry) = data.get_entry(is_darkness) {
                    let _ = entry.body_model.map(|body| result.body_model = body);
                    let _ = entry.dress_model.map(|dress| result.dress_model = dress);
                    let _ = entry.head_model.map(|head| result.head_model = head);
                    let _ = entry.hair_model.map(|hair| result.hair_model = hair);
                    result.sound.voice = entry.voice;
                    if entry.info_anim.is_some() { result.info_anims = entry.info_anim; }
        
                    result_commit_scaling(result, entry);
                    if mode == 1 && result.scale_stuff[16] < 2.6 {  result.scale_stuff[16] = 2.6; }
                    entry.accessories.iter()
                        .filter(|access| access.locator.is_some() && access.model.is_some() )
                        .for_each(|access| result.accessory_list.try_add(access) );
                }
            }
        );
    }
    pub fn get_engage_outfit(&self, result: &mut AssetTableResult, mode: i32, god_index: i32, gender: Gender, override_outfit: bool, _conds: ConditionFlags) {
        result.ride_dress_model = "null".into();
        result.ride_model = "null".into();
        change_accessory(result.accessory_list, "null", "c_hip_loc");
        change_accessory(result.accessory_list, "null", "l_shld_loc");
        result.scale_stuff[16] = 2.6;
        let new_gender = if god_index == 13 { Gender::None } else { gender };
        if god_index == 13 && gender == Gender::Female {
            if let Some(search) = self.god.iter().find(|data| data.index == god_index && data.mode == mode ){
                let asset_table = AssetTable::try_index_get(search.entry_index).unwrap();
                let _ = asset_table.body_model.map(|body| result.body_model = body);
                let _ = asset_table.dress_model.map(|dress| result.dress_model = dress);
            }
        }
        if let Some(search) = self.engage.iter().find(|data| data.index == god_index && data.mode == mode && data.gender == new_gender ){
            let asset_table = AssetTable::try_index_get(search.entry_index).unwrap();
            if override_outfit || god_index == 13 {
                let _ = asset_table.body_model.map(|body| result.body_model = body);
                let _ = asset_table.dress_model.map(|dress| result.dress_model = dress);
            }
            if god_index != 13 {    // Hair Colors
                if asset_table.hair_r > 0 { result.unity_colors[0].r = asset_table.unity_colors[0].r; }
                if asset_table.hair_g > 0 { result.unity_colors[0].g = asset_table.unity_colors[0].g; }
                if asset_table.hair_b > 0 { result.unity_colors[0].b = asset_table.unity_colors[0].b; }

                if asset_table.grad_r > 0 { result.unity_colors[1].r = asset_table.unity_colors[1].r; }
                if asset_table.grad_g > 0 { result.unity_colors[1].g = asset_table.unity_colors[1].g; }
                if asset_table.grad_b > 0 { result.unity_colors[1].b = asset_table.unity_colors[1].b; }
            }
        }
    }
    pub fn set_job_dress(&self, result: &mut AssetTableResult, job: &JobData, gender: Gender, mode: i32, conditions: ConditionFlags) {
        if let Some(job_set) = self.job.iter().find(|j| j.job_hash == job.parent.hash && j.mode == mode) { 
            let is_morph = conditions.contains(ConditionFlags::Corrupted);
            if mode == 2 {
                if !result.dress_model.is_null() {
                    if result.dress_model.to_string().contains("Swd0A") {   
                        let _ = job_set.get_dress(gender, is_morph).map(|dress| result.dress_model = dress); 
                    }
                }
                let _ = job_set.get_body_rig(gender).map(|rig| result.body_model = rig);
                if !conditions.contains(ConditionFlags::DismountMask) {
                    let _ = job_set.get_ride_dress(is_morph).map(|dress| result.ride_dress_model = dress);
                    let _ = job_set.get_ride_rig().map(|ride_rig| result.ride_model = ride_rig);
                }
            }
            else {
                if let Some(job_set) = self.job.iter().find(|j| j.job_hash == job.parent.hash) {
                    if !result.body_model.is_null() {
                        if result.body_model.to_string().contains("Swd0A") { 
                            let _ = job_set.get_obody(gender, is_morph).map(|dress| result.body_model = dress);  
                        }
                    }
                    if !conditions.contains(ConditionFlags::DismountMask) { 
                        let _ = job_set.get_ride_obody(is_morph).map(|dress| result.ride_model = dress);
                        let _ = job_set.get_map_wing_scaling().map(|scale| result.scale_stuff[18] = scale);
                    }
                    let _ = job_set.get_map_all_scaling().map(|scale| result.scale_stuff[16] = scale);
                }
            }
        }
    }
    pub fn random_head(&self, result: &mut AssetTableResult, unit: &Unit, conditions: ConditionFlags, with_dress: bool) {
        let rng = Random::instantiate().unwrap();
        rng.initialize(unit.grow_seed as u32);
        if conditions.contains(ConditionFlags::Male) || conditions.contains(ConditionFlags::Female) {
            self.bid.iter().nth( rng.get_value( self.bid.len() as i32) as usize)
                .map(|&i| AssetTable::try_index_get(i).map(|entry| result.commit_asset_table(entry))
            );
            let set = if conditions.contains(ConditionFlags::Male) {&self.m_body } else { &self.f_body };
            let size = set.len();
            if size < 5 { return; }

            if let Some(entry) = set.iter().nth(rng.get_value(size as i32) as usize).and_then(|&index|AssetTable::try_index_get(index)) {
                result_commit_scaling(result, entry);
                if with_dress {
                    if !GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) { entry.dress_model.map(|dress| result.dress_model = dress);   }
                }
                if entry.head_model.is_some_and(|head| !head.to_string().contains("null")) {  result.head_model = entry.head_model.unwrap(); }
                if entry.accessory_list.list.iter()
                    .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("_Hair"))) { result.hair_model = "uHair_null".into(); }

                else if entry.hair_model.is_some_and(|hair| !hair.to_string().contains("null")) {  result.hair_model = entry.hair_model.unwrap(); }

                crate::assets::accessory::accessory_clear_all(result.accessory_list);

                entry.accessory_list.list.iter()
                    .filter(|acc| 
                        acc.model.is_some_and(|model| !model.to_string().contains("shld")) && acc.locator.is_some())
                    .for_each(|acc| result.accessory_list.try_add(acc) );

                clear_accessory_at_locator(result.accessory_list, "c_trans");
                clear_accessory_at_locator(result.accessory_list, "c_hip_jnt");
            }
        }
    }
    pub fn adjust_head(&self, result: &mut AssetTableResult, unit: &Unit) {
        if !result.dress_model.is_null() {  // Somboro Head if using Somboro Body
            if result.dress_model.to_string().contains("Sdk0AM") {
                result.head_model = "uHead_c504".into();
                result.magic = "MG_Obscurite".into();
            }
        }
        if !result.head_model.is_null() {
            let head = result.head_model.to_string();
            if self.job_is_unique(unit.job) {
                if head.contains("801") { self.random_head(result, unit, ConditionFlags::Male, false);  }
                else if head.contains("851") { self.random_head(result, unit, ConditionFlags::Female, false);  }
            } 
        }
    }

    pub fn get_gender_condition(&self, gender: i32) -> i32 {
        if gender == 1 { self.male_index } else if gender == 2 { self.female_index } else { 0 }
    }
    pub fn random_aoc(&self, unit: &Unit, result: &mut AssetTableResult, conditions: ConditionFlags) {
        if conditions.contains(ConditionFlags::TikiEngage) { return; }
        if conditions.contains(ConditionFlags::Male) || conditions.contains(ConditionFlags::Female) {
            let hash = if unit.person.get_asset_force() == 0 { unit.person.parent.hash } else { unit.grow_seed };
            let rng = crate::utils::create_rng(hash, 1);
            if unit.status.value & 8388608 != 0 { rng.get_value(100); }
            let aoc = if conditions.contains(ConditionFlags::Male) { &self.aoc_m[ rng.get_value( self.aoc_m.len() as i32 ) as usize] }
                else { &self.aoc_f[ rng.get_value( self.aoc_f.len() as i32 ) as usize] };
            result.info_anims = Some(concat_string!("AOC_Info_c", aoc).into());
        }
    }
    pub fn job_can_use_canon(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.cannon) }
    pub fn job_can_use_dragonstone(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.dragon_stone) }
    pub fn job_is_unique(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.unique) }
}

pub struct SearchData {
    pub search_type: SearchType,   
    pub gender: Gender,
    pub hash: i32,
    pub mode: i32,
    pub index: i32, 
    pub entry_index: i32,
    pub mount: Mount,
}
impl SearchData {
    pub fn new(ty: SearchType, hash: i32, gender: Gender, index: i32, entry: &AssetTable) -> Self {
        Self {
            search_type: ty,   // person
            hash: hash,
            gender: gender,
            index: index,
            mode: entry.mode,
            entry_index: entry.parent.index,
            mount: determine_mount(entry),
        }
    }
}

#[derive(PartialEq)]
pub enum SearchType {
    Person,
    Body,
    Job,
    God, 
    Engage,
    Animation,
    Item,
}


pub fn initialize_search_list() {
    initialize_anim_data();
    SEARCH_LIST.get_or_init(||{
        let mut slist = AssetData::new();
        // Playable Persons
        slist.add_person("MPID_Lueur", Gender::Male);
        slist.add_person("MPID_Lueur", Gender::Female);
        
        PersonData::get_list().unwrap().iter()
            .filter(|person| person.gender & 3 != 0 && person.parent.index > 1)
            .for_each(|person| slist.add_person_data(person) );


        let mut aoc = include_str!("data/data/heads.txt").lines().enumerate();
        aoc.next().map(|aoc|slist.aoc_m =  aoc.1.split_whitespace().map(|str| str.to_string()).collect());
        aoc.next().map(|aoc| slist.aoc_f = aoc.1.split_whitespace().map(|str| str.to_string()).collect());

        // Tiki Engage
        if let Some(entry) =  asset_table_search(1, &vec!["EID_チキ"]) { slist.engage.push( SearchData::new(SearchType::Person,0, Gender::None, 13, entry) ) }
        if let Some(entry) =  asset_table_search(2, &vec!["EID_チキ"]) { slist.engage.push( SearchData::new(SearchType::Person,0, Gender::None, 13, entry) ) }

        // Emblems
        let emblem_list =  EMBLEM_LIST.get().unwrap();
        for x in 0..emblem_list.len() { slist.add_god(GodData::try_get_hash(emblem_list[x as usize ]).unwrap(), x as i32);  }
        JobData::get_list().unwrap().iter().for_each(|job| slist.add_job(job) );

        ItemData::get_list().unwrap().iter().for_each(|item|
            { search_by_iid(item.iid).map(|entry| slist.items.add(item, entry.parent.index)); }
        );

        println!("Job Assets: {}", slist.job.len());
        println!("Person Assets Mode 0 : {} / {}", slist.m0.len(), slist.f0.len());
        println!("AOC Count: {} / {}", slist.aoc_m.len(), slist.aoc_f.len());
        println!("Person Assets Mode 2 : {} / {}", slist.m_body.len(), slist.f_body.len());
        slist
    });
}



// AID_異形兵