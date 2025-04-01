use unity::{system::{Dictionary, List}, prelude::*};
use skyline::patching::Patch;
use engage::{
    proc::desc::ProcDesc,
    force::*, gamedata::{achieve::*, dispos::ChapterData, item::ItemData, ring::*, unit::*, *}, gameuserdata::*, gamevariable::*, godpool::GodPool, menu::{config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}, BasicMenuResult}, noticemanager::NoticeManager, proc::{ProcInst, ProcVoidMethod}, random::*, sequence::{commonrewardsequence::CommonRewardSequence, wellsequence::WellSequence} 
};
use super::CONFIG;
use crate::{randomizer::*, utils::*};
const DLC_CIDS: [&str; 15] = ["M005", "S001", "M006", "G001", "S002", "G002", "M007", "G003", "M008", "G004", "M009", "G005", "G006", "M010", "M011"];

pub mod random;
pub mod postchapter;
pub mod sortie;
pub struct ContiniousMode;
impl ConfigBasicMenuItemSwitchMethods for ContiniousMode {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result;
        if dlc_check() { result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().continuous, 0, 5, 1); }
        else { result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().continuous, 0, 4, 1); }
        if CONFIG.lock().unwrap().continuous != result {
            CONFIG.lock().unwrap().continuous = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let mode = CONFIG.lock().unwrap().continuous;
        this.help_text = if dlc_check() {
            if mode == 1 {  "Game will progress map to map."}
            else if mode == 2 {  "Game will progress map to map without DLC." }
            else if mode == 3 { "Game will progress map to map in random order."}
            else if mode == 4 { "Chapters are unlocked with some restrictions."}
            else if mode == 5 { "Chapters are scaled and unlocked with some restrictions."}
            else {"Game will progress with access to the Somniel and World Map" }
        }
        else {
            if mode == 1 { "Game will progress map to map." }
            else if mode == 2 { "Game will progress map to map in random order."}
            else if mode == 3 { "Chapters are unlocked with some restrictions."}
            else if mode == 4 { "Chapters are scaled and unlocked with some restrictions."}
            else { "Game will progress with access to the Somniel and World Map" }
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let mode = CONFIG.lock().unwrap().continuous;
        this.command_text =  if dlc_check() {
            match mode {
                1 => { "Enabled" }
                2 => { "Enabled w/o DLC" }
                3 => { "Random" }
                4 => { "Open" }
                5 => { "Open Scaled" }
                _ => { "Disabled" }
            }
        }
        else {
            match mode {
                1 => { "Enabled" }
                2 => { "Random" }
                3 => { "Open" }
                4 => { "Open Scaled" }
                _ => { "Disabled" }
            }
        }.into();
    }
}
pub fn setting_adjustment() {
    GameVariableManager::make_entry("G_ConSet", 0);
    let c_mode = GameVariableManager::get_number(DVCVariables::CONTINIOUS);
    if c_mode >= 3  && !dlc_check() && !GameVariableManager::get_bool("G_ConSet") {
        GameVariableManager::set_bool("G_ConSet", true);
        GameVariableManager::set_number(DVCVariables::CONTINIOUS, c_mode + 1);
    }
    else if c_mode == 2 && !dlc_check() {
        GameVariableManager::set_bool("G_ConSet", true);
        GameVariableManager::set_number(DVCVariables::CONTINIOUS, 3);
    }
}
// Continious Mode Stuff
pub fn do_continious_mode() {
    setting_adjustment();
    let c_mode = GameVariableManager::get_number(DVCVariables::CONTINIOUS);
    if GameVariableManager::get_number(DVCVariables::DEPLOYMENT_KEY) == 3 && !GameUserData::is_evil_map() { GameUserData::get_status().value &= !12352; }
    if c_mode > 0 {
        Patch::in_text(0x01f7e9c8).bytes(&[0xE0, 0x03, 0x15, 0x2A ]).unwrap(); // Forces SP = EXP for all cases
        Patch::in_text(0x01f7eab8).bytes(&[0xE0, 0x03, 0x01, 0x2A ]).unwrap();
    }
    else { 
        Patch::in_text(0x028a80d0).bytes(&[0xff, 0x43, 0x01, 0xd1 ]).unwrap();  // HubFac IsComplete
        Patch::in_text(0x028a80d4).bytes(&[0xfd, 0x7b, 0x01, 0xa9]).unwrap(); 
        return; 
    }
    if c_mode >= 4 {
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
        return;
    }

    if c_mode != 0 && !DVCVariables::is_main_chapter_complete(26)  {
        GameUserData::set_grow_mode(0);
        GameVariableManager::set_bool("G_拠点_神竜導入イベント再生済み", false);
        let current_chapter = GameUserData::get_chapter();
        if DVCVariables::is_main_chapter_complete(4) {
            current_chapter.set_flag(current_chapter.get_flag() & !114 );
            if c_mode == 3 { 
                crate::randomizer::terrain::adjust_miasma_tiles();
                if !GameUserData::is_evil_map() { GameUserData::get_status().value &= !12352; }
                crate::utils::return_true(0x028a80d0);
            }
        }
    }
     else {
        let current_chapter = GameUserData::get_chapter();
        let cid =  current_chapter.cid.to_string();
        if cid.contains("G00") || cid.contains("S0") { current_chapter.set_flag(current_chapter.get_flag() | 48); }
        else if cid.contains("M0") {
            if cid != "CID_M010" && cid != "CID_M021" && DVCVariables::is_main_chapter_complete(4) { 
                current_chapter.set_flag(current_chapter.get_flag() | 48); 
            }
         }
        Patch::in_text(0x01d76320).bytes(&[0xfd, 0x7b, 0xbd, 0xa9]).unwrap();   // Revert Back menu item in Sortie
        Patch::in_text(0x01d76324).bytes(&[0xf6, 0x57, 0x01, 0xa9]).unwrap(); 
    // Revert to normal exp -> sp
        Patch::in_text(0x01f7e9c8).bytes(&[0x86, 0xed, 0xea, 0x97 ]).unwrap(); // 86 ed ea 97
        Patch::in_text(0x01f7eab8).bytes(&[0x4a, 0xed, 0xea, 0x97 ]).unwrap();  // 4a ed ea 97
        Patch::in_text(0x01d78ee4).bytes(&[0x8b, 0xe9, 0x1d, 0x94]).unwrap(); 
    
        Patch::in_text(0x028a80d0).bytes(&[0xff, 0x43, 0x01, 0xd1 ]).unwrap();  // HubFac IsComplete
        Patch::in_text(0x028a80d4).bytes(&[0xfd, 0x7b, 0x01, 0xa9]).unwrap(); 
    }
}

