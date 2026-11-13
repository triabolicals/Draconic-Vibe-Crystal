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
        procinst::*, unitedit::*, spriteatlasmanager_2::*,
        ChapterData, IStructData_1Methods, ISingletonPool_2,
        GodPool, UnitPool, FaceThumbnail, UnitReliance
    },
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior},
};
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
    utils::*,
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
pub enum HeroAppearanceMode { Normal, Training, }

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
    #[allow(static_mut_refs)] pub fn get() -> &'static mut Self { unsafe { &mut STATUS } }
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
                    if Self::get().print_events { println!("[Proc Jump]: {} [{}], Desc: {}/{}, Label: {}", proc.get_name(), proc.get_hash_code(),  proc.get_desc_index(), proc.m_descs().len(),  *label); }
                    Self::on_proc_jump(*proc, *label);
                }
                SystemEvent::ProcInstBind { proc, parent: _} => {
                    let proc = proc.borrow();
                    if Self::get().print_events {
                        let desc = proc.get_desc_index();
                        let hash = proc.get_hash_code();
                        let descs = proc.m_descs();
                        println!("[Proc Bind]: {} ({}), Desc: {}/{} Parent: {}", (*proc).get_name(), hash, desc , descs.len(), (*proc).get_super().get_name());
                    }
                    Self::on_proc_bind(*proc);
                }
                SystemEvent::SaveLoaded {ty, slot_id} => {
                    if Self::get().print_events { println!("[SaveLoaded Event] Type: {}, Slot: {}", *ty, *slot_id); }
                    if *ty > 1 { DVC::get().on_save_load(); }
                }
                _ => {}
            }
        }
    }
    pub fn initialize(&mut self) {
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
            RecruitmentRandomizer::correct_emblem_order();
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
    pub fn on_proc_jump(proc: ProcInst, label: i32){ sequences::proc_inst_jump_event(proc, label); }
    pub fn on_proc_bind(proc: ProcInst) { sequences::proc_inst_bind_event(DVC::get(), proc); }
}
