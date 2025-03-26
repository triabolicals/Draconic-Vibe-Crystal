use emblem::EMBLEM_LIST;
use person::PLAYABLE;
pub use unity::prelude::*;
use skyline::patching::Patch;
pub use engage::{
    dialog::yesno::*,
    menu::{*, BasicMenuResult, config::{ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    proc::ProcInst,
    gamevariable::*,
    gameuserdata::*,
    hub::access::*,
    mess::*,
    pad::Pad,
    random::*,
    gamedata::{*, unit::*, ring::RingData, item::RewardData, skill::*, item::*, god::*, dispos::*},
};
pub use super::enums::*;
pub use super::config::*;
pub use std::sync::{RwLock, OnceLock};

use std::{fs::{self, File}, io::Write};
use crate::utils::{self, can_rand, dlc_check, fnv_hash_string};

pub mod status;
pub mod bgm;
pub mod grow;
pub mod item;
pub mod person;
pub mod interact;
pub mod styles;
pub mod emblem;
pub mod skill;
pub mod job;
pub mod names;
pub mod map;
pub mod terrain;

use engage::{godpool::GodPool, proc::*, script::DynValue};
use engage::proc::desc::ProcDesc;

pub use super::{CONFIG, VERSION};
pub static mut LINKED: [i32; 20] = [-1; 20];

pub static RANDOMIZER_STATUS: RwLock<status::RandomizerStatus> = RwLock::new(status::RandomizerStatus{
        alear_person_set: false,
        well_randomized: false,
        enemy_emblem_randomized: false,
        enemy_unit_randomized: false,
        emblem_unit_skill_randomized: false,
        skill_randomized: false,
        emblem_data_randomized: false,
        emblem_apt_randomized: false,
        shop_randomized: false,
        enabled: false,
        stat_caps: false,
        accessory: false,
        kizuna_replacements: false,
        map_tile: false,
        learn_skill: false,
        seed: 0,
        continious_random_chapter: String::new(),
        enemy_edelgard: false,
        inspectors_set: false,
    }
);

#[unity::class("App", "SoftwareKeyboard")]
pub struct SoftwareKeyboard {
    pub proc: ProcInstFields,
    pub keyboard_mode: i32,
}
impl Bindable for SoftwareKeyboard {}
#[skyline::from_offset(0x01fddeb0)]
fn keyboard_ctor(this: &SoftwareKeyboard, length: i32, text: &Il2CppString, header: &Il2CppString, sub: &Il2CppString, preset: i32, call_back: u64, method_info: OptionalMethod);

#[skyline::from_offset(0x01fddf50)]
fn keyboard_desc(this: &SoftwareKeyboard, method_info: OptionalMethod) -> &'static Array<&'static mut ProcDesc>;

#[skyline::from_offset(0x01fdea50)]
fn get_result(method_info: OptionalMethod) -> Option<&'static Il2CppString>;

pub fn get_new_seed() -> i32 {
    if let Some(input) = unsafe { get_result(None) } {
        let string = input.to_string();
        if string.len() == 0 { return Random::get_system().value() }
        println!("String: {}", string);
        let parsed = string.parse::<i32>();
        if parsed.is_ok(){
            let value = parsed.unwrap();
            println!("String Value: {}", value);
            if value == 0 { return Random::get_system().value() as i32; }
            return value;
        }
        let hash = unsafe { fnv_hash_string(input, None) };
        if hash < 0 { return -1*hash; }
        else { return hash; }
    }
    Random::get_system().value() as i32
}
pub struct SeedRandomizer;
impl ConfigBasicMenuItemCommandMethods for SeedRandomizer {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance =  engage::util::get_instance::<Pad>();
        if pad_instance.npad_state.buttons.plus() {
            let keyboard = SoftwareKeyboard::instantiate().unwrap();
            let current_seed = format!("{}", crate::utils::get_random_number_for_seed() as i32);
            unsafe { keyboard_ctor(keyboard, 10, current_seed.into(), "Enter value for new save file seed.".into(), format!("Range: 0 to {}. 0 for random seed.", i32::MAX).into(), 0, 0, None) };    
            let desc = unsafe { keyboard_desc(keyboard, None) };
            unsafe { procinst_createbind(keyboard, this.menu, desc, "Keyboard".into(), None) };
            BasicMenuResult::se_cursor()
        }
        else if pad_instance.npad_state.buttons.a(){
            let str = unsafe { get_result(None) }.unwrap();
            if str.to_string().len() > 0 {
                let new_seed = get_new_seed();
                let text = format!("Set seed for New Game saves to {}?", new_seed);
                YesNoDialog::bind::<SeedConfirm>(this.menu, text, "Do it!", "Nah..");
                BasicMenuResult::se_cursor()
            }
            else { BasicMenuResult::new() }
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "Set Seed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        let set_seed = CONFIG.lock().unwrap().seed as i32;
        this.help_text = if set_seed != 0 { format!("Press + to change the set seed. Currently set to: {}", set_seed) }
            else { format!("Press + to manually set seed.") }.into();

    }
}

pub struct ReseedRandomizer;
impl ConfigBasicMenuItemCommandMethods for ReseedRandomizer {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance =  engage::util::get_instance::<Pad>();
        if pad_instance.npad_state.buttons.plus() {
            let keyboard = SoftwareKeyboard::instantiate().unwrap();
            let current_seed = format!("{}", GameVariableManager::get_number(DVCVariables::SEED));
            let desc = unsafe { keyboard_desc(keyboard, None) };
            unsafe { keyboard_ctor(keyboard, 10, current_seed.into(), "Enter New Seed".into(), format!("Range: 0 to {}. 0 for random seed.", i32::MAX).into(), 0, 0, None) };    
            keyboard.keyboard_mode = 1;
            unsafe { procinst_createbind(keyboard, this.menu, desc, "Keyboard".into(), None) };
            BasicMenuResult::se_cursor()
        }
        else if pad_instance.npad_state.buttons.a(){
            let new_seed = get_new_seed();
            let text = format!("Change randomization seed to {}?\nRecruitment order will not be affected.\nRequires saving and reloading.", new_seed);
            GameVariableManager::make_entry("NewSeed", new_seed as i32);
            GameVariableManager::set_number("NewSeed", new_seed as i32);
            YesNoDialog::bind::<ReseedConfirm>(this.menu, text, "Do it!", "Nah..");
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { this.command_text = "Reseed".into(); }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) { 
        this.help_text = format!("Press + to manually set Seed. Current: {}", GameVariableManager::get_number(DVCVariables::SEED)).into();
    }
}

pub struct SeedConfirm;
impl TwoChoiceDialogMethods for SeedConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        let new_seed = get_new_seed();
        unsafe {
            CONFIG.lock().unwrap().seed = new_seed as u32;
            CONFIG.lock().unwrap().save();
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap());
            let index = menu.select_index;
            SeedRandomizer::set_help_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
} 

pub struct ReseedConfirm;
impl TwoChoiceDialogMethods for ReseedConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        reseed();
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        ReseedRandomizer::set_help_text(menu.menu_item_list[index as usize], None);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
} 

