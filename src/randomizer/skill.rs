use unity::prelude::*;
use super::{person::{*, PLAYABLE}, emblem::emblem_skill::*};
use engage::{
    menu::{BasicMenuResult, config::{*, ConfigBasicMenuItem}},
    gameuserdata::GameUserData,
    gamedata::god::*,
};

use std::sync::Mutex;
use super::CONFIG;
use crate::{enums::*, utils::*};

pub static SKILL_POOL: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static MADDENING_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());

pub struct SkillIndex {
    pub index: i32,
    pub in_use: bool,
    pub linked_use: bool,
}

impl SkillIndex { pub fn new(value: i32) -> Self { Self { index: value, in_use: false, linked_use: false, }} }

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
        this.help_text = if CONFIG.lock().unwrap().random_skill {  "Personals and class skills are randomized." }
            else { "No changes to personal and class skills." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().random_skill { "Randomize" }  else { "Default" }.into();
    }
}
pub struct EnemySkillGauge;
impl ConfigBasicMenuItemGaugeMethods for EnemySkillGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().random_enemy_skill_rate as f32 / 100.0 }
            else {GameVariableManager::get_number("G_EnemySkillGauge") as f32 / 100.0 }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let gauge = if GameUserData::get_sequence() == 0  { CONFIG.lock().unwrap().random_enemy_skill_rate  as f32 / 100.0 } 
            else { GameVariableManager::get_number("G_EnemySkillGauge")  as f32 / 100.0 };

        let result = ConfigBasicMenuItem::change_key_value_f(gauge, 0.0, 1.0, 0.25);
        if gauge != result {
            if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().random_enemy_skill_rate = ( result * 100.0 ) as i32; CONFIG.lock().unwrap().save(); } 
            else { GameVariableManager::set_number("G_EnemySkillGauge", (result * 100.0) as i32); }
            this.gauge_ratio = result;
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ this.help_text = "Percentage of enemy units will gain a random skill.".into(); }
}
pub extern "C" fn vibe_skill_gauge() -> &'static mut ConfigBasicMenuItem {  
    let skill_gauge = ConfigBasicMenuItem::new_gauge::<EnemySkillGauge>("Random Enemy Skill Rate");
    skill_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_skill_gauge as _);
    skill_gauge
}

// Skill randomization and Emblem skills/stat boost randomization upon msbt loading
pub fn create_skill_pool() {
    let skill_list = SkillData::get_list_mut().unwrap();
    let mut inherit_list: Vec<i32> = Vec::new();
    let godgrowlist = GodGrowthData::get_list().unwrap();
    for x in 0..godgrowlist.len() { 
        for y in 0..godgrowlist[x].len() {
            let level = godgrowlist[x][y].get_inheritance_skills();
            if level.is_none() {continue; }
            let inherit_skills = level.unwrap();
            for z in 0..inherit_skills.len() {
                let skill = SkillData::get(&inherit_skills[z].get_string().unwrap());
                if skill.is_some() { inherit_list.push(skill.unwrap().parent.index); }
            }
        }
    }
    for x in 1..skill_list.len() {
        let skill = &skill_list[x]; // fix bad inheritance data
        if skill.parent.index == 382 { continue; }
        if skill.get_inheritance_cost() > 0 && skill.get_inheritance_sort() == 0 {
            if inherit_list.iter().find(|y| **y == skill.parent.index ).is_none() {
                skill.set_inherit_cost(0);
            }
        }
        let current_priority = skill.get_priority() % 100;
        let previous_priority = skill_list[x - 1].get_priority() % 100;
        if current_priority < 2 { continue; }
        if current_priority > 1 && previous_priority != current_priority - 1 {
            skill.set_priority(0);
        }
    }
    if SKILL_POOL.lock().unwrap().len() != 0 { return; }
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_計略", true, true); 
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_無し", true, true);
    for x in 0..skill_list.len() {
        if skill_list[x].help.is_none() { continue; }
        if skill_list[x].name.is_none() { continue; }
        let sid = skill_list[x as usize].sid.get_string().unwrap();
        if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x == sid ).is_some() { continue;}
        let mut skip = false;
        let flag = skill_list[x].get_flag();
        if Mess::get(skill_list[x].name.unwrap()).get_string().unwrap().len() < 1 || Mess::get(skill_list[x].help.unwrap()).get_string().unwrap().len() < 1 { continue; }
        if skill_list[x].get_inheritance_cost() != 0 && skill_list[x].get_inheritance_sort() != 0 { 
            SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill_list[x], false);
        }
        if skill_list[x].is_style_skill() { continue; }
        for y in 0..8 {
            if flag & (1 << y ) != 0 {
                skip = true;
                break;
            }
        }
        if !skip {
            if str_contains(skill_list[x].sid ,"E00"){ continue; }
            if str_contains(skill_list[x].sid ,"G00"){ continue; }
            if flag & 1 == 1 { continue; }
            SKILL_POOL.lock().unwrap().push(SkillIndex::new(x as i32));
            SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill_list[x], false); // Chaos Skills
            if MADDENING_BLACK_LIST.iter().find(|lol| **lol == sid ).is_none() {
                MADDENING_POOL.lock().unwrap().push(x as i32);
                ENGAGE_SKILLS_CHAOS.lock().unwrap().push(SkillIndex::new(x as i32));
            }
        }
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_超越_闇", true, false); 

    println!("Total Maddening Skills in Pool: {}",  MADDENING_POOL.lock().unwrap().len());
    println!("Total Skills in Pool: {}", SKILL_POOL.lock().unwrap().len());
    create_emblem_skill_pool();
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
    for x in 0..skill_pool_count { SKILL_POOL.lock().unwrap()[x as usize].in_use = false; }
    reset_emblem_skills();
}

