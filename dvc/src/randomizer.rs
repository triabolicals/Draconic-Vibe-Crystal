pub use unity::prelude::*;
use skyline::patching::Patch;
pub use engage::{
    dialog::yesno::*,
    unit::Unit,
    gamedata::{dispos::*, god::*, item::RewardData, item::*, ring::RingData, skill::*, *},
    gameuserdata::*,
    gamevariable::*,
    hub::access::*,
    mess::*,
    pad::Pad,
    proc::ProcInst,
    proc::*, random::*,
    script::DynValue,
};
pub use super::enums::*;
pub use super::config::*;
pub use std::sync::{OnceLock, RwLock};

use std::io::Write;
use std::sync::{Mutex, RwLockReadGuard, RwLockWriteGuard};
use engage::gamedata::hub::{HubDisposData, HubFacilityData};
use engage::god::GodPool;
use engage::menu::BasicMenuResult;
use engage::menu::menu_item::config::{ConfigBasicMenuItemCommandMethods};
use crate::utils::{self, can_rand, dlc_check};

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
pub mod data;
pub mod blacklist;
pub(crate) mod latertalk;

use num_traits::FromPrimitive;
use outfit_core::UnitAssetMenuData;
use crate::assets::emblem::{get_random_engage_voice, has_engage_decide};
use crate::randomizer::blacklist::DVCBlackLists;
use crate::randomizer::data::{GameData, RandomizedGameData};
pub use super::{CONFIG, VERSION};

pub static RANDOMIZER_DATA: OnceLock<GameData> = OnceLock::new();
pub static DVC_BLACK_LIST: OnceLock<RwLock<DVCBlackLists>> = OnceLock::new();
pub static RANDOMIZED_DATA: OnceLock<RwLock<RandomizedGameData>> = OnceLock::new();

pub fn get_dvc_black_list_read() ->  RwLockReadGuard<'static, DVCBlackLists> { DVC_BLACK_LIST.get().unwrap().read().unwrap() }
pub fn get_data_read() ->&'static GameData { RANDOMIZER_DATA.get_or_init(||GameData::init()) }
pub fn get_rand_data_read() -> RwLockReadGuard<'static, RandomizedGameData> { RANDOMIZED_DATA.get().unwrap().read().unwrap() }
pub fn get_rand_data_write() -> RwLockWriteGuard<'static, RandomizedGameData> { RANDOMIZED_DATA.get().unwrap().write().unwrap() }

