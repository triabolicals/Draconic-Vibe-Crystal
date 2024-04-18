use unity::prelude::*;
use engage::{
    menu::{*, BasicMenuResult, config::{ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserglobaldata::*,
    gameuserdata::*,
    proc::ProcInst,
    hub::access::*,
    mess::*,
    gamedata::{*, skill::*, god::*, dispos::*},
    pad::Pad,
    util::get_instance,
};
use std::{fs, fs::File, io::Write};
use crate::{deploy, person, emblem, item, skill, grow, ironman, utils::*};
use super::{VERSION, CONFIG, DeploymentConfig};

pub static mut CURRENT_SEED: i32 = -1;
pub fn write_seed_output_file() {
    let seed = GameVariableManager::get_number("G_Random_Seed");
    let _ = fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    let filename = format!("sd:/Draconic Vibe Crystal/Slot {} - {}.log", GameUserGlobalData::get_last_save_data_index(), get_player_name());
    let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
    writeln!(&mut f, "------------- Triabolical Randomization Settings - Version {} -------------", VERSION).unwrap();
    writeln!(&mut f, "* Seed: {}", seed).unwrap();
    writeln!(&mut f, "* Random Recruitment: {}", GameVariableManager::get_bool("G_Random_Recruitment")).unwrap();
    let emblem_mode =  GameVariableManager::get_number("G_Emblem_Mode");
    if emblem_mode == 0 { writeln!(&mut f, "* Emblem Recruitment Mode: No Randomization").unwrap();  }
    else if emblem_mode == 1 { writeln!(&mut f, "* Emblem Recruitment Mode: Random").unwrap();  }
    else if emblem_mode == 2 { writeln!(&mut f, "* Emblem Recruitment Mode: Reverse").unwrap(); }
    let god_mode =  GameVariableManager::get_number("G_Random_God_Mode");
    if god_mode == 0 { writeln!(&mut f, "* Emblem Data: No Randomization").unwrap();  }
    else if god_mode == 1 { writeln!(&mut f, "* Emblem Data: Engraves / Inheritable Skills").unwrap();  }
    else if god_mode == 2 { writeln!(&mut f, "* Emblem Data: Engage Attack / Engage Link").unwrap(); }
    else if god_mode == 3 { writeln!(&mut f, "* Emblem Data: Engraves / Inheritable / Attack / Link").unwrap(); }
    writeln!(&mut f, "* Random Classes: {}", GameVariableManager::get_bool("G_Random_Job")).unwrap();
    writeln!(&mut f, "* Random Skills: {}", GameVariableManager::get_bool("G_Random_Skills")).unwrap();
    writeln!(&mut f, "* Random Items: {}", GameVariableManager::get_bool("G_Random_Item")).unwrap();
    let growth_mode = GameVariableManager::get_number("G_Random_Grow_Mode");
    match growth_mode {
        1 => { writeln!(&mut f, "* Growth Rate Mode: Personal").unwrap(); },
        2 => { writeln!(&mut f, "* Growth Rate Mode: Class Mods").unwrap(); },
        3 => { writeln!(&mut f, "* Growth Rate Mode: Personal + Class Mods").unwrap(); },
        _ => { writeln!(&mut f, "* Growth Rate Mode: No Randomization").unwrap(); },
    }
    let sync_mode = GameVariableManager::get_number("G_Random_God_Sync");
    match sync_mode {
        1 => { writeln!(&mut f, "* Emblem Sync Data: Stat Bonuses").unwrap(); },
        2 => { writeln!(&mut f, "* Emblem Sync Data: Sync/Engage Skills").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
    }
    if GameVariableManager::get_bool("G_Random_Recruitment") {
        writeln!(&mut f, "\n--------------- Person Recruitment Order Randomization ---------------").unwrap();
        let mut count = 0;
        for x in person::PIDS{
            let string = format!("G_R_{}", x);
            let name1 = Mess::get( PersonData::get(x).unwrap().get_name().unwrap() ).get_string().unwrap();
            let new_pid = GameVariableManager::get_string(&string);
            let mut name2 = String::new();
            if PersonData::get(&new_pid.get_string().unwrap()).is_some() {
                name2 = Mess::get( PersonData::get(&new_pid.get_string().unwrap()).unwrap().get_name().unwrap() ).get_string().unwrap();
            } 
            count += 1;
            writeln!(&mut f, "* {} - {} ({}) -> {} ({})", count, name1, x, name2, new_pid.get_string().unwrap()).unwrap();
        }
    }
    if emblem_mode != 0 {
        writeln!(&mut f, "\n-------------- Emblems Recruitment Order Randomization ---------------").unwrap();
        let mut count = 0;
        for x in deploy::EMBLEM_GIDS { 
            let string = format!("G_R_{}", x);
            let name1 = Mess::get( GodData::get(x).unwrap().mid ).get_string().unwrap();
            let new_gid = GameVariableManager::get_string(&string);
            let mut name2 = String::new();
            if GodData::get(&new_gid.get_string().unwrap()).is_some() {
                name2 = Mess::get( GodData::get(&new_gid.get_string().unwrap()).unwrap().mid).get_string().unwrap();
            }
            count += 1;
            writeln!(&mut f, "* {} - {} ({}) -> {} ({})", count, name1, x, name2, new_gid.get_string().unwrap()).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_Grow_Mode") == 1 || GameVariableManager::get_number("G_Random_Grow_Mode") == 3 {
        writeln!(&mut f, "\n--------------- Personal Growth Rates Randomization ---------------").unwrap();
        let person_list = PersonData::get_list().unwrap();
        for x in 0..person_list.len() {
            let grow = person_list[x].get_grow();
            if grow.is_zero() { continue; } 
            let line = get_person_growth_line(person_list[x]);
            writeln!(&mut f, "* {} - {}", x+1, line).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_Grow_Mode") >= 2 {
        let job_list = JobData::get_list_mut().unwrap();
        writeln!(&mut f, "\n--------------- Class Growth Rates Modifers Randomization ---------------").unwrap();
        for x in 0..job_list.len() {
            let grow = job_list[x].get_diff_grow();
            if grow.is_zero() { continue; } 
            let jid = job_list[x].jid.get_string().unwrap();
            let job_name = Mess::get(job_list[x].name).get_string().unwrap();
            writeln!(&mut f, "* {} - {} ({})\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |", x+1, job_name, jid, 
            Mess::get("MID_SYS_HP").get_string().unwrap(), grow[0], Mess::get("MID_SYS_Str").get_string().unwrap(), grow[1], Mess::get("MID_SYS_Mag").get_string().unwrap(), grow[6], 
            Mess::get("MID_SYS_Tec").get_string().unwrap(), grow[2], Mess::get("MID_SYS_Spd").get_string().unwrap(), grow[3], Mess::get("MID_SYS_Lck").get_string().unwrap(), grow[4],
            Mess::get("MID_SYS_Def").get_string().unwrap(), grow[5], Mess::get("MID_SYS_Res").get_string().unwrap(), grow[7], Mess::get("MID_SYS_Phy").get_string().unwrap(), grow[8]).unwrap();
        }
    }
    if GameVariableManager::get_bool("G_Random_Skills") {
        writeln!(&mut f, "\n--------------- Personal Skills Randomization ---------------").unwrap();
        for x in person::PIDS {
            let person = PersonData::get(x).unwrap();
            let name = Mess::get(person.get_name().unwrap()).get_string().unwrap();
            let personal_sid = person.get_common_sids().unwrap();
            for y in 0..personal_sid.len() {
                let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
                let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
                if skill.get_flag() & 1 == 0 {
                    let skill_name = Mess::get( SkillData::get(&personal_sid[y as usize].get_string().unwrap()).unwrap().name.unwrap() ).get_string().unwrap();
                    let sid = SkillData::get(&personal_sid[y as usize].get_string().unwrap()).unwrap().sid.get_string().unwrap();
                    writeln!(&mut f, "* {} ({}):\t{} ({})",  name, x, skill_name, sid).unwrap();
                    break;
                }
            }
        }
        writeln!(&mut f, "\n--------------- Class Learn Skill / Lunatic Skill Randomization --------------").unwrap();
        let job_list = JobData::get_list_mut().unwrap();
        for x in 0..job_list.len() {
            let job = &job_list[x as usize];
            let job_name = Mess::get(job.name).get_string().unwrap();
            let mut string = " ------  ".into();
            let mut string2 = "  ------ ".into();
            if job.learn_skill.is_some() {
                let skill_name = SkillData::get(&job.learn_skill.unwrap().get_string().unwrap()).unwrap().name.unwrap();
                string = format!("{} ({})", Mess::get( skill_name ).get_string().unwrap(), job.learn_skill.unwrap().get_string().unwrap());
            }   
            if job.lunatic_skill.is_some() {
                let skill_name = SkillData::get(&job.lunatic_skill.unwrap().get_string().unwrap()).unwrap().name.unwrap();
                string2 = format!("{} ({})", Mess::get(skill_name ).get_string().unwrap(), job.lunatic_skill.unwrap().get_string().unwrap());
            }
            if job.learn_skill.is_none() && job.lunatic_skill.is_none() { continue;}
            else { 
                let jid = job.jid.get_string().unwrap();
                writeln!(&mut f, "* {} - {} ({}):\t {} / {}", x, job_name, jid, string, string2).unwrap();
            }
        }
        let n_skills = skill::SKILL_POOL.lock().unwrap().len();
        let skill_list = SkillData::get_list().unwrap();
        let ring_list = RingData::get_list().unwrap();
        writeln!(&mut f, "\n--------------- Bond Ring Randomization --------------").unwrap();
        for x in 0..ring_list.len() {
            if ring_list[x].rank != 3 { continue; }

            let skills = skill_array_string(ring_list[x].get_equip_skills());
            let name = Mess::get(ring_list[x].name).get_string().unwrap();
            if ring_list[x].gid.is_some() {
                writeln!(&mut f, "* {}: {} - {}", get_emblem_name(&ring_list[x].gid.unwrap().get_string().unwrap()), name, skills).unwrap();
            }
            else {
                writeln!(&mut f, "* {} - {}", name, skills).unwrap();
            }
        }
        writeln!(&mut f, "\n--------------- Randomization Skill Pool Availiablity ({} skills)  ---------------", n_skills).unwrap();
        for x in 0..n_skills {
            let skill_index = skill::SKILL_POOL.lock().unwrap()[x as usize].index as usize;
            let skill = &skill_list[skill_index ];
            let sid = skill.sid.get_string().unwrap();
            let name = Mess::get(skill_list[skill_index ].name.unwrap()).get_string().unwrap();
            let mut personal = "X / X";
            if skill.get_inheritance_cost() != 0 {
                if skill.can_override_skill() { personal = "Enemy"; }
                else { personal = "Class | Enemy"; }
            }
            else {
                if skill.can_override_skill() { personal = "Personal | Enemy"; }
                else { personal = "Personal | Class | Enemy"; }
            }
            writeln!(&mut f, "* {} ({})\t {}", name, sid, personal).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_God_Mode") >= 2 {
        writeln!(&mut f, "\n--------------- Emblem Engage / Linked Engage Attack Randomization ---------------").unwrap();
        for x in deploy::EMBLEM_GIDS { 
            let line = god_engage_random_str(x);
            writeln!(&mut f, "{}", line).unwrap();
        }
    }
    writeln!(&mut f, "\n--------------- Emblem Engrave Data ---------------").unwrap();
    for x in deploy::EMBLEM_GIDS { 
        let god = GodData::get(*x).unwrap();
        let line = format!("* {} - \t{}: {}, {}: {}, {}: {}, {}: {}, {}: {}, {}: {}", 
        mess_get(god.mid), 
        get_stat_label(11), god.get_engrave_avoid(),  get_stat_label(12), god.get_engrave_critical(), get_stat_label(13), god.get_engrave_hit(), 
        get_stat_label(14), god.get_engrave_power(), get_stat_label(15), god.get_engrave_secure(), get_stat_label(16), god.get_engrave_weight());

        writeln!(&mut f, "{}", line).unwrap();
    }

    writeln!(&mut f, "\n--------------- Emblem Sync / Engage Data --------------").unwrap();
    match god_mode {
        1 => { writeln!(&mut f, "* Emblem Data: Engraves / Inheritable Skills").unwrap();  }
        2 => { writeln!(&mut f, "* Emblem Data: Engage Attack / Engage Link").unwrap(); }
        3 => { writeln!(&mut f, "* Emblem Data: Engraves / Inheritable / Engage Attack / Engage Link").unwrap(); }
        _ => { writeln!(&mut f, "* Emblem Data: No Randomization").unwrap();  }
    }
    match sync_mode {
        1 => { writeln!(&mut f, "* Emblem Sync Data: Stat Bonuses").unwrap(); },
        2 => { writeln!(&mut f, "* Emblem Sync Data: Sync/Engage Skills").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
    }
    for x in skill::EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        let god_id = format!("GID_{}", x);
        let engage_skill = level_data[0].engage_skills.list.item[0].get_skill().unwrap();
        let god = GodData::get(*x).unwrap(); 
        let god_grow = GodGrowthData::try_get_from_god_data(god).unwrap();
        writeln!(&mut f, "\n*** {} Engage Skill: {}, Engage Atk/Link: {}\n", get_emblem_name(&god_id), get_skill_name(engage_skill), god_engage_random_str(&god_id)).unwrap();
        for y in 1..level_data.len() {
            writeln!(&mut f, "\t* {} Lv. {} Stats: {}", get_emblem_name(&god_id), y, stats_from_skill_array(level_data[y as usize].synchro_skills)).unwrap();
            writeln!(&mut f, "\t* Weapon: {}", level_data[y as usize].aptitude.value);
            writeln!(&mut f, "\t\tSyncho Skills:  {}", skill_array_string(level_data[y as usize].synchro_skills)).unwrap();
            writeln!(&mut f, "\t\tEngaged Skills: {}", skill_array_string(level_data[y as usize].engaged_skills)).unwrap();
            if y-1 < god_grow.len() {
                let level = god_grow[y-1].get_inheritance_skills();
                if level.is_none() { writeln!(&mut f, "").unwrap(); continue;}
                let inherit_skills = level.unwrap();
                writeln!(&mut f, "\t\tInherit Skills: {}", sid_array_string(inherit_skills)).unwrap();
            }
            writeln!(&mut f, "").unwrap();
        }
    }
    println!("Randomization Print to file");
}

 // Hooks 
 #[skyline::hook(offset=0x021a3310)]
 pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    if GameUserData::get_sequence() == 6 { emblem::emblem_gmap_spot_adjust(); }
    let result = call_original!(dyn_value, method_info);
    if result.is_none() { return result; }
    let result_string = result.unwrap();
    unsafe {
        if string_start_with(result_string, "GID_".into(), None) {
            if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
            let gid = result_string.get_string().unwrap();
            let string = format!("G_R_{}", gid);
            let new_gid = GameVariableManager::get_string(&string);
            if !is_null_empty(new_gid, None) { return Some(new_gid); }
        }
        else if string_start_with(result_string, "PID_".into(), None) {
            if !GameVariableManager::get_bool("G_Random_Recruitment") { return result; }
            if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022" && result_string.get_string().unwrap() != "PID_ヴェイル" { return result; }
            let pid = result_string.get_string().unwrap();
            let string = format!("G_R_{}", pid);
            let new_pid = GameVariableManager::get_string(&string);
            if !is_null_empty(new_pid, None) { return Some(new_pid);  }
        }
        else if string_start_with(result_string, "IID_".into(), None){
            if GameVariableManager::get_number("G_Random_Item") == 0 || GameVariableManager::get_number("G_Random_Item") == 2  { return result; }
            else { return Some(item::get_random_item(result.unwrap())); }
        }
        else if string_start_with(result_string, "TUTID_紋章士".into(), None){
            if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
            let key = replace_str(result_string, "TUTID_紋章士".into(), "G_R_GID_".into(), None);
            let new_gid = GameVariableManager::get_string(&key.get_string().unwrap());
            let new_tut = replace_str(new_gid, "GID_".into(), "TUTID_紋章士".into(), None);
            return Some(new_tut);
        }
    }
    return result;
}
// Switching PersonData indexes in scripts for event function calls
#[skyline::hook(offset=0x01cb5eb0)]
pub fn try_get_index(dyn_value: u64, index: i32, method_info: OptionalMethod) -> i32 {
    let result = call_original!(dyn_value, index, method_info);
    if !GameVariableManager::get_bool("G_Random_Recruitment")  { return result; }
    let person_list = PersonData::get_list().unwrap();
    let person_count = PersonData::get_count();
    if result  < 0 || result >= person_count {  return result; }
    let person = &person_list[ result  as usize ];
    if !person::is_player_unit(person){ return result; }
    let new_person = person::switch_person(person);
    let new_index = PersonData::get_index( new_person.pid );
    println!("New Index from {} -> {}", result , new_index);
    return new_index;
}

pub fn start_new_game(){
    *CONFIG.lock().unwrap() = DeploymentConfig::new();
    CONFIG.lock().unwrap().correct_rates();
    let seed = CONFIG.lock().unwrap().seed;
    if seed == 0 {  GameVariableManager::make_entry("G_Random_Seed", get_random_number_for_seed() as i32); }
    else { GameVariableManager::make_entry("G_Random_Seed", CONFIG.lock().unwrap().seed as i32); }
    GameVariableManager::make_entry("G_Emblem_Mode", CONFIG.lock().unwrap().emblem_mode as i32);
    GameVariableManager::make_entry("G_Random_Recruitment", CONFIG.lock().unwrap().random_recruitment as i32);
    GameVariableManager::make_entry("G_Random_Job", CONFIG.lock().unwrap().random_job as i32);
    GameVariableManager::make_entry("G_Lueur_Random", 0);
    GameVariableManager::make_entry("G_Random_Skills", CONFIG.lock().unwrap().random_skill as i32);
    GameVariableManager::make_entry("G_Random_Grow_Mode", CONFIG.lock().unwrap().random_grow as i32);
    GameVariableManager::make_entry("G_Random_God_Mode",  CONFIG.lock().unwrap().random_god_mode as i32);
    GameVariableManager::make_entry("G_Random_Item",  CONFIG.lock().unwrap().random_item as i32);
    GameVariableManager::make_entry("G_Random_God_Sync", CONFIG.lock().unwrap().random_god_sync_mode as i32);
    GameVariableManager::make_entry("G_Random_Engage_Weps", CONFIG.lock().unwrap().random_engage_weapon as i32);
    person::randomize_person();
    emblem::randomize_emblems();
    skill::reset_skills();
    skill::randomize_skills();
    grow::random_grow();
    skill::randomized_god_data();
    write_seed_output_file();
    unsafe { CURRENT_SEED = GameVariableManager::get_number("G_Random_Seed"); }
}

pub fn reset_gamedata() {
    println!("Resetting GameData");

    JobData::unload();
    JobData::load();
    let jobs = JobData::get_list_mut().unwrap();
    // Fix the job style
    unsafe { for j in 0..jobs.len() { job_on_complete(jobs[j], None); } }

    PersonData::unload();
    PersonData::load();
    
    let persons = PersonData::get_list_mut().unwrap();
    for p in 0..persons.len() {  
        persons[p].on_complete(); 
        unsafe { person_data_on_complete_end(persons[p], None); }
    }
    GodData::unload();
    GodData::load();

    GodGrowthData::unload();
    GodGrowthData::load();

    RingData::unload();
    RingData::load();

    let god = GodData::get_list_mut().unwrap();
    for g in 0..god.len() {
        god[g].on_complete();
        let ggd = GodGrowthData::try_get_from_god_data(god[g]);
        if ggd.is_some() {
            let growth = ggd.unwrap();
            for x in 0..growth.len() {
                growth[x].on_complete();
            }
        }
    }
    GodGrowthData::on_complete_end();
    HubDisposData::unload();
    HubDisposData::load();

    HubFacilityData::unload();
    unsafe { hub_facility_data_load(None); }

    ChapterData::unload();
   // ChapterData::load_data();
    unsafe {chapter_data_load(None); }
    skill::reset_skills();
    item::ENGAGE_ITEMS.lock().unwrap().reset();
    item::ENGAGE_ITEMS.lock().unwrap().commit();
    unsafe { CURRENT_SEED = -1; }
}
#[unity::from_offset("App", "HubFacilityData", "Load")]
pub fn hub_facility_data_load(method_info: OptionalMethod);

pub fn randomize_stuff() {
    *CONFIG.lock().unwrap() = DeploymentConfig::new();
    CONFIG.lock().unwrap().correct_rates();
    if GameVariableManager::get_number("G_Random_Seed") == 0 { return; }
    if CONFIG.lock().unwrap().add_new_settings {
        println!("Adding new settings");
        // Random Items
        GameVariableManager::make_entry("G_Random_Item",  CONFIG.lock().unwrap().random_item as i32);
        // Update Emblem Mode if switched to 'All'
        if GameVariableManager::get_number("G_Random_God_Mode") != 0 {
            if CONFIG.lock().unwrap().random_god_mode == 3 { GameVariableManager::set_number("G_Random_God_Mode", 3);}
            if CONFIG.lock().unwrap().random_god_sync_mode != 0 { GameVariableManager::make_entry("G_Random_God_Sync", CONFIG.lock().unwrap().random_god_sync_mode);}
        }
        if GameVariableManager::get_number("G_Random_Item") == 1 {
            if CONFIG.lock().unwrap().random_item != 0 { GameVariableManager::set_number("G_Random_Item", CONFIG.lock().unwrap().random_item as i32); }
        }
        if GameVariableManager::get_number("G_Random_Job") != 0 {
           if CONFIG.lock().unwrap().random_job != 0 { GameVariableManager::set_number("G_Random_Job", CONFIG.lock().unwrap().random_job as i32);  }
        }
        if !GameVariableManager::exist("G_Random_Engage_Weps") {
            if CONFIG.lock().unwrap().random_engage_weapon { GameVariableManager::make_entry("G_Random_Engage_Weps", 1); }
        }
    }
    unsafe {
        if GameVariableManager::get_number("G_Random_Seed") != CURRENT_SEED  {
            println!("Randomized Stuff with Save File Seed {}", GameVariableManager::get_number("G_Random_Seed"));
            emblem::randomize_emblems();
            person::randomize_person();
            skill::randomize_skills();
            grow::random_grow();
            skill::randomized_god_data();
            write_seed_output_file();
            CURRENT_SEED = GameVariableManager::get_number("G_Random_Seed");
        }
    }
}

fn god_engage_random_str(gid: &str) -> String {
    let god = GodData::get(gid).unwrap();
    let emblem_name = Mess::get( god.mid).get_string().unwrap();
    unsafe {
        let engage_attack = Mess::get( SkillData::get( &god.get_engage_attack().get_string().unwrap() ).unwrap().name.unwrap() ).get_string().unwrap();
        let mut string = " ------  ".into();
        let mut string2 = "  ------ ".into();
    
        if god_data_get_link_gid(god, None).is_some() {
            let gid = god_data_get_link_gid(god, None).unwrap();
            string = Mess::get( GodData::get(&gid.get_string().unwrap()).unwrap().mid).get_string().unwrap();
            if god_data_get_engage_link(god, None).is_some() {
                let sid = god_data_get_engage_link(god, None).unwrap();
                string2 = Mess::get( SkillData::get(&sid.get_string().unwrap()).unwrap().name.unwrap()).get_string().unwrap();
            }
        }
        return format!("* {}: {} / {} ({})", emblem_name, engage_attack, string2, string);
    }
}
pub fn skip_m000(){
    if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M000" {
        GameVariableManager::set_bool("勝利".into(), true);
    }
}

#[unity::from_offset("App", "JobData", "OnCompleted")]
fn job_on_complete(this: &JobData, method_info: OptionalMethod);

#[unity::from_offset("App", "ChapterData", "Load")]
fn chapter_data_load(method_info: OptionalMethod);

#[unity::from_offset("App", "PersonData", "OnCompletedEnd")]
fn person_data_on_complete_end(this: &PersonData, method_info: OptionalMethod);

#[unity::from_offset("App", "GodData", "get_LinkGid")]
fn god_data_get_link_gid(this: &GodData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "GodData", "get_EngageAttackLink")]
fn god_data_get_engage_link(this: &GodData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;


extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent, method.method_info);
    });
}
pub struct TriabolicalMenu;
impl ConfigBasicMenuItemCommandMethods for TriabolicalMenu {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();
        // Check if A is pressed before executing any of this
        if pad_instance.npad_state.buttons.b() {CONFIG.lock().unwrap().save();}
        if pad_instance.npad_state.buttons.a() {
            if pad_instance.npad_state.buttons.a() {
            // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
                let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
                    close_anime_all(this.menu, method.method_info);
                });
            // Initialize the menu
                ConfigMenu::create_bind(this.menu);
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

            // Register a OnDispose callback to restore the previous menu
                config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
                // Clear the buttons in the List so we can add our own
                config_menu.full_menu_item_list.clear();
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<ironman::IronmanMod>("Ironman Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::autolevel::AutolevelMod>("Level Scale Units")); 
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<person::RandomPersonMod>("Unit Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemMod>("Emblem Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomJobMod>("Random Classes"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomGrowMod>("Random Growth Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomSkillMod>("Randomize Skills"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomItemMod>("Randomize Obtained Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomGodMod>("Randomize Emblem Data"));       
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomSynchoMod>("Randomize Emblem Sync Data"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomEngageWepMod>("Engage Items/Weapons"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::bgm::RandomBGMMod>("Randomize Map BGM")); 
                BasicMenuResult::se_cursor()
            }   
            else { BasicMenuResult::new() }
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "All will be Revealed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.help_text = "Open up the Draconic Vibe Crystal settings.".into(); }
}
extern "C" fn vibe() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_command::<TriabolicalMenu>("Draconic Vibe Crystal") } 
pub fn install_vibe() { cobapi::install_global_game_setting(vibe); }