pub extern "C" fn vibe_reseed() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_command::<ReseedRandomizer>("Change Randomizer Seed");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::not_in_map_sortie_build_attr as _);
    switch
}
pub extern "C" fn vibe_seed() -> &'static mut ConfigBasicMenuItem {  ConfigBasicMenuItem::new_command::<SeedRandomizer>("Set New Game Seed") }

/// Tutorial clear and provide DLC seal usages
pub fn tutorial_check(){
    let list = GameVariableManager::find_starts_with("G_解説_");
    if !GameVariableManager::get_bool("G_解説_TUTID_クラスチェンジ") {
        for i in 0..list.len() {
            let string = list[i].to_string();
            GameVariableManager::set_bool(&string, true);
            if string == "G_解説_TUTID_クラスチェンジ" { return; }
        }
    }
    GameVariableManager::find_starts_with("G_進化_").iter().for_each(|key| GameVariableManager::set_bool(key.to_string(), true));
    if dlc_check() && can_rand() {
        GameVariableManager::set_bool("G_CC_エンチャント", true);
        GameVariableManager::set_bool("G_CC_マージカノン", true);
    }
    if CONFIG.lock().unwrap().debug {
        GameVariableManager::find_starts_with("G_GmapSpot_").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 3));
        EMBLEM_LIST.get().unwrap().iter().flat_map(|&i| GodData::try_get_hash(i))
            .for_each(|god| { GodPool::create(god); });
    }
}
#[skyline::from_offset(0x01fde850)]
pub fn keyboard_create_bind<P: Bindable>(this: &P, length: i32, text: &Il2CppString, header: &Il2CppString, sub: &Il2CppString, preset: i32, call_back: u64, method_info: OptionalMethod);