pub fn replace_all_sid_person(person: &PersonData) {
    if person.pid.get_string().unwrap() == "PID_ヴェイル" { return; } // ignore Veyle swaps
    let new_person = switch_person(person);
    let old_job = person.get_job().unwrap();
    let new_job = new_person.get_job().unwrap();
    let jid = 
        if old_job.is_high() && ( new_job.is_high() || (new_job.is_low() && new_job.max_level == 40 ) ) { new_job.jid }
        else if old_job.is_high() && ( new_job.is_low() && new_job.has_high() ) { new_job.get_high_jobs()[0].jid }
        else if old_job.is_low() &&  new_job.is_high() {
            if new_job.get_low_jobs().len() == 0 { "JID_ソードファイター".into() }
            else { new_job.get_low_jobs()[0].jid }
        }
        else { new_job.jid }; 

    let new_flag_value = if new_person.pid.get_string().unwrap() == "PID_リュール" { new_person.get_flag().value | 1536  }  
                         else { new_person.get_flag().value | 512 };

    let person_list = PersonData::get_list_mut().unwrap();
    let name = person.get_name().unwrap().get_string().unwrap();
    let grow = new_person.get_grow();
    let personal_sids = new_person.get_common_sids().unwrap();
    let mut new_sid = personal_sids[0];
    if GameVariableManager::get_number(&format!("G_P_{}", new_person.pid.get_string().unwrap())) != 0 {
        let new_skill_index = GameVariableManager::get_number(&format!("G_P_{}", new_person.pid.get_string().unwrap()));
        let new_skill = SkillData::try_index_get(new_skill_index);
        if new_skill.is_none() { return; }
        new_sid = new_skill.unwrap().sid;
    }
    else {
        for y in 0..personal_sids.len() {
            let skill = SkillData::get( &personal_sids[y as usize].get_string().unwrap() ).unwrap();
            if skill.get_flag() & 1 == 0 {
                new_sid = personal_sids[y as usize];
                break;
            }
        }
    }
    let icon_name = new_person.get_unit_icon_id().get_string().unwrap();
    let help_name = new_person.get_help().get_string().unwrap();
    for x in 2..person_list.len() {
        let person_x = &person_list[x as usize];
        if person_x.get_flag().value & 128 != 0 { continue; }
        if person_x.parent.index == person.parent.index { continue;}
        if person_x.get_name().is_none() { continue; }
        if person_x.get_name().unwrap().get_string().unwrap() != name {continue; }
        if person_x.get_common_sids().is_none() { continue; }
        if person_x.get_flag().value & 512 != 0 { continue; }   // prevents already changed from changing again
        person_x.set_name(new_person.get_name().unwrap().get_string().unwrap().into());
        let personal_sid = person_x.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).unwrap();
            if skill.get_flag() & 1 == 0 {
                personal_sid[y as usize] = new_sid;
                break;
            }
        }
        let job_x = person_x.get_job().unwrap();
        let level = person_x.get_level();
        if ( job_x.is_low() && job_x.max_level == 20 ) && new_job.is_high() { person_x.set_level(level); }
        else if ( job_x.is_low() && job_x.max_level == 40 ) && new_job.is_high() {
            if level > 20 { 
                person_x.set_level(level - 20 ); 
                person_x.set_internal_level(20); 
            }
        }
        else if job_x.is_high() && new_job.max_level == 40 { 
            let total = if person_x.get_internal_level() == 0 { 20 } else { person_x.get_internal_level() } as u8 + level;
            person_x.set_level( total ); 
            person_x.set_internal_level(0); 
        }
        person_x.get_flag().value = new_flag_value;
        let grow_x = person_x.get_grow();
        person_x.set_jid(jid);
        person_x.set_gender(new_person.get_gender());
        person_x.set_attrs(new_person.get_attrs());
        person_x.set_unit_icon_id(icon_name.clone().into());
        person_x.set_help(help_name.clone().into());
        if !grow_x.is_zero() {
            for x in 0..11 { 
                if grow_x[x as usize] == 0 { continue; }
                if grow_x[x as usize] < grow[x as usize] {  grow_x[x as usize] = grow[x as usize];  }
            } // Growths
        }
        person_x.on_complete();
    }
}