pub fn continous_mode_post_battle_stuff(proc: &ProcInst){
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 0 { return; }
    if GameUserData::get_chapter().cid.to_string() == "CID_M026" || DVCVariables::is_main_chapter_complete(26) { return; }

    GameVariableManager::set_bool(GameUserData::get_chapter().get_cleared_flag_name(), true);
    do_continious_mode();
    postchapter::add_support_points();
    do_dlc();
    postchapter::create_bond_rings();
    WellSequence::set_use_flag(2);
    postchapter::update_bonds();
    let item_list = generate_item_list(proc);
    WellSequence::set_use_flag(0);
    let common_rewards_sequence = CommonRewardSequence::instantiate().unwrap();
    let methods = common_rewards_sequence.get_class().get_methods();
    let ctor_parameters = methods[3].get_parameters();
    let para = unity::prelude::Il2CppClass::from_il2cpptype( ctor_parameters[0].parameter_type ).unwrap();
    if let Ok(e_list) = il2cpp::instantiate_class::<Dictionary<&Unit, i32>>(para) {
        let dictionary_methods = e_list.get_class().get_methods();
        unsafe { dictionary_ctor(e_list, Some(dictionary_methods[0])); }
        let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
        let mut base_exp_gain = 10*(3 + 2*(2 - (GameUserData::get_difficulty(false) as i32 )) );
        let mut level_cap = get_recommended_level_main() as i32;
        let random_map = GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3;
        if random_map { 
            let map_completed = crate::continuous::get_story_chapters_completed();
            base_exp_gain = 50 - (GameUserData::get_difficulty(false) as i32)*10; 
            level_cap = if map_completed < 7  {1 + map_completed    }
            else { crate::utils::max( (crate::continuous::get_story_chapters_completed()-6)*2, crate::continuous::get_story_chapters_completed()+4) }
        }
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            for unit in force_iter {
                if unit.status.value & 35184372088832 != 0 { continue; }    // Lyn doubles are a no-no
                if unit.level == unit.job.max_level { 
                    if random_map { unit.add_sp( base_exp_gain * 2 ); }
                    else { unit.add_sp(base_exp_gain); }
                }
                else {
                    let total_level = unit.level as i32 + unit.internal_level as i32;
                    if total_level < level_cap { 
                        if random_map {
                            let scale_exp = clamp_value(base_exp_gain * ( level_cap - 1 ) / total_level, base_exp_gain, 99);
                            e_list.add(unit, scale_exp );
                            unit.add_sp(scale_exp);
                        }
                        else {
                            e_list.add(unit, base_exp_gain); 
                            unit.add_sp(base_exp_gain);
                        }
                    }
                    else {
                        let diff = total_level - level_cap;
                        let exp_gain = base_exp_gain / (  2 + diff );
                        e_list.add(unit, exp_gain);
                        unit.add_sp(exp_gain);
                    }
                }
            }
            base_exp_gain = clamp_value(base_exp_gain * 5 / 3 , base_exp_gain, 99);
            level_cap -= 3; 
            if base_exp_gain <= 0 { break; }
        }
            // Heroes DLC
        if !GameVariableManager::get_bool("G_拠点_コンテンツ報酬受け取り済") && DVCVariables::is_main_chapter_complete(4) && unsafe { has_content(1, None) } {
            item_list.add(ItemData::get_mut( "IID_フェンサリル" ).unwrap());
            item_list.add(ItemData::get_mut( "IID_ノーアトゥーン" ).unwrap());
            item_list.add(ItemData::get_mut( "IID_フォルクヴァング" ).unwrap());
            UnitRingPool::add_ring("RNID_DLC1コモン_1_S".into(), None, 1 );
            UnitRingPool::add_ring("RNID_DLC1コモン_2_S".into(), None, 1 );
            UnitRingPool::add_ring("RNID_DLC1コモン_3_S".into(), None, 1 );
            GameVariableManager::make_entry("G_拠点_コンテンツ報酬受け取り済", 1);
            GameVariableManager::set_bool("G_拠点_コンテンツ報酬受け取り済", true);
        }
        CommonRewardSequence::create_bind(proc, e_list, item_list, 0, false);
        let desc = proc.child.as_ref().unwrap().descs.get();
        unsafe {
            (*desc)[3] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
            (*desc)[0xc] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
            (*desc)[0xd] = ProcDesc::call(ProcVoidMethod::new(None, nothing_proc));
        }

    }
}
// Item List for well drops and gifts
fn generate_item_list(_proc: &ProcInst) -> &'static mut List<ItemData> {
    WellSequence::set_use_flag(2);
    let current_chapter = GameUserData::get_chapter();
    let current_cid = current_chapter.cid.to_string(); 
    let seed = Random::get_system().value() as u32;
    let random = Random::instantiate().unwrap();
    random.ctor(seed);
    let rand_map = GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3;
    let completed = get_continious_total_map_complete_count();
    if (!rand_map && !DVCVariables::is_main_chapter_complete(6)) || (rand_map && completed <= 7 ) {
        if current_cid == "CID_M004" {
            if let Some(patch_items) =  RewardData::calc_rewards("Patch0特典".into()) {
                if continuous_mode_dlc_allowed() {
                    RewardData::calc_rewards("DLC購入特典0".into()).unwrap().iter().for_each(|item|{
                        if let Some(item1) = ItemData::get_mut(item.iid) { patch_items.add(item1); }
                    });
                    set_patch_flag("G_拠点_DLC特典アイテム0受け取り済み");
                }
                set_patch_flag("G_拠点_Patch0特典アイテム受け取り済み");
                return patch_items;
            }
            else { return WellSequence::calc_item_exchange(2, random); }
        }
        else if (!rand_map && current_cid == "CID_M005") ||  (rand_map && completed == 6 ) {
            if let Some(patch_items) =  RewardData::calc_rewards("Patch3特典".into()) {
                if continuous_mode_dlc_allowed() {
                    RewardData::calc_rewards("DLC購入特典1".into()).unwrap().iter().for_each(|item|{
                        if let Some(item1) = ItemData::get_mut(item.iid) { patch_items.add(item1); }
                    });
                    set_patch_flag("G_拠点_DLC特典アイテム1受け取り済み"); 
                }
                set_patch_flag("G_拠点_Patch3特典アイテム受け取り済み");
                return patch_items;
            }
            else { return WellSequence::calc_item_exchange(2, random); }
        }
        else {
            let well_items = WellSequence::calc_item_exchange(1, random);
            if (!rand_map && current_cid == "CID_M006") ||  (rand_map && completed == 7 ) {
                well_items.add(ItemData::get_mut("IID_トライゾン").unwrap());
                well_items.add(ItemData::get_mut("IID_ルヴァンシュ").unwrap());
                if continuous_mode_dlc_allowed() { well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap()); }
                WellSequence::set_evil_weapon_state(3);
            }
            return well_items;
        }
    }
    WellSequence::set_evil_weapon_state(3);
    if WellSequence::get_exchange_level() != 0 { return WellSequence::calc_item_exchange(WellSequence::get_exchange_level(), random); }
    if (!rand_map && !DVCVariables::is_main_chapter_complete(10) ) || (rand_map && completed < 9){
        let well_items = WellSequence::calc_item_exchange(2, random);
        if current_cid == "CID_M008" || ( current_cid == "CID_G002" || current_cid == "CID_G005" ) {
            well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap());
            well_items.add(ItemData::get_mut("IID_チェンジプルフ").unwrap());
        }
        return well_items;
    }
    else if (!rand_map && !DVCVariables::is_main_chapter_complete(17)) || ( rand_map && completed < 16 ) { return WellSequence::calc_item_exchange(3, random); }
    else if (!rand_map && !DVCVariables::is_main_chapter_complete(22)) || ( rand_map && completed < 21 ) { return WellSequence::calc_item_exchange(4, random) }
    else { return WellSequence::calc_item_exchange(5, random); }
}

