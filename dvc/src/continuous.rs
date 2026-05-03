use unity::{system::{SystemDictionary, Dictionary, List}, prelude::*};
use engage::{
    proc::desc::ProcDesc, gamevariable::*, force::*, unit::Unit, god::GodPool,
    menu::BasicMenuItem, gameuserdata::*, sequence::wellsequence::WellSequenceUseFlags,
    gamedata::{hub::HubFacilityData, chapter::ChapterData, item::ItemData, *},
    manager::NoticeManager, proc::{ProcInst, ProcVoidMethod}, random::*, 
    sequence::{commonrewardsequence::CommonRewardSequence, wellsequence::WellSequence}
};
use crate::{randomizer::*, utils::*, procs::nothing_proc};
const DLC_CIDS: [&str; 15] = ["M005", "S001", "M006", "G001", "S002", "G002", "M007", "G003", "M008", "G004", "M009", "G005", "G006", "M010", "M011"];

pub mod random;
pub mod postchapter;
pub mod sortie;
pub fn continuous_mode_data_edit() {
    let c_mode = DVCVariables::Continuous.get_value();
    if c_mode == 2 || c_mode == 1 {
        HubFacilityData::get_list_mut().unwrap().iter_mut()
            .for_each(|item| { item.condition_cid = "CID_M004".into(); });

        ChapterData::get_list_mut().unwrap().iter_mut()
            .for_each(|chapter|{ chapter.flag &= !114; });
    }
    if c_mode == 2 { GameUserData::set_grow_mode(0); }
}
pub fn do_continious_mode() {
    if DVCVariables::is_main_chapter_complete(26) { return; }
    let c_mode = DVCVariables::Continuous.get_value();
    if DVCVariables::UnitDeployment.get_value() == 3 && !GameUserData::is_evil_map() { GameUserData::get_status().value &= !12352; }
    let current_chapter = GameUserData::get_chapter();
    match c_mode {
        1|2 => {    // Continuous/Random
            current_chapter.flag &= !48;
            if c_mode == 2 {
                crate::randomizer::terrain::adjust_miasma_tiles();
                if !GameUserData::is_evil_map() { GameUserData::get_status().value &= !12352; }
            }
        }
        3|4 => {    // Open
            if DVCVariables::is_main_chapter_complete(10) {
                for x in 11..21 { GameVariableManager::set_number(format!("G_GmapSpot_M0{}", x), 3); }
                GameVariableManager::set_number("G_GmapSpot_M021M022", 3);
            }
            else {
                for x in 4..10 { GameVariableManager::set_number(format!("G_GmapSpot_M00{}", x), 3); }
                GameVariableManager::set_number("G_GmapSpot_M010", 3);
                GameVariableManager::set_number("G_GmapSpot_S001", 3);
                GameVariableManager::set_number("G_GmapSpot_S002", 3);
            }
            if DVCVariables::is_main_chapter_complete(21) && !DVCVariables::is_main_chapter_complete(22) {
                for x in 0..12 {
                    if let Some(god) = DVCVariables::get_god_from_index(x, true) {
                        if let Some(g_unit) = GodPool::create(god) {
                            if x > 0 { g_unit.set_escape(true);  }
                        }
                    }
                }
                for x in 23..26 { GameVariableManager::set_number(format!("G_GmapSpot_M0{}", x), 3); }
            }
        }
        _ => {
            let cid =  current_chapter.cid.to_string();
            if cid.contains("G00") || cid.contains("S0") { current_chapter.flag |= 48; }
            else if cid.contains("M0") {
                if cid != "CID_M010" && cid != "CID_M021" && DVCVariables::is_main_chapter_complete(4) {
                    current_chapter.flag |= 48;
                }
            }
        }
    }
}

