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
    GameVariableManager::find_starts_with("G_進化_").iter().for_each(|key| GameVariableManager::set_bool(key.to_string(), true));
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
pub fn write_seed_output_file() -> bool { false }



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
}

/// Used to randomized enemy emblem stuff if loading save from map
pub fn in_map_randomize() { person::unit::reload_all_actors(); }

/// Routine after NG is started to randomize gamedata
pub fn start_new_game(){
    DeploymentConfig::get().correct_rates();
    GameVariableManager::make_entry("G_DVC_Version", 5);
    let seed = DeploymentConfig::get().seed;
    GameVariableManager::make_entry_norewind(DVCVariables::DVC_STATUS, 0);

    let iron_man = DeploymentConfig::get().ironman;
    let randomized = DeploymentConfig::get().randomized;
    let ran_seed = if randomized { if seed == 0 { utils::get_random_number_for_seed() } else { seed } } else { 0 };

    println!("Starting new game Seed: {} [{}]", ran_seed, randomized);
    DVCVariables::Seed.init_var(ran_seed as i32, true);
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

    if let Ok(mut lock) = RANDOMIZER_STATUS.try_write() { lock.reset(); }
}
fn upgrade() {
    if !GameVariableManager::exist("G_DVC_Version") { GameVariableManager::make_entry_norewind("G_DVC_Version", 1); }
    let version = GameVariableManager::get_number("G_DVC_Version");
    if version < 3 { migrate_to_v3(); }
    if version < 5 { migrate_to_v5(); }
    if version < 7 {
        for x in 0..19 {
            let s = DVCVariables::get_dvc_emblem_index(x, false);
            DVCVariables::set_emblem_recruitment(x, s as i32);
        }
    }
    GameVariableManager::set_number("G_DVC_Version", 7);
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
/*
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
*/

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