// When loading save at exploration
pub fn update_next_chapter() {
    if GameVariableManager::get_number(DVCVariables::CONTINIOUS) != 0 { 
        set_next_chapter(); 
        random::continous_rand_emblem_adjustment();
        continuous_mode_next_chapter_notice();
    }
}
// DLC Check for continous mode
fn continuous_mode_dlc_allowed() -> bool {
    dlc_check() && (GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 1 || GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3)
}

fn set_next_chapter(){
    let mode = GameVariableManager::get_number(DVCVariables::CONTINIOUS);
    if mode == 0 || !DVCVariables::is_main_chapter_complete(4) { return; }
    let current_chapter = GameUserData::get_chapter();
    let current_cid = current_chapter.cid.to_string();
    if current_cid == "CID_M026" { return; }
    if mode == 3 && DVCVariables::is_main_chapter_complete(4) {
        GameVariableManager::set_bool("G_初回アクセス_錬成屋", true);
        random::set_next_random_chapter(current_chapter);
        return;
    }
    emblem::emblem_gmap_spot_adjust();
    let dlc = continuous_mode_dlc_allowed();
    //switch or updated without DLC, moves back to main chapters from Divine Paralogue
    if !dlc {
        for x in 1..7 { 
            GameVariableManager::set_bool(format!("G_Cleared_G00{}", x), false);  
            GameVariableManager::set_bool(format!("G_Cleared_E00{}", x), false); 
        }
        for x in 12..19 { random::escape_god(EMBLEM_GIDS[x], true); }
    }
    if !DVCVariables::is_main_chapter_complete(10) {
        if dlc{
            if let Some(new_chapter) = DLC_CIDS.iter().find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x) ) {
                current_chapter.set_next_chapter(format!("CID_{}", new_chapter).as_str());
            }
        }
        else {
            if let Some(new_chapter) = DLC_CIDS.iter().filter(|c| !c.contains("G00")).find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x)) {
                current_chapter.set_next_chapter(format!("CID_{}", new_chapter).as_str());
            }
        }
    }
    if current_cid == "CID_M015" || DVCVariables::is_main_chapter_complete(15) {
        if current_cid == "CID_M021" { return; }
        let rec_level = get_recommended_level_main();
        let chapter_list = ChapterData::get_list_mut().unwrap();
        // paralogue check
        let mut min_rec_level = rec_level;
        let mut chapter_index = 0;
        for x in 29..42 {
            let paralogue = &chapter_list[x];
            if paralogue.cid.to_string() == current_cid { continue; }
            if GameVariableManager::get_bool(paralogue.get_cleared_flag_name()) { continue; }    // already completed
            let paralogue_level = paralogue.get_recommended_level();
            let open = paralogue.get_gmap_open_condition().to_string();
            if open.contains("G00") || GameVariableManager::get_bool(format!("G_Cleared_{}", open)) {
                if paralogue_level < rec_level {
                    if paralogue_level < min_rec_level {
                        min_rec_level = paralogue_level;
                        chapter_index = x;
                    }
                }
            }
        }
        if chapter_index > 0 {
            let paralogue = &chapter_list[chapter_index as usize];
            current_chapter.set_next_chapter(&paralogue.cid.to_string());
            return;
        }
        for x in 16..28 {
            let main = &chapter_list[x];
            if main.cid.to_string() == current_cid { continue; }
            if GameVariableManager::get_bool(main.get_cleared_flag_name()) { continue; }
            current_chapter.set_next_chapter(&main.cid.to_string());
            return;
        }
    }
}

fn do_dlc() {
    if !continuous_mode_dlc_allowed() { return; }
    let current_cid = GameUserData::get_chapter().cid.to_string();
    let random = GameVariableManager::get_number(DVCVariables::CONTINIOUS) == 3;
    let completed = get_story_chapters_completed();
    if (!random && current_cid == "CID_M006" ) || ( random && completed >= 4 ) {
        let god =
        if GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) == 0 { GodData::get("GID_エーデルガルト") }
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
            if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) != 0 { PersonData::get(DVCVariables::get_dvc_person(x as i32, false)) }
            else { PersonData::get(PIDS[x]) }.unwrap();
            UnitUtil::join_unit_person(person_data);
        }
        GameVariableManager::set_bool("MapRecruit", false);
    }
}

pub fn get_continious_total_map_complete_count() -> i32 {
    ChapterData::get_list().unwrap().iter().filter(|chapter| GameUserData::is_chapter_completed(chapter)).count() as i32
}
pub fn get_story_chapters_completed() -> i32 {
    GameVariableManager::find_starts_with("G_Cleared_M0").iter().filter(|cleared| GameVariableManager::get_bool(cleared.to_string())).count() as i32
}

fn get_recommended_level_main() -> u8 {
    let chapters = ChapterData::get_list_mut().expect(":D");
    let current_cid = GameUserData::get_chapter().cid.to_string();
    if current_cid == "CID_M026" { return 0; }
    for x in 1..27 {
        if chapters[x].cid.to_string() == current_cid { continue; }
        if !GameUserData::is_chapter_completed(chapters[x]) {  return chapters[x].get_recommended_level();  }
    }
    return  GameUserData::get_chapter().get_recommended_level();
}

#[skyline::from_offset(0x01ddfc50)]
fn enable_map_rewind(method_info: OptionalMethod);

#[skyline::from_offset(0x03cbca00)]
fn dictionary_ctor(this: &Dictionary<&Unit, i32>, method_info: OptionalMethod);

pub fn continuous_mode_next_chapter_notice(){
    if DVCVariables::is_random_map() && ( GameUserData::get_sequence() & 4 != 0) && GameUserData::get_chapter().get_next_chapter().is_some() {
        NoticeManager::add_by_mid( "MID_Hub_Next_Go1");
    }
}

pub fn hub_menu_next_help_text(_this: &BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    if DVCVariables::is_random_map() && GameUserData::get_chapter().get_next_chapter().is_some() {  Mess::get("MID_Hub_Next_Go1")  }
    else { Mess::get("MID_MENU_KIZUNA_DEPART_HELP") }
}

pub extern "C" fn nothing_proc(_proc: &mut ProcInst, _method_info: OptionalMethod) {}