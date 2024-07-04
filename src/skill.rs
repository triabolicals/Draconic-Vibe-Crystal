use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    mess::*,
    gamedata::{*, skill::*, GodData, god::*},
};

use std::sync::Mutex;
use super::CONFIG;
use crate::{enums::*, person, utils::*};

pub static SKILL_POOL: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static MADDENING_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static INHERIT_SKILLS:  Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());

pub struct SkillIndex {
    pub index: i32,
    pub in_use: bool,
    pub linked_use: bool,
}

impl SkillIndex { pub fn new(value: i32) -> Self { Self { index: value, in_use: false, linked_use: false, }} }

// Skill randomization and Emblem skills/stat boost randomization upon msbt loading
pub fn print_bad_inherit_skill() {
    let skill_list = SkillData::get_list().unwrap();
    let mut inherit_from_gods: Vec<usize> = Vec::new();
    for x in EMBLEM_GIDS {
        let god = GodData::get(*x).unwrap();
        let ggid = GodGrowthData::try_get_from_god_data(god);
        if ggid.is_none() { continue; }
        let god_grow = ggid.unwrap(); 
        for y in 0..god_grow.len() {
            let level = god_grow[y].get_inheritance_skills();
            if level.is_none() {continue; }
            let inherit_skills = level.unwrap();
            for z in 0..inherit_skills.len() {
                let sid = inherit_skills[z].get_string().unwrap();
                let sk = SkillData::get(&sid);
                if sk.is_some() {
                    let index: usize = sk.unwrap().parent.index as usize;
                    inherit_from_gods.push(index);
                }
            }
        }
    }
    for x in 0..skill_list.len() {
        let skill = &skill_list[x];
        let flag = skill.get_flag();
        if ( skill.get_inheritance_cost() != 0 && skill.get_inheritance_sort() == 0 ) &&  ( flag & 1 == 0 ) { 
            if inherit_from_gods.iter().find(|index| x == **index).is_none() {
                skill.set_inherit_cost(0);
            }
        }
    }
}

pub fn create_skill_pool() {
    println!("Generating skill pool");
    let skill_list = SkillData::get_list().unwrap();
    if SKILL_POOL.lock().unwrap().len() != 0 { return; }
    for x in 0..skill_list.len() {
        let sid = skill_list[x as usize].sid.get_string().unwrap();
        if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x == sid ).is_some() { continue;}
        let mut skip = false;
        let flag = skill_list[x].get_flag();
        if skill_list[x].help.is_none() { continue; }
        if skill_list[x].name.is_none() { continue; }
        if skill_list[x].get_inheritance_cost() != 0 && skill_list[x].get_inheritance_sort() != 0 { 
            let skill_name = Mess::get( skill_list[x].name.unwrap() ).get_string().unwrap();
            if skill_name.len() == 0 { continue;  }
            let skill_help = Mess::get( skill_list[x].help.unwrap() ).get_string().unwrap();
            if skill_help.len() == 0 { continue; }
            INHERIT_SKILLS.lock().unwrap().push(SkillIndex::new(x as i32));
        }
        if skill_list[x].is_style_skill() { continue; }
        for y in 0..8 {
            if flag & (1 << y ) != 0 {
                skip = true;
                break;
            }
        }
        if !skip {
            let skill_name = Mess::get( skill_list[x].name.unwrap() ).get_string().unwrap();
            if skill_name.len() == 0 { continue;  }
            let skill_help = Mess::get( skill_list[x].help.unwrap() ).get_string().unwrap();
            if skill_help.len() == 0 { continue; }
            if str_contains(skill_list[x].sid ,"E00"){ continue; }
            if str_contains(skill_list[x].sid ,"G00"){ continue; }
            if flag & 1 == 1 { continue; }
            SKILL_POOL.lock().unwrap().push(SkillIndex::new(x as i32));
            if MADDENING_BLACK_LIST.iter().find(|lol| **lol == sid ).is_none() {
                MADDENING_POOL.lock().unwrap().push(x as i32);
            }
        }
    }
    println!("Total Maddening Skills in Pool: {}",  MADDENING_POOL.lock().unwrap().len());
    println!("Total Skills in Pool: {}", SKILL_POOL.lock().unwrap().len());
    println!("Total Inherit Skills in Pool: {}", INHERIT_SKILLS.lock().unwrap().len());
    crate::emblem::emblem_skill::create_emblem_skill_pool();
}

