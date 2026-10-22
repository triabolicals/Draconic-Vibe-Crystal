use super::*;
pub use gamedata::*;
pub use engage::{
    app::{singletonclass_1::*, singletonpool_2::*},
    List_1Ext,
};
use std::sync::RwLock;
use cobapi::{Event, SystemEvent};
use engage::{
    app::{
        HubPlayTalkAfter_SelectListMenu, IBasicMenuItemMethods,
        procinst::*, unitedit::*, proc::*, spriteatlasmanager_2::*,
        ChapterData, IStructData_1Methods, IBasicMenu, IBasicMenuMethods, ISingletonPool_2,
        ProcVoidMethod, GodPool,
        Stream_2, UnitPool, FaceThumbnail, HubPlayTalk, UnitReliance
    },
    ProcVoidMethodExt,
    combat::Kaneko,
    unity_engine::{GameObject, IComponentMethods, IGameObjectMethods, IObject_2Methods, IRectTransformMethods, RectTransform},
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior},
};
use engage::app::{HubPlayTalkAfter, IHubAccessMethods, IHubDisposDataMethods, IHubPlayTalkMethods, IMapSequence, MapSequence};
use outfit_core::{UnitAssetData, UnitAssetMenuData};
use skyline::install_hooks;
use unity::{Cast, OptionalMethod};
use crate::{
    dvc::{
        asset::AppearanceManager,
        randomizer::{
            RecruitmentRandomizer,
            shop::{DVCShopItemMenuItem, DVCShopWeaponMenuItem}
        },
        talk::tag::TalkTagTextReplace,
        unit::for_each_unit,
        autolevel::get_difficulty_adjusted_average_level,
        unit::set_unit_lil
    },
    sequences,
    utils::override_vtable,
    menus::{DVCSettingMenu, UnitRecruitMenuItem},
    utils::to_opt
};

pub mod deployment;
pub mod unit;
pub mod randomizer;
pub mod ironman;
pub mod script;
pub mod continuous;
pub mod autolevel;
pub mod blacklist;
pub mod asset;
pub mod mess;
pub mod talk;
pub mod unitstatus;

pub use blacklist::*;

pub static mut STATUS: DVC = DVC::new();

#[derive(PartialEq, Clone, Copy)]
pub enum HeroAppearanceMode {
    Normal,
    Training,
}