pub fn replace_enemy_version() {
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 || !GameVariableManager::get_bool("G_Random_Skills") { return; } 
    for x in 0..41 {
        let person = PersonData::get(PIDS[x]).unwrap();
        replace_all_sid_person(person);
    }
}
pub fn randomize_skills() {
    if !GameVariableManager::get_bool("G_Random_Skills") || !crate::utils::can_rand() { return; }
    println!("Randomizing skills");
    let skill_list = SkillData::get_list().unwrap();
    let rng = Random::instantiate().unwrap();
    let seed = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_pool_count = SKILL_POOL.lock().unwrap().len() as i32;
    // Playables
    let playable_size = PLAYABLE.lock().unwrap().len();
    let person_list = PersonData::get_list().unwrap();
    for x in 0..playable_size {
        let p_index = PLAYABLE.lock().unwrap()[x as usize] as usize;
        let person = &person_list[p_index]; 
        let personal_sid = person.get_common_sids().
            expect(
                format!("Person #{}: {} doesn't have skills!", person.parent.index, person.get_name().unwrap().get_string().unwrap()).as_str()
            );

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
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
        let mut skill_index2;
        loop {
            skill_index2 = rng.get_value(skill_pool_count) as usize;
            if skill_list[ SKILL_POOL.lock().unwrap()[ skill_index2 ].index as usize ].can_override_skill() { continue; }
            if skill_index2 == skill_index { continue; }
            else {break; }
        }
        job.set_learning_skill( skill_list[ SKILL_POOL.lock().unwrap()[ skill_index  ].index as usize ].sid ); 
        job.set_lunatic_skill(  skill_list[ SKILL_POOL.lock().unwrap()[ skill_index2 ].index as usize ].sid ); 
    }
    randomize_bond_ring_skills();

}
pub fn randomize_bond_ring_skills(){
    let maddening_pool_size = MADDENING_POOL.lock().unwrap().len() as i32;
    let ring_list = RingData::get_list_mut().unwrap();
    let skill_list = SkillData::get_list().unwrap();
    let ranks = [3, 2, 1, 0]; 
    let ranks_rate: [i32; 4] = CONFIG.lock().unwrap().get_bond_ring_rates();
    let rng_rings = crate::utils::get_rng();
    for y in 0..4 {
        let current_rank = ranks[y as usize];
        let odds = ranks_rate[y as usize];
        if odds == 0 { continue; }
        let mut ring_skill_set: [bool; 1000] = [false; 1000];
        for x in 0..ring_list.len() {
            if ring_list[x].rank != current_rank { continue; }
            let mut skill_odds = odds;
            if skill_odds < rng_rings.get_value(100)  { continue; }
            let equip_skills = ring_list[x].get_equip_skills();
            let mut index = rng_rings.get_value(maddening_pool_size) as usize;
            let mut count = 0;
            while ring_skill_set[ index ] && count < 50 { 
                index = rng_rings.get_value(maddening_pool_size) as usize; 
                count += 1;
            }
            let s_index = MADDENING_POOL.lock().unwrap()[index];
            let s_high = get_highest_priority(s_index);
            equip_skills.clear();
            equip_skills.add_skill(&skill_list[s_high as usize],6, 0);
            ring_skill_set[ index ] = true;
            if y < 3 {
                skill_odds = skill_odds / (3 - y) + (y+3)*5;
                if skill_odds < rng_rings.get_value(100) { continue; }
                let mut index2 = rng_rings.get_value(maddening_pool_size) as usize;
                let mut s_high2 = get_highest_priority(MADDENING_POOL.lock().unwrap()[index2]);
                count = 0;
                while ring_skill_set[ index2 ] || s_high == s_high2  { 
                    index2 = rng_rings.get_value(maddening_pool_size) as usize; 
                    s_high2 = get_highest_priority(MADDENING_POOL.lock().unwrap()[index2]);
                    count += 1;
                    if count > 50 &&  s_high != s_high2 { break; }
                }
                equip_skills.add_skill(&skill_list[s_high2 as usize],6, 0);
                ring_skill_set[ index2 ] = true;
            }
        }
    }
}