pub fn write_seed_output_file() {
    let seed = GameVariableManager::get_number(DVCVariables::SEED);
    let _ = fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    let filename = format!("sd:/Draconic Vibe Crystal/{}.log", utils::get_player_name());
    let file = File::options().create(true).write(true).truncate(true).open(filename);
    let mut f = if file.is_err() { println!("Cannot create output file");
        File::options().create(true).write(true).truncate(true).open("sd:/Draconic Vibe Crystal/output.log") }
         else { file}.unwrap();
    writeln!(&mut f, "------------- Triabolical Randomization Settings - Version {} -------------", VERSION).unwrap();
    if GameVariableManager::get_bool("G_Ironman") { writeln!(&mut f, "* Ironman Mode").unwrap(); }
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) != 0 { writeln!(&mut f, "* Continuious Mode").unwrap(); }
    writeln!(&mut f, "* Seed: {}", seed).unwrap();
    match GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) {
        1 => { writeln!(&mut f, "* Random Recruitment").unwrap(); },
        2 => { writeln!(&mut f, "* Reverse Recruitment").unwrap();}
        3 => { writeln!(&mut f, "* Custom Recruitment").unwrap(); },
        _ => {},
    }
    let emblem_mode =  GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY);
    match emblem_mode {
        1 => { writeln!(&mut f, "* Emblem Recruitment Mode: Random").unwrap();  },
        2 => { writeln!(&mut f, "* Emblem Recruitment Mode: Reverse").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Recruitment Mode: Custom").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Recruitment Mode: Normal").unwrap();  },
    }
    let god_mode =  GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_KEY);
    if god_mode == 0 { writeln!(&mut f, "* Emblem Data: No Randomization").unwrap();  }
    else if god_mode == 1 { writeln!(&mut f, "* Emblem Data: Inheritable Skills").unwrap();  }
    else if god_mode == 2 { writeln!(&mut f, "* Emblem Data: Engage Attack / Engage Link").unwrap(); }
    else if god_mode == 3 { writeln!(&mut f, "* Emblem Data: Inheritable / Attack / Link").unwrap(); }
    writeln!(&mut f, "* Random Classes: {}", GameVariableManager::get_bool(DVCVariables::JOB_KEY)).unwrap();
    writeln!(&mut f, "* Random Skills: {}", GameVariableManager::get_bool(DVCVariables::SKILL_KEY)).unwrap();
    writeln!(&mut f, "* Random Items: {}", GameVariableManager::get_bool(DVCVariables::ITEM_KEY)).unwrap();
    let growth_mode = GameVariableManager::get_number(DVCVariables::GROWTH_KEY);
    match growth_mode {
        1 => { writeln!(&mut f, "* Growth Rate Mode: Personal").unwrap(); },
        2 => { writeln!(&mut f, "* Growth Rate Mode: Class Mods").unwrap(); },
        3 => { writeln!(&mut f, "* Growth Rate Mode: Personal + Class Mods").unwrap(); },
        _ => { writeln!(&mut f, "* Growth Rate Mode: No Randomization").unwrap(); },
    }
    let sync_mode = GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY);
    match sync_mode {
        1 => { writeln!(&mut f, "* Emblem Sync Data: Stat Bonuses").unwrap(); },
        2 => { writeln!(&mut f, "* Emblem Sync Data: Sync / Engage Skills").unwrap(); },
        3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
        _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
    }
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
        writeln!(&mut f, "\n--------------- Person Recruitment Order ---------------").unwrap();
        let mut count = 0;
        let playable = PLAYABLE.get().unwrap();
        playable.iter().for_each(|&pindex|{
            let person = PersonData::try_index_get(pindex).unwrap();
            let name1 = Mess::get_name(person.pid).to_string();
            let key = format!("G_R_{}", person.pid.to_string());
            count +=1; 
            if GameVariableManager::exist(key.as_str()) {
                let new_pid = GameVariableManager::get_string(key.as_str());
                let mut name2 = String::new();
                if PersonData::get(&new_pid.to_string()).is_some() {
                    name2 = Mess::get_name(new_pid).to_string();
                } 
                writeln!(&mut f, "* {} - {} ({}) -> {} ({})", count, name1, person.pid.to_string(), name2, new_pid.to_string()).unwrap();
            }
            else { writeln!(&mut f, "* {} - {} ({}) -> {} ({})", count, name1, person.pid.to_string(), name1, person.pid.to_string()).unwrap(); }
        });
    }
    if emblem_mode != 0 {
        writeln!(&mut f, "\n-------------- Emblems Recruitment Order Randomization ---------------").unwrap();
        let mut count = 0;
        for x in 0..19 { 
            let string = format!("G_R_{}", EMBLEM_GIDS[x]);
            let name1 = Mess::get( GodData::get(EMBLEM_GIDS[x]).unwrap().mid ).to_string();
            let new_gid = GameVariableManager::get_string(&string);
            let mut name2 = String::new();
            if GodData::get(&new_gid.to_string()).is_some() { name2 = Mess::get( GodData::get(&new_gid.to_string()).unwrap().mid).to_string(); }
            count += 1;
            writeln!(&mut f, "* {} - {} ({}) -> {} ({})", count, name1, x, name2, new_gid.to_string()).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_Grow_Mode") & 1 != 0 {
        writeln!(&mut f, "\n--------------- Personal Growth Rates Randomization ---------------").unwrap();
        let person_list = PersonData::get_list().unwrap();
        for x in 0..person_list.len() {
            let grow = person_list[x].get_grow();
            if grow.is_zero() { continue; } 
            let line = utils::get_person_growth_line(person_list[x]);
            writeln!(&mut f, "* {} - {}", x+1, line).unwrap();
        }
    }
    if GameVariableManager::get_number("G_Random_Grow_Mode") & 2 != 0 {
        let job_list = JobData::get_list_mut().unwrap();
        writeln!(&mut f, "\n--------------- Class Growth Rates Modifers Randomization ---------------").unwrap();
        for x in 0..job_list.len() {
            let grow = job_list[x].get_diff_grow();
            if grow.is_zero() { continue; } 
            let jid = job_list[x].jid.to_string();
            let job_name = Mess::get(job_list[x].name).to_string();
            writeln!(&mut f, "* {} - {} ({})\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |", x+1, job_name, jid, 
            Mess::get("MID_SYS_HP").to_string(), grow[0], Mess::get("MID_SYS_Str").to_string(), grow[1], Mess::get("MID_SYS_Mag").to_string(), grow[6], 
            Mess::get("MID_SYS_Tec").to_string(), grow[2], Mess::get("MID_SYS_Spd").to_string(), grow[3], Mess::get("MID_SYS_Lck").to_string(), grow[4],
            Mess::get("MID_SYS_Def").to_string(), grow[5], Mess::get("MID_SYS_Res").to_string(), grow[7], Mess::get("MID_SYS_Phy").to_string(), grow[8]).unwrap();
        }
    }
    if GameVariableManager::get_bool(DVCVariables::SKILL_KEY) {
        writeln!(&mut f, "\n--------------- Personal Skills Randomization ---------------").unwrap();
        let playable_size = person::PLAYABLE.get().unwrap().len();
        let person_list = PersonData::get_list().unwrap();
        for x in 0..playable_size {
            let p_index = person::PLAYABLE.get().unwrap()[x as usize] as usize;
            let person = &person_list[p_index]; 
            let mut name = Mess::get(person.get_name().unwrap()).to_string();
            if name.len() == 0 { name = person.get_name().unwrap().to_string(); }
            let personal_sid = person.get_common_sids().unwrap();
            for y in 0..personal_sid.len() {
                let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().to_string(), y);
                let skill = SkillData::get( &personal_sid[y as usize].to_string() ).expect(&error_message);
                if skill.get_flag() & 1 == 0 {
                    let skill_name = Mess::get( SkillData::get(&personal_sid[y as usize].to_string()).unwrap().name.unwrap() ).to_string();
                    let sid = SkillData::get(&personal_sid[y as usize].to_string()).unwrap().sid.to_string();
                    writeln!(&mut f, "* {} ({}):\t{} ({})",  name, person.pid.to_string(), skill_name, sid).unwrap();
                    break;
                }
            }
        }
        writeln!(&mut f, "\n--------------- Class Learn Skill / Lunatic Skill Randomization --------------").unwrap();
        let job_list = JobData::get_list_mut().unwrap();
        for x in 0..job_list.len() {
            let job = &job_list[x as usize];
            let job_name = Mess::get(job.name).to_string();
            let mut string = " ------  ".into();
            let mut string2 = "  ------ ".into();
            if job.learn_skill.is_some() {
                let skill_name = SkillData::get(&job.learn_skill.unwrap().to_string()).unwrap().name.unwrap();
                string = format!("{} ({})", Mess::get( skill_name ).to_string(), job.learn_skill.unwrap().to_string());
            }   
            if job.lunatic_skill.is_some() {
                let skill_name = SkillData::get(&job.lunatic_skill.unwrap().to_string()).unwrap().name.unwrap();
                string2 = format!("{} ({})", Mess::get(skill_name ).to_string(), job.lunatic_skill.unwrap().to_string());
            }
            if job.learn_skill.is_none() && job.lunatic_skill.is_none() { continue;}
            else { 
                let jid = job.jid.to_string();
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
            let skills = utils::skill_array_string(ring_list[x].get_equip_skills());
            if skills.len() <= 2 { continue; }
            let name = Mess::get(ring_list[x].name).to_string();
            let rank;
            if ring_list[x].rank == 3 { rank = "S"; }
            else if ring_list[x].rank == 2 { rank = "A"; }
            else if ring_list[x].rank == 1 { rank = "B"; }
            else { rank = "C"; }
            if ring_list[x].gid.is_some() { writeln!(&mut f, "* {}: {} {} - {}", utils::get_emblem_name(&ring_list[x].gid.unwrap().to_string()), name, rank, skills).unwrap(); }
            else { writeln!(&mut f, "* {} - {}", name, skills).unwrap(); }
        }
        writeln!(&mut f, "\n--------------- Randomization Skill Pool Availiablity ({} skills)  ---------------", n_skills).unwrap();
        for x in 0..n_skills {
            let skill_index = skill::SKILL_POOL.lock().unwrap()[x as usize].index as usize;
            let skill = &skill_list[skill_index ];
            let sid = skill.sid.to_string();
            let name = Mess::get(skill_list[skill_index ].name.unwrap()).to_string();
            let personal;
            let cost; 
            if skill.get_inheritance_cost() != 0 {
                if skill.can_override_skill() { personal = "Enemy"; }
                else { personal = "Class | Enemy"; }
                cost = format!("{} SP", skill.get_inheritance_cost());
            }
            else {
                if skill.can_override_skill() { personal = "Personal | Enemy"; }
                else { personal = "Personal | Class | Enemy"; }
                cost = "N/A".to_string();
            }
            writeln!(&mut f, "* {} ({}) \t {} \tCost: {}", name, sid, personal, cost).unwrap();
        }
    }
    if GameVariableManager::get_number("G_InteractSetting") != 0 {
        let kinds = ["None", "Sword", "Lance", "Axe", "Bow", "Dagger", "Tome", "Rod", "Arts", "Special"];
        let interact_list = interact::InteractData::get_list().unwrap();
        writeln!(&mut f, "\n--------------- Weapon Triangle Interactions ---------------").unwrap();
        for x in 1..10 {
            let mut string = format!("{}: ", kinds[x]);
            let flag_value = interact_list[x].flag.value;
            for y in 1..10 {
                if flag_value & ( 1 << y ) != 0 {
                    string = format!("{}{} (S) ", string, kinds[y]);
                }
                if flag_value & ( 1 << (y + 10) ) != 0 {
                    string = format!("{}{} (W) ", string, kinds[y]);
                } 
            }
            writeln!(&mut f, "#{} - {}", x, string).unwrap();
        }
        for x in 1..10 {
            let mut string = format!("{}: ", kinds[x]);
            let flag_value = interact_list[x].flag.value;
            for y in 1..10 {
                if flag_value & ( 1 << y ) != 0 {
                    string = format!("{}{} (S) ", string, kinds[y]);
                }
                if flag_value & ( 1 << (y + 10) ) != 0 {
                    string = format!("{}{} (W) ", string, kinds[y]);
                } 
            }
            writeln!(&mut f, "# Reversed {} - {}", x, string).unwrap();
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
    for x in EMBLEM_GIDS {
        let god = GodData::get(x).unwrap();
        let line = format!("* {} - \t{}: {}, {}: {}, {}: {}, {}: {}, {}: {}, {}: {}", 
        utils::mess_get(god.mid), 
        utils::get_stat_label(11), god.get_engrave_avoid(),  utils::get_stat_label(12), god.get_engrave_critical(), utils::get_stat_label(13), god.get_engrave_hit(), 
        utils::get_stat_label(14), god.get_engrave_power(), utils::get_stat_label(15), god.get_engrave_secure(), utils::get_stat_label(16), god.get_engrave_weight());
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
        writeln!(&mut f, "\n*** {} Engage Skill: {}, Engage Atk/Link: {}\n", utils::get_emblem_name(&god_id), utils::get_skill_name(engage_skill), crate::message::god_engage_random_str(&god_id)).unwrap();
        let weapons_str = emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().print(index as i32, 0);
        writeln!(&mut f, "\t* Engage Weapons 1: {}", weapons_str).unwrap();
        let weapons_str2 = emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().print(index as i32, 1);
        writeln!(&mut f, "\t* Engage Weapons 2: {}", weapons_str2).unwrap();
        let weapons_str3 = emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().print(index as i32, 2);
        writeln!(&mut f, "\t* Engage Weapons 3: {}\n", weapons_str3).unwrap();
        for y in 1..level_data.len() {
            writeln!(&mut f, "\t* {} Lv. {} Stats: {}", utils::get_emblem_name(&god_id), y, utils::stats_from_skill_array(level_data[y as usize].synchro_skills)).unwrap();
            writeln!(&mut f, "\t\tSyncho Skills:  {}", utils::skill_array_string(level_data[y as usize].synchro_skills)).unwrap();
            writeln!(&mut f, "\t\tEngaged Skills: {}", utils::skill_array_string(level_data[y as usize].engaged_skills)).unwrap();
            if y-1 < god_grow.len() {
                let level = god_grow[y-1].get_inheritance_skills();
                if level.is_none() { writeln!(&mut f, "").unwrap(); continue;}
                let inherit_skills = level.unwrap();
                writeln!(&mut f, "\t\tInherit Skills: {}", utils::sid_array_string(inherit_skills)).unwrap();
            }
            writeln!(&mut f, "").unwrap();
        }
        index += 1;
    }
    println!("Randomization Print to file");
}

/// Switching PersonData indexes in scripts for event function calls
#[skyline::hook(offset=0x01cb5eb0)]
pub fn try_get_index(dyn_value: &DynValue, index: i32, method_info: OptionalMethod) -> i32 {
    let result = call_original!(dyn_value, index, method_info);
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return result; }
    if let Some(person) = PersonData::try_index_get(result) {
        if utils::is_player_unit(person){
            let new_person = person::switch_person(person);
            return PersonData::get_index( new_person.pid );
        }
    }
    return result;
}
/// SaveLoad Event Randomizing for Cobalt 1.21+
pub fn save_file_load() {
    tutorial_check();
    if !DVCVariables::random_enabled() {  return;  }
    CONFIG.lock().unwrap().create_game_variables_after_new_game();

    if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) == 4 { 
        GameVariableManager::set_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY, 0); 
    }
    
    if DVCVariables::get_seed() != RANDOMIZER_STATUS.read().unwrap().seed {
        if GameVariableManager::get_number(DVCVariables::SHOP_KEY) == 0 && CONFIG.lock().unwrap().random_shop_items {
            GameVariableManager::set_number(DVCVariables::SHOP_KEY,  CONFIG.lock().unwrap().random_shop_items as i32 );
        }
        println!("[SaveLoad Event] Randomized Save File Seed {}", DVCVariables::get_seed());
        
        if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) != 0  { item::change_liberation_type(); }
        if GameVariableManager::get_bool(DVCVariables::JOB_KEY) && GameVariableManager::get_number(DVCVariables::MISERCODE_TYPE) != 0 { item::change_misercode_type(); }

        person::change_lueur_for_recruitment(false);
        skill::randomize_skills();
        skill::replace_enemy_version();
        item::shop::randomize_shop_data();
        // emblem::emblem_skill::randomized_god_data();
        if GameUserData::get_sequence() == 5 { person::hub::change_kizuna_dispos(); }
    }
}

