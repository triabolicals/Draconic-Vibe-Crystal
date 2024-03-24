use unity::prelude::*;
use engage::{
    menu::{*, BasicMenuResult, config::{ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    proc::ProcInst,
    random::*,
    force::*,
    hub::access::*,
    mess::*,
    gamedata::{*, skill::*, god::*},
    pad::Pad,
    util::get_instance,
};
use std::{fs::File, io::Write};
use crate::{deploy, person, emblem, item, skill, grow, ironman};
use super::{VERSION, CONFIG};

pub static mut CURRENT_SEED: i32 = -1;

//DLC Check 
#[unity::from_offset("App", "DLCManager", "HasContent")]
pub fn has_content(content: i32, method_info: OptionalMethod) -> bool;

// Frame Count
#[skyline::from_offset(0x0250c6a0)]
pub fn get_frame_count(method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x3784700)]
pub fn string_start_with(this: &Il2CppString, value: &Il2CppString, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x037815b0)]
pub fn sub_string(this: &Il2CppString, start: i32, method_info: OptionalMethod) -> &'static Il2CppString;

#[skyline::from_offset(0x3780700)]
pub fn is_null_empty(this: &Il2CppString, method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x03773720)]
pub fn replace_str(this: &Il2CppString, old_value: &Il2CppString, new_value: &Il2CppString, method_info: OptionalMethod) -> &'static Il2CppString;

#[unity::from_offset("System", "String", "Contains")]
pub fn string_contains(this: &Il2CppString, value: &Il2CppString, method_info: OptionalMethod) -> bool;

pub fn str_contains(this: &Il2CppString, value: &str) -> bool { unsafe {string_contains(this, value.into(), None) } }

// Getting Player's name for file name
pub fn get_player_name() -> String {
    let f_type: [ForceType; 5] = [ForceType::Player, ForceType::Enemy, ForceType::Absent, ForceType::Dead, ForceType::Lost];
    for f in f_type {
        let force = Force::get(f).unwrap();
        let mut force_iter = Force::iter(force);
        while let Some(unit) = force_iter.next() {
            if unit.person.pid.get_string().unwrap() == "PID_リュール" {
                if unit.edit.name.is_some(){ return unit.edit.name.unwrap().get_string().unwrap(); }
            }
        }
    }
    return "randomized".to_string();
}

