use std::collections::HashMap;
use std::sync::OnceLock;
use accessory::BustData;
use concat_string::concat_string;
use god::{EngageAtkAsset, GodAssets};
use item::WeaponAssets;
use engage::{gamedata::{*, skill::*, unit::*, assettable::*}, random::Random, gamedata::item::ItemData, gamevariable::GameVariableManager};
use engage::mess::Mess;
use engage::resourcemanager::{ResourceManager, ResourceManagerStaticFields};
use crate::{randomizer::emblem::EMBLEM_LIST, utils::str_contains, DVCVariables};
use job::{Mount, *};
use search::*;
use crate::enums::ENGAGE_PREFIX;
use super::{result_commit_scaling, ConditionFlags, EMBLEM_ASSET, accessory::{change_accessory, clear_accessory_at_locator}, get_unit_outfit_mode, AnimSetDB};

pub static SEARCH_LIST: OnceLock<AssetData> = OnceLock::new();

pub mod item;
pub mod job;
pub mod accessory;
pub mod search;
pub mod god;

pub fn initialize_anim_data() {}

pub struct AssetData {
    pub bid: Vec<i32>,
    pub m0: Vec<i32>,
    pub f0: Vec<i32>,
    pub m_body: Vec<i32>,
    pub f_body: Vec<i32>,
    pub personal_outfits: HashMap<&'static str, &'static str>,
    pub fx_classes: HashMap<i32, i32>,
    pub aoc_m: Vec<String>,
    pub aoc_f: Vec<String>,
    pub job: Vec<JobAssetSets>,
    pub god: Vec<GodAssets>,
    pub engage: Vec<SearchData>,
    pub unique_0: HashMap<i32, i32>,
    pub engage_atks: Vec<EngageAtkAsset>,
    pub male_index: i32,
    pub female_index: i32,
    pub weapon_conditions: [i32; 10],
    pub other_conditions: [i32; 6],
    pub items: WeaponAssets,
    pub bust: BustData,
    pub personal_data: Vec<PersonAsset>,
    pub u_male: Vec<(String, i32)>,
    pub o_male: Vec<(String, i32)>,
    pub u_female: Vec<(String, i32)>,
    pub o_female: Vec<(String, i32)>,
    pub bond_face: Vec<String>,
    pub engaging: Vec<i32>,
}