pub static RANDOMIZER_STATUS: RwLock<status::RandomizerStatus> =
    RwLock::new(
    status::RandomizerStatus{
        alear_person_set: false,
        well_randomized: false,
        enabled: false,
        kizuna_replacements: false,
        map_tile: false,
        learn_skill: false,
        seed: 0,
        inspectors_set: false,
        init: false,
        tilabolical: [0; 1024],
    }
);
/// Tutorial clear and provide DLC seal usages
pub fn tutorial_check() {
    let list = GameVariableManager::find_starts_with("G_解説_");
    if !GameVariableManager::get_bool("G_解説_TUTID_クラスチェンジ") {
        for i in 0..list.len() {
            let string = list[i].to_string();
            GameVariableManager::set_bool(&string, true);
            if string == "G_解説_TUTID_クラスチェンジ" { return; }
        }
    }
    GameVariableManager::find_starts_with("G_進化_").iter()
        .for_each(|key| GameVariableManager::set_bool(key.to_string(), true));

    if dlc_check() && can_rand() {
        GameVariableManager::set_bool("G_CC_エンチャント", true);
        GameVariableManager::set_bool("G_CC_マージカノン", true);
    }

    /*
        if DeploymentConfig::get().debug {
        GameVariableManager::find_starts_with("G_Cleared_M0").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 0));
        GameVariableManager::find_starts_with("G_GmapSpot_").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 3));
        GameData::get_playable_god_list().iter().for_each(|g|{
            if let Some(g_unit) = GodPool::create(g) { g_unit.set_escape(false); }
        });
    }
        GameData::get_playable_god_list().iter().for_each(|g|{
            if let Some(g_unit) = GodPool::create(g) { g_unit.set_escape(false); }
        });
        GameVariableManager::find_starts_with("G_Cleared_M01").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 0));
        GameVariableManager::find_starts_with("G_Cleared_M02").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 0));
        GameVariableManager::find_starts_with("G_GmapSpot_").iter().for_each(|key| GameVariableManager::set_number(key.to_string(), 3));
    }}

     */
}
pub fn write_seed_output_file() -> bool {
    false
        /*
    let seed = GameVariableManager::get_number(DVCVariables::SEED);
    let _ = fs::create_dir_all("sd:/Draconic Vibe Crystal/");
    let filename = format!("sd:/Draconic Vibe Crystal/{}.log", utils::get_player_name());
    if let Ok(mut f) = File::options().create(true).write(true).truncate(true).open(filename) { ;
        writeln!(&mut f, "------------- Triabolical Draconic Vibe Crystal Output - Version {} -------------", VERSION).unwrap();

        writeln!(&mut f, "* Seed: {}", seed).unwrap();
        let continuous = GameVariableManager::get_number(DVCVariables::CONTINUOUS);
        match continuous {
            1|2 => { writeln!(&mut f, "* Map Progression Mode: ").unwrap(); }
            3 => { writeln!(&mut f, "* Map Progression Mode: Random").unwrap(); }
            4 => { writeln!(&mut f, "* Open World Map").unwrap(); }
            5 => { writeln!(&mut f, "* Open World Map with Scaling").unwrap(); }
            _ => {}
        };
        if DVCFlags::Ironman.get_value() { writeln!(&mut f, "* Ironman Mode").unwrap(); }
        writeln!(&mut f, "** Recruitment Settings **").unwrap();
        writeln!(&mut f, "\t* Unit Recruitment Order: {}",
            match GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) {
                1 => { "Random" }
                2 => { "Reverse" }
                3 => { "List" }
                _ => { "Normal" }
            }
        ).unwrap();
        writeln!(&mut f, "\t* Emblem Recruitment Order: {}",
                 match GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) {
                     1 => { "Random" }
                     2 => { "Reverse" }
                     3 => { "List" }
                     _ => { "Normal" }
                 }
        ).unwrap();
        writeln!(&mut f, "\t* Emblem Random Recruitment with Custom Emblems: {}", DVCLocalizer::on_off(DVCFlags::CustomEmblemsRecruit.get_value())).unwrap();
        writeln!(&mut f, "** Emblem Randomization Settings **").unwrap();
        writeln!(&mut f, "\t* Emblem Engage Attack Randomization: {}", DVCLocalizer::on_off(DVCFlags::EngageAttacks.get_value())).unwrap();
        writeln!(&mut f, "\t* Emblem Engage Weapon Randomization: {}", DVCLocalizer::on_off(DVCFlags::EngageWeapons.get_value())).unwrap();
        writeln!(&mut f, "\t* Emblem Sync Stat Randomization: {}", DVCLocalizer::on_off(DVCFlags::EmblemStats.get_value())).unwrap();
        writeln!(&mut f, "\t* Engrave Variance Randomization Level: {}", GameVariableManager::get_number(DVCVariables::ENGRAVE_KEY)).unwrap();
        writeln!(&mut f, "\t* Emblem Skill Inheritance: {}",
            match GameVariableManager::get_number(DVCVariables::EMBLEM_INHERIT_MODE) { 1 => { "Random" }, 2 => { "Chaos" }, 3 => { "Unit" }, _ => { "Normal" } }).unwrap();
        writeln!(&mut f, "\t* Emblem Sync Skill Randomization: {}", 
                 match GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY) { 1 => { "Random" }, 2 => { "Chaos" }, _ => { "Normal" } }
        ).unwrap();
        
        writeln!(&mut f, "\t* Emblem Engage Skill Randomization: {}",
             match GameVariableManager::get_number(DVCVariables::EMBLEM_ENGAGE_SKILL_KEY){ 1 => { "Random" }, 2 => { "Chaos" }, _ => { "Normal" } }
        ).unwrap();


        writeln!(&mut f, "* Personal Skill Randomization: {}", DVCLocalizer::on_off(DVCFlags::PersonalSkills.get_value())).unwrap();


        //  writeln!(&mut f, "* Random Classes: {}", GameVariableManager::get_bool(DVCVariables::JOB_KEY)).unwrap();
        match GameVariableManager::get_number(DVCVariables::SKILL_KEY) {
            1|4 => writeln!(&mut f, "* Random Skills: Personal").unwrap(),
            2 => writeln!(&mut f, "* Random Skills: Class").unwrap(),
            3 => writeln!(&mut f, "* Random Skills: Personal + Class").unwrap(),
            _ => writeln!(&mut f, "* Random Skills: None").unwrap(),
        };
        writeln!(&mut f, "* Random Items: {}", GameVariableManager::get_bool(DVCVariables::ITEM_KEY)).unwrap();
        let growth_mode = GameVariableManager::get_number(DVCVariables::GROWTH_KEY);
        match growth_mode {
            1 => { writeln!(&mut f, "* Growth Rate Mode: Personal").unwrap(); },
            2 => { writeln!(&mut f, "* Growth Rate Mode: Class Mods").unwrap(); },
            3 => { writeln!(&mut f, "* Growth Rate Mode: Personal + Class Mods").unwrap(); },
            _ => { writeln!(&mut f, "* Growth Rate Mode: No Randomization").unwrap(); },
        }
        let sync_mode = GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY) & 255;
        match sync_mode {
            1 => { writeln!(&mut f, "* Emblem Sync Data: Stat Bonuses").unwrap(); },
            2 => { writeln!(&mut f, "* Emblem Sync Data: Sync / Engage Skills").unwrap(); },
            3 => { writeln!(&mut f, "* Emblem Sync Data: Stats / Sync Skills / Engage Skills").unwrap(); },
            _ => { writeln!(&mut f, "* Emblem Sync Data: No Randomization").unwrap(); },
        }
        if DVCFlags::RandomSP.get_value() {
            writeln!(&mut f, "* Random SP Cost").unwrap();
        }
        let data = GameData::get();
        if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 {
            writeln!(&mut f, "\n--------------- Person Recruitment Order ---------------").unwrap();
            data.playables.iter().flat_map(|i| PersonData::try_get_hash(i.hash)).enumerate()
                .for_each(|(index, person)|{
                    let key = format!("G_R_{}", person.pid.to_string());
                    if GameVariableManager::exist(key.as_str()) {
                        let new_pid = GameVariableManager::get_string(key.as_str());
                        writeln!(&mut f, "* {} - {} ({}) -> {} ({})", index, Mess::get_name(person.pid), person.pid, Mess::get_name(new_pid), new_pid).unwrap();
                    }
                    else { writeln!(&mut f, "* {} - {} ({}) -> {} ({})", index, Mess::get_name(person.pid), person.pid, Mess::get_name(person.pid), person.pid).unwrap(); }
                }
                );
        }
        if emblem_mode != 0 {
            writeln!(&mut f, "\n-------------- Emblems Recruitment Order Randomization ---------------").unwrap();
            GameData::get_playable_god_list().iter().enumerate()
                .for_each(|(index, god)|{
                    let key = format!("G_R_{}", god.gid.to_string());
                    if GameVariableManager::exist(key.as_str()) {
                        let new_god = GodData::get(GameVariableManager::get_string(key.as_str())).unwrap();
                        writeln!(&mut f, "* {} - {} ({}) -> {} ({})", index,  Mess::get(god.mid), god.gid,  Mess::get(new_god.mid), new_god.gid).unwrap();
                    }
                    else { writeln!(&mut f, "* {} - {} ({}) -> {} ({})", index,  Mess::get(god.mid), god.gid,   Mess::get(god.mid), god.gid).unwrap(); }
                }
                )
        }
        if GameVariableManager::get_number("G_Random_Grow_Mode") & 1 != 0 {
            writeln!(&mut f, "\n--------------- Personal Growth Rates Randomization ---------------").unwrap();
            PersonData::get_list().unwrap().iter()
                .filter(|p| p.parent.index > 0 && !p.get_grow().is_zero())
                .for_each(|person|{
                    writeln!(&mut f, "* {} - {}", person.parent.index, utils::get_person_growth_line(person)).unwrap();
                }
                );
        }
        if GameVariableManager::get_number("G_Random_Grow_Mode") & 2 != 0 {
            writeln!(&mut f, "\n--------------- Class Growth Rates Modifers Randomization ---------------").unwrap();
            JobData::get_list_mut().unwrap().iter()
                .filter(|job| job.parent.index > 0 && !job.get_diff_grow().is_zero())
                .for_each(|job|{
                    let grow = job.get_diff_grow();
                    writeln!(&mut f, "* {} - {} ({})\n\t| {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% | {} {}% |", job.parent.index, Mess::get_name(job.jid), job.jid,
                             Mess::get("MID_SYS_HP").to_string(), grow[0], Mess::get("MID_SYS_Str").to_string(), grow[1], Mess::get("MID_SYS_Mag").to_string(), grow[6],
                             Mess::get("MID_SYS_Tec").to_string(), grow[2], Mess::get("MID_SYS_Spd").to_string(), grow[3], Mess::get("MID_SYS_Lck").to_string(), grow[4],
                             Mess::get("MID_SYS_Def").to_string(), grow[5], Mess::get("MID_SYS_Res").to_string(), grow[7], Mess::get("MID_SYS_Phy").to_string(), grow[8]).unwrap();
                }
                );
        }
        if GameVariableManager::get_number(DVCVariables::SKILL_KEY) != 0 {
            writeln!(&mut f, "\n--------------- Personal Skills Randomization ---------------").unwrap();
            data.playables.iter()
                .flat_map(|i| PersonData::try_get_hash(i.hash))
                .for_each(|person|{
                    let skill_name =
                        person.get_common_skills().iter()
                            .flat_map(|skill| skill.get_skill())
                            .find(|skill| skill.flag & 1 == 0)
                            .and_then(|skill| skill.name)
                            .map_or_else(|| "-".into(), |msid| Mess::get(msid));
                    writeln!(&mut f, "* {} ({}):\t{}",  Mess::get_name(person.pid), person.pid, skill_name).unwrap();
                }
                );
            writeln!(&mut f, "\n--------------- Class Learn Skills --------------").unwrap();
            JobData::get_list_mut().unwrap().iter().filter(|job| job.learn_skill.is_some())
                .for_each(|job|{
                    let learn_skill_name = job.learn_skill
                        .and_then(|sid| SkillData::get(sid))
                        .and_then(|skill| skill.name)
                        .map_or_else(|| "-----".into(), |name| Mess::get(name) );

                    writeln!(&mut f, "* {} - {} ({}):\t {}", job.parent.index, Mess::get_name(job.jid), job.jid, learn_skill_name).unwrap();
                }
                );
            writeln!(&mut f, "\n--------------- Bond Ring Skill Randomization --------------").unwrap();
            let bond_ring_rates =crate::DeploymentConfig::get().get_bond_ring_rates();
            let ranks = ["S", "A", "B", "C"];
            for x in 0..4 { writeln!(&mut f, "-- {} Rank Rate: {}", ranks[x as usize], bond_ring_rates[x as usize]).unwrap(); }
            RingData::get_list().unwrap().iter()
                .for_each(|ring|{
                    let skills = utils::skill_array_string(ring.get_equip_skills());
                    if skills.len() > 0 {
                        let rank = ranks.get(ring.rank as usize).map_or_else(||"??", |f| *f);
                        let god_name = ring.gid.map_or_else(|| "".into(), |f| Mess::get(GodData::get(f).unwrap().mid));
                        writeln!(&mut f, "* {} {} {}:\n\tSkills: {}", god_name, Mess::get(ring.name), rank, skills).unwrap();
                    }
                }
                );
        }
        if GameVariableManager::get_number("G_InteractSetting") != 0 {
            let kinds = ["None", "Sword", "Lance", "Axe", "Bow", "Dagger", "Tome", "Rod", "Arts", "Special"];
            let interact_list = interact::InteractData::get_list().unwrap();
            writeln!(&mut f, "\n--------------- Weapon Triangle Interactions ---------------").unwrap();
            for x in 1..10 {
                let mut string = format!("{}: ", kinds[x]);
                let flag_value = interact_list[x].flag.value;
                for y in 1..10 {
                    if flag_value & ( 1 << y ) != 0 { string = format!("{}{} (S) ", string, kinds[y]); }
                    if flag_value & ( 1 << (y + 10) ) != 0 { string = format!("{}{} (W) ", string, kinds[y]); }
                }
                writeln!(&mut f, "#{} - {}", x, string).unwrap();
            }
            for x in 1..10 {
                let mut string = format!("{}: ", kinds[x]);
                let flag_value = interact_list[x].flag.value;
                for y in 1..10 {
                    if flag_value & ( 1 << y ) != 0 { string = format!("{}{} (S) ", string, kinds[y]); }
                    if flag_value & ( 1 << (y + 10) ) != 0 { string = format!("{}{} (W) ", string, kinds[y]); }
                }
                writeln!(&mut f, "# Reversed {} - {}", x, string).unwrap();
            }
        }
        if GameVariableManager::get_number("G_Random_God_Mode") >= 2 {
            writeln!(&mut f, "\n--------------- Emblem Engage / Linked Engage Attack Randomization ---------------").unwrap();
            emblem_list.iter().flat_map(|&h| GodData::try_get_hash(h))
                .for_each(|god|{
                    writeln!(&mut f, "{}", crate::message::god_engage_random_str(god)).unwrap();
                }
                );
        }
        writeln!(&mut f, "\n--------------- Emblem Engrave Data ---------------").unwrap();
        emblem_list.iter().flat_map(|&h| GodData::try_get_hash(h))
            .for_each(|god|{
                let line = format!("* {} - \t{}: {}, {}: {}, {}: {}, {}: {}, {}: {}, {}: {}",
                                   Mess::get(god.mid),
                                   utils::get_stat_label(11), god.get_engrave_avoid(),  utils::get_stat_label(12), god.get_engrave_critical(), utils::get_stat_label(13), god.get_engrave_hit(),
                                   utils::get_stat_label(14), god.get_engrave_power(), utils::get_stat_label(15), god.get_engrave_secure(), utils::get_stat_label(16), god.get_engrave_weight()
                );
                writeln!(&mut f, "{}", line).unwrap();
            }
            );
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
        emblem_list.iter().flat_map(|&h| GodData::try_get_hash(h)).enumerate()
            .for_each(|(index, god)|{
                let level_data = god.get_level_data().unwrap();
                let grow_data = GodGrowthData::try_get_from_god_data(god).unwrap();
                let engage_skill = level_data[0].engage_skills.list.item[0].get_skill().map_or_else(|| String::from(" ------- "), |skill| Mess::get(skill.name.unwrap()).to_string());
                let god_name =                  Mess::get(god.mid);
                writeln!(&mut f, "\n****** {} *******\nEngage Skill: {}, Engage Atk/Link: {} / {} with ({} / {} )\n",
                         god_name,
                         engage_skill,
                         utils::get_skill_name_from_sid(god.get_engage_attack()), crate::message::god_link_engage_atk_str(god),
                         crate::message::god_link_god(god), crate::message::god_link_pid(god)
                ).unwrap();

                let blevels = [1, 10, 15];
                for weapon_slot in 0..3 {
                    writeln!(&mut f, "\t* Engage Weapons {}: {}",  blevels[weapon_slot], emblem::emblem_item::ENGAGE_ITEMS.lock().unwrap().print(index as i32,  weapon_slot as i32)).unwrap();
                }
                writeln!(&mut f, "").unwrap();
                for y in 1..level_data.len() {
                    writeln!(&mut f, "\t* {} Lv. {} Stats: {}", god_name, y, utils::stats_from_skill_array(level_data[y].synchro_skills)).unwrap();
                    writeln!(&mut f, "\t\tSyncho Skills:  {}", utils::skill_array_string(level_data[y].synchro_skills)).unwrap();
                    writeln!(&mut f, "\t\tEngaged Skills: {}", utils::skill_array_string(level_data[y].engaged_skills)).unwrap();
                    if y-1 < grow_data.len() {
                        let level = grow_data[y-1].get_inheritance_skills();
                        if level.is_none() { writeln!(&mut f, "").unwrap(); continue;}
                        let inherit_skills = level.unwrap();
                        writeln!(&mut f, "\t\tInherit Skills: {}", utils::sid_array_string(inherit_skills)).unwrap();
                    }
                    writeln!(&mut f, "").unwrap();
                }
            }
            );
        println!("Randomization Print to file");
    }
    else {

    }
    */
}



