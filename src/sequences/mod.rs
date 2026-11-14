pub use engage::{
    app::{
        procinst::*, proc::*, procdesc::*, procvoidmethod::*, procboolmethod::*,
        singletonclass_1::*, singletonprocinst_1::*
    },
    ext::{ProcBoolMethodExt, ProcVoidFunctionExt, ProcVoidMethodExt},
};
use crate::dvc::{asset, autolevel, randomizer, HeroAppearanceMode, DVC};
use super::*;

pub mod hub;
pub mod map;
pub mod main;
pub mod mainmenu;
pub mod levelup;
pub mod summon;
pub mod unitgrow;
pub mod eventdemo;
pub mod gmap;
pub mod latertalk;
pub mod hubmenu;
pub mod hubplaytalk;
pub mod die;
mod itemuse;
pub mod muscle;
mod title;

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
const HUB_PLAY_TALK_AFTER: i32 = 966197538;
const HUB_PLAY_TALK_AFTER_REPLACE_PERSON: i32 = -590766076;
const COMMON_REWARD_MENU: i32 = -1755657852;
const MUSCLE_START: i32 = -100577619;
const MUSCLE_EXERCISE: i32 = 1877583707;
const FISHING_SEQUENCE: i32 = -1301819996;
const HUB_COOKING_SEQUENCE: i32 = -366846237;
const MUSCLE_GAME: i32 = -141149838;
pub fn proc_inst_jump_event(proc: ProcInst, label: i32) {
    let hash_code = proc.get_hash_code();
    match hash_code {
        MAIN_SEQUENCE => if label == 2 || label == 14 { DVC::get().reset(); },
        MAP_SEQUENCE => map::map_sequence_completed_jump_event(proc, label),
        MAINMENU_SEQUENCE => mainmenu::main_menu_jump_event(proc, label),
        TITLE_SEQUENCE => title::title_sequence_event_jump(label),
        _ => {}
    }
}
pub fn proc_inst_bind_event(dvc: &mut DVC, proc: ProcInst){
    let hash_code = proc.get_hash_code();
    match hash_code {
        MAIN_SEQUENCE => main::main_sequence_descs_edit(proc),
        DIE_EVENT => die::die_event_edit(proc),
        MUSCLE_START => proc.m_descs().set(37, Proc::call_2(ProcVoidMethod::from_fn(proc, hub::muscle_exit).unwrap())),
        MUSCLE_EXERCISE|MUSCLE_GAME => { dvc.hero_appearance = Some(HeroAppearanceMode::Training); }
        FISHING_SEQUENCE => {    // FishingSequence
            dvc.hero_appearance = Some(HeroAppearanceMode::Normal);
            let count = proc.m_descs().iter().count();
            let i = if count == 16 { 14 } else { 11 };
            proc.m_descs().set(i, Proc::call_2(ProcVoidMethod::from_fn(proc, hub::fishing_exit).unwrap()));
        }
        HUB_SEQUENCE => {
            dvc.kizuna_person_hashes.clear();
            hub::hub_sequence_desc_edit(proc);
        },
        HUB_MENU_SEQUENCE => hubmenu::hub_menu_sequence_proc_edit(proc),
        HUB_COOKING_SEQUENCE => hub::hub_cooking_sequence_edit(proc),
        HUB_PLAY_TALK_AFTER_REPLACE_PERSON|HUB_PLAY_TALK => hubplaytalk::hub_play_talk_descs_edit(proc),
        HUB_PLAY_TALK_AFTER => hubplaytalk::hub_play_talk_after_descs_edit(proc),
        WELL_SEQUENCE => hub::well_sequence_desc_edit(proc),
        GMAP_SEQUENCE => gmap::gmap_desc_edit(proc),
        MAP_SEQUENCE => map::map_sequence_desc_edit(proc),
        -1643925050 => if DVCVariables::is_continuous() { proc.jump(4); },
        EVENT_DEMO => eventdemo::event_demo_proc_edit(proc),
        SORTIE_SEQUENCE|MAP_SEQUENCE_BATTLE|MAP_SEQUENCE_BATTLE_ACTION|COMBAT_COMBAT_SEQUENCE => randomizer::bgm::proc_descs_bgm_edit(proc),
        CONFIG_MENU => menus::DVCSettingMenu::add_setting_menu_item(proc),
        COMMON_REWARD_MENU => dvc::continuous::adjust_exp_game_object(),
        -1403413982 => dvc.kizuna_person_hashes.clear(),
        MAP_ENGAGE_SUMMON => summon::map_engage_summon_edit(proc),
        UNIT_SELECT_SUB_MENU => {}
        LEVEL_UP => levelup::level_up_sequence_edit(proc),
        LATER_TALK => latertalk::later_talk_edit(proc),
        GAME_SAVE_DATA_PROC_READ => {}
        _ => {
            let name = proc.get_name().to_rust_string();
            if name.contains("ProcBond") { asset::facethumb::telop_proc_edit(proc); }
            else if name == "QuickExercise" {
                proc.m_descs().set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, muscle::hub_quick_exercise_get_prize_bonus).unwrap()));
            }
            else if name.contains("RefineShopRefineTargetMenu") { randomizer::shop::edit_refine_evolve_items(proc); }
            else if name.contains("DragonRideSequence") {
                dvc.hero_appearance = Some(HeroAppearanceMode::Normal);
                proc.m_descs().set(42, Proc::call_2(ProcVoidMethod::from_fn(proc, hub::dragon_ride_exit).unwrap()));
            }
        }
    }
}