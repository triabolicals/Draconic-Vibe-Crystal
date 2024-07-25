use unity::prelude::*;
use skyline::patching::Patch;
use engage::{
    menu::{*, BasicMenuResult, config::{ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    gamevariable::*,
    gameuserglobaldata::*,
    gameuserdata::*,
    proc::ProcInst,
    hub::access::*,
    mess::*,
    gamedata::{*, item::RewardData, skill::*, item::*, god::*, dispos::*},
    pad::Pad,
    util::get_instance,
};
use std::{fs, fs::File, io::Write};
use crate::{enums::*, deploy, person, emblem, item, skill, grow, utils::*};
use crate::emblem::emblem_item::ENGAGE_ITEMS;

use super::{VERSION, CONFIG, DeploymentConfig};
pub static mut LINKED: [i32; 20] = [-1; 20];
pub static mut CURRENT_SEED: i32 = -1;

pub fn tutorial_check(){
    let list = GameVariableManager::find_starts_with("G_解説_");
    for i in 0..list.len() {
        let string = list[i].get_string().unwrap();
        GameVariableManager::set_bool(&string, true);
        if string == "G_解説_TUTID_クラスチェンジ" { return; }
    }
}

pub fn write_seed_output_file() {
    let seed = GameVariableManager::get_number("G_Random_Seed");
    let _ = fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    let filename = format!("sd:/Draconic Vibe Crystal/Slot {} - {}.log", GameUserGlobalData::get_last_save_data_index(), get_player_name());
    let file = File::options().create(true).write(true).truncate(true).open(filename);
    if file.is_err() { println!("Cannot create output file"); return; }
    let mut f = file.unwrap();
    writeln!(&mut f, "------------- Triabolical Randomization Settings - Version {} -------------", VERSION).unwrap();
    if GameVariableManager::get_bool("G_Ironman") { writeln!(&mut f, "* Ironman Mode").unwrap(); }
    if GameVariableManager::get_number("G_Continuous") != 0 { writeln!(&mut f, "* Continuious Mode").unwrap(); }
    writeln!(&mut f, "* Seed: {}", seed).unwrap();
    match GameVariableManager::get_number("G_Random_Recruitment") {
        1 => { writeln!(&mut f, "* Random Recruitment").unwrap(); },
        2 => { writeln!(&mut f, "* Reverse Recruitment").unwrap();}
        3 => { writeln!(&mut f, "* Custom Recruitment").unwrap(); },
        _ => {},
    }
    let emblem_mode =  GameVariableManager::get_number("G_Emblem_Mode");
    match emblem_mode {
        1 => { writeln!(&mut f, "* Emblem Recruitment Mode: Random").unwrap();  },
        2 => { writeln!(&mut f, "* Emblem Recruitment Mode: Reverse").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Recruitment Mode: Normal").unwrap();  },
    }
    let god_mode =  GameVariableManager::get_number("G_Random_God_Mode");
    if god_mode == 0 { writeln!(&mut f, "* Emblem Data: No Randomization").unwrap();  }
    else if god_mode == 1 { writeln!(&mut f, "* Emblem Data: Inheritable Skills").unwrap();  }
    else if god_mode == 2 { writeln!(&mut f, "* Emblem Data: Engage Attack / Engage Link").unwrap(); }
    else if god_mode == 3 { writeln!(&mut f, "* Emblem Data: Inheritable / Attack / Link").unwrap(); }
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
        2 => { writeln!(&mut f, "* Emblem Sync Data: Sync / Engage Skills").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
    }
    if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
        writeln!(&mut f, "\n--------------- Person Recruitment Order ---------------").unwrap();
        let mut count = 0;
        for x in PIDS{
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
        for x in EMBLEM_GIDS { 
            let string = format!("G_R_{}", x);
            let name1 = Mess::get( GodData::get(x).unwrap().mid ).get_string().unwrap();
            let new_gid = GameVariableManager::get_string(&string);
            let mut name2 = String::new();
            if GodData::get(&new_gid.get_string().unwrap()).is_some() { name2 = Mess::get( GodData::get(&new_gid.get_string().unwrap()).unwrap().mid).get_string().unwrap(); }
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
        let playable_size = person::PLAYABLE.lock().unwrap().len();
        let person_list = PersonData::get_list().unwrap();
        for x in 0..playable_size {
            let p_index = person::PLAYABLE.lock().unwrap()[x as usize] as usize;
            let person = &person_list[p_index]; 
            let mut name = Mess::get(person.get_name().unwrap()).get_string().unwrap();
            if name.len() == 0 { name = person.get_name().unwrap().get_string().unwrap(); }
            let personal_sid = person.get_common_sids().unwrap();
            for y in 0..personal_sid.len() {
                let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
                let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
                if skill.get_flag() & 1 == 0 {
                    let skill_name = Mess::get( SkillData::get(&personal_sid[y as usize].get_string().unwrap()).unwrap().name.unwrap() ).get_string().unwrap();
                    let sid = SkillData::get(&personal_sid[y as usize].get_string().unwrap()).unwrap().sid.get_string().unwrap();
                    writeln!(&mut f, "* {} ({}):\t{} ({})",  name, person.pid.get_string().unwrap(), skill_name, sid).unwrap();
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
        let bond_ring_rates = CONFIG.lock().unwrap().get_bond_ring_rates();
        let ranks = ["S", "A", "B", "C"];
        for x in 0..4 {
            writeln!(&mut f, "-- {} Rank Rate: {}", ranks[x as usize], bond_ring_rates[x as usize]).unwrap();
        }
        for x in 0..ring_list.len() {
            let skills = skill_array_string(ring_list[x].get_equip_skills());
            if skills.len() <= 2 { continue; }
            let name = Mess::get(ring_list[x].name).get_string().unwrap();
            let rank;
            if ring_list[x].rank == 3 { rank = "S"; }
            else if ring_list[x].rank == 2 { rank = "A"; }
            else if ring_list[x].rank == 1 { rank = "B"; }
            else { rank = "C"; }
            if ring_list[x].gid.is_some() { writeln!(&mut f, "* {}: {} {} - {}", get_emblem_name(&ring_list[x].gid.unwrap().get_string().unwrap()), name, rank, skills).unwrap(); }
            else { writeln!(&mut f, "* {} - {}", name, skills).unwrap(); }
        }
        writeln!(&mut f, "\n--------------- Randomization Skill Pool Availiablity ({} skills)  ---------------", n_skills).unwrap();
        for x in 0..n_skills {
            let skill_index = skill::SKILL_POOL.lock().unwrap()[x as usize].index as usize;
            let skill = &skill_list[skill_index ];
            let sid = skill.sid.get_string().unwrap();
            let name = Mess::get(skill_list[skill_index ].name.unwrap()).get_string().unwrap();
            let personal;
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
        for x in 0..20 {
            let gid = format!("GID_{}", EMBLEM_ASSET[x as usize]); 
            let line = crate::message::god_engage_random_str(&gid);
            writeln!(&mut f, "{}", line).unwrap();
        }
    }
    writeln!(&mut f, "\n--------------- Emblem Engrave Data ---------------").unwrap();
    for x in 0..20 {
        let gid = format!("GID_{}", EMBLEM_ASSET[x as usize]); 
        let god = GodData::get(&gid).unwrap();
        let line = format!("* {} - \t{}: {}, {}: {}, {}: {}, {}: {}, {}: {}, {}: {}", 
        mess_get(god.mid), 
        get_stat_label(11), god.get_engrave_avoid(),  get_stat_label(12), god.get_engrave_critical(), get_stat_label(13), god.get_engrave_hit(), 
        get_stat_label(14), god.get_engrave_power(), get_stat_label(15), god.get_engrave_secure(), get_stat_label(16), god.get_engrave_weight());
        writeln!(&mut f, "{}", line).unwrap();
    }
    writeln!(&mut f, "\n--------------- Emblem Sync / Engage Data --------------").unwrap();
    match god_mode {
        1 => { writeln!(&mut f, "* Emblem Data: Inheritable Skills").unwrap();  }
        2 => { writeln!(&mut f, "* Emblem Data: Engage Attack / Engage Link").unwrap(); }
        3 => { writeln!(&mut f, "* Emblem Data: Inheritable / Engage Attack / Engage Link").unwrap(); }
        _ => { writeln!(&mut f, "* Emblem Data: No Randomization").unwrap();  }
    }
    match sync_mode {
        1 => { writeln!(&mut f, "* Emblem Sync Data: Stat Bonuses").unwrap(); },
        2 => { writeln!(&mut f, "* Emblem Sync Data: Sync / Engage Skills").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
    }
    let mut index: usize = 0;
    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        let god_id = format!("GID_{}", x);
        let engage_skill = level_data[0].engage_skills.list.item[0].get_skill().unwrap();
        let god = GodData::get(x).unwrap(); 
        let god_grow = GodGrowthData::try_get_from_god_data(god).unwrap();
        writeln!(&mut f, "\n*** {} Engage Skill: {}, Engage Atk/Link: {}\n", get_emblem_name(&god_id), get_skill_name(engage_skill), crate::message::god_engage_random_str(&god_id)).unwrap();
        let weapons_str = ENGAGE_ITEMS.lock().unwrap().print(index as i32, 0);
        writeln!(&mut f, "\t* Engage Weapons 1: {}", weapons_str).unwrap();
        let weapons_str2 = ENGAGE_ITEMS.lock().unwrap().print(index as i32, 1);
        writeln!(&mut f, "\t* Engage Weapons 2: {}", weapons_str2).unwrap();
        let weapons_str3 = ENGAGE_ITEMS.lock().unwrap().print(index as i32, 2);
        writeln!(&mut f, "\t* Engage Weapons 3: {}\n", weapons_str3).unwrap();
        for y in 1..level_data.len() {
            writeln!(&mut f, "\t* {} Lv. {} Stats: {}", get_emblem_name(&god_id), y, stats_from_skill_array(level_data[y as usize].synchro_skills)).unwrap();
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
        index += 1;
    }
    println!("Randomization Print to file");
}

 // Hooks 
// Switching PersonData indexes in scripts for event function calls
#[skyline::hook(offset=0x01cb5eb0)]
pub fn try_get_index(dyn_value: u64, index: i32, method_info: OptionalMethod) -> i32 {
    let result = call_original!(dyn_value, index, method_info);
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return result; }
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
fn create_changables() {
    if !GameVariableManager::exist("G_HubItem") { GameVariableManager::make_entry("G_HubItem", CONFIG.lock().unwrap().exploration_items); }
    if !GameVariableManager::exist("G_EngagePlus") { GameVariableManager::make_entry("G_EngagePlus", CONFIG.lock().unwrap().engage_link as i32); }
    if !GameVariableManager::exist("G_EnemySkillGauge")  { GameVariableManager::make_entry("G_EnemySkillGauge", CONFIG.lock().unwrap().random_enemy_skill_rate); }
    if !GameVariableManager::exist("G_EnemyJobGauge")  { GameVariableManager::make_entry("G_EnemyJobGauge", CONFIG.lock().unwrap().random_enemy_job_rate); }
    if !GameVariableManager::exist("G_EnemyEmblemGauge")  { GameVariableManager::make_entry("G_EnemyEmblemGauge", CONFIG.lock().unwrap().enemy_emblem_rate); }
    if !GameVariableManager::exist("G_DeploymentMode")  { GameVariableManager::make_entry("G_DeploymentMode", CONFIG.lock().unwrap().deployment_type); }
    if !GameVariableManager::exist("G_EmblemDeployMode")  { GameVariableManager::make_entry("G_EmblemDeployMode", CONFIG.lock().unwrap().emblem_deployment); }
    if !GameVariableManager::exist("G_DVC_Autolevel") { GameVariableManager::make_entry("G_DVC_Autolevel", CONFIG.lock().unwrap().autolevel as i32); }
    if !GameVariableManager::exist("G_RandomBGM")  { GameVariableManager::make_entry("G_RandomBGM", CONFIG.lock().unwrap().random_map_bgm as i32 ); }
    if !GameVariableManager::exist("G_EnemyRevivalStone") { GameVariableManager::make_entry("G_EnemyRevivalStone", CONFIG.lock().unwrap().revival_stone_rate); }
    if !GameVariableManager::exist("G_ItemGauge") { GameVariableManager::make_entry("G_ItemGauge", CONFIG.lock().unwrap().replaced_item_price); }
    if !GameVariableManager::exist("G_BattleStyles") { GameVariableManager::make_entry("G_BattleStyles", CONFIG.lock().unwrap().random_battle_styles as i32); }
    if !GameVariableManager::exist("G_EngraveSetting") { GameVariableManager::make_entry("G_EngraveSetting", CONFIG.lock().unwrap().engrave_settings as i32); }
    if !GameVariableManager::exist("G_EngraveSetting2") { GameVariableManager::make_entry("G_EngraveSetting2", CONFIG.lock().unwrap().engrave_settings as i32); }
}

pub fn start_new_game(){
    *CONFIG.lock().unwrap() = DeploymentConfig::new();
    CONFIG.lock().unwrap().correct_rates();
    let seed = CONFIG.lock().unwrap().seed;
    if CONFIG.lock().unwrap().iron_man { 
        GameVariableManager::make_entry("G_Ironman", 1);
        crate::ironman::ironman_code_edits();
    }
    GameVariableManager::make_entry("G_Continuous", CONFIG.lock().unwrap().continuous);
    crate::shop::randomize_shop_data();
    if CONFIG.lock().unwrap().randomized {
        if seed == 0 {  GameVariableManager::make_entry("G_Random_Seed", get_random_number_for_seed() as i32); }
        else { GameVariableManager::make_entry("G_Random_Seed", CONFIG.lock().unwrap().seed as i32); }
    }
    else { 
        GameVariableManager::make_entry("G_Random_Seed", 0);
        let m001 = ChapterData::get("CID_M001").unwrap();
        GameVariableManager::set_bool("G_Cleared_M000", true);
        GameUserData::set_chapter(m001);
        create_changables();
        return;
    }
    GameVariableManager::make_entry("G_Emblem_Mode", CONFIG.lock().unwrap().emblem_mode as i32);
    GameVariableManager::make_entry("G_Random_Recruitment", CONFIG.lock().unwrap().random_recruitment as i32);
    GameVariableManager::make_entry("G_Random_Job", CONFIG.lock().unwrap().random_job as i32);
    GameVariableManager::make_entry("G_Lueur_Random", 0);
    GameVariableManager::make_entry("G_Random_Skills", CONFIG.lock().unwrap().random_skill as i32);
    GameVariableManager::make_entry("G_Random_Grow_Mode", CONFIG.lock().unwrap().random_grow as i32);
    GameVariableManager::make_entry("G_Random_God_Mode",  CONFIG.lock().unwrap().random_god_mode as i32);
    GameVariableManager::make_entry("G_Random_Item",  CONFIG.lock().unwrap().random_item as i32);
    GameVariableManager::make_entry("G_Random_God_Sync", CONFIG.lock().unwrap().random_god_sync_mode as i32);
    GameVariableManager::make_entry("G_ChaosMode", CONFIG.lock().unwrap().emblem_skill_chaos as i32);
    GameVariableManager::make_entry("G_Random_Engage_Weps", CONFIG.lock().unwrap().random_engage_weapon as i32);
    create_changables();     // Changables 
    crate::shop::randomize_shop_data();
    emblem::randomize_emblems();
    crate::utils::get_lueur_name_gender();
    person::randomize_person();
    skill::randomize_skills();
    grow::random_grow();
    grow::randomize_job_styles();
    crate::emblem::emblem_skill::randomized_god_data();
    randomize_engage_links();
    item::randomize_well_rewards();
    write_seed_output_file();
    tutorial_check();
    let m001 = ChapterData::get("CID_M001").unwrap();
    GameVariableManager::set_bool("G_Cleared_M000", true);
    GameUserData::set_chapter(m001);
    crate::person::change_lueur_for_recruitment(true);
    skill::replace_enemy_version();
    unsafe { 
        CURRENT_SEED = GameVariableManager::get_number("G_Random_Seed"); 
        LUEUR_CHANGE = true;
    }
    if CONFIG.lock().unwrap().random_job != 0 { crate::asset::unlock_royal_classes(); }
}

pub fn reset_gamedata() {
    println!("Resetting GameData");
    skill::reset_skills();
    ItemData::unload();
    ItemData::load_data();
    let items = ItemData::get_list_mut().unwrap();
    for j in 0..items.len() {items[j].on_completed(); }
    JobData::unload();
    JobData::load();
    let jobs = JobData::get_list_mut().unwrap();
    for j in 0..jobs.len() { jobs[j].on_completed(); } 
    PersonData::unload();
    PersonData::load();
    let persons = PersonData::get_list_mut().unwrap();
    for p in 0..persons.len() {  persons[p].on_completed();  }
    GodData::unload();
    GodData::load();
    GodGrowthData::unload();
    GodGrowthData::load();
    RingData::unload();
    RingData::load_data();
    crate::shop::reset_shopdata();
    let god = GodData::get_list_mut().unwrap();
    for g in 0..god.len() {
        god[g].on_complete();
        if god[g].gid.get_string().unwrap() == "GID_リュール" {
            god[g].get_flag().value |= -2147483648; // adding back the hero flag if removed for emblem alear
        }
        let ggd = GodGrowthData::try_get_from_god_data(god[g]);
        if ggd.is_some() {
            let growth = ggd.unwrap();
            for x in 0..growth.len() { growth[x].on_complete(); }
        }
    }
    GodGrowthData::on_complete_end();
    HubDisposData::unload();
    HubDisposData::load();

    RewardData::unload();
    RewardData::load();
    HubFacilityData::unload();
    HubFacilityData::load_data();
    ChapterData::unload();
    ChapterData::load_data();

    ENGAGE_ITEMS.lock().unwrap().reset();
    ENGAGE_ITEMS.lock().unwrap().commit();

    Patch::in_text(0x01dc9f8c).bytes(&[0xb5, 0xd9, 0x15, 0x94]).unwrap();   //  Reset God Exp bypass check for Engage+ Links
    Patch::in_text(0x01a39fe4).bytes(&[0x68,0x00, 0x00, 0xb4]).unwrap();    // Reset SP = EXP 
    Patch::in_text(0x01d76320).bytes(&[0xfd, 0x7b, 0xbd, 0xa9]).unwrap();   // Revert Back menu item in Sortie
    Patch::in_text(0x01d76324).bytes(&[0xf6, 0x57, 0x01, 0xa9]).unwrap(); 
    // Alear Randomization Revert
    Patch::in_text(0x02d524e0).bytes(&[0x1f, 0x00, 0x00, 0x72]).unwrap();   // Lueur God Face Stuff
    Patch::in_text(0x02d524e4).bytes(&[0x08, 0x11, 0x89, 0x9a]).unwrap();   
    Patch::in_text(0x02d524e8).bytes(&[0x08, 0x01, 0x40, 0xb9]).unwrap();

    Patch::in_text(0x0233f104).bytes(&[0x01, 0x00, 0xb0, 0x52]).unwrap();   // Emblem Alear Stuff
    Patch::in_text(0x02d51dec).bytes(&[0xb1, 0x60, 0xc7, 0x97]).unwrap();   //FaceThumbnail removes check for hero 
    Patch::in_text(0x021e12ac).bytes(&[0x81, 0x23, 0xf5, 0x97]).unwrap();   //GetBondLevelFacePath
    Patch::in_text(0x02915844).bytes(&[0x1b, 0x52, 0xd8, 0x97]).unwrap();   //InfoUtil$$SetGodName to prevent the Emblem name to disable for the Hero with Emblem Alear
    Patch::in_text(0x02915694).bytes(&[0x87, 0x52, 0xd8, 0x97]).unwrap();   //SetUnitName - prevents Emblem X on hero unit when engaged with Alear
    Patch::in_text(0x01c66588).bytes(&[0xca, 0x0e, 0x0b, 0x94]).unwrap();   // Bond Exp Gauge-Related Hero check
    Patch::in_text(0x01c666ac).bytes(&[0x81, 0x0e, 0x0b, 0x94]).unwrap();   // Bond Exp Gauge-Related Hero Check
    Patch::in_text(0x02081edc).bytes(&[0x75, 0xa0, 0xfa, 0x97]).unwrap();   // god face for hero + emblem alear
    Patch::in_text(0x01c69d60).bytes(&[0xd4, 0x00, 0x0b, 0x94]).unwrap();   // hero disappear when selecting emblem alear

    Patch::in_text(0x02ae9000).bytes(&[0x60, 0xc7, 0xfd, 0x97]).unwrap(); // Gender animation for the replacement unit 
    Patch::in_text(0x02ae8d28).bytes(&[0x16, 0xc8, 0xfd, 0x97]).unwrap();
    Patch::in_text(0x02a5d0f4).bytes(&[0x23, 0xf7, 0xff, 0x97]).unwrap();
    Patch::in_text(0x01cfd4c4).bytes(&[0x2f, 0x76, 0x35, 0x94]).unwrap();
    Patch::in_text(0x01d03184).bytes(&[0xff, 0x5e, 0x35, 0x94]).unwrap();
    Patch::in_text(0x01e5fe00).bytes(&[0xe0, 0xeb, 0x2f, 0x94]).unwrap();
    Patch::in_text(0x01e5ff4c).bytes(&[0x8d, 0xeb, 0x2f, 0x94]).unwrap();
    Patch::in_text(0x027049c8).bytes(&[0xee, 0x58, 0x0d, 0x94]).unwrap();
    unsafe {
        CURRENT_SEED = -1; 
        LUEUR_CHANGE = false;
        for x in 0..20 {  LINKED[x as usize] = -1; }    // Linked Units for Engage+
    }
}

pub fn randomize_stuff() {
    *CONFIG.lock().unwrap() = DeploymentConfig::new();
    CONFIG.lock().unwrap().correct_rates();
    if !can_rand() { return; }
    tutorial_check();
    unsafe {
        if GameVariableManager::get_number("G_Random_Seed") != CURRENT_SEED  {
            GameVariableManager::make_entry("G_ChaosMode", CONFIG.lock().unwrap().emblem_skill_chaos as i32);
            if GameVariableManager::get_number("G_Random_Job") != 0 {  crate::asset::unlock_royal_classes(); }
            println!("Randomized Stuff with Save File Seed {}", GameVariableManager::get_number("G_Random_Seed"));
            create_changables();    //In-game menus
            crate::shop::randomize_shop_data();
            emblem::randomize_emblems();
            person::randomize_person();
            skill::randomize_skills();            
            grow::random_grow();
            crate::person::change_lueur_for_recruitment(false);
            skill::replace_enemy_version();
            grow::randomize_job_styles();
            crate::emblem::emblem_skill::randomized_god_data();
            emblem::random_engrave_by_setting( GameVariableManager::get_number("G_EngraveSetting"));
            randomize_engage_links();
            item::randomize_well_rewards();
            write_seed_output_file();
            if ( GameVariableManager::get_bool("G_Cleared_M002") && GameVariableManager::get_bool("G_Random_Job") && 
                GameVariableManager::get_bool("G_Lueur_Random") ) && ( GameVariableManager::get_number("G_Liberation_Type") != 0 ) {
                let liberation = ItemData::get_mut("IID_リベラシオン").unwrap();
                let l_type = GameVariableManager::get_number("G_Liberation_Type") as u32;
                liberation.kind = l_type;
                if l_type == 4 {
                    liberation.range_i = 2;
                    liberation.range_o = 3;
                    liberation.set_cannon_effect("弓砲台".into());
                    liberation.on_complete();
                    liberation.get_equip_skills().add_sid("SID_飛行特効",4, 0);
                }
                else if l_type == 5 || l_type == 6 {
                    liberation.range_o = 2;
                    liberation.range_i = 1;
                    if l_type == 6 {
                        liberation.set_cannon_effect("魔砲台炎".into());
                        liberation.set_hit_effect( "エルファイアー".into());
                        liberation.on_complete();
                    }
                    else { liberation.get_give_skills().add_sid("SID_毒",3, 0); }
                }
                else if l_type == 8 {
                    liberation.get_equip_skills().add_sid("SID_気功",4, 0);
                    liberation.get_equip_skills().add_sid("SID_２回行動",4,0);
                }
                else {
                    liberation.range_i = 1;
                    liberation.range_o = 1;
                }
            }
            CURRENT_SEED = GameVariableManager::get_number("G_Random_Seed");
            LUEUR_CHANGE = true;
            if GameVariableManager::get_bool("G_Random_Job") && GameVariableManager::exist("G_Misercode_Type") {
                let misercode = ItemData::get_mut("IID_ミセリコルデ").unwrap();
                misercode.kind = GameVariableManager::get_number("G_Misercode_Type") as u32;
                let misercode_type = GameVariableManager::get_number("G_Misercode_Type");
                misercode.get_give_skills().clear();
                misercode.get_equip_skills().clear();
                if misercode_type == 4 {
                    misercode.range_o = 2; misercode.range_i = 2;
                    misercode.set_cannon_effect("弓砲台".into());
                    misercode.on_complete();
                    misercode.get_equip_skills().add_sid("SID_飛行特効",4, 0);
                }
                else if misercode_type == 5 || misercode_type == 6 {
                    misercode.range_i = 1;
                    misercode.range_o = 2;
                    if misercode_type == 6 {
                        misercode.set_cannon_effect("魔砲台炎".into());
                        misercode.set_hit_effect( "オヴスキュリテ".into());
                    }
                    else { misercode.get_give_skills().add_sid("SID_毒",3, 0); }
                    misercode.on_complete();
                }   
                else if misercode_type == 8 { 
                    misercode.range_i = 1;
                    misercode.range_o = 1;
                    misercode.get_equip_skills().add_sid("SID_２回行動",4,0); 
                }
                else {
                    misercode.range_i = 1;
                    misercode.range_o = 2;
                }
            }
        }
    }
}

pub fn randomize_engage_links() {
    if !can_rand() { return; }
    if !CONFIG.lock().unwrap().engage_link { return; }
    let mut pid_set: [bool; 41] = [false; 41];
    pid_set[0] = true;
    let rng = get_rng();
    let dic = GodData::get_link_dictionary();
    Patch::in_text(0x01dc9f8c).bytes(&[0x20, 0x00, 0x80, 0x52]).unwrap();     // God Exp bypass check
    unsafe {
        let emblem_count;
        let person_count;
        if has_content(0, None) {
            emblem_count = 19;
            person_count = 41;
        }
        else {
            emblem_count = 12;
            person_count = 36;
        }
        for x in 0..emblem_count {
            let gid = format!("GID_{}", EMBLEM_ASSET[x as usize]);
            let god = GodData::get(&gid).unwrap();
            let mut index: usize = rng.get_value(person_count as i32) as usize;
            let mut pid = PIDS[index];
            while pid_set[index] || GodData::try_get_link(PersonData::get(&pid).unwrap()).is_some()  {
                index = rng.get_value(person_count as i32) as usize;
                pid = PIDS[index];
            }
            LINKED[ x as usize ] = index as i32;
            god.on_complete();
            let person = PersonData::get(&pid).unwrap();
            dic.add(PIDS[index].into(), god);
            person.on_complete();
            pid_set[index] = true;
        }
    }
}

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
        if pad_instance.npad_state.buttons.b() {CONFIG.lock().unwrap().save();}
        if pad_instance.npad_state.buttons.a() {
            if pad_instance.npad_state.buttons.a() {
            // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
                let close_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
                    close_anime_all(this.menu, method.method_info);
                });
                ConfigMenu::create_bind(this.menu);
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

                config_menu.get_class_mut().get_virtual_method_mut("OnDispose").map(|method| method.method_ptr = open_anime_all_ondispose as _).unwrap();
                config_menu.full_menu_item_list.clear();
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<RandomEnable>("Enable Randomization"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::continuous::ContiniousMode>("Continuous Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::ironman::IronmanMod>("Ironman Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::autolevel::AutolevelMod>("Level Scale Units")); 
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<person::RandomPersonMod>("Unit Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemMod>("Emblem Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomJobMod>("Random Classes"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<item::EnemyJobGauge>("Random Enemy Class Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomGrowMod>("Random Growth Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomBattleStyles>("Randomize Class Types"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomSkillMod>("Randomize Skills"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<skill::EnemySkillGauge>("Random Enemy Skill Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomItemMod>("Randomize Obtained Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::item::ItemPriceGauge>("Randomized Item Value"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomGiftMod>("Reward/Gift Item Settings"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::shop::RandomShopMod>("Shop Setting"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<crate::shop::RandomHubItemMod>("Exploration Items"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomGodMod>("Randomize Emblem Data"));       
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomSynchoMod>("Randomize Emblem Sync Data"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::emblem_skill::EmblemSkillChaos>("Emblem Skill Chaos Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::EngraveSettings>("Engrave Randomization Level"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEngageWepMod>("Engage Items/Weapons"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemLinkMod>("Engage+ Links"));
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
extern "C" fn vibe() -> &'static mut ConfigBasicMenuItem { 
    let title = format!("Draconic Vibe Crystal {}", super::VERSION);
    ConfigBasicMenuItem::new_command::<TriabolicalMenu>(title) 
}
// InGame Menu Stuff
extern "C" fn vibe_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map2 as _);
    switch
} 
extern "C" fn vibe_emblem_deployment() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map2 as _);
    switch
} 
extern "C" fn vibe_autolevel() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<crate::autolevel::AutolevelMod>("Level Scale Units");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
} 
extern "C" fn vibe_hub_items() -> &'static mut ConfigBasicMenuItem {  
    let hub_items = ConfigBasicMenuItem::new_switch::<crate::shop::RandomHubItemMod2>("Exploration Items");
    hub_items.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_hub_items as _);
    hub_items
}
extern "C" fn vibe_bgm() -> &'static mut ConfigBasicMenuItem {  
    let switch =  ConfigBasicMenuItem::new_switch::<crate::bgm::RandomBGMMod>("Randomize Map BGM");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
}
extern "C" fn vibe_engage_links() -> &'static mut ConfigBasicMenuItem {  
    let switch = ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemLinkMod>("Engage+ Links");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch
}
extern "C" fn vibe_enemy_emblem() -> &'static mut ConfigBasicMenuItem { 
    let enemy_emblem = ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyEmblemGauge>("Enemy Emblem Rate"); 
    enemy_emblem.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    enemy_emblem
}
extern "C" fn vibe_enemy_stones() -> &'static mut ConfigBasicMenuItem { 
    let enemy_stones = ConfigBasicMenuItem::new_gauge::<crate::autolevel::EnemyRevivalStones>("Enemy Revival Stone Rate"); 
    enemy_stones.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    enemy_stones
}
extern "C" fn vibe_job_gauge() -> &'static mut ConfigBasicMenuItem {  
    let class_gauge = ConfigBasicMenuItem::new_gauge::<item::EnemyJobGauge>("Random Enemy Class Rate"); 
    class_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_job_gauge as _);
    class_gauge
}
extern "C" fn vibe_skill_gauge() -> &'static mut ConfigBasicMenuItem {  
    let skill_gauge = ConfigBasicMenuItem::new_gauge::<skill::EnemySkillGauge>("Random Enemy Skill Rate");
    skill_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_skill_gauge as _);
    skill_gauge
}
extern "C" fn vibe_item_gauge() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_gauge::<item::ItemPriceGauge>("Randomized Item Value");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_hub_items as _);
    item_gauge
}
extern "C" fn vibe_styles() -> &'static mut ConfigBasicMenuItem {  
    let item_gauge = ConfigBasicMenuItem::new_switch::<grow::RandomBattleStyles>("Random Class Types");
    item_gauge.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    item_gauge
}
extern "C" fn vibe_engrave() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<emblem::EngraveSettings>("Engrave Randomization Level");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = build_attribute_not_in_map as _);
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr =  emblem::engrave_setting_acall as _);
    switch
}
fn build_attribute_job_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number("G_Random_Job") > 1 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