/// Main Randomizing Event and after starting NG (include SaveLoad Event if not using Cobalt 1.21)
fn randomize_gamedata(is_new_game: bool) {
    let sequence = GameUserData::get_sequence();
    println!("Current Chapter: {}, Sequence: {}", GameUserData::get_chapter().cid, sequence);

    emblem::randomize_emblems();
    crate::utils::get_lueur_name_gender();
    person::randomize_person();
    person::change_lueur_for_recruitment(is_new_game);
    skill::randomize_skills();
    item::shop::randomize_shop_data();
    emblem::emblem_skill::randomized_god_data();
    item::randomize_well_rewards();
    if !utils::in_map_chapter()  {   // Some issues when attempting to this when the scene loads
        let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.accessory = true ); 
        skill::replace_enemy_version();
        emblem::enemy::randomize_enemy_emblems();
        emblem::emblem_skill::adjust_emblem_common_skills();
    }
    if sequence == 5 { person::hub::change_kizuna_dispos(); }

    emblem::emblem_item::randomized_emblem_apts();
    emblem::engrave::random_engrave_by_setting( GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY), true);
    emblem::randomize_engage_links(false);
    
    interact::change_interaction_data( GameVariableManager::get_number(DVCVariables::INTERACT_KEY), true);
    grow::random_grow();
    styles::randomize_job_styles();
    if GameVariableManager::get_bool(DVCVariables::EMBLEM_NAME_KEY) { names::randomize_emblem_names();  }

    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
        lock.seed = DVCVariables::get_seed();
        lock.enabled = true;
    }
    item::shop::add_personal_outfits();
    if GameVariableManager::get_number(DVCVariables::JOB_KEY) != 0 { crate::assets::unlock_royal_classes(); }
    println!("Game data randomized");
}