pub fn continous_mode_post_battle_stuff(proc: &ProcInst){
    if DVCVariables::Continuous.get_value() == 0 { return}
    GameVariableManager::set_bool(GameUserData::get_chapter().get_cleared_flag_name(), true);
    if DVCVariables::is_main_chapter_complete(26)  { return; }
    do_continious_mode();
    postchapter::add_support_points();
    do_dlc();
    WellSequence::set_use_flag(WellSequenceUseFlags::NotUse);
    let e_list = get_generic_class!(SystemDictionary<Unit, i32>)
        .and_then(|klass|{ klass.instantiate_as::<Dictionary<&Unit, i32>>() }).unwrap();
    e_list.ctor();
    let mut base_exp_gain = 30 + 20*(2 - (GameUserData::get_difficulty(false)));
    let mut level_cap = get_recommended_level_main() as i32;
    let random_map = DVCVariables::is_random_map();
    if random_map {
        let map_completed = get_story_chapters_completed();
        base_exp_gain = 50 - (GameUserData::get_difficulty(false) as i32)*10;
        level_cap = 
            if map_completed < 7  { 1 + map_completed } 
            else { max( (get_story_chapters_completed()-6)*2, get_story_chapters_completed()+4) }
    }
    Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Absent).unwrap().iter())
        .for_each(|unit|{
            if unit.status.value & 35184372088832 == 0 {
                let exp = if unit.force.is_some_and(|f| f.force_type == 3) {
                    base_exp_gain / 2 
                }
                else { base_exp_gain };
                if unit.level == unit.job.max_level {
                    let sp = if random_map { 2 } else { 1} * exp;
                    unit.add_sp(sp);
                }
                else {
                    let total_level = unit.level as i32 + unit.internal_level as i32;
                    if total_level < level_cap {
                        if random_map {
                            let scale_exp = clamp_value(exp * ( level_cap - 1 ) / total_level, exp, 99);
                            e_list.add(unit, scale_exp );
                            unit.add_sp(scale_exp);
                        }
                        else {
                            e_list.add(unit, base_exp_gain);
                            unit.add_sp(base_exp_gain);
                        }
                    }
                    else {
                        let exp_gain = exp / ( 2 + total_level - level_cap);
                        e_list.add(unit, exp_gain);
                        unit.add_sp(exp_gain);
                    }
                }
            }
        });
    let item_list = generate_item_list(proc);
    if !DVCFlags::ContinuousModeItems.get_value() {
        item_list.clear();
        if let Some(bond_frags) = ItemData::get("IID_絆のかけら500") { item_list.add(bond_frags); }
    }
    CommonRewardSequence::create_bind(proc, e_list, item_list, 0, false);
    if let Some(child) = proc.get_child() { // Skips the blank fade
        let descs = child.descs.get_mut();
        descs[3] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
        descs[0xc] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
        descs[0xd] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
    }
}
// Item List for well drops and gifts
fn generate_item_list(_proc: &ProcInst) -> &'static mut List<ItemData> {
    WellSequence::set_use_flag(WellSequenceUseFlags::ItemReturn);
    let current_cid = GameUserData::get_chapter().cid.to_string();
    let random = Random::get_hub_item();
    let rand_map = DVCVariables::Continuous.get_value() == 2;
    let completed = get_continious_total_map_complete_count();
    WellSequence::set_evil_weapon_event_state(3);
    if WellSequence::get_exchange_level() != 0 { WellSequence::calc_item_exchange(WellSequence::get_exchange_level(), random) }
    else {
        let mut level = 1;
        for c in [10, 17, 22]{
            if (rand_map && completed >= (c-1)) || (!rand_map && !DVCVariables::is_main_chapter_complete(c)){ level += 1; }
        }
        if DVCVariables::is_main_chapter_complete(22) { level += 1; }
        let items = WellSequence::calc_item_exchange(level, random);
        if (!rand_map && current_cid == "CID_M006") ||  (rand_map && completed == 7 ) {
            items.add(ItemData::get_mut("IID_トライゾン").unwrap());
            items.add(ItemData::get_mut("IID_ルヴァンシュ").unwrap());
        }
        else if current_cid == "CID_M008" || ( current_cid == "CID_G002" || current_cid == "CID_G005" ){
            items.add(ItemData::get_mut("IID_マスタープルフ").unwrap());
            items.add(ItemData::get_mut("IID_チェンジプルフ").unwrap());
        }
        items
    }
}

pub fn update_next_chapter() {
    if DVCVariables::Continuous.get_value() != 0 && GameUserData::get_sequence() != 0 {
        set_next_chapter(); 
        random::continous_rand_emblem_adjustment();
    }
}
// DLC Check for continous mode
fn continuous_mode_dlc_allowed() -> bool { dlc_check() && DVCFlags::ContinuousDLC.get_value() }