fn build_attribute_skill_gauge(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_bool("G_Random_Skills") { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}

fn build_attribute_hub_items(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Enable }
    else if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2  { BasicMenuItemAttribute::Hide }
    else if !crate::utils::can_rand() { BasicMenuItemAttribute::Hide }
    else if GameVariableManager::get_number("G_Random_Item") != 0 { BasicMenuItemAttribute::Enable }
    else { BasicMenuItemAttribute::Hide }
}
pub fn build_attribute_not_in_map(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else if !crate::utils::can_rand() && !GameUserData::get_sequence() == 0 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}
pub fn build_attribute_not_in_map2(_this: &mut ConfigBasicMenuItem,  _method_info: OptionalMethod) -> BasicMenuItemAttribute  {
    if GameUserData::get_sequence() == 3 || GameUserData::get_sequence() == 2 { BasicMenuItemAttribute::Hide }
    else { BasicMenuItemAttribute::Enable }
}

pub struct RandomEnable;
impl ConfigBasicMenuItemSwitchMethods for RandomEnable {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = CONFIG.lock().unwrap().randomized;
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            CONFIG.lock().unwrap().randomized = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if CONFIG.lock().unwrap().randomized { "Enables randomization settings on a new save." } 
            else {"Disables randomization settings on a new save." }.into();
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if CONFIG.lock().unwrap().randomized { "Enable" } else { "Disable" }.into();
    }
}
pub fn install_vibe() { 
    cobapi::install_global_game_setting(vibe); 
    cobapi::install_game_setting(vibe_deployment);
    cobapi::install_game_setting(vibe_emblem_deployment);
    cobapi::install_game_setting(vibe_autolevel);
    cobapi::install_game_setting(vibe_skill_gauge);
    cobapi::install_game_setting(vibe_job_gauge);
    cobapi::install_game_setting(vibe_hub_items);
    cobapi::install_game_setting(vibe_item_gauge);
    cobapi::install_game_setting(vibe_bgm);
    cobapi::install_game_setting(vibe_styles);
    cobapi::install_game_setting(vibe_engrave);
    cobapi::install_game_setting(vibe_enemy_emblem);
    cobapi::install_game_setting(vibe_enemy_stones);
    cobapi::install_game_setting(vibe_engage_links);
}