/// Used to randomized enemy emblem stuff if loading save from map
pub fn in_map_randomize() {
    skill::replace_enemy_version();
    emblem::enemy::randomize_enemy_emblems();
    emblem::emblem_skill::adjust_emblem_common_skills();
    person::unit::reload_all_actors();
}
/// Routine after NG is started to randomize gamedata
pub fn start_new_game(){
    *CONFIG.lock().unwrap() = DeploymentConfig::new();
    CONFIG.lock().unwrap().correct_rates();
    let seed = CONFIG.lock().unwrap().seed;
    // Settings that does not get added
    if CONFIG.lock().unwrap().iron_man { 
        GameVariableManager::make_entry(DVCVariables::IRONMAN, 1);
        crate::ironman::ironman_code_edits();
    }
    GameVariableManager::make_entry(DVCVariables::CONTINIOUS, CONFIG.lock().unwrap().continuous);
    if CONFIG.lock().unwrap().randomized {
        if seed == 0 {  GameVariableManager::make_entry(DVCVariables::SEED, utils::get_random_number_for_seed() as i32); }
        else { GameVariableManager::make_entry(DVCVariables::SEED, CONFIG.lock().unwrap().seed as i32); }
        CONFIG.lock().unwrap().create_game_variables(true);
        randomize_gamedata(true);
    }
    else { 
        GameVariableManager::make_entry(DVCVariables::SEED, 0);
        CONFIG.lock().unwrap().create_game_variables(false);
    }
    GameUserData::set_chapter(ChapterData::get("CID_M001").unwrap());
    GameVariableManager::set_bool("G_Cleared_M000", true);
}
/// Resets all gamedata to normal when returning to the title screen
pub fn reset_gamedata() {
    println!("Resetting GameData");
    skill::reset_skills();
    ItemData::unload();
    ItemData::load_data();
    ItemData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed());

    interact::InteractData::unload();
    interact::InteractData::load_data();

    JobData::unload();
    JobData::load();
    JobData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed() );
    job::correct_job_base_stats();
    PersonData::unload();
    PersonData::load();
    let dic = GodData::get_link_dictionary();
    for x in 1..PIDS.len() {
        let person = PersonData::get(PIDS[x]).unwrap();
        if GodData::try_get_link(person).is_some() {
            dic.remove(person.pid);
            person.set_link_god(None);
        }
    }
    names::give_names_to_generics();
    PersonData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed() );
    person::check_playable_classes();
    GodData::unload();
    GodData::load();
    GodGrowthData::unload();
    GodGrowthData::load();
    RingData::unload();
    RingData::load_data();
    item::shop::reset_shopdata();
    GodData::get_list_mut().unwrap().iter()
        .for_each(|god|{
            god.on_completed();
            if let Some(growth) = GodGrowthData::try_get_from_god_data(god) {
                growth.iter().for_each(|level| level.on_complete());
            }
        }
    );
    engage_count();

    GodGrowthData::on_complete_end();
    HubDisposData::unload();
    HubDisposData::load();

    RewardData::unload();
    RewardData::load();
    HubFacilityData::unload();
    HubFacilityData::load_data();
    ChapterData::unload();
    ChapterData::load_data();

    SkillData::unload();
    SkillData::load_data();
    SkillData::get_list().unwrap().iter().for_each(|skill| skill.on_completed() );
    SkillData::try_index_get(0).unwrap().on_completed_end();
    skill::fixed_skill_inherits();
    skill::fix_priority_data();

    emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().reset();
    emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().commit();
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
    Patch::in_text(0x01c77620).bytes(&[0xfd, 0x7b, 0xbc, 0xa9]).unwrap();   // Summon Delete Impl
    Patch::in_text(0x01dee3a8).bytes(&[0x42, 0x00, 0x80, 0x52]).unwrap();
    
    unsafe { for x in 0..20 {  LINKED[x as usize] = -1; }    }
    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
        lock.reset();
        println!("Randomizer Status is reset");
    }

}

