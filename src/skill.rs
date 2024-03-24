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
use crate::{person, deploy, random};

pub struct SkillIndex {
    pub index: i32,
    pub in_use: bool,
}
impl SkillIndex {
    fn new(value: i32) -> Self { Self { index: value, in_use: false }}
}
pub static SKILL_POOL: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
static PERSONS_SKILLS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
static LEARN_SKILLS:  Mutex<Vec<i32>> = Mutex::new(Vec::new());
static LUNATIC_SKILLS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

static INHERIT_SKILLS:  Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());

// Skill randomization and Emblem skills/stat boost randomization upon msbt loading
pub fn create_skill_pool() {
    if SKILL_POOL.lock().unwrap().len() != 0 { return; }
    let skill_list = SkillData::get_list().unwrap();
    for x in person::PIDS {
        let person = PersonData::get(x).unwrap();
        let personal_sid = person.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
            if skill.name.is_none() { continue; }
            if skill.get_flag() & 1 == 1 { continue; }
            let index = SkillData::get_index(skill.sid);
            PERSONS_SKILLS.lock().unwrap().push(index);
            println!("Skill: {} from {} is added to personal skill pool position {}", skill.name.unwrap().get_string().unwrap(), person.get_name().unwrap().get_string().unwrap(), y);
            break;
        }
    }
    println!("Personal Skill Size {}", PERSONS_SKILLS.lock().unwrap().len());
    for x in 0..skill_list.len() {
        let flag = skill_list[x].get_flag();
        let mut skip = false;
        if skill_list[x].get_inheritance_cost() != 0 { 
            let skill_name = Mess::get( skill_list[x].name.unwrap() ).get_string().unwrap();
            if skill_name.len() == 0 { continue;  }
            let skill_help = Mess::get( skill_list[x].help.unwrap() ).get_string().unwrap();
            if skill_help.len() == 0 { continue; }
            INHERIT_SKILLS.lock().unwrap().push(SkillIndex::new(x as i32));
         }
        if skill_list[x].help.is_none() { continue; }
        if skill_list[x].name.is_none() { continue; }
        if skill_list[x].is_style_skill() { continue; }
        if skill_list[x].can_override_skill() && skill_list[x].get_priority() % 2 == 0 { continue; }
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
            if random::str_contains(skill_list[x].sid ,"E00"){ continue; }
            if random::str_contains(skill_list[x].sid ,"G00"){ continue; }
           // println!("Skill #{}: {} is added to the pool", x, skill_name);
            SKILL_POOL.lock().unwrap().push(SkillIndex::new(x as i32));
        }
    }
    println!("Total Skills in Pool: {}", SKILL_POOL.lock().unwrap().len());
    let job_list = JobData::get_list().unwrap();
    
    unsafe {
        for x in 0..job_list.len() {
            let job = &job_list[x];
            let mut index_learn = -1;
            let mut index_lunatic = -1;
            let learn_skill = job.learn_skill;
            if learn_skill.is_some() {
                let sid = learn_skill.unwrap();
                if !random::is_null_empty(sid, None) {  index_learn = SkillData::get_index(sid);  }
            }
            let lunatic_skill = job.lunatic_skill;
            if lunatic_skill.is_some() {
                let sid = lunatic_skill.unwrap();
                if !random::is_null_empty(sid, None) { index_lunatic = SkillData::get_index(sid);  }
            }
            LEARN_SKILLS.lock().unwrap().push( index_learn );
            LUNATIC_SKILLS.lock().unwrap().push( index_lunatic );
        }
    }
}

pub fn reset_skills() {
    println!("Resetting skills to normal");
    /*
    let mut index = 0;
    let skill_list = SkillData::get_list().unwrap();
    for x in person::PIDS {
        let skill_index = PERSONS_SKILLS.lock().unwrap()[index as usize];
        let person = PersonData::get(x).unwrap();
        let personal_sid = person.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
            if skill.get_flag() & 1 == 0 {
                personal_sid[y as usize] = skill_list[ skill_index as usize].sid;
                break;
            }
        }
        person.on_complete();
        index += 1;
    }
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        let index_learn = LEARN_SKILLS.lock().unwrap()[x as usize];
        let index_lunatic = LUNATIC_SKILLS.lock().unwrap()[x as usize];
        if index_learn != -1 {  job.set_learning_skill( skill_list[ index_learn as usize ].sid ); }
        if index_lunatic != -1 {  job.set_lunatic_skill( skill_list[ index_lunatic as usize].sid );  }
    }

    */
    let skill_pool_count = SKILL_POOL.lock().unwrap().len();
    let inherit_count = INHERIT_SKILLS.lock().unwrap().len();
    for x in 0..skill_pool_count { SKILL_POOL.lock().unwrap()[x as usize].in_use = false; }
    for x in 0..inherit_count { INHERIT_SKILLS.lock().unwrap()[x as usize].in_use = false; }

}
pub fn randomize_skills() {
    if !GameVariableManager::get_bool("G_Random_Skills") { return; }
    println!("randomizing skills");
    let skill_list = SkillData::get_list().unwrap();
    let rng = Random::instantiate().unwrap();
    let seed = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_pool_count = SKILL_POOL.lock().unwrap().len() as i32;
    for x in person::PIDS {
        let person = PersonData::get(x).unwrap();
        let personal_sid = person.get_common_sids().unwrap();
        let mut skill_index = rng.get_value(skill_pool_count) as usize;
        while SKILL_POOL.lock().unwrap()[skill_index].in_use { skill_index = rng.get_value(skill_pool_count) as usize; }
        for y in 0..personal_sid.len() {
            let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
            if skill.get_flag() & 1 == 0 {
                let index = SKILL_POOL.lock().unwrap()[skill_index].index;
                personal_sid[y as usize] = skill_list[ index  as usize].sid;
                SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
           //     println!("{} has personal skill {} in position {}", person.get_name().unwrap().get_string().unwrap(), skill_list[ index as usize].help.unwrap().get_string().unwrap(), y);
                break;
            }
        }
        person.on_complete();
    }
    println!("Person Skills complete");
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        let index_learn = LEARN_SKILLS.lock().unwrap()[x as usize];
        if index_learn == -1 { continue; }
        let mut skill_index = rng.get_value(skill_pool_count) as usize;

        while SKILL_POOL.lock().unwrap()[skill_index].in_use { 
            skill_index = rng.get_value(skill_pool_count) as usize;
        }
        let index = SKILL_POOL.lock().unwrap()[ skill_index  ].index as usize;
        job.set_learning_skill( skill_list[ index ].sid ); 
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
    }
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        let index_lunatic = LUNATIC_SKILLS.lock().unwrap()[x as usize];
        if index_lunatic == -1 { continue; }
        let mut skill_index = rng.get_value(skill_pool_count) as usize;
        while SKILL_POOL.lock().unwrap()[skill_index].in_use {
            skill_index = rng.get_value(skill_pool_count) as usize;
        }
        let index = SKILL_POOL.lock().unwrap()[ skill_index ].index as usize;
        job.set_lunatic_skill( skill_list[ index ].sid ); 
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
    }
}