fn get_highest_priority(index: i32) -> i32 {
    let skill_list = SkillData::get_list().unwrap();
    let skill = &skill_list[ index as usize]; 
    let priority = skill.get_priority();
    if priority == 0 || priority > 9 { return index; }
    let sid = skill.sid.get_string().unwrap(); 
    //Eirika Skills
    for x in 0..EIRIKA_TWIN_SKILLS.len() {
        if sid == EIRIKA_TWIN_SKILLS[x as usize] {
            if x < 6 { return SkillData::get( EIRIKA_TWIN_SKILLS[ (6 + x) as usize]).unwrap().parent.index; }
            else { return SkillData::get( EIRIKA_TWIN_SKILLS[x as usize]).unwrap().parent.index; }
        }
    }
    let mut new_index = index+1;
    let mut current_priority = priority;
    loop {
        let new_skill = &skill_list[new_index as usize];
        if current_priority < new_skill.get_priority() {
            current_priority = new_skill.get_priority();
            new_index += 1;
        }
        else { return new_index - 1;  }
    }
}

pub fn get_random_skill(difficulty: i32, rng: &Random) -> &'static SkillData {
    let skill_pool_size;
    let mut skill_index;
    let skill_list = SkillData::get_list().unwrap();
    if difficulty == 2 {
        skill_pool_size = MADDENING_POOL.lock().unwrap().len();
        skill_index = MADDENING_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize];
        if GameVariableManager::get_bool("G_Cleared_M017") { skill_index = get_highest_priority(skill_index); }
        return &skill_list[ skill_index as usize];
    }
    else {
        skill_pool_size = SKILL_POOL.lock().unwrap().len();
        skill_index =  SKILL_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize].index;
        return &skill_list[ skill_index as usize];
    }
}
pub fn reset_skills() {
    println!("Resetting skills to normal");
    let skill_pool_count = SKILL_POOL.lock().unwrap().len();
    let inherit_count = INHERIT_SKILLS.lock().unwrap().len();
    for x in 0..skill_pool_count { SKILL_POOL.lock().unwrap()[x as usize].in_use = false; }
    for x in 0..inherit_count { INHERIT_SKILLS.lock().unwrap()[x as usize].in_use = false; }
    crate::emblem::emblem_skill::reset_emblem_skills();
}

pub fn replace_all_sid_person(person: &PersonData, sid: &Il2CppString, new_sid: &Il2CppString) {
    let person_list = PersonData::get_list_mut().unwrap();
    let name = person.get_name().unwrap().get_string().unwrap();
    let sid_comp = sid.get_string().unwrap();
    for x in 2..person_list.len() {
        let person_x = &person_list[x as usize];
        if person_x.parent.index == person.parent.index { continue;}
        if person_x.get_name().is_none() { continue; }
        if person_x.get_name().unwrap().get_string().unwrap() != name {continue; }
        if person_x.get_common_sids().is_none() { continue; }
        let personal_sid = person_x.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            if personal_sid[y as usize].get_string().unwrap() == sid_comp {
                personal_sid[y as usize] = new_sid;
                person_x.on_complete();
                break;
            }
        }
    }
}