pub fn randomize_stuff() {

    if !utils::can_rand() {  return;  }
    if RANDOMIZER_STATUS.read().unwrap().seed == 0 {
        *CONFIG.lock().unwrap() = DeploymentConfig::new();
        CONFIG.lock().unwrap().correct_rates();
        CONFIG.lock().unwrap().create_game_variables_after_new_game();
        tutorial_check();
    }
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) == 4 {  GameVariableManager::set_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY, 0);   }

    if DVCVariables::get_seed() != RANDOMIZER_STATUS.read().unwrap().seed {
        if GameVariableManager::get_number(DVCVariables::SHOP_KEY) == 0 && CONFIG.lock().unwrap().random_shop_items {
            GameVariableManager::set_number(DVCVariables::SHOP_KEY,  CONFIG.lock().unwrap().random_shop_items as i32 );
        }
        println!("Randomized Stuff with Save File Seed {}", DVCVariables::get_seed());
        randomize_gamedata(false);
        if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) != 0  { item::change_liberation_type(); }
        if GameVariableManager::get_bool(DVCVariables::JOB_KEY) && GameVariableManager::get_number(DVCVariables::MISERCODE_TYPE) != 0 { item::change_misercode_type(); }

        item::unit_items::adjust_items();   //Meteor Adjustment
        if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
            lock.enabled = true;
            lock.seed =  DVCVariables::get_seed();
        }
    }
    if DVCVariables::random_enabled() && RANDOMIZER_STATUS.read().unwrap().enabled && !utils::in_map_chapter() && GameUserData::get_sequence() != 5 {
        skill::replace_enemy_version();
        emblem::enemy::randomize_enemy_emblems();
        emblem::emblem_skill::adjust_emblem_common_skills();
    }
}

pub fn intitalize_game_data() {  
    bgm::initalize_bgm_pool();
    person::ai::create_custom_ai();
    person::get_playable_list();
    crate::talk::fill_name_array();
    emblem::initialize_emblem_list();
    // assets::auto_adjust_asset_table( IS_GHAST);
    interact::get_style_interact_default_values();
    skill::create_skill_pool();
    emblem::engrave::get_engrave_stats();
    item::create_item_pool();
    engage_count();
    emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().intialize_list();
    job::correct_job_base_stats();
    grow::get_growth_min_max();
    emblem::emblem_skill::get_pid_emblems();
    person::check_playable_classes();
    person::get_all_enemy_persons();
    emblem::enemy::initalize_dark_emblems();
    skill::fixed_skill_inherits();
    skill::learn::initialize_job_skill_restrictions();
    crate::assets::data::initialize_search_list();
    crate::assets::animation::fix_common_male_swords();
    CONFIG.lock().unwrap().seed = 0;
}
/// Data that does not depend on game data
pub fn intialize_added_data() { 
    // crate::assets::data::HEAD_DATA.get_or_init(||crate::assets::data::person::get_head_data());
}

pub fn engage_count() {
    let god_data = GodData::get_list_mut().unwrap();
    god_data.iter_mut()
        .filter(|god| god.engage_count > 0)
        .for_each(|god| god.engage_count = 7);

    emblem::ENEMY_EMBLEM_LIST.get().unwrap().iter().flat_map(|&x|GodData::try_index_get_mut(x))
        .for_each(|x|{
            x.force_type = 1;
        }
    );
}

pub fn reseed() {
    let seed = GameVariableManager::get_number("NewSeed");
    if seed == GameVariableManager::get_number(DVCVariables::SEED) { return; }
    if seed == 0 {
        let new_seed = Random::get_system().value();
        GameVariableManager::set_number(DVCVariables::SEED, new_seed);
    }
    else {
        GameVariableManager::set_number(DVCVariables::SEED, seed as i32);
    }
    GameVariableManager::set_number("NewSeed", 0);
    let _ = RANDOMIZER_STATUS.try_write().map(|mut lock| lock.seed = GameVariableManager::get_number(DVCVariables::SEED));
    // Set Personal and Job Learn Skill to 0 
    GameVariableManager::find_starts_with("G_P_PID").iter().for_each(|person_key| GameVariableManager::set_number(person_key.to_string().as_str(), 0));
    GameVariableManager::find_starts_with("G_L_JID").iter().for_each(|job_key| GameVariableManager::set_number(job_key.to_string().as_str(), 0));
    // Adaptive Growths
    GameVariableManager::find_starts_with("G_JG_").iter().for_each(|growth_key| GameVariableManager::set_number(growth_key.to_string().as_str(), 0));
    //randomize_gamedata(false);
}