pub(crate) fn set_next_chapter(){
    let mode = DVCVariables::Continuous.get_value();
    if mode == 0 || !DVCVariables::is_main_chapter_complete(4) { return; }
    if !GameVariableManager::exist("G_DVC_Next") {
        GameVariableManager::make_entry("G_DVC_Next", 0);
    }
    let current_chapter = GameUserData::get_chapter();
    if GameVariableManager::get_number("G_DVC_Next") == current_chapter.parent.hash {
        GameVariableManager::set_number("G_DVC_Next", 0);
    }
    if let Some(chapter) = ChapterData::try_get_hash(GameVariableManager::get_number("G_DVC_Next")) {
        if !GameUserData::is_chapter_completed(chapter) { return;}
    }
    let current_cid = current_chapter.cid.to_string();
    if current_cid == "CID_M026" { return; }
    let mut next = None;
    if mode == 2 && DVCVariables::is_main_chapter_complete(4) {
        GameVariableManager::set_bool("G_初回アクセス_錬成屋", true);
        next = random::set_next_random_chapter(current_chapter);
    }
    let dlc = continuous_mode_dlc_allowed();
    if next.is_none() && current_chapter.get_next_chapter().is_some() && current_chapter.flag & 64 != 0 {
        next = current_chapter.get_next_chapter();
    }
    if next.is_none() {
        if !DVCVariables::is_main_chapter_complete(10) {
            if dlc{
                if let Some(new_chapter) = DLC_CIDS.iter().find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x) ) {
                    next = ChapterData::get(format!("CID_{}", new_chapter));
                }
            }
            else {
                if let Some(new_chapter) = DLC_CIDS.iter()
                    .filter(|c| !c.contains("G00"))
                    .find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x))
                {
                    next = ChapterData::get(format!("CID_{}", new_chapter));
                }
            }
        }
        else if dlc {
            for x in 1..7 {
                if !GameVariableManager::get_bool(format!("G_Cleared_G00{}", x)) {
                    next = ChapterData::get(format!("CID_G00{}", x));
                }
            }
        }
    }
    if next.is_none() {
        let emblem_paralogues = DVCVariables::is_main_chapter_complete(15);
        next =
        ChapterData::get_list().unwrap().iter()
            .filter(|x|
                x.gmap_spot_open_condition.is_none_or(|con| GameVariableManager::get_bool(format!("G_Cleared_{}", con))) &&
                !GameUserData::is_chapter_completed(x) && (
                    x.cid.str_contains("M0") || x.cid.str_contains("S001") || x.cid.str_contains("S002") || (x.cid.str_contains("S0") && emblem_paralogues)
                )
            )
            .map(|x| (x.parent.hash, x.recommended_level))
            .min_by(|chapter1, chapter2| chapter1.1.cmp(&chapter2.1))
            .and_then(|x| ChapterData::try_get_hash(x.0));
    }
    if let Some(next_chapter) = next { GameVariableManager::set_number("G_DVC_Next", next_chapter.parent.hash); }
}

fn do_dlc() {
    if !continuous_mode_dlc_allowed() { return; }
    let current_cid = GameUserData::get_chapter().cid.to_string();
    let random = DVCVariables::Continuous.get_value() == 2;
    let completed = get_story_chapters_completed();
    if (!random && current_cid == "CID_M006" ) || ( random && completed >= 4 ) {
        let god =
        if DVCVariables::EmblemRecruitment.get_value() == 0 { GodData::get("GID_エーデルガルト") }
        else { GodData::get( GameVariableManager::get_string("G_R_GID_エーデルガルト")) }.unwrap();
        GodPool::create(god);
    }
    if (!random && current_cid == "CID_M017" ) || ( random && completed == 16 ) {
        GameVariableManager::set_bool("G_CC_エンチャント", true);   // enable dlc seals
        GameVariableManager::set_bool("G_CC_マージカノン", true);
        GameVariableManager::set_number("G_所持_IID_マージカノン専用プルフ", GameVariableManager::get_number("G_所持_IID_マージカノン専用プルフ") + 1); // add dlc deals
        GameVariableManager::set_number("G_所持_IID_エンチャント専用プルフ", GameVariableManager::get_number("G_所持_IID_エンチャント専用プルフ") + 1);
        GameVariableManager::make_entry("MapRecruit", 1);
        GameVariableManager::set_bool("MapRecruit", true);
        for x in 36..41 {
            let person_data = 
            if DVCVariables::UnitRecruitment.get_value() != 0 { PersonData::get(DVCVariables::get_dvc_person(x as i32, false)) }
            else { PersonData::get(PIDS[x]) }.unwrap();
            engage::unit::UnitUtil::join_unit_person(person_data);
        }
        GameVariableManager::set_bool("MapRecruit", false);
    }
}

pub fn get_continious_total_map_complete_count() -> i32 {
    let main = get_story_chapters_completed();
    let side = GameVariableManager::find_starts_with("G_Cleared_S0").iter()
        .filter(|cleared| GameVariableManager::get_bool(cleared.to_string())).count() as i32;
    main + side
}
pub fn get_story_chapters_completed() -> i32 {
    GameVariableManager::find_starts_with("G_Cleared_M0").iter()
        .filter(|cleared| GameVariableManager::get_bool(cleared.to_string())).count() as i32
}

fn get_recommended_level_main() -> u8 {
    let chapters = ChapterData::get_list_mut().expect(":D");
    let current_cid = GameUserData::get_chapter().cid.to_string();
    if current_cid == "CID_M026" { return 0; }
    for x in 1..27 {
        if chapters[x].cid.to_string() == current_cid { continue; }
        if !GameUserData::is_chapter_completed(chapters[x]) {  return chapters[x].recommended_level;  }
    }
    GameUserData::get_chapter().recommended_level
}
pub fn hub_menu_next_help_text(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    if DVCVariables::is_random_map() && GameUserData::get_chapter().get_next_chapter().is_some() {  Mess::get("MID_Hub_Next_Go1")  }
    else { Mess::get("MID_MENU_KIZUNA_DEPART_HELP") }
}