impl AssetData {
    pub fn new() -> Self { 
        Self{
            personal_data: Vec::new(),
            aoc_m: Vec::new(),
            aoc_f: Vec::new(),
            bid: vec![],
            bust: BustData::new(),
            items: WeaponAssets::new(),
            m0: vec![],
            m_body: vec![],
            f0: vec![],
            f_body: vec![],
            u_male: vec![],
            o_male: vec![],
            u_female: vec![],
            o_female: vec![],
            bond_face: vec![],
            engaging: vec![],
            unique_0: HashMap::new(),
            fx_classes: HashMap::new(),
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
            ],
            other_conditions: [
                AssetTableStaticFields::get_condition_index("エンゲージ技"),
                AssetTableStaticFields::get_condition_index("エンゲージ中"),
                AssetTableStaticFields::get_condition_index("踊り"),  //  Dance
                AssetTableStaticFields::get_condition_index("弾丸"),  // DragonStone
                AssetTableStaticFields::get_condition_index("竜石"),  // Bullet
                AssetTableStaticFields::get_condition_index("素手"),
            ],
            personal_outfits: HashMap::new(),
        }
    }
    pub fn add_gendered_assets(&mut self, entry: &AssetTable, gender: Gender) {
        let entry_index = entry.parent.index;
        if gender == Gender::Male {
            if entry.mode == 0 && !self.m0.contains(&entry_index) {
                self.m0.push(entry_index);
            }
            else if entry.mode == 2 && !self.m_body.contains(&entry_index) {
                self.m_body.push(entry_index);
            }
        }
        else if gender == Gender::Female {
            if entry.mode == 0 && !self.m0.contains(&entry_index) {
                self.f0.push(entry_index);
            }
            else if entry.mode == 2 && !self.m_body.contains(&entry_index) {
                self.f_body.push(entry_index);
            }
        }

    }
    pub fn add_person(&mut self, mpid: &str, gender:Gender) {
        if gender != Gender::None {
            let gender_con = if gender == Gender::Female { "女装" } else { "男装" };
            for x in [0, 2] {
                if let Some(entry) = search_by_2_keys(x, mpid, gender_con) {
                    self.add_gendered_assets(&entry, gender);
                }
            }
        }
    }
    pub fn add_person_data(&mut self, person: &PersonData) {
        let gender = person.get_gender();
        if gender == 0 || gender > 2 || person.get_bmap_size() > 1 { return;}
        let cross_dressing = person.get_flag().value & 32 != 0;
        let dress_gender = if (gender == 1 && !cross_dressing) || (gender == 2 && cross_dressing) { Gender::Male } else { Gender::Female };
        if let Some(name) = person.get_name() {
            if let Some(entry) = person.aid.and_then(|aid| search_by_key(0, aid, None)){
                self.add_gendered_assets(entry, dress_gender);
            }
            if let Some(entry) = person.belong.and_then(|b| search_by_key(0, b, None))
            {
                self.bid.push(entry.parent.index)
            }
            for x in [0, 2] {
                if let Some(entry) = search_by_key(x, name, None) {
                    self.add_gendered_assets(entry, dress_gender);
                }
                if let Some(entry) = search_by_key_with_dress(x, person.pid) {
                    self.add_gendered_assets(entry, dress_gender);
                }
            }
        }
        else {
            for x in [0, 2] {
                if let Some(entry) = search_by_key_with_dress(x, person.pid) {
                    self.add_gendered_assets(entry, dress_gender);
                }
            }
        }
    }
    pub fn add_god(&mut self, god: &GodData, index: i32) {
        let hash = god.parent.hash;
        let eid = god.gid.to_string().replace("GID", "EID");
        for mode in 1..3 {
            if let Some(entry) = asset_table_search(mode, &vec![eid.as_str(), "女装"]) {
                self.engage.push( SearchData::new(SearchType::Engage, hash, Gender::Female, index, entry) );
            }
            if let Some(entry) = asset_table_search(mode, &vec![eid.as_str(), "男装"]) {
                self.engage.push( SearchData::new(SearchType::Engage, hash, Gender::Male, index , entry) );
            }
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
        if let Some(entry) = search_by_key_with_dress(2, god.gid){
            let god_gender = if god.female == 1 { Gender::Female } else { Gender::Male };
            self.add_gendered_assets(entry, god_gender);
        }
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
        let canon = job.weapon_levels[9] > 1 && job.mask_skills
            .find_sid("SID_弾丸装備").is_some();
        let dragonstone = job.weapon_levels[9] > 1 && job.mask_skills
            .find_sid("SID_弾丸装備").is_none();
        for mode in 1..3 { 
            let mut job_data = JobAssetSets{ 
                job_hash: hash, 
                mode,
                mount: Mount::None, 
                entries: Vec::new(), 
                transform: Vec::new(),
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
                        let mut count = 0;
                        let s = [person.pid.clone(), person.name.unwrap().clone()];
                        for x in s {
                            let mut start_index = 0;
                            while let Some(entry) = search_by_key(mode, x, Some(start_index)) {
                                if entry.dress_model.is_some_and(|b| b.str_contains("M_c")) || entry.dress_model.is_some_and(|b| b.str_contains("M_c")) {
                                    job_data.unique = true;
                                    job_data.gender_flag |= 1;
                                    job_data.entries.push(entry.parent.index);

                                }
                                else if entry.dress_model.is_some_and(|b| b.str_contains("F_c")) || entry.dress_model.is_some_and(|b| b.str_contains("F_c")) {
                                    job_data.unique = true;
                                    job_data.gender_flag |= 2;
                                    job_data.entries.push(entry.parent.index);
                                }
                                if entry.body_anim.is_some() {
                                    job_data.unique = true;
                                    job_data.entries.push(entry.parent.index);
                                }
                                start_index = entry.parent.index;
                                count += 1;
                            }
                            if count > 0 { break; }
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
    pub fn get_engage_outfit(&self, result: &mut AssetTableResult, mode: i32, god_index: i32, gender: Gender, override_outfit: bool, conds: &ConditionFlags) {
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
                return;
            }
        }
        if conds.contains(ConditionFlags::Engaged) && god_index < 20 {
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
            if god_index != 13 && god_index < 20 && override_outfit {
                let gen_str = if gender == Gender::Male { "M" } else { "F" };
                if mode == 1 { result.body_model = format!("oBody_{}1A{}_c000", ENGAGE_PREFIX[god_index as usize], gen_str ).into(); }
                else { result.dress_model = format!("uBody_{}1A{}_c000", ENGAGE_PREFIX[god_index as usize], gen_str ).into(); }
            }
        }
    }
    pub fn set_job_dress(&self, result: &mut AssetTableResult, job: &JobData, gender: Gender, mode: i32, conditions: &ConditionFlags) {
        if let Some(job_set) = self.job.iter().find(|j| j.job_hash == job.parent.hash && j.mode == mode) { 
            let is_morph = conditions.contains(ConditionFlags::Corrupted);
            if mode == 2 {
                if !result.dress_model.is_null() {
                    if result.dress_model.str_contains("Swd0A") {
                        if let Some(dress) = job_set.get_dress(gender, is_morph ){
                            println!("JOB {} Dress: {}", Mess::get_name(job.jid), dress);
                            result.dress_model = dress;
                        }
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
                        let body_model = result.body_model.to_string();
                        if body_model.contains("Swd0A") && !body_model.contains("c251")  {
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
            if job.parent.hash == 185671037 { job_set.apply_hair_color(result, mode, gender); }
        }
        else { println!("Job {} does not exists", Mess::get_name(job.jid)); }
    }
    pub fn set_job_mount_dress(&self, result: &mut AssetTableResult, job: &JobData, gender: Gender, mode: i32, conditions:&ConditionFlags) {
        if conditions.contains(ConditionFlags::DismountMask) { return; }
        if let Some(job_set) = self.job.iter().find(|j| j.job_hash == job.parent.hash && j.mode == mode) {
            let is_morph = conditions.contains(ConditionFlags::Corrupted);
            if mode == 2 {
                let _ = job_set.get_body_rig(gender).map(|rig| result.body_model = rig);
                if let Some(dress) = job_set.get_ride_dress(is_morph) {
                    let dress_str = dress.to_string();
                    if dress_str == "null" || dress_str.len() < 14 {
                        result.ride_dress_model = dress;
                    }
                    else {
                        let dress_str = &dress.to_string()[..13];
                        if result.ride_dress_model.is_null() {
                            result.ride_dress_model = dress;
                        }
                        else if !result.ride_dress_model.str_contains(&dress_str) {
                            result.ride_dress_model = dress;
                        }
                    }
                }
                let _ = job_set.get_ride_rig().map(|ride_rig| result.ride_model = ride_rig);
            }
            else {
                let _ = job_set.get_ride_obody(is_morph).map(|dress| result.ride_model = dress);
                let _ = job_set.get_map_wing_scaling().map(|scale| result.scale_stuff[18] = scale);
                let _ = job_set.get_map_all_scaling().map(|scale| result.scale_stuff[16] = scale);
            }
            if job.parent.hash == 185671037 { job_set.apply_hair_color(result, mode, gender); }
        }
    }
    pub fn random_head(&self, result: &mut AssetTableResult, unit: &Unit, conditions: ConditionFlags, with_dress: bool) {
        let bid_ran = ((unit.drop_seed as u32) % (self.bid.len() as u32)) as usize;
        if conditions.contains(ConditionFlags::Male) || conditions.contains(ConditionFlags::Female) {
            if let Some(bid_asset) = self.bid.get(bid_ran).and_then(|&i| AssetTable::try_index_get(i)) {
                result.commit_asset_table(bid_asset);
            }
            let set = if conditions.contains(ConditionFlags::Male) {&self.m_body } else { &self.f_body };
            let size = set.len();
            if size < 5 { return; }
            let body_sel = (((unit.drop_seed as u32) >> 4) % size as u32) as usize;
            if let Some(entry) = set.get(body_sel).and_then(|&index|AssetTable::try_index_get(index)) {
                result_commit_scaling(result, entry);
                if with_dress && !GameVariableManager::get_bool(DVCVariables::ENEMY_OUTFIT_KEY) {
                    entry.dress_model.map(|dress| result.dress_model = dress);
                }
                if let Some(head) = entry.head_model.filter(|head| !head.to_string().contains("null") ) {
                    result.head_model = head;
                }
                let hair_accessory = entry.accessory_list.list.iter()
                    .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("_Hair")));

                let shld = result.accessory_list.list.iter()
                    .find(|x|x.locator.is_some_and(|loc| str_contains(loc, "shld"))

                ).map_or_else(|| String::new(), |acc| acc.model.map_or_else(|| String::new(), |model| model.to_string()));

                if hair_accessory { result.hair_model = "uHair_null".into(); }
                else if entry.hair_model.is_some_and(|hair| !hair.str_contains("null")) {
                    result.hair_model = entry.hair_model.unwrap();
                }

                if with_dress || hair_accessory {
                    result.accessory_list.clear();
                    entry.accessory_list.list.iter().for_each(|acc|{ result.accessory_list.try_add(acc); });
                    clear_accessory_at_locator(result.accessory_list, "c_trans");
                    clear_accessory_at_locator(result.accessory_list, "c_hip_jnt");
                    if !shld.is_empty() { change_accessory(result.accessory_list, shld.as_str(), "l_shld_loc"); }
                    else { change_accessory(result.accessory_list, "null", "l_shld_loc"); }
                }
                // Fixing Skin for Heads
                if !result.head_model.is_null() {
                    let head = result.head_model.to_string();
                    if head.len() >= 10 {
                        if let Ok(id) = head.split_at(7).1[..3].parse::<i32>() {
                            if let Some(entry0) = self.unique_0.get(&id)
                                .and_then(|&index| AssetTable::try_index_get(index))
                            {
                                result.commit_asset_table(entry0);
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn adjust_head(&self, result: &mut AssetTableResult, unit: &Unit) {
        if !result.dress_model.is_null() {  // Somboro Head if using Somboro Body
            if result.dress_model.str_contains("Sdk0AM") {
                result.head_model = "uHead_c504".into();
                result.magic = "MG_Obscurite".into();
            }
        }
        if !result.head_model.is_null() && unit.force.is_some_and(|f| f.force_type == 1 || f.force_type == 2) {
            let head = result.head_model.to_string();
            if self.job_is_unique(unit.job) &&
                result.info_anims.is_some_and(|s| s.str_contains("c000") || s.str_contains("c050"))
            {
                if head.contains("c801") { self.random_head(result, unit, ConditionFlags::Male, false);  }
                else if head.contains("c851") { self.random_head(result, unit, ConditionFlags::Female, false);  }
            } 
        }
    }

    pub fn get_gender_condition(&self, gender: i32) -> i32 {
        if gender == 1 { self.male_index } 
        else if gender == 2 { self.female_index } 
        else { 0 }
    }
    pub fn random_aoc(&self, unit: &Unit, result: &mut AssetTableResult, conditions: ConditionFlags, skip: usize) {
        if conditions.contains(ConditionFlags::TikiEngage) { return; }
        if conditions.contains(ConditionFlags::Male) || conditions.contains(ConditionFlags::Female) {
            let hash = if unit.person.get_asset_force() == 0 { unit.person.parent.hash } else { unit.grow_seed };
            let rng = crate::utils::create_rng(hash, 1);
            if unit.status.value & 8388608 != 0 { rng.get_value(1); }
            for _ in 0..skip { rng.get_value(1); }
            let aoc = if conditions.contains(ConditionFlags::Male) { &self.aoc_m[ rng.get_value( self.aoc_m.len() as i32 ) as usize] }
                else { &self.aoc_f[ rng.get_value( self.aoc_f.len() as i32 ) as usize] };
            result.info_anims = Some(concat_string!("AOC_Info_c", aoc).into());
        }
    }
    pub fn job_can_use_canon(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.cannon) }
    pub fn job_can_use_dragonstone(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.dragon_stone) }
    pub fn job_is_unique(&self, job_data: &JobData) -> bool { self.job.iter().find(|x| x.job_hash == job_data.parent.hash).map_or_else(|| false, |x| x.unique) }

    pub fn get_random_appearance(&self, result: &mut AssetTableResult, mode: i32, unit: &Unit, conditions: &ConditionFlags) -> bool {
        let outfit = get_unit_outfit_mode(unit);
        let mode = if mode == 3 { 1 } else { mode };
        if outfit & 32 == 0 || conditions.contains(ConditionFlags::TikiEngage) {
            if outfit & 64 != 0 { crate::assets::dress::change_result_colors_by_unit(unit, result); }
            if !conditions.contains(ConditionFlags::TikiEngage) && mode == 1 && conditions.contains(ConditionFlags::CausalClothes) && !result.body_model.is_null() {
                let o_body = result.body_model.get_hash_code();
                if !self.o_male.iter().any(|x| x.1 == o_body) && !self.o_female.iter().any(|x| x.1 == o_body) {
                    self.get_default_battle_outfits(result, mode, unit);
                }
            }
            else if !conditions.contains(ConditionFlags::TikiEngage) && mode == 2 && conditions.contains(ConditionFlags::CausalClothes) && !result.dress_model.is_null() {
                let o_body = result.dress_model.get_hash_code();
                if !self.u_male.iter().any(|x| x.1 == o_body) && !self.u_female.iter().any(|x| x.1 == o_body) {
                    self.get_default_battle_outfits(result, mode, unit);
                }
            }
            if GameVariableManager::get_number("G_RandAsset") > 1 {
                let skip = (unit.status.value & 0x800000 != 0) as usize;
                self.random_aoc(unit, result, conditions.clone(), skip);
            }
            return false;
        }
        let key =
        if unit.person.parent.index == 1 {
            if unit.edit.gender == 2 { "G_AMPID_LueurF".to_string() }
            else { "G_AMPID_LueurM".to_string() }
        }
        else { format!("G_A{}", unit.person.name.unwrap()) };
        if GameVariableManager::exist(key.as_str()) {
            let index = GameVariableManager::get_number(key.as_str());
            if let Some(data) = self.personal_data.get(index as usize) {
                let stating_job_hash = GameVariableManager::get_number(format!("G_JG_{}", unit.person.pid));
                let dress = JobData::try_get_hash(stating_job_hash).is_some_and(|j|
                    j.parent.hash == unit.job.parent.hash
                );
                data.commit_result(result, mode, dress);
                if (outfit & 3 != 0) && unit.accessory_list.unit_accessory_array[0].index > 0 {
                    self.get_random_job_dress(result, 0, mode, unit, conditions);
                }
                let skip = (GameVariableManager::get_number("G_RandAsset") > 1) as usize + (unit.status.value & 0x800000 != 0) as usize + 2;
                self.random_aoc(unit, result, conditions.clone(), skip);
                if outfit & 64 != 0 {
                    crate::assets::dress::change_result_colors_by_unit(unit, result);
                }
                return true;
            }
        }
        false
    }
    pub fn get_random_job_dress(&self, result: &mut AssetTableResult, ty: i32, mode: i32, unit: &Unit, conditions: &ConditionFlags) {
        let mut hash = (unit.person.parent.hash as u32 >> 2) + (DVCVariables::get_seed() as u32 >> 1);
        if ty == 0 {
            hash += unit.accessory_list.unit_accessory_array[0].index as u32 + (unit.job.parent.hash as u32 >> 2);
        }
        else if ty == 1 || ty == 11 {
            let rng =  Random::new(hash);
            if ty > 10 { rng.get_value(100000); }
            let value = rng.get_value(100000) as u32;
            let mask = 2 + (unit.selected_weapon_mask.value != 0) as i32;
            hash = (unit.job.parent.hash >> 1) as u32 + value + rng.get_value(mask) as u32;
        }
        else if ty == 2 || ty == 12 {
            let rng =  Random::new(unit.grow_seed as u32);
            if ty > 10 { rng.get_value(100000); }
            let value = rng.get_value(100000) as u32;
            hash = value + (unit.job.parent.hash as u32 >> 1);
        }
        else if ty >= 20 && ty < 100 {
            let s = GodData::try_index_get(ty- 20).unwrap().parent.hash;
            hash += s as u32 >> 1;
        }
        else { hash += ty as u32 >> 1; }
        let is_female = conditions.contains(ConditionFlags::Female);
        if mode == 1 || mode == 3 { result.body_model = self.get_random_outfit(mode, hash, is_female).into(); }
        else { result.dress_model = self.get_random_outfit(mode, hash, is_female).into(); }
    }

    pub fn get_random_outfit(&self, mode: i32, hash: u32, is_female: bool) -> String {
        let set =
        match mode {
            1|3 => { if !is_female { &self.o_male } else { &self.o_female } },
            _ => { if !is_female { &self.u_male } else { &self.u_female } },
        };
        let index = hash % (set.len() as u32);
        set.get(index as usize).map(|m| m.0.clone()).unwrap()
    }
    pub fn get_default_battle_outfits(&self, result: &mut AssetTableResult, mode: i32, unit: &Unit) -> bool {
        if unit.person.parent.index == 1 {
            let suffix = if unit.edit.gender == 1 { "M_c001" } else { "F_c051" };
            if mode == 1 { result.body_model = format!("oBody_Drg0A{}", suffix).into(); }
            else if mode == 2 { result.dress_model = format!("uBody_Drg0A{}", suffix).into(); }
            else {
                result.body_model = format!("gBody_Drg0A{}", suffix).into();
            }
            return true;
        }
        else {
            if let Some(body_suffix) = unit.person.get_name().and_then(|name|
                self.personal_outfits.get(name.to_string().as_str())
            ){
                if mode == 1 || mode == 3 { result.body_model = format!("oBody_{}", body_suffix).into(); }
                else { result.dress_model = format!("uBody_{}", body_suffix).into(); }
                return true;
            }
        }
        false
    }
    pub fn check_dress_body(&self, result: &mut AssetTableResult, unit: Option<&Unit>, hash: u32, mode: i32, is_female: bool) {
        if self.is_missing_body(result, mode) {
            if !unit.is_some_and(|unit|  self.get_default_battle_outfits(result, mode, unit)) {
                let body =  self.get_random_outfit(mode, hash, is_female);
                if mode != 2 { result.body_model = body.into(); }
                else { result.dress_model = body.into(); }
            }
        }
    }
    pub fn is_missing_body(&self, result: &mut AssetTableResult, mode: i32) -> bool {
        let is_mode_1 = mode == 1 || mode == 3;
        if is_mode_1 { // oBody
            result.body_model.is_null() || {
                let body_hash = result.body_model.get_hash_code();
                !self.o_male.iter().any(|x| x.1 == body_hash) && !self.o_female.iter().any(|x| x.1 == body_hash)
            }
        }
        else {  // uBody
            result.dress_model.is_null() || {
                let body_hash = result.dress_model.get_hash_code();
                !self.u_male.iter().any(|x| x.1 == body_hash) && !self.u_female.iter().any(|x| x.1 == body_hash)
            }
        }
    }
}

pub struct PersonAsset {
    pub is_female: bool,
    pub asset_table_mode: [i32; 3],
    pub icon: String,
    pub name: String,
    pub default_outfit: Option<String>,
}

impl PersonAsset {
    pub fn new(line: &str) -> Self {
        let params = line.split_whitespace().collect::<Vec<&str>>();
        let is_female = params[2].parse::<i32>().unwrap() == 2;
        let icon = params[3].to_string();
        let name =
        if params[0].starts_with("MPID") { params[0].to_string() }
        else if params[0].starts_with("PID") {
            PersonData::get(params[0]).and_then(|p| p.name)
                .map(|name| name.to_string()).unwrap_or(String::from("MPID_Lueur"))
        }
        else if params[0].starts_with("JID") {
            "MPID_PastLueur".to_string()
        }
        else { String::from("MPID_Lueur") };
        let mut asset_table_mode: [i32; 3] = [-1; 3];
        if params[0].contains("Lueur") || params[0].starts_with("JID")  {
            let gen_con = if is_female { "女装" } else { "男装" };
            for mode in 0..3 {
                if let Some(x) = search_by_2_keys(mode, params[0], gen_con) {
                    asset_table_mode[mode as usize] = x.parent.index;
                }
            }
        }
        else {
            let asset_table_count = AssetTable::get_count();
            let mut start = 1000;
            for mode in 0..3 {
                while start < asset_table_count {
                    if let Some(x) = search_by_key(mode, params[0], Some(start)) {
                        if mode == 0 {
                            asset_table_mode[mode as usize] = x.parent.index;
                            break;
                        }
                        if x.head_model.is_some() || x.hair_model.is_some() {
                            asset_table_mode[mode as usize] = x.parent.index;
                            break;
                        }
                        start = x.parent.index + 1;
                    }
                    else { break; }
                }
            }
        }
        Self {
            asset_table_mode,
            name,
            is_female,
            icon,
            default_outfit: if params[1] == "none" { None } else { Some(params[1].to_string())}
        }
    }
    pub fn commit_result(&self, result: &mut AssetTableResult, mode: i32, dress: bool) -> bool {
        let mode = if mode == 3 { 1 } else { mode };
        if self.name.contains("PastLueur") {
            let suffix = if self.is_female { "051" } else { "001" };
            if mode == 1 {
                result.head_model = format!("oHair_h{}", suffix).into();
                change_accessory(result.accessory_list, "uAcc_spine2_Hair051", "c_spine1_jnt");
            } else {
                result.head_model = format!("uHead_c{}", suffix).into();
                if self.is_female { result.hair_model = "uHair_null".into(); }
                else { result.hair_model = "uHair_h001".into(); }
            }
        }
        if let Some(s) = AssetTable::try_index_get(self.asset_table_mode[0]) {
            result.commit_asset_table(s);
        }
        if let Some(s) = AssetTable::try_index_get(self.asset_table_mode[mode as usize]) {
            if let Some(head) = s.head_model { result.head_model = head.clone(); }
            result.accessory_list.clear();
            if let Some(hair) = s.hair_model { result.hair_model = hair.clone(); }
            if dress && self.default_outfit.is_some() {
                let outfit = self.default_outfit.as_ref().unwrap();
                if mode == 1 { result.body_model = format!("oBody_{}", outfit).into(); }
                else { result.dress_model = format!("uBody_{}", outfit).into(); }
            }

            s.accessory_list.list.iter()
                .filter(|x|
                    x.locator.is_some() && x.model.is_some_and(|x| !x.str_contains("Emblem"))
                )
                .for_each(|x|{
                    let model = x.model.unwrap().to_string();
                    let locator = x.locator.unwrap().to_string();
                    change_accessory(result.accessory_list, &model, &locator);
            });

            return true;
        }
        false
    }
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
        let contains_m = ["M_c", "M1_c", "M2_c", "M3_c", "M4_c", "M5_c", "Pha0AF_c002", "Pha0AF_c004", "Pha0AF_c015", "Pha0AF_c012", "Pha0AF_c018" ];
        let contains_f = ["F_c", "F1_c", "F2_c", "F3_c", "F4_c", "F5_c" ];
        slist.bond_face = ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>().files.entries.iter()
            .filter(|x|x.key.is_some_and(|x| x.str_contains("Telop/LevelUp/FaceThumb/")))
            .map(|x| x.key.unwrap().to_string()).collect();
        let v: Vec<_> =
        ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>().files.entries.iter().filter(|x|
            x.key.is_some_and(|x| {
                let s = x.to_string();
                s.contains("uBody_") && !s.contains("Body_Box") && !s.contains("null") &&
                    ( contains_m.iter().any(|x| s.contains(x)) | contains_f.iter().any(|x| s.contains(x)))
            })
        ).map(|x|{
            let part = x.key.unwrap().to_string().split("/").last().unwrap().to_string();
            let il2p: &Il2CppString =  part.clone().into();
            let hash = il2p.get_hash_code();
            (part, hash)
        }).collect();

        slist.u_male = v.iter().filter(|x| contains_m.iter().any(|s| x.0.contains(s))  )
            .map(|x| x).map(|v|v.clone()).collect();

        slist.u_female = v.iter().filter(|x| contains_f.iter().any(|s| x.0.contains(s)) && !contains_m.iter().any(|s| x.0.contains(s)))
            .map(|x| x).map(|v|v.clone()).collect();

        slist.engaging = AnimSetDB::get_list().unwrap().iter().filter(|x| x.parent.index > 0 && x.get_engage1().is_some_and(|x| x.to_string() == "=")).map(|x| x.parent.index).collect();
        println!("Engaging {}", slist.engaging.len());
        let v: Vec<_> =
            ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>().files.entries.iter().filter(|x|
                x.key.is_some_and(|x| {
                    let s = x.to_string();
                    s.contains("oBody_") && !s.contains("Body_Box") && !s.contains("null") &&
                        ( contains_m.iter().any(|x| s.contains(x)) | contains_f.iter().any(|x| s.contains(x)))
                })
            ).map(|x|{
                let part = x.key.unwrap().to_string().split("/").last().unwrap().to_string();
                let il2p: &Il2CppString = part.clone().into();
                let hash = il2p.get_hash_code();
                (part, hash)
            })
                .collect();
        slist.o_male = v.iter().filter(|x|
            contains_m.iter().any(|s| x.0.contains(s)))
            .map(|v|v.clone()).collect();
        slist.o_female = v.iter().filter(|x|
            contains_f.iter().any(|s| x.0.contains(s))
            && !contains_m.iter().any(|s| x.0.contains(s))
        )
            .map(|v|v.clone()).collect();

        aoc.for_each(|v|{
            if v.0 < 53 {
                let s: Vec<_> = v.1.split_whitespace().collect();
                slist.personal_outfits.insert(s[0], s[1]);
            }
            if v.0 >= 2 {
                slist.personal_data.push(PersonAsset::new(v.1));
            }
        });  // Tiki Engage
        if let Some(entry) =  asset_table_search(1, &vec!["EID_チキ"]) {
            slist.engage.push( SearchData::new(SearchType::Person,0, Gender::None, 13, entry) )
        }
        if let Some(entry) =  asset_table_search(2, &vec!["EID_チキ"]) {
            slist.engage.push( SearchData::new(SearchType::Person,0, Gender::None, 13, entry) )
        }

        // Emblems
        let emblem_list =  EMBLEM_LIST.get().unwrap();
        for x in 0..emblem_list.len() {
            slist.add_god(GodData::try_get_hash(emblem_list[x]).unwrap(), x as i32);
        }
        JobData::get_list().unwrap().iter().for_each(|job|{
            slist.add_job(job);
            let jid = job.jid.to_string();
            if jid.contains("_E") {
                if let Some(original_job) = JobData::get( jid.replace("_E", "下級").as_str()) {
                    slist.fx_classes.insert(job.parent.hash, original_job.parent.hash);
                }
            }
        });
        ItemData::get_list().unwrap().iter().for_each(|item|
            { search_by_iid(item.iid, 2).map(|entry| slist.items.add(item, entry.parent.index)); }
        );

        slist
    });
}