pub fn randomized_god_data(){
    let mode = GameVariableManager::get_number("G_Random_God_Mode");
    if mode == 0 { return; }
    //Engraves + 
    println!("Randommizing God Data...");
    if mode == 1 || mode == 3 {
        let list_size = INHERIT_SKILLS.lock().unwrap().len();
        for x in 0..list_size { INHERIT_SKILLS.lock().unwrap()[x].in_use = false; }
        let rng = Random::instantiate().unwrap();
        let seed = 3*GameVariableManager::get_number("G_Random_Seed") as u32;
        rng.ctor(seed);
        let mut max_engrave_stat: [i8; 6] = [0; 6];
        let skill_list = SkillData::get_list().unwrap();
        // get engrave min and max and change inheritable skills
        for x in deploy::EMBLEM_GIDS {
            let god = GodData::get(*x).unwrap();
            if max_engrave_stat[0] < god.get_engrave_avoid() { max_engrave_stat[0] = god.get_engrave_avoid(); }
            if max_engrave_stat[1] < god.get_engrave_critical() { max_engrave_stat[1] = god.get_engrave_critical();}
            if max_engrave_stat[2] < god.get_engrave_hit() { max_engrave_stat[2] = god.get_engrave_hit(); }
            if max_engrave_stat[3] < god.get_engrave_power() { max_engrave_stat[3] = god.get_engrave_power(); }
            if max_engrave_stat[4] < god.get_engrave_secure() { max_engrave_stat[4] = god.get_engrave_secure(); } 
            if max_engrave_stat[5] < god.get_engrave_weight() { max_engrave_stat[5] = god.get_engrave_weight(); } 
            let ggid = GodGrowthData::try_get_from_god_data(god);
            if ggid.is_none() { continue; }
            let god_grow = ggid.unwrap(); 
            for y in 0..god_grow.len() {
                let level = god_grow[y].get_inheritance_skills();
                if level.is_none() {continue; }
                let inherit_skills = level.unwrap();
                for z in 0..inherit_skills.len() {
                    let mut value = rng.get_value(list_size as i32) as usize;
                    if value >= list_size { continue; }
                    while INHERIT_SKILLS.lock().unwrap()[value].in_use { value = rng.get_value(list_size as i32) as usize; }
                    inherit_skills[z] = skill_list[ INHERIT_SKILLS.lock().unwrap()[value].index as usize ].sid;
                    INHERIT_SKILLS.lock().unwrap()[value].in_use = false;
                }
                god_grow[y].on_complete(); 
            }
        }
        // randomization of engrave data
        for x in 0..6 {
             max_engrave_stat[x] = max_engrave_stat[x] / 5;
        }
        for x in deploy::EMBLEM_GIDS {
            let god = GodData::get(*x).unwrap();
            for i in 0..6 {
                let value =  rng.get_min_max( -5*max_engrave_stat[i as usize] as i32 , 5*max_engrave_stat[i as usize ] as i32) as i8;
                god.set_engrave( i as i32, value);
            }
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
        if CONFIG.lock().unwrap().random_skill { this.command_text = "On".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

pub struct RandomGodMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGodMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_mode, 0, 1, 1);
        if CONFIG.lock().unwrap().random_god_mode != result {
            CONFIG.lock().unwrap().random_god_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_mode == 1 {  this.help_text = "Engraves and inheritiable skills are randomized".into(); }
        else { this.help_text = "No changes to emblem data.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_mode == 1 { this.command_text = "Engraves + Inherits".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

#[no_mangle]
extern "C" fn skill_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomSkillMod>("Randomize Skills") } 
extern "C" fn god_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomGodMod>("Randomize Emblem Data") } 
pub fn install_skill_rnd() { 
    cobapi::install_global_game_setting(skill_rnd); 
    cobapi::install_global_game_setting(god_rnd);
}