/// SaveLoad Event Randomizing for Cobalt 1.21+
pub fn save_file_load() {
    tutorial_check();
    let seed_key = DVCVariables::Seed.get_key();
    GameVariableManager::make_entry_norewind(FLAGNAME2, 0);
    GameVariableManager::make_entry_norewind(FLAGNAME, 0);
    if !GameVariableManager::exist(seed_key) { GameVariableManager::make_entry_norewind(seed_key, 0); }
    if !DVCVariables::random_enabled() {  return;  }
    upgrade();
    remove_old_keys();
    if DVCVariables::get_seed() != RANDOMIZER_STATUS.read().unwrap().seed {
        println!("[SaveLoad Event] Randomized Save File Seed {}", DVCVariables::get_seed());
        person::change_lueur_for_recruitment(false);
        if GameUserData::get_sequence() == 5 { person::hub::change_kizuna_dispos(); }
        DVCFlags::Initialized.set_value(false);
        randomize_gamedata(false);
    }
}

/// Main Randomizing Event and after starting NG (include SaveLoad Event if not using Cobalt 1.21)
pub(crate) fn randomize_gamedata(is_new_game: bool) {
    job::single::single_class_exists();
    let sequence = GameUserData::get_sequence();
    println!("[RandomizeGameData] Is New Game {}", is_new_game);
    emblem::randomize_emblems();
    utils::get_lueur_name_gender();
    person::randomize_person();
    person::change_lueur_for_recruitment(is_new_game);
    crate::continuous::continuous_mode_data_edit();
    if !DVCFlags::Initialized.get_value() || is_new_game {
        let data = GameData::get();
        let mut random = get_rand_data_write();
        random.randomize(data);
        random.commit(data);
    }
    if sequence == 5 { person::hub::change_kizuna_dispos(); }
    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
        lock.seed = DVCVariables::get_seed();
        lock.enabled = true;
    }
    if GameVariableManager::get_number(DVCVariables::LIBERATION_TYPE) != 0  { item::change_liberation_type(); }
    if GameVariableManager::get_number(DVCVariables::MISERCODE_TYPE) != 0 { item::change_misercode_type(); }
    for x in 0..33 {
        if let Some(key) = DVCVariables::from(x).map(|v| v.get_key()) { DVCVariables::log_variable(key); }
    }

    println!("Game data randomized");
}