pub fn write_seed_output_file() {
    let seed = GameVariableManager::get_number("G_Random_Seed");
    let filename = format!("sd:/engage/{}.log", get_player_name());
    let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
    writeln!(&mut f, "------------- Triabolical Randomization Settings - Version {} -------------", VERSION).unwrap();
    writeln!(&mut f, "* Seed: {}", seed).unwrap();
    writeln!(&mut f, "* Random Recruitment: {}", GameVariableManager::get_bool("G_Random_Recruitment")).unwrap();
    let emblem_mode =  GameVariableManager::get_number("G_Emblem_Mode");
    if emblem_mode == 0 { writeln!(&mut f, "* Emblem Recruitment Mode: No Randomization").unwrap();  }
    else if emblem_mode == 1 { writeln!(&mut f, "* Emblem Recruitment Mode: Random").unwrap();  }
    else if emblem_mode == 2 { writeln!(&mut f, "* Emblem Recruitment Mode: Reverse").unwrap(); }
    writeln!(&mut f, "* Random Classes: {}", GameVariableManager::get_bool("G_Random_Job")).unwrap();
    writeln!(&mut f, "* Random Skills: {}", GameVariableManager::get_bool("G_Random_Skills")).unwrap();

    let growth_mode = GameVariableManager::get_number("G_Random_Grow_Mode");
    match growth_mode {
        1 => { writeln!(&mut f, "* Growth Rate Mode: Personal").unwrap(); },
        2 => { writeln!(&mut f, "* Growth Rate Mode: Class Mods").unwrap(); },
        3 => { writeln!(&mut f, "* Growth Rate Mode: Personal + Class Mods").unwrap(); },
        _ => { writeln!(&mut f, "* Growth Rate Mode: No Randomization").unwrap(); },
    }

    writeln!(&mut f, "\n--------------- Person Recruitment Order Randomization ---------------").unwrap();
    if GameVariableManager::get_bool("G_Random_Recruitment") {
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
    if GameVariableManager::get_number("G_Random_Grow_Mode") == 1 ||  GameVariableManager::get_number("G_Random_Grow_Mode") == 3 {
        writeln!(&mut f, "\n--------------- Personal Growth Rates Randomization ---------------").unwrap();
        let person_list = PersonData::get_list().unwrap();
        for x in 0..person_list.len() {
            let grow = person_list[x].get_grow();
            if grow.is_zero() { continue; } 
            let pid = person_list[x].pid.get_string().unwrap();
            let mut name = " -- ".to_string();
            if person_list[x].get_name().is_some() {
                name = Mess::get(person_list[x].get_name().unwrap()).get_string().unwrap();
            }
            writeln!(&mut f, "* {} - {} ({})\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |", x+1, name, pid, 
            Mess::get("MID_SYS_HP").get_string().unwrap(), grow[0], Mess::get("MID_SYS_Str").get_string().unwrap(), grow[1], Mess::get("MID_SYS_Mag").get_string().unwrap(), grow[6], 
            Mess::get("MID_SYS_Tec").get_string().unwrap(), grow[2], Mess::get("MID_SYS_Spd").get_string().unwrap(), grow[3], Mess::get("MID_SYS_Lck").get_string().unwrap(), grow[4],
            Mess::get("MID_SYS_Def").get_string().unwrap(), grow[5], Mess::get("MID_SYS_Res").get_string().unwrap(), grow[7], Mess::get("MID_SYS_Phy").get_string().unwrap(), grow[8]).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_Grow_Mode") == 2 ||  GameVariableManager::get_number("G_Random_Grow_Mode") == 3 {
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
        writeln!(&mut f, "\n--------------- Skills in the Randomization Pool ({} skills) ---------------", n_skills).unwrap();
        for x in 0..n_skills {
            let skill_index = skill::SKILL_POOL.lock().unwrap()[x as usize].index as usize;
            let sid = skill_list[skill_index ].sid.get_string().unwrap();
            let name = Mess::get(skill_list[skill_index ].name.unwrap()).get_string().unwrap();
            writeln!(&mut f, "- {} - {} ({})", x+1, name, sid).unwrap();
        }
    }
    println!("Randomization Print to file");
}

pub fn get_random_number_for_seed() -> u32 {
    unsafe {
        let seed = get_frame_count(None);
        let rng = Random::get_system();
        rng.initialize(seed as u32);
        let result = rng.value() as u32;
        return result;
    }
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
    if !GameVariableManager::get_bool("G_Random_Recruitment") == 0 { return result; }
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
    CONFIG.lock().unwrap().seed += get_random_number_for_seed();
    GameVariableManager::make_entry("G_Random_Seed", CONFIG.lock().unwrap().seed as i32);
    GameVariableManager::make_entry("G_Emblem_Mode", CONFIG.lock().unwrap().emblem_mode as i32);
    GameVariableManager::make_entry("G_Random_Recruitment", CONFIG.lock().unwrap().random_recruitment as i32);
    GameVariableManager::make_entry("G_Random_Job", CONFIG.lock().unwrap().random_job as i32);
    GameVariableManager::make_entry("G_Lueur_Random", 0);
    GameVariableManager::make_entry("G_Random_Skills", CONFIG.lock().unwrap().random_skill as i32);
    GameVariableManager::make_entry("G_Random_Grow_Mode", CONFIG.lock().unwrap().random_grow as i32);
    GameVariableManager::make_entry("G_Random_God_Mode",  CONFIG.lock().unwrap().random_god_mode as i32);
    CONFIG.lock().unwrap().save();
    person::randomize_person();
    emblem::randomize_emblems();
    skill::reset_skills();
    skill::randomize_skills();
    grow::random_grow();
    skill::randomized_god_data();
    write_seed_output_file();
    unsafe { CURRENT_SEED = CONFIG.lock().unwrap().seed as i32; }
}

pub fn reset_gamedata() {
    println!("Resetting GameData");
    JobData::unload();
    JobData::load();
    let jobs = JobData::get_list_mut().unwrap();
    unsafe {
        for j in 0..jobs.len() {
            on_complete(jobs[j], None);
        }
    }

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
    let god = GodData::get_list_mut().unwrap();

    unsafe {
        for g in 0..god.len() {
            god_data_on_complete(god[g], None);
            let ggd = GodGrowthData::try_get_from_god_data(god[g]);
            if ggd.is_some() {
                let growth = ggd.unwrap();
                for x in 0..growth.len() {
                    growth[x].on_complete();
                    ggd_data_on_complete_end(growth[x], None);
                }
            }
        }
    }

    HubDisposData::unload();
    HubDisposData::load();

    skill::reset_skills();
    unsafe { CURRENT_SEED = -1; }
}
pub fn randomize_stuff() {
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

#[unity::from_offset("App", "JobData", "OnCompleted")]
fn on_complete(this: &JobData, method_info: OptionalMethod);

#[unity::from_offset("App", "GodData", "OnCompleted")]
fn god_data_on_complete(this: &GodData, method_info: OptionalMethod);

#[unity::from_offset("App", "GodData", "OnBuild")]
fn god_data_on_build(this: &GodData, method_info: OptionalMethod);

#[unity::from_offset("App", "GodGrowthData", "OnCompleted")]
fn ggd_data_on_complete(this: &GodGrowthData, method_info: OptionalMethod);

#[unity::from_offset("App", "GodGrowthData", "OnCompletedEnd")]
fn ggd_data_on_complete_end(this: &GodGrowthData, method_info: OptionalMethod);

#[unity::from_offset("App", "PersonData", "OnCompletedEnd")]
fn person_data_on_complete_end(this: &PersonData, method_info: OptionalMethod);


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
        if pad_instance.npad_state.buttons.a() {
            if pad_instance.npad_state.buttons.a() {
            // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
                let close_anime_all =
                    unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<ConfigBasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
                    close_anime_all(this.menu, method.method_info);
                });

            // Initialize the menu
                ConfigMenu::create_bind(this.menu);
            
                let config_menu = this.menu.proc.child.cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

            // Register a OnDispose callback to restore the previous menu
                config_menu
                    .get_class_mut()
                    .get_virtual_method_mut("OnDispose")
                    .map(|method| method.method_ptr = open_anime_all_ondispose as _)
                    .unwrap();

                // Clear the buttons in the List so we can add our own
                config_menu.full_menu_item_list.clear();

                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::DeploymentMod>("Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<deploy::EmblemMod>("Emblem Deployment Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<ironman::IronmanMod>("Ironman Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<person::RandomPersonMod>("Unit Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<emblem::RandomEmblemMod>("Emblem Recruitment Order"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<item::RandomJobMod>("Random Classes"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<grow::RandomGrowMod>("Random Growth Mode"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomSkillMod>("Randomize Skills"));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<skill::RandomGodMod>("Randomize Emblem Data"));       
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