pub struct DVC {
    pub alear_person_set: bool,
    pub well_randomized: bool,
    pub enabled: bool,
    pub kizuna_replacements: bool,
    pub map_tile: bool,
    pub seed: i32,
    pub learn_skill: bool,
    pub inspectors_set: bool,
    pub init: bool,
    pub print_events: bool,
    pub kizuna_person_hashes: Vec<i32>,
    pub kizuna_recruit: Vec<i32>,
    pub god_recruit: i32,
    pub hero_appearance: Option<HeroAppearanceMode>,
    pub con_chapter: Option<i32>,
    pub random_reclass: [i32; 2],
    pub set_reclass: Vec<i32>,
    pub unit_job: [i32; 41],
}
#[allow(unused)]
impl DVC {
    const DIE_EVENT: i32 = 1559281067;
    const CONFIG_MENU: i32 = -1028322135;
    const MAIN_SEQUENCE: i32 = -339912801;
    const TITLE_SEQUENCE: i32 = -988690862;
    const MAINMENU_SEQUENCE: i32 = -1912552174;
    const SORTIE_SEQUENCE: i32 = -1624221522;
    const MAP_SEQUENCE: i32 = 1650205480;
    const HUB_SEQUENCE: i32 = 570083351;
    const GMAP_SEQUENCE: i32 = 971901799;
    const MAP_HUMAN_SEQUENCE: i32 = 1525873615;
    const MAP_SEQUENCE_BATTLE: i32 = -1562529994;
    const MAP_ENGAGE_SUMMON: i32 = 31745184;
    const MAP_SEQUENCE_BATTLE_ACTION: i32 = 123970300;
    const GAME_SAVE_DATA_PROC_READ: i32 = -1512678141;
    const WELL_SEQUENCE: i32 = -651818312;
    const COMBAT_COMBAT_SEQUENCE: i32 = -1353170183;
    const COMBAT_ENGAGE_SEQUENCE: i32 = -283318138;
    const UNIT_SELECT_SUB_MENU: i32 = -845322556;
    const LATER_TALK: i32 = 1918405982;
    const EVENT_DEMO: i32 = -2147118281;
    const HUB_MENU_SEQUENCE: i32 = 1811932770;
    const LEVEL_UP: i32 = 1959640519;
    const HUB_MENU_SUB_SYSTEM_MENU: i32 = -334095976;
    const SORTIE_TOP_MENU_SUB_SYSTEM_MENU: i32 = -614519810;
    const MAP_SYSTEM_MENU: i32 = -1831912859;
    const HUB_PLAY_TALK: i32 = 183720196;
    const COMMON_REWARD_MENU: i32 = -1755657852;
    const MUSCLE_START: i32 = 1877583707;
    pub const fn new() -> Self {
        DVC{
            alear_person_set: false,
            well_randomized: false,
            enabled: false,
            kizuna_replacements: false,
            map_tile: false,
            learn_skill: false,
            seed: 0,
            init: false,
            inspectors_set: false,
            print_events: false,
            kizuna_person_hashes: Vec::new(),
            kizuna_recruit: Vec::new(),
            god_recruit: 0,
            con_chapter: None,
            random_reclass: [0; 2],
            set_reclass: Vec::new(),
            hero_appearance: None,
            unit_job: [0; 41],
        }
    }
    pub fn reset_set_jobs(&mut self) {
        if DVCVariable::ClassMode.get_setting_value() != 6 { return; }
        let playables = &PersonPool::get().playable;
        for i in 0..41 {
            let job_hash = self.unit_job[i];
            let data = playables[i].get_person_data();
            let key = GrowthData::get_starting_class_key(data);
            if UnitPool::get_from_person(data, false).is_null() {
                let temp_key = format!("TJG_{}", data.get_pid());
                if GameVariableManager::is_exist(temp_key.as_str()) {
                    let value = GameVariableManager::get_number(temp_key.as_str());
                    GameVariableManager::set_number(key.as_str(), value);
                    self.unit_job[i] = value;
                }
            }
        }
    }
    pub fn set_jobs(&mut self, init: bool) {
        if DVCVariable::ClassMode.get_setting_value() == 6 {
            let playables = &PersonPool::get().playable;
            for i in 0..41 {
                let job_hash = self.unit_job[i];
                let data = playables[i].get_person_data();
                let key = GrowthData::get_starting_class_key(data);
                if init {
                    GameVariableManager::make_entry_norewind(key.as_str(), 0);
                    match job_hash {
                        -1|0 => { GameVariableManager::set_number( format!("G_JG_{}", data.get_pid()), job_hash); }
                        1000 => {
                            let female = if i == 0 { DVCVariable::LueurGender.get_value() == 2 }
                            else { playables[i].get_person_data().get_dress_gender().value == 2 };
                            let hash = if female { 30291080 } else { 30291392 };
                            self.unit_job[i] = hash;
                            GameVariableManager::set_number( key.as_str(), hash);
                        }
                        _ => {
                            if let Some(job) = to_opt(JobData::try_get_from_hash(job_hash)) {
                                GrowthData::set_starting_class_2(playables[i].get_person_data(), job);
                            }
                        }
                    }
                }
                else {
                    self.unit_job[i] =
                    if UnitPool::get_from_person(data, false).is_null() {
                        let temp_key = format!("TJG_{}", data.get_pid());
                        if GameVariableManager::is_exist(temp_key.as_str()) {
                            let value = GameVariableManager::get_number(temp_key.as_str());
                            GameVariableManager::set_number(key.as_str(), value);
                            value
                        }
                        else { GameVariableManager::get_number(key.as_str()) }
                    }
                    else { GameVariableManager::get_number(key.as_str()) };
                }
            }
        }
    }
    pub fn get() -> &'static mut Self { unsafe { &mut STATUS } }
    pub fn reset(&mut self) {
        self.alear_person_set = false;
        self.well_randomized = false;
        self.enabled = false;
        self.kizuna_replacements = false;
        self.map_tile = false;
        self.seed = 0;
        self.init = false;
        self.learn_skill = false;
        self.inspectors_set = false;
        self.god_recruit = 0;
        self.con_chapter = None;
        self.unit_job = [0; 41];
        GameData::get().reset();
        RecruitmentRandomizer::reset_hub();
        let thumb = FaceThumbnail::s_face_thumb();
        let lueur_sprite = thumb.get("TempDVCLueur0");
        thumb.m_cache_table().try_insert("Lueur".into(), lueur_sprite, InsertionBehavior::overwrite_existing());
        patches::reset_patch();
    }
    pub fn install() {
        println!("Installing Draconic Vibe Crystal v{}", VERSION);
        Self::get().print_events = DVCConfig::get().debug;
        cobapi::register_system_event_handler(Self::event_handler);
        menus::inject_menu_classes();
        if cobapi::injection::register::<DVCShopItemMenuItem>().is_ok()  { println!("DVCShopItemMenuItem is Injected"); }
        if cobapi::injection::register::<DVCShopWeaponMenuItem>().is_ok() { println!("DVCShopWeaponMenuItem is Injected"); }
        if cobapi::injection::register::<TalkTagTextReplace>().is_ok() { println!("TalkTagTextReplace is Injected"); }
        if cobapi::injection::register::<sequences::hubmenu::DVCNextChapterMenuItem>().is_ok() { println!("DVCNextChapterMenuItem is Injected"); }
        cobapi::install_lua_command_registerer(script::install_dvc_script_functions);
    }
    pub extern "C" fn event_handler(event: &Event<SystemEvent>) {
        if let Event::Args(ev) = event {
            match ev {
                SystemEvent::ProcInstJump {proc, label } => {
                 //   if Self::get().print_events { println!("[Proc Jump]: {} [{}], Desc: {}/{}, Label: {}", proc.get_name(), proc.get_hash_code(),  proc.get_desc_index(), proc.m_descs().len(),  *label); }
                    println!("[Proc Jump]: {} [{}], Desc: {}/{}, Label: {}", proc.get_name(), proc.get_hash_code(),  proc.get_desc_index(), proc.m_descs().len(),  *label);
                    Self::on_proc_jump(*proc, *label);
                }
                SystemEvent::ProcInstBind { proc, parent: _} => {
                    let proc = proc.borrow();
                    //if Self::get().print_events {
                        let desc = proc.get_desc_index();
                        let hash = proc.get_hash_code();
                        let descs = proc.m_descs();
                        //
                   // }
                    println!("[Proc Bind]: {} ({}), Desc: {}/{} Parent: {}", (*proc).get_name(), hash, desc , descs.len(), (*proc).get_super().get_name());
                    Self::on_proc_bind(*proc);
                }
                SystemEvent::SaveLoaded {ty, slot_id} => {
                   // if Self::get().print_events { println!("[SaveLoaded Event] Type: {}, Slot: {}", *ty, *slot_id); }
                    if *ty > 1 { DVC::get().on_save_load(); }
                }
                _ => {}
            }
        }
    }
    pub fn initialize(&mut self) {
        #[cfg(feature = "debug")]{ self.print_events = true; }
        script::script_vtable_override();
        ironman::iron_man_vtable_edit();
        asset::facethumb::override_facethumb_menu_item_build();
        deployment::sortie_deployment_menu_install();
        randomizer::reclass::class_change_vtable_edit();
        randomizer::rings::ring_skill_inheritance_vtable_edits();
        randomizer::hub::hub_method_edits();
        continuous::continuous_mode_vtable_edit();
        talk::talk_tag_vtable_edit();
        sequences::levelup::vtable_build_common_reward();
        unit::ai::ai_think_method_edit();
        sequences::gmap::vtable_gmap_enter_chapter_item();
        sequences::muscle::exercise_prize_method_edits();
        let data = RANDOMIZER_DATA.get_or_init(|| GameData::init());
        RANDOMIZED_DATA.get_or_init(|| RwLock::new(RandomizedGameData::new(data.person_pool.playable.len())));
        asset::other::OtherAssetData::init();
        install_hooks!(unit::unit_create_impl_2_hook, randomizer::shop::item_refine_data_try_get);
    }
    pub fn on_map_complete(&mut self) {
        self.inspectors_set = false;
        self.map_tile = false;
        randomizer::shop::update_added_shop_items(true);
        if DVCVariables::get_chapter_index() == 22 || DVCVariables::is_main_chapter_complete(22){
            if let Some(hero) = to_opt(UnitPool::get_hero(false)){
                hero.remove_private_skill("SID_異形兵");
                hero.update_state();
            }
        }
        for_each_unit(31, |u|{ set_unit_lil(u, 0, 0); });
    }
    pub fn on_new_game_start(&mut self) {
        GameVariableManager::make_entry("G_DVC_Version", VARIABLE_VERSION);
        let config = DVCConfig::get();
        let user_data = DVCVariables::user_data();
        self.enabled = true;
        if config.seed != 0 { DVCVariable::Seed.init_var(config.seed as i32, true); }
        else { DVCVariable::Seed.init_var(DVCVariables::get_random_number_for_seed() as i32, true); }
        config.create_game_variables(config.randomized);
        self.randomize(true);
        user_data.set_chapter(ChapterData::get("CID_M001".into()));
    }
    pub fn on_save_load(&mut self) {
        self.enabled = true;
        continuous::save_load_emblem_adjustment();
        GameVariableManager::make_entry_norewind("G_DVC_Version", VARIABLE_VERSION);
        let config = DVCConfig::get();
        config.create_game_variables(false);
        if DVCVariable::Version.get_value() < VARIABLE_VERSION {
            let seed = DVCVariable::Seed.get_setting_value();
            DVCFlags::RandomEnable.set_value(seed != 0);
            if seed == 0 { DVCVariable::Seed.set_value(DVCVariables::get_random_number_for_seed() as i32); }
            DVCVariable::Version.set_value(VARIABLE_VERSION);
        }
        let debug = DVCConfig::new().debug;
        if !debug || DVCVariable::AverageLevel.get_value() == 0 {
            DVCVariable::AverageLevel.init_var(get_difficulty_adjusted_average_level(), true);
        }
        println!("DVC Version: {}", DVCVariable::Version.get_setting_value());
        self.randomize(false);
        unit::learn_skill::update_all_unit_learn_skills();
        unit::adjust_missing_weapon_mask();
        randomizer::item::adjust_non_unit_items_inventory();
        GodPool::get_instance().m_sort().iter().for_each(|v|{ randomizer::godbond::check_fix_god_bonds(v); });
        if DVCVariables::get_sequence() == 2 || DVCVariables::get_sequence() == 3 {
            for_each_unit(3u32, |u|{ u.item_equip(); });
            if DVCVariables::get_chapter_index() == 22 {
                for_each_unit(2u32, |u|{ unit::ai::set_ai_for_m022(u); });
            }
        }
        randomizer::shop::update_added_shop_items(false);
    }
    pub fn randomize(&mut self, new_game: bool) {
        println!("Seed: {}", DVCVariable::Seed.get_setting_value());
        randomizer::reclass::class_change_vtable_edit();
        randomizer::hub::hub_method_edits();
        self.set_jobs(new_game);
        if new_game {
            let female = UnitPool::get_from_person_2(enums::PIDS[0], false).m_edit().m_gender().value == 2;
            RecruitmentRandomizer::setup_unit_recruitment_order(female);
            RecruitmentRandomizer::setup_emblem_recruitment_order();
            let asset_data = UnitAssetMenuData::get();
            asset_data.data.clear();
            let random_appearance = DVCFlags::PlayerAppearance.get_value();
            PersonPool::get().playable.iter().for_each(|p|{ asset_data.data.push(UnitAssetData::new_hash(p.hash, random_appearance )); });
            PersonPool::setup_reliance_data();
            UnitReliance::reset();
        }
        else {
            RecruitmentRandomizer::setup_hero(false);
            RecruitmentRandomizer::setup_hub();
        }
        let data = GameData::get();
        AppearanceManager::get().randomize();
        if !self.init {
            let mut random = RandomizedGameData::get_write();
            random.randomize(data);
            random.commit(data);
            self.init = true;
        }
        unit::item::change_liberation_type();
        if DVCVariable::MisercodeKind.get_setting_value() > 0 { unit::item::change_misercode_type(); }
        for x in 0..30 { if let Some(key) = DVCVariable::from(x).map(|v| v.get_key()) { DVCVariables::log_variable(key); } }
        let seals = DVCVariables::has_dlc() && (DVCFlags::RandomEnable.get_value() || DVCVariables::is_main_chapter_complete(67));
        GameVariableManager::set_bool("G_CC_エンチャント", seals);
        GameVariableManager::set_bool("G_CC_マージカノン", seals);
        if DVCVariables::is_main_chapter_complete(22) {
            if let Some(hero) = to_opt(UnitPool::get_hero(false)){ hero.remove_private_skill("SID_異形兵"); }
        }
        GameVariableManager::set_bool("G_解説_TUTID_クラスチェンジ", true);
    }
    pub fn on_proc_jump(proc: ProcInst, label: i32){
        let hash_code = proc.get_hash_code();
        match hash_code {
            Self::MAINMENU_SEQUENCE => {
                if label == 6 {
                    DVC::get().init = false;
                    DVCSettingMenu::create_bind(proc);
                }
                else if label == 28 {
                    DVCConfig::get().save();
                    Self::get().on_new_game_start();
                }
            }
            Self::MAIN_SEQUENCE => {
                match label {
                    2|14 => { Self::get().reset(); }
                    16 => {}
                    _ => {}
                }
            }
            Self::TITLE_SEQUENCE => {
                UnitAssetMenuData::get().is_loaded = false;
                UnitAssetMenuData::get().data.clear();
                if label == 0 {
                    Self::get().enabled = false;
                    if !UnitAssetMenuData::get().init {
                        outfit_core::install_outfit_plugin(true);
                        Patch::in_text(0x01bb272c).nop().unwrap();
                        install_hooks!(
                            asset::asset_table_result_setup_hook,
                            asset::emblem::asset_table_result_god_setup,
                            asset::asset_table_setup_person,
                            asset::color::combat_character_play_facial,
                            mess::unit_get_name,
                        );
                        override_vtable("App", "GameUserData", "OnDeserialize", game_user_data_on_deserialize as _);
                    }
                }
                else {}
            }
            Self::MAP_SEQUENCE => {
                match label {
                    3|4 => { randomizer::map::fow::update_fog(); }
                    // 12 => {} // Turn Hunam
                    16 => { randomizer::map::fow::update_fog(); }  // Turn End
                    17 => { // Complete
                        let con = DVCVariable::Continuous.get_value();
                        if con == 2 || con == 1 {
                            let user_data = DVCVariables::user_data();
                            let chapter = user_data.get_chapter();
                            if let Some(chapter) = continuous::set_next_chapter(chapter){
                                DVCVariables::set_next_chapter(chapter);
                            }
                        }
                        Self::get().on_map_complete();
                        continuous::post_chapter_continuous_mode(proc);
                        if con != 1 && con != 2 && DVCFlags::PostChapterAutolevel.get_value() {
                            autolevel::auto_level_units_bind(proc, false);
                        }
                        if let Some(map) = proc.try_cast::<MapSequence>() {
                            map.set_m_is_completed(true);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    pub fn on_proc_bind(proc: ProcInst) {
        let hash_code = proc.get_hash_code();
        match hash_code {
            Self::DIE_EVENT => { sequences::die::die_event_edit(proc); }
            -100577619 => { proc.m_descs().set(37, Proc::call_2(ProcVoidMethod::from_fn(proc, sequences::hub::muscle_exit).unwrap())); } // MuscleStart
            Self::MUSCLE_START => { Self::get().hero_appearance = Some(HeroAppearanceMode::Training); } // MuscleExerice
            966197538 => {
                if let Some(hub_play_talk_after) = proc.try_cast::<HubPlayTalkAfter>() {
                    proc.m_descs().set(5, Proc::call_2(ProcVoidMethod::from_fn(hub_play_talk_after, sequences::hubplaytalk::hub_play_talk_after_create_select).unwrap()))
                }
            }
            -1301819996 => {    // FishingSequence
                Self::get().hero_appearance = Some(HeroAppearanceMode::Normal);
                let count = proc.m_descs().iter().count();
                let i = if count == 16 { 14 } else { 11 };
                proc.m_descs().set(i, Proc::call_2(ProcVoidMethod::from_fn(proc, sequences::hub::fishing_exit).unwrap()));
            }
            -141149838 => { Self::get().hero_appearance = Some(HeroAppearanceMode::Training); }
            -366846237 => { sequences::hub::hub_cooking_sequence_edit(proc); }
            -590766076|Self::HUB_PLAY_TALK => {
                if DVCVariables::get_sequence() == 5 {
                    if let Some(talk) = proc.try_cast::<HubPlayTalk>() {
                        let descs = talk.m_descs();
                        descs.set(7, Proc::call_2(ProcVoidMethod::from_fn(talk.into(), sequences::hubplaytalk::hub_play_talk_exec_talk_after).unwrap()));
                    }
                }
            },
            Self::MAIN_SEQUENCE => sequences::main::main_sequence_descs_edit(proc),
            Self::WELL_SEQUENCE => sequences::hub::well_sequence_desc_edit(proc),
            Self::HUB_SEQUENCE => {
                let dvc = Self::get();
                dvc.kizuna_person_hashes.clear();
                sequences::hub::hub_sequence_desc_edit(proc);
            },
            Self::HUB_MENU_SEQUENCE => { sequences::hubmenu::hub_menu_sequence_proc_edit(proc); }
            Self::GMAP_SEQUENCE => sequences::gmap::gmap_desc_edit(proc),
            Self::MAP_SEQUENCE => {
                AppearanceManager::get().randomize_summons();
                sequences::map::map_sequence_desc_edit(proc)
            },
            -1643925050 => { if DVCVariables::is_continuous() { proc.jump(4); } }
            Self::EVENT_DEMO => sequences::eventdemo::event_demo_proc_edit(proc),
            Self::SORTIE_SEQUENCE|Self::MAP_SEQUENCE_BATTLE|Self::MAP_SEQUENCE_BATTLE_ACTION|Self::COMBAT_COMBAT_SEQUENCE => randomizer::bgm::proc_descs_bgm_edit(proc),
            Self::CONFIG_MENU => {
                if let Some(proc) = proc.try_cast::<engage::app::BasicMenu>() {
                    proc.m_full_menu_item_list().add(DVCSettingMenu::create_settings_menu_item().into());
                    proc.set_m_desc_index(0);
                    proc.set_m_show_row_num(9);
                    proc.rebuild();
                }
            }
            Self::COMMON_REWARD_MENU => {
                if let Some(root) = to_opt(GameObject::find("WdwEXP")){
                    if let Some(rect) = to_opt(root.get_component_in_children_3::<RectTransform>()) {
                        let mut size = rect.get_size_delta();
                        size.x += 100.0;
                        rect.set_size_delta(size);
                        for i in 0..12 {
                            if let Some(transform) = to_opt(Kaneko::find_in_children(rect, format!("Exp{}", i))){
                                let go = transform.get_game_object();
                                go.get_components_in_children_3::<RectTransform>(true).iter().for_each(|rect|{
                                    let name = rect.get_name().to_rust_string();
                                    if name == "Max" || name == "ExpValue" {
                                        let mut anchored_pos = rect.get_anchored_position();
                                        anchored_pos.x += 50.0;
                                        rect.set_anchored_position(anchored_pos);
                                    }
                                });
                            }
                        }
                    }
                }
            }
            -1403413982 => { Self::get().kizuna_person_hashes.clear(); }
            Self::MAP_ENGAGE_SUMMON => sequences::summon::map_engage_summon_edit(proc),
            Self::UNIT_SELECT_SUB_MENU => {}
            Self::LEVEL_UP => sequences::levelup::level_up_sequence_edit(proc),
            Self::LATER_TALK => sequences::latertalk::later_talk_edit(proc),
            Self::GAME_SAVE_DATA_PROC_READ => {}
            _ => {
                let name = proc.get_name().to_rust_string();
                if name.contains("ProcBond") { asset::facethumb::telop_proc_edit(proc); }
                else if name == "QuickExercise" {
                    proc.m_descs().set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, sequences::muscle::hub_quick_exercise_get_prize_bonus).unwrap()));
                }
                else if name.contains("RefineShopRefineTargetMenu") { randomizer::shop::edit_refine_evolve_items(proc); }
                else if name.contains("DragonRideSequence") {
                    Self::get().hero_appearance = Some(HeroAppearanceMode::Normal);
                    proc.m_descs().set(42, Proc::call_2(ProcVoidMethod::from_fn(proc, sequences::hub::dragon_ride_exit).unwrap()));
                }
            }
        }
    }
}
fn game_user_data_on_deserialize(this: GameUserData, stream: Stream_2, version: i32, _: OptionalMethod){
    unsafe { GameUserData::on_deserialize(this, stream, version) };
    outfit_core::deserialize_outfit_data(stream, version);
    PersonPool::setup_reliance_data();
}