pub fn randomize_skills() {
    if !GameVariableManager::get_bool("G_Random_Skills") { return; }
    println!("randomizing skills");
    let skill_list = SkillData::get_list().unwrap();
    let rng = Random::instantiate().unwrap();
    let seed = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_pool_count = SKILL_POOL.lock().unwrap().len() as i32;
    // Playables
    let playable_size = person::PLAYABLE.lock().unwrap().len();
    let person_list = PersonData::get_list().unwrap();
    for x in 0..playable_size {
        let p_index = person::PLAYABLE.lock().unwrap()[x as usize] as usize;
        let person = &person_list[p_index]; 
        let personal_sid = person.get_common_sids().unwrap();
        let personal_key = format!("G_P_{}", person.pid.get_string().unwrap());
        let mut skill_index: usize; 
        let mut index; 
        if GameVariableManager::exist(&personal_key) {
            index = GameVariableManager::get_number(&personal_key) as usize; 
            let pos = SKILL_POOL.lock().unwrap().iter().position(|x| x.index == index as i32);
            if pos.is_some() {
                skill_index = pos.unwrap() as usize; 
                for y in 0..personal_sid.len() {
                    let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).unwrap();
                    if skill.get_flag() & 1 == 0 {
                        replace_all_sid_person(person, personal_sid[y as usize], skill_list[ index  as usize].sid);
                        personal_sid[y as usize] = skill_list[ index  as usize].sid;
                        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
                        break;
                    }
                }
                person.on_complete();
                continue;
            }
        }
        skill_index = rng.get_value(skill_pool_count) as usize;
        index = SKILL_POOL.lock().unwrap()[skill_index].index as usize; 
        let mut skill_sid = skill_list[index as usize].sid.get_string().unwrap();
        let mut count = 0;
        while count < 50 && ( ( SKILL_POOL.lock().unwrap()[skill_index].in_use || skill_list[index].get_inheritance_cost() != 0 ) || PERSONAL_BLIST.lock().unwrap().iter().find(|x| **x == skill_sid).is_some() ) { 
            skill_index = rng.get_value(skill_pool_count) as usize;
            index = SKILL_POOL.lock().unwrap()[skill_index].index as usize;
            if index > 1250 { continue; }
            skill_sid = skill_list[index as usize].sid.get_string().unwrap();
            if skill_sid == "SID_双聖" { SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true; }
            count+=1;
        }
        for y in 0..personal_sid.len() {
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).unwrap();
            if skill.get_flag() & 1 == 0 {
                replace_all_sid_person(person, personal_sid[y as usize], skill_list[ index  as usize].sid);
                personal_sid[y as usize] = skill_list[ index  as usize].sid;
                SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
                GameVariableManager::make_entry(&personal_key, index as i32);
                break;
            }
        }
        person.on_complete();
    }
    // the rest 
    println!("Person Skills complete");
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        if job.is_low() && job.max_level == 20 { continue; }
        let mut skill_index;
        let mut count = 0;
        loop {
            count += 1;
            skill_index = rng.get_value(skill_pool_count) as usize;
            if skill_list[ SKILL_POOL.lock().unwrap()[ skill_index ].index as usize ].can_override_skill() { continue; }
            if !SKILL_POOL.lock().unwrap()[skill_index].in_use || count >= 50 { break; }
        }
        count = 0;
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
        let mut skill_index2;
        loop {
            count += 1;
            skill_index2 = rng.get_value(skill_pool_count) as usize;
            if skill_list[ SKILL_POOL.lock().unwrap()[ skill_index2 ].index as usize ].can_override_skill() { continue; }
            if skill_index2 == skill_index { continue; }
            else {
                break;
            }
        }
        job.set_learning_skill( skill_list[ SKILL_POOL.lock().unwrap()[ skill_index  ].index as usize ].sid ); 
        job.set_lunatic_skill(  skill_list[ SKILL_POOL.lock().unwrap()[ skill_index2 ].index as usize ].sid ); 
        //SKILL_POOL.lock().unwrap()[ skill_index2 ].in_use = true;
    }
    let maddening_pool_size = MADDENING_POOL.lock().unwrap().len() as i32;
    let mut ring_skill_set: [bool; 1000] = [false; 1000];
    let ring_list = RingData::get_list_mut().unwrap();

    // Bond Rings
    let ranks = [3, 2, 1, 0]; 
    let ranks_rate: [i32; 4] = CONFIG.lock().unwrap().get_bond_ring_rates();
    let rng_rings = Random::instantiate().unwrap();
    let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
    rng_rings.ctor(seed);
    for y in 0..4 {
        let current_rank = ranks[y as usize];
        let odds = ranks_rate[y as usize];
        if odds == 0 { continue; }
        for x in 0..ring_list.len() {
            if ring_list[x].rank != current_rank { continue; }
            if odds < rng_rings.get_value(100) { continue; }
            let equip_skills = ring_list[x].get_equip_skills();
            let mut index = rng.get_value(maddening_pool_size) as usize;
            let mut count = 0;
            while ring_skill_set[ index ] && count < 50 { 
                index = rng.get_value(maddening_pool_size) as usize; 
                count += 1;
            }
            let s_index = MADDENING_POOL.lock().unwrap()[index];
            let s_high = get_highest_priority(s_index);
            equip_skills.clear();
            equip_skills.add_skill(&skill_list[s_high as usize],6, 0);
            ring_skill_set[ index ] = true;
        }
    }
}

pub struct RandomSkillMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSkillMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_skill);
        if CONFIG.lock().unwrap().random_skill != result {
            CONFIG.lock().unwrap().random_skill  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_skill {  this.help_text = "Personals and class skills are randomized.".into(); }
        else { this.help_text = "No changes to personal and class skills.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_skill { this.command_text = "Player + Enemy".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}