/// Used to randomized enemy emblem stuff if loading save from map
pub fn in_map_randomize() {
    person::unit::reload_all_actors();
}
/// Routine after NG is started to randomize gamedata
pub fn start_new_game(){
    DeploymentConfig::get().correct_rates();
    GameVariableManager::make_entry("G_DVC_Version", 5);
    let seed = DeploymentConfig::get().seed;
    // Settings that does not get added
    GameVariableManager::make_entry_norewind(DVCVariables::DVC_STATUS, 0);

    let iron_man = DeploymentConfig::get().ironman;
    let randomized = DeploymentConfig::get().randomized;
    let ran_seed =
        if randomized {
            if seed == 0 { utils::get_random_number_for_seed() } else { seed }
        }
        else { 0 };

    println!("Starting new game Seed: {} [{}]", ran_seed, randomized);
    GameVariableManager::make_entry(DVCVariables::SEED, ran_seed as i32);
    DeploymentConfig::get().create_game_variables(randomized);
    if iron_man { crate::ironman::ironman_code_edits(); }
    DVCFlags::Initialized.set_value(false);
    if randomized { randomize_gamedata(true); }

    let asset_data = UnitAssetMenuData::get();
    asset_data.data.clear();
    let random_appearance = if DVCFlags::PlayerAppearance.get_value() { 8 } else { 0 };
    GameData::get().playables.iter().for_each(|p|{ 
        asset_data.data.push(outfit_core::UnitAssetData::new_hash(p.hash, random_appearance != 0));
    });
    if let Some(starting_chapter) = ChapterData::get("CID_M001") {
        GameUserData::set_chapter(starting_chapter);
        GameVariableManager::set_bool("G_Cleared_M000", true);
    }
}
pub fn reload<T: Gamedata>() {
    T::unload();
    loop {
        T::load();
        if T::get_count() > 0 { break;}
        println!("FAILED TO RELOAD: {}", T::class().get_name());
    }
}
/// Resets all gamedata to normal when returning to the title screen
pub fn reset_gamedata() {
    if RANDOMIZER_STATUS.try_write().map(|v| v.seed == 0 ).unwrap_or(false) { return; }
    println!("Resetting GameData");
    reload::<ItemData>();
    ItemData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed());
    reload::<InteractData>();
    reload::<JobData>();
    JobData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed() );
    job::correct_job_base_stats();
    reload::<PersonData>();
    reload::<GodData>();
    PersonData::get_list_mut().unwrap().iter().for_each(|x| x.on_completed() );
    person::check_playable_classes();
    GodGrowthData::unload();
    GodGrowthData::load();
    reload::<RingData>();
    GodData::get_list_mut().unwrap().iter()
        .for_each(|god|{
            god.on_completed();
            if let Some(growth) = GodGrowthData::try_get_from_god_data(god) { growth.iter().for_each(|level| level.on_completed()); }
        }
    );
    engage_count();

    // GodGrowthData::on_completed_end();
    HubDisposData::unload();
    HubDisposData::load();

    RewardData::unload();
    RewardData::load();
    HubFacilityData::unload();
    HubFacilityData::load();
    reload::<ChapterData>();
    reload::<SkillData>();
    SkillData::get_list().unwrap().iter().for_each(|skill| skill.on_completed() );
    SkillData::try_index_get(0).unwrap().on_completed_end();

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

    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
        lock.reset();
        println!("Randomizer Status is reset");
    }
}
fn upgrade() {
    if !GameVariableManager::exist("G_DVC_Version") {
        GameVariableManager::make_entry_norewind("G_DVC_Version", 1);
    }
    let version = GameVariableManager::get_number("G_DVC_Version");
    // if version < 2 { migrate_to_v2(); }
    if version < 3 { migrate_to_v3(); }
    if version < 5 { migrate_to_v5(); }

}
pub fn randomize_stuff() {

    upgrade();
    for x in 0..10 {    // ShopItems
        GameVariableManager::make_entry_norewind(format!("G_DVC_I{}", x).as_str(), 0);
        GameVariableManager::make_entry_norewind(format!("G_DVC_W{}", x).as_str(), 0);
    }
    println!("[DVC Randomization] {}", can_rand());
    if !can_rand() {  return;  }
    if RANDOMIZER_STATUS.read().unwrap().seed == 0 {
        DeploymentConfig::get().correct_rates();
        tutorial_check();
    }
    if DVCVariables::get_seed() != RANDOMIZER_STATUS.read().unwrap().seed {
        println!("Randomized Stuff with Save File Seed {}", DVCVariables::get_seed());
        randomize_gamedata(false);

        if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() {
            lock.enabled = true;
            lock.seed =  DVCVariables::get_seed();
        }
    }
}

pub fn initialize_game_data() {
    DVC_BLACK_LIST.get_or_init(|| RwLock::new(DVCBlackLists::init()));
    RANDOMIZER_DATA.get_or_init(|| GameData::init());
    let emblems = GameData::get_playable_god_list().len();
    let playables = GameData::get().playables.len();
    RANDOMIZED_DATA.get_or_init(|| RwLock::new(RandomizedGameData::new(emblems, playables)));
    bgm::initalize_bgm_pool();
    person::ai::create_custom_ai();
    emblem::initialize_emblem_list();
    engage_count();
    person::check_playable_classes();
    crate::assets::data::initialize_search_list();
    crate::talk::fill_name_array();
    job::correct_job_base_stats();
    DeploymentConfig::get().seed = 0;
    println!("Finished Initialization GameData");
}

pub fn engage_count() {
    let god_data = GodData::get_list_mut().unwrap();
    god_data.iter_mut()
        .filter(|god| god.engage_count > 0)
        .for_each(|god| god.engage_count = 7);

    emblem::ENEMY_EMBLEM_LIST.get().unwrap()
        .iter()
        .flat_map(|&x|GodData::try_index_get_mut(x))
        .for_each(|x| { x.force_type = 1; });
}

#[skyline::hook(offset=0x02291fd0)]
pub fn person_sound(
    person_switch_name: &Il2CppString,
    engage_switch_name: Option<&Il2CppString>,
    event_name: &Il2CppString,
    character: u64,
    method_info: OptionalMethod)
{
    println!("Voice Event Name: {} / {}", event_name, person_switch_name);
    if event_name.to_string() == "V_Engage_Respond" {
        let person_switch = person_switch_name.to_string();
        if person_switch == "Lueur1" {
            call_original!("PlayerM".into(), engage_switch_name, event_name, character, method_info);
            return;
        }
        else if person_switch == "Lueur2" {
            call_original!("PlayerF".into(), engage_switch_name, event_name, character, method_info);
            return;
        }
    }
    else if event_name.to_string() == "V_Engage_Decide" {
        let person_switch = person_switch_name.to_string();
        let new_voice =
            match person_switch.as_str() {
                "PlayerM_Boss" => { "PlayerM" },
                "PlayerF_Boss" => { "PlayerF" },
                "Veyre_Boss" => { "Veyre" },
                "Sepia" => { "DLC_44" },
                "Gris" => { "DLC_45"},
                "Marron" => { "DLC_46" },
                _ => {
                    if !has_engage_decide(person_switch.as_str()) { get_random_engage_voice() }
                    else { person_switch.as_str() }
                }
            };
        if let Some(engage_switch) = engage_switch_name {
            if engage_switch.contains("PlayerM") {
                call_original!(new_voice.into(), Some("Lueur1".into()), event_name, character, method_info);
                return;
            }
            else if engage_switch.contains("PlayerF") {
                call_original!(new_voice.into(), Some("Lueur2".into()), event_name, character, method_info);
                return;
            }
        }
        call_original!(new_voice.into(), engage_switch_name, event_name, character, method_info);
        return;
    }
    call_original!(person_switch_name, engage_switch_name, event_name, character, method_info);
}

pub trait Randomizer<T> {
    fn get_random_element(&self, rng: &Random) -> Option<&T>;
    fn get_remove(&mut self, rng: &Random) -> Option<T>;
    fn get_remove_filter(&mut self, rng: &Random, filter: impl Fn(&T) -> bool ) -> Option<T>;
    fn get_filter(&self, rng: &Random, filter: impl Fn(&T) -> bool) -> Option<&T>;
    fn shuffle(&mut self, rng: &Random, cycles: i32);
}
impl<T> Randomizer<T> for Vec<T> {
    fn get_random_element(&self, rng: &Random) -> Option<&T> {
        let len = self.len();
        if len > 1 { self.get(rng.get_value( len as i32) as usize) }
        else if len == 1 { self.get(0) }
        else { None }
    }
    fn get_remove(&mut self, rng: &Random) -> Option<T> {
        let len = self.len();
        let selection = if len > 1 { rng.get_value( len as i32) as usize } else { 0 };
        if len > 0 { Some(self.swap_remove(selection)) }
        else { None }
    }
    fn get_remove_filter(&mut self, rng: &Random, filter: impl Fn(&T) -> bool ) -> Option<T> {
        let list: Vec<usize> = self.iter().enumerate()
            .filter(|(_, element)| filter(element))
            .map(|(index, element)| index).collect();
        
        list.get_random_element(rng).map(|&index| self.remove(index))
    }
    fn get_filter(&self, rng: &Random, filter: impl Fn(&T) -> bool ) -> Option<&T> {
        let list: Vec<usize> = self.iter().enumerate()
            .filter(|(_, element)| filter(element))
            .map(|(index, element)| index).collect();
        list.get_random_element(rng).and_then(|&index| self.get(index))
    }
    fn shuffle(&mut self, rng: &Random, cycle: i32) {
        let range = self.len();
        if cycle == 0 || range < 4 { return; }
        for _ in 0..cycle {
            for x in 0..range { self.swap(x, rng.get_value(range as i32) as usize); }
        }
    }
}