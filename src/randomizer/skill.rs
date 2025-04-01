use unity::{engine::Sprite, prelude::*};
use super::{person::{*, PLAYABLE}, emblem::emblem_skill::*};
use engage::{
    dialog::yesno::*, 
    gamedata::ring::RingData, 
    gameicon::GameIcon, 
    gameuserdata::GameUserData, 
    menu::{config::{ConfigBasicMenuItem, *}, BasicMenuResult}, 
    sortie::SortieSelectionUnitManager,
    tmpro::TextMeshProUGUI
};

use std::sync::Mutex;
use super::{DVCVariables, CONFIG};
use crate::{enums::*, utils::*};

pub mod learn;
pub mod menu;

pub static SKILL_POOL: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static MADDENING_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());


pub struct SkillIndex {
    pub index: i32,
    pub in_use: bool,
    pub linked_use: bool,
}
impl SkillIndex { pub fn new(value: i32) -> Self { Self { index: value, in_use: false, linked_use: false, }} }

// Skill randomization and Emblem skills/stat boost randomization upon msbt loading

fn add_skill_to_pool(skill: &SkillData) {
    let index = skill.parent.index;
    let sid = skill.sid.to_string();
    if sid.contains("E00") || sid.contains("G00") { return; }
    if skill.help.is_none() ||  skill.name.is_none() || skill.is_style_skill() || SKILL_BLACK_LIST.lock().unwrap().iter().any(|&y| y == index ) { return; }
    if Mess::get(skill.name.unwrap()).to_string().len() < 1 || Mess::get(skill.help.unwrap()).to_string().len() < 1 { return; }
    if skill.get_inheritance_cost() != 0 && skill.get_inheritance_sort() != 0 { SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, false); }
    if skill.get_flag() & 511 == 0 {
        SKILL_POOL.lock().unwrap().push(SkillIndex::new(index as i32));
        SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, false); // Chaos Skills
        if MADDENING_BLACK_LIST.iter().find(|lol| **lol == skill.sid.to_string() ).is_none() { MADDENING_POOL.lock().unwrap().push(index);  }
    }
}
pub fn create_skill_pool() {
    let skill_list = SkillData::get_list_mut().unwrap();
    fix_priority_data();
    if SKILL_POOL.lock().unwrap().len() != 0 { return; }
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_計略", true, true); 
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_無し", true, true);
    skill_list.iter().for_each(|skill| add_skill_to_pool(skill));

    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_超越_闇", true, false); 
    ENGAGE_SKILLS_CHAOS.get_or_init(|| MADDENING_POOL.lock().unwrap().iter().map(|x| *x).collect());

    println!("Total Maddening Skills in Pool: {}",  MADDENING_POOL.lock().unwrap().len());
    println!("Total Skills in Pool: {}", SKILL_POOL.lock().unwrap().len());
    create_emblem_skill_pool();
}

pub fn fix_priority_data() {
    let skill_list = SkillData::get_list_mut().unwrap();
    for x in 1..skill_list.len() {
        let skill = &skill_list[x]; // fix bad inheritance data
        if skill.parent.index == 382 { continue; }
        let current_priority = skill.get_priority() % 100;
        let previous_priority = skill_list[x - 1].get_priority() % 100;
        if current_priority < 2 { continue; }
        if current_priority > 1 && previous_priority != current_priority - 1 { skill.set_priority(0); }
    }
}

fn get_highest_priority(index: i32) -> i32 {
    let skill_list = SkillData::get_list().unwrap();
    let skill = &skill_list[ index as usize]; 
    let priority = skill.get_priority();
    if priority == 0 || priority > 9 { return index; }
    let sid = skill.sid.to_string(); 
    if index == SkillData::get_count() - 1 { return index; }
    //Eirika Skills
    if let Some(pos) =  EIRIKA_TWIN_SKILLS.iter().position(|&str| str == sid ) {
        if pos < 6 { return SkillData::get( EIRIKA_TWIN_SKILLS[ 6 + pos ]).unwrap().parent.index; }
        else { return SkillData::get( EIRIKA_TWIN_SKILLS[pos]).unwrap().parent.index; }
    }
    let mut new_index = index+1;
    let mut current_priority = priority;

    loop {
        if new_index == SkillData::get_count() { return new_index-1;  }
        let new_skill = &skill_list[new_index as usize];
        if current_priority < new_skill.get_priority() {
            current_priority = new_skill.get_priority();
            new_index += 1;
        }
        else { return new_index - 1;  }
    }
}

pub fn get_random_skill_job(difficulty: i32, rng: &Random, unit: &Unit) -> Option<&'static SkillData> {
    let restrict_list = learn::JOB_RESTRICT_SKILLS_LIST.get().unwrap();
    let job_mask = unit.selected_weapon_mask.value | unit.job.get_weapon_mask().value;
    let mut count = 0;
    while count < 50 {
        let skill = get_random_skill(difficulty, rng);
        if restrict_list.is_valid_for_weapon_mask(skill, job_mask) && !crate::randomizer::person::unit::has_skill(unit, skill) { return Some(skill); }
        count+= 1;
    }
    None
}

pub fn get_random_skill_dispos(difficulty: i32, rng: &Random) -> &'static SkillData {
    let restrict_list = learn::JOB_RESTRICT_SKILLS_LIST.get().unwrap();
    loop {
        let skill = get_random_skill(difficulty, rng);
        if !restrict_list.list.iter().any(|x| x.hash == skill.parent.hash) { return skill; }
    }
}

pub fn get_random_skill(difficulty: i32, rng: &Random) -> &'static SkillData {
    let skill_pool_size;
    let mut skill_index;
    if difficulty == 2 {
        skill_pool_size = MADDENING_POOL.lock().unwrap().len();
        skill_index = MADDENING_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize];
        if DVCVariables::is_main_chapter_complete(22) { skill_index = get_highest_priority(skill_index); }
    }
    else {
        skill_pool_size = SKILL_POOL.lock().unwrap().len();
        skill_index =  SKILL_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize].index;
    }
    return SkillData::try_index_get(skill_index).unwrap();
}


pub fn reset_skills() {
    println!("Resetting skills to normal");
    SKILL_POOL.lock().unwrap().iter_mut().for_each(|x|x.in_use = false);
    reset_emblem_skills();
}

pub fn replace_all_sid_person(person_index: i32) {
    let enemy_list: Vec<_> = ENEMY_PERSONS.get().unwrap().iter().filter(|x| x.0 == person_index).collect();
    if enemy_list.len() == 0 { return; }
    let person = PersonData::get(PIDS[person_index as usize]).unwrap();
    let new_person = switch_person(person);
    let old_job = person.get_job().expect(
        format!("Person #{}: {} does not have a valid class.\nPlease change the JID of Person #{} in Person.xml.", person.parent.index, Mess::get_name(person.pid), person.parent.index).as_str()
    );
    let new_job = new_person.get_job().expect( 
        format!("Person #{}: {} does not have a valid class.\nPlease change the JID of Person #{} in Person.xml.", new_person.parent.index, Mess::get_name(new_person.pid), new_person.parent.index).as_str()
    );
    let jid = 
        if old_job.is_high() && ( new_job.is_high() || (new_job.is_low() && new_job.max_level == 40 ) ) { new_job.jid }
        else if old_job.is_high() && ( new_job.is_low() && new_job.has_high() ) { new_job.get_high_jobs()[0].jid }
        else if old_job.is_low() &&  new_job.is_high() {
            if new_job.get_low_jobs().len() == 0 { "JID_ソードファイター".into() }
            else { new_job.get_low_jobs()[0].jid }
        }
        else { new_job.jid }; 

    let new_flag_value = if new_person.pid.to_string() == PIDS[0] { new_person.get_flag().value | 1536  }  
        else { new_person.get_flag().value | 512 };

    let grow = new_person.get_grow();
    let personal_sids = new_person.get_common_sids().unwrap();
    let new_skill_index = if GameVariableManager::get_number(&format!("G_P_{}", new_person.pid.to_string())) != 0 {
        GameVariableManager::get_number(&format!("G_P_{}", new_person.pid))
    }
    else {
        SkillData::get(personal_sids.iter().find(|&sid|SkillData::get(sid.to_string()).unwrap().get_flag() & 1 == 0 ).unwrap().to_string()).unwrap().parent.hash
    };
    let valid = new_person.get_unit_icon_id().is_some();
    let icon_name = if valid { new_person.get_unit_icon_id().unwrap().to_string() } else { "".to_string() };
    let help_name = new_person.get_help().to_string();

    enemy_list.iter().for_each(|enemy|{
        if let Some(person_x) = PersonData::try_index_get_mut(enemy.1){
            // println!("Found Enemy Person #{}: {}", enemy.1, Mess::get_name(person_x.pid));
            change_personal_sid(person_x, SkillData::try_get_hash(new_skill_index).unwrap());
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
            person_x.set_name(new_person.get_name().unwrap());
            person_x.get_flag().value = new_flag_value;
            let grow_x = person_x.get_grow();
            person_x.set_jid(jid);
            person_x.set_gender(new_person.get_gender());
            person_x.set_attrs(new_person.get_attrs());
            if valid { person_x.set_unit_icon_id(icon_name.clone().into()); }
            person_x.set_help(help_name.clone().into());
            if !grow_x.is_zero() {
                for x in 0..11 { 
                    if grow_x[x as usize] == 0 { continue; }
                    if grow_x[x as usize] < grow[x as usize] {  grow_x[x as usize] = grow[x as usize];  }
                } // Growths
            }
        }
    } );
}

pub fn replace_enemy_version() {
    if !can_rand() { return; } 
    if GameUserData::get_chapter().cid.to_string().contains("G00") { return; }
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().skill_randomized { return; }
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().enemy_unit_randomized {
        println!("Replacing Enemy Character's Skills");
        [4, 7, 11, 14, 17, 18, 19, 20, 23, 26, 28, 29, 33, 36, 37, 38, 39, 40].iter().for_each(|&x| replace_all_sid_person(x));
        crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.enemy_unit_randomized = true).unwrap();
        if GameUserData::get_sequence() == 2 || GameUserData::get_sequence() == 3 { super::person::unit::reload_all_actors(); }
    }
}
pub fn randomize_skills() {
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().skill_randomized { return; }
    if !GameVariableManager::get_bool(DVCVariables::SKILL_KEY) || !DVCVariables::random_enabled() {
        crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.skill_randomized = true).unwrap();
        return; 
    }
    println!("Randomizing skills");
    let skill_list = SkillData::get_list().unwrap();
    let rng = Random::instantiate().unwrap();
    let seed = 2*DVCVariables::get_seed();
    rng.ctor(seed as u32);

    let mut playable = PLAYABLE.get().unwrap().clone();
    let mut skill_pool = SKILL_POOL.lock().unwrap();
    let person_b_list = PERSONAL_BLIST.lock().unwrap();
    // convert indexes to hash
    let mut need_personals: Vec<i32> = Vec::new();

    playable.iter()
        .flat_map(|&index| PersonData::try_index_get_mut(index))
        .for_each(|person|{
            let personal_key = format!("G_P_{}", person.pid);
            let mut personal_set = false;
            if GameVariableManager::exist(&personal_key) {
                let hash = GameVariableManager::get_number(&personal_key);
                if let Some(skill) = SkillData::try_get_hash(hash) { 
                    if let Some(s) = skill_pool.iter_mut().find(|s| s.index == skill.parent.index) {
                        s.in_use = true;
                        change_personal_sid(person, skill);
                        personal_set = true;
                    }
                }
            }
            else {  GameVariableManager::make_entry(&personal_key, 0); }
            if !personal_set { need_personals.push(person.parent.index); }
        }
    );
    if need_personals.len() > 0 {
        println!("{} need personals", need_personals.len());
        need_personals.iter()
            .flat_map(|&index| PersonData::try_index_get_mut(index))
            .for_each(|person| {
                let mut available: Vec<&mut SkillIndex> = skill_pool.iter_mut().filter(|a| 
                    !a.in_use && 
                    !skill_list[a.index as usize].can_override_skill() &&
                    !skill_list[a.index as usize].is_style_skill() && 
                    !person_b_list.iter().any(|&y| y == a.index)
                ).collect();
                let len = available.len() as i32;
                if len == 0 { panic!("No Available Skills to give as personals to {}",  Mess::get_name(person.pid).to_string()); }
                let new_skill_index = &mut available[ rng.get_value(len) as usize];
                let skill = SkillData::try_index_get(new_skill_index.index).expect("Attempting to assign personal skill: Bad Skill? :O");
                println!("Person Skill Changed: {}: {}", Mess::get_name(person.pid), Mess::get(skill.name.unwrap()));
                change_personal_sid(person, skill);
                new_skill_index.in_use = true;
                GameVariableManager::set_number(&format!("G_P_{}", person.pid), skill.parent.hash);
            }
        );
    }
    // the rest 
    let restriction_list = learn::JOB_RESTRICT_SKILLS_LIST.get().unwrap();
    let job_list = JobData::get_list_mut().unwrap();
    let mut jobs: Vec<i32> = Vec::new();
    job_list.iter()
        .filter(|job|{ job.is_high() || ( job.is_low() && job.max_level == 40 ) })
        .for_each(|job|{
            let mut weapon_mask = 0;
            for x in 1..9 { if job.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }
            let mut learn_skill_set = false;
            let index = job.parent.index;
            let job_key = format!("G_L_{}", job.jid);
            if GameVariableManager::exist(job_key.as_str()) {
                if let Some(skill) = SkillData::try_get_hash( GameVariableManager::get_number(job_key.as_str()) ) {
                    if restriction_list.is_valid_for_weapon_mask(skill, weapon_mask) {
                        if let Some(skill_index1) = skill_pool.iter_mut().find(|x| x.index == skill.parent.index) {
                            skill_index1.in_use = true;
                            learn_skill_set = true;
                            job.set_learning_skill( skill.sid );
                        }
                    }
                }
            }
            else {  GameVariableManager::make_entry(job_key.as_str(), 0);   }
            if !learn_skill_set { jobs.push(index);  }
        }
    );
    jobs.iter().for_each(|&job_index|{
        let mut available: Vec<&mut SkillIndex> = skill_pool.iter_mut().filter(|a| !a.in_use && !skill_list[a.index as usize].can_override_skill() ).collect();
        if let Some(job) = JobData::try_index_get_mut(job_index){
            let len = available.len() as i32;
            let mut weapon_mask = 0;
            for x in 1..9 { if job.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }
            if len > 0 {
                let mut count = 0;
                let mut new_skill_index = &mut available[ rng.get_value(len) as usize];
                let mut skill = SkillData::try_index_get(new_skill_index.index).expect("Attempting to assign job learn skill: Bad Skill?");
                loop {  // Keep randomizing if possible to prevent non-tome/non-staff jobs from getting tome/staff exclusive skills
                    if restriction_list.is_valid_for_weapon_mask(skill, weapon_mask) || count >= 50 { break; }
                    count += 1;
                    new_skill_index = &mut available[ rng.get_value(len) as usize];
                    skill = SkillData::try_index_get(new_skill_index.index).expect("Attempting to assign job learn skill: Bad Skill? :O");
                }
                job.set_learning_skill( skill.sid );
                new_skill_index.in_use = true;
                GameVariableManager::set_number(format!("G_L_{}", job.jid), skill.parent.hash);
            }
        }
    });   
    super::emblem::emblem_skill::change_weapon_restrict("SID_全弾発射", 1023); 
    randomize_bond_ring_skills();
    crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.skill_randomized = true).unwrap();
}

pub fn randomize_bond_ring_skills(){
    let maddening_pool_size = MADDENING_POOL.lock().unwrap().len() as i32;
    let ring_list = RingData::get_list_mut().unwrap();
    let ranks = [3, 2, 1, 0]; 
    let ranks_rate: [i32; 4] = CONFIG.lock().unwrap().get_bond_ring_rates();
    let rng_rings = crate::utils::get_rng();
    ring_list.iter_mut().for_each(|ring| { ring.get_equip_skills().clear(); } );
    for y in 0..4 {
        let current_rank = ranks[y as usize];
        let odds = ranks_rate[y as usize];
        if odds == 0 { continue; }
        let mut pool = MADDENING_POOL.lock().unwrap().clone();
        ring_list.iter_mut()
            .filter(|ring| ring.rank == current_rank && rng_rings.get_value(100) < odds )
            .for_each(|ring|{
                let equip_skills = ring.get_equip_skills();
                let mut skill_count = 0;
                let mut skill_odds = odds;
                while rng_rings.get_value(100) < skill_odds  && skill_count < 4 {
                    if let Some(skill) = crate::utils::get_random_and_remove(&mut pool, rng_rings).and_then(|i| SkillData::try_index_get(get_highest_priority(i))) {
                        equip_skills.add_skill(skill, 6, 0);
                        skill_count += 1;
                    }
                    else { break; }  // no more skills
                    skill_odds = 1/ ( 1 + skill_count + y)* skill_odds + 5*current_rank;
                }
            }
        );
    }
}

fn change_personal_sid(person: &mut PersonData, skill: &SkillData) {
    let personal_sid = person.get_common_sids().expect( format!("Expected Person #{}: {} to have commond sids!", person.parent.index, Mess::get_name(person.pid)).as_str() );

    if let Some(sid) = personal_sid.iter_mut().find(|sid| SkillData::get(sid.to_string()).is_some_and(|s| s.flag & 1 == 0)){
        person.get_common_skills().replace_sid(sid, skill);
        person.get_normal_skills().replace_sid(sid, skill);
        person.get_hard_skills().replace_sid(sid, skill);
        person.get_lunatic_skills().replace_sid(sid, skill);
        *sid = skill.sid;
    }
} 

pub fn fixed_skill_inherits() {
    let skill_list = SkillData::get_list_mut().unwrap();
    skill_list.iter_mut()
        .filter(|skill| skill.get_inheritance_cost() > 0 && ( skill.get_flag() & 63 != 0 || NO_INHERITS.iter().any(|sid| str_contains(skill.sid, sid)) ))
        .for_each(|inherit| { inherit.set_inherit_cost(0);});
}

