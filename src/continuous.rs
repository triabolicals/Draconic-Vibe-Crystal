use unity::{system::{Dictionary, List}, prelude::*};
use skyline::patching::Patch;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    force::*,
    proc::ProcInst,
    random::*,
    gamedata::{unit::*, item::ItemData, god::RingData, dispos::ChapterData, *},
    godpool::GodPool,
};
use super::CONFIG;
use crate::{randomizer::*, utils::*};
const DLC_CIDS: [&str; 14] = ["M005", "S001", "M006", "G001", "S002", "G002", "M007", "G003", "M008", "G004", "M009", "G005", "G006", "M010"];
#[unity::class("App", "GodBond")]
pub struct GodBond {
    pub god: &'static GodData,
    reliance_s: u64,
    pub pid: &'static Il2CppString,
    pub level: u8,
    __: u8,
    pub exp: u16,
}

#[unity::class("App", "UnitRelianceData")]
pub struct UnitRelianceData {
    reliance: u64,
    pub level: i32,
    pub exp: i8,
    pub score: i8,
}

#[unity::class("App", "AchieveData")]
pub struct AchieveData { }
impl Gamedata for AchieveData {}

#[unity::class("App", "CommonRewardSequence")]
pub struct CommonRewardSequence {
    proc: [u8; 0x60],
    menu: u64,
    pub exp_list: &'static Dictionary<&'static Unit, i32>,
    pub item_reward_list: &'static List<ItemData>,
}

// Continious Mode Stuff
pub fn do_continious_mode() {
    if GameVariableManager::get_number("G_Continuous") != 0 && !GameVariableManager::get_bool("G_Cleared_M026")  {
        if GameVariableManager::get_number("G_Continuous") == 2 && !dlc_check() { GameVariableManager::set_number("G_Continuous", 3); }
        let current_chapter = GameUserData::get_chapter();
        let current_cid = current_chapter.cid.to_string();
        if GameVariableManager::get_bool("G_Cleared_M004") {
            current_chapter.set_flag(current_chapter.get_flag() & !114 );
            if GameVariableManager::get_number("G_Continuous") == 3 { 
                crate::deployment::fulldeploy::adjust_miasma_tiles();
                GameUserData::get_status().value &= !12352;
                crate::utils::return_true(0x028a80d0);
            }
            else {
                if current_cid == "CID_G006" && GameVariableManager::get_bool("G_Cleared_G006")  { current_chapter.set_next_chapter("CID_M010"); }
            }
        }
        Patch::in_text(0x02533768).bytes(&[0xE0,0x03, 0x1F, 0xAA]).unwrap();    //  Prevent Fade out after EXP screen
        Patch::in_text(0x02533770).bytes(&[0x14,0xA0, 0x0B, 0x94]).unwrap();    //  Prevent Fade out after EXP screen
        Patch::in_text(0x01f7e9c8).bytes(&[0xE0, 0x03, 0x15, 0x2A ]).unwrap(); // Forces SP = EXP for all cases
        Patch::in_text(0x01f7eab8).bytes(&[0xE0, 0x03, 0x01, 0x2A ]).unwrap();
        crate::utils::return_4(0x01d76320); // Hides Back menu item in Sortie
        crate::utils::mov_x0_0(0x01d78ee4); // Can return disable sortie b call


    }
     else {
        Patch::in_text(0x01d76320).bytes(&[0xfd, 0x7b, 0xbd, 0xa9]).unwrap();   // Revert Back menu item in Sortie
        Patch::in_text(0x01d76324).bytes(&[0xf6, 0x57, 0x01, 0xa9]).unwrap(); 
            // Revert to normal exp -> sp
        Patch::in_text(0x01f7e9c8).bytes(&[0x86, 0xed, 0xea, 0x97 ]).unwrap(); // 86 ed ea 97
        Patch::in_text(0x01f7eab8).bytes(&[0x4a, 0xed, 0xea, 0x97 ]).unwrap();  // 4a ed ea 97
        Patch::in_text(0x01d78ee4).bytes(&[0x8b, 0xe9, 0x1d, 0x94]).unwrap(); 
    
        Patch::in_text(0x028a80d0).bytes(&[0xff, 0x43, 0x01, 0xd1 ]).unwrap();  // HubFac IsComplete
        Patch::in_text(0x028a80d4).bytes(&[0xfd, 0x7b, 0x01, 0xa9]).unwrap(); 
    }
    /*
    ["Vandre", "Diamand", "Ivy", "Fogato", "Hortensia", "Alfred", "Yunaka", "Seadas", "Staluke", "Misutira", "Goldmary", "Celine"].iter().for_each(|g|{
        if let Some(god) = GodData::get(format!("GID_{}", g)) {
            unsafe { godpool_create(god, None); }
        }
    });
    */
}

pub fn continous_mode_post_battle_stuff(proc: &ProcInst){
    if GameVariableManager::get_number("G_Continuous") == 0 { return; }
    if GameUserData::get_chapter().cid.to_string() == "CID_M026" { return; }
    if GameVariableManager::get_bool("G_Cleared_M026") { return; }
    GameVariableManager::set_bool(GameUserData::get_chapter().get_cleared_flag_name(), true);
    do_continious_mode();
    add_support_points();
    do_dlc();
    update_bonds();
    create_bond_rings();
    unsafe {
        set_well_use_flag(2, None);
        let item_list = generate_item_list(proc);
        let common_rewards_sequence = CommonRewardSequence::instantiate().unwrap();
        let methods = common_rewards_sequence.get_class().get_methods();
        let ctor_parameters = methods[3].get_parameters();
        let para = unity::prelude::Il2CppClass::from_il2cpptype( ctor_parameters[0].parameter_type ).unwrap();
        let exp_list = il2cpp::instantiate_class::<Dictionary<&Unit, i32>>(para);
        if exp_list.is_ok() {
            let e_list = exp_list.unwrap();
            let dictionary_methods = e_list.get_class().get_methods();
            dictionary_ctor(e_list, Some(dictionary_methods[0]));
            let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
            let mut base_exp_gain = 10*(3 + 2*(2 - (GameUserData::get_difficulty(false) as i32 )) );
            let mut level_cap = get_recommended_level_main() as i32;
            let random_map = GameVariableManager::get_number("G_Continuous") == 3;
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
            if !GameVariableManager::get_bool("G_拠点_コンテンツ報酬受け取り済") && GameVariableManager::get_bool("G_Cleared_M004") && has_content(1, None) {
                item_list.add(ItemData::get_mut( "IID_フェンサリル" ).unwrap());
                item_list.add(ItemData::get_mut( "IID_ノーアトゥーン" ).unwrap());
                item_list.add(ItemData::get_mut( "IID_フォルクヴァング" ).unwrap());
                add_ring_to_pool( "RNID_DLC1コモン_1_S".into(), None, 1, None);
                add_ring_to_pool( "RNID_DLC1コモン_2_S".into(), None, 1, None);
                add_ring_to_pool( "RNID_DLC1コモン_3_S".into(), None, 1, None);
                GameVariableManager::make_entry("G_拠点_コンテンツ報酬受け取り済", 1);
                GameVariableManager::set_bool("G_拠点_コンテンツ報酬受け取り済", true);
            }
            create_common_reward_bind(proc, e_list, item_list, 0, false, None);
        }
    }

}
// Item List for well drops and gifts
fn generate_item_list(proc: &ProcInst) -> &'static mut List<ItemData> {
    unsafe { 
        set_well_use_flag(2, None);
        let current_chapter = GameUserData::get_chapter();
        let current_cid = current_chapter.cid.to_string(); 
        let seed = Random::get_system().value() as u32;
        let random = Random::instantiate().unwrap();
        random.ctor(seed);
        let rand_map = GameVariableManager::get_number("G_Continuous") == 3;
        let completed = get_number_main_chapters_completed2();
        if (!rand_map && !GameVariableManager::get_bool("G_Cleared_M006")) || (rand_map && completed <= 7 ) {
            if current_cid == "CID_M004" {
                let patch_items = calc_rewards("Patch0特典".into(), None);
                if continuous_mode_dlc_allowed() {
                    calc_rewards("DLC購入特典0".into(), None).iter().for_each(|item|{
                        if let Some(item1) = ItemData::get_mut(item.iid) { patch_items.add(item1); }
                    });
                    set_patch_flag("G_拠点_DLC特典アイテム0受け取り済み");
                }
                set_patch_flag("G_拠点_Patch0特典アイテム受け取り済み");
                return patch_items;
            }
            else if (!rand_map && current_cid == "CID_M005") ||  (rand_map && completed == 6 ) {
                let patch_items = calc_rewards("Patch3特典".into(), None);
                if continuous_mode_dlc_allowed() {
                    calc_rewards("DLC購入特典1".into(), None).iter().for_each(|item|{
                        if let Some(item1) = ItemData::get_mut(item.iid) { patch_items.add(item1); }
                    });
                    set_patch_flag("G_拠点_DLC特典アイテム1受け取り済み"); 
                }
                set_patch_flag("G_拠点_Patch3特典アイテム受け取り済み");
                return patch_items;
            }
            else {
                let well_items = calc_well_item(proc, 1, random, None);
                if (!rand_map && current_cid == "CID_M006") ||  (rand_map && completed == 7 ) {
                    well_items.add(ItemData::get_mut("IID_トライゾン").unwrap());
                    well_items.add(ItemData::get_mut("IID_ルヴァンシュ").unwrap());
                    if continuous_mode_dlc_allowed() { well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap()); }
                }
                return well_items;
            }
        }
        if (!rand_map && !GameVariableManager::get_bool("G_Cleared_M010") ) || (rand_map && completed < 9){
            let well_items = calc_well_item(proc, 2, random, None);
            if current_cid == "CID_M008" || ( current_cid == "CID_G002" || current_cid == "CID_G005" ) {
                well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap());
                well_items.add(ItemData::get_mut("IID_チェンジプルフ").unwrap());
            }
            return well_items;
        }
        else if (!rand_map && !GameVariableManager::get_bool("G_Cleared_M017")) || ( rand_map && completed < 16 ) { return calc_well_item(proc, 3, random, None); }
        else if (!rand_map && !GameVariableManager::get_bool("G_Cleared_M022")) || ( rand_map && completed < 21 ) { return calc_well_item(proc, 4, random, None); }
        else { return calc_well_item(proc, 5, random, None);  }
    }
}

// When loading save at exploration
pub fn update_next_chapter() {
    if GameVariableManager::get_number("G_Continuous") != 0 { 
        set_next_chapter(); 
        continous_rand_emblem_adjustment();
    }
}
// DLC Check for continous mode
fn continuous_mode_dlc_allowed() -> bool {
    if dlc_check() {
        return GameVariableManager::get_number("G_Continuous") == 1 || GameVariableManager::get_number("G_Continuous")  == 3;
    }
    return false;
}
fn set_patch_flag(flag: &str) {
    GameVariableManager::make_entry(flag, 1);
    GameVariableManager::set_bool(flag, true);
}
fn set_next_chapter() {
    let mode = GameVariableManager::get_number("G_Continuous");
    if mode == 0 || !GameVariableManager::get_bool("G_Cleared_M004") { return; }
    let current_chapter = GameUserData::get_chapter();
    let current_cid = current_chapter.cid.to_string();
    if mode == 3 && GameVariableManager::get_bool("G_Cleared_M004") {
        GameVariableManager::set_bool("G_初回アクセス_錬成屋", true);
        set_next_random_chapter(current_chapter);
        return;
    }
    emblem::emblem_gmap_spot_adjust();
    let dlc = continuous_mode_dlc_allowed();
    //switch or updated without DLC, moves back to main chapters from Divine Paralogue
    if !dlc && current_cid.contains("CID_G") {   
        if let Some(new_chapter) = DLC_CIDS.iter()
            .filter(|c| !c.contains("G00"))
            .find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x)) {
                current_chapter.set_next_chapter(format!("CID_{}", new_chapter).as_str());
        }
        else { current_chapter.set_next_chapter("CID_M010") }
    }
    if dlc && !GameVariableManager::get_bool("G_Cleared_G006") {    // Divine Paralogue order with DLC
        if let Some(new_chapter) = DLC_CIDS.iter().find(|&x| !GameVariableManager::get_bool(format!("G_Cleared_{}", x)) && !current_cid.contains(x)) {
            current_chapter.set_next_chapter(format!("CID_{}", new_chapter).as_str());
        }
        else { current_chapter.set_next_chapter("CID_M010") }
        return;
    }
    if current_cid == "CID_M015" || GameVariableManager::get_bool("G_Cleared_M015") {
        if current_cid == "CID_M021" { return; }
        let rec_level = get_recommended_level_main();
        let chapter_list = ChapterData::get_list_mut().unwrap();
        // paralogue check
        let mut min_rec_level = rec_level;
        let mut chapter_index = 0;
        for x in 29..42 {
            let paralogue = &chapter_list[x];
            if paralogue.cid.to_string() == current_cid { continue; }
            if GameVariableManager::get_bool(&paralogue.get_cleared_flag_name().to_string()) { continue; }    // already completed
            let paralogue_level = paralogue.get_recommended_level();
            // If map to unlock is divine paralogue and the map required to unlock is cleared
            //println!("{} level: {} and {} level: {}", paralogue.cid.to_string(), paralogue_level, current_cid, rec_level );
            let open = paralogue.get_gmap_open_condition().to_string();
            if open.contains("G00") || 
               GameVariableManager::get_bool(&format!("G_Cleared_{}", open)) {
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
            //println!("New Chapter Paralogue: {} at level {}", paralogue.cid.to_string(), paralogue.get_recommended_level());
            return;
        }
        // main chapter
        for x in 16..28 {
            let main = &chapter_list[x];
            if main.cid.to_string() == current_cid { continue; }
            if GameVariableManager::get_bool(&main.get_cleared_flag_name().to_string()) { continue; }
            current_chapter.set_next_chapter(&main.cid.to_string());
            //println!("New Chapter {} (main)", main.cid.to_string());
            return;
        }
    }
}

fn set_next_random_chapter(current_chapter: &ChapterData) {
    let prefixless = current_chapter.get_prefixless_cid().to_string();
    continous_rand_emblem_adjustment();
    if unsafe { crate::randomizer::STATUS.continious_random_chapter == prefixless } && !GameVariableManager::get_bool(format!("G_Cleared_{}", prefixless)) { return; }
    let dlc = continuous_mode_dlc_allowed();

    let completed = GameVariableManager::find_starts_with("G_Cleared_").iter().filter(|key| GameVariableManager::get_bool(key.to_string())).count();
    let mut available: Vec<String> = Vec::new();
    let m011_cleared = GameVariableManager::get_bool("G_Cleared_M011");
    let m011 = GameVariableManager::get_bool("G_Cleared_M006") && GameVariableManager::get_bool("G_Cleared_M008");
    ["M005", "M006", "M007", "M008", "M009", "M010", "M012", "M013", "M014", "M015", "M016", "M018", "S001", "S002"].iter()
        .for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) {available.push(key.to_string());} );

    if m011_cleared { ["M017", "M019", "M020", "M021"].iter().for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) { available.push(key.to_string()); } ); }
    else if m011 { available.push("M011".to_string());  }

    if completed >= 15 {    // Paralogues
        for x in 0..12 {
            let e_index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[x as usize].to_string(), false);
            let para = EMBELM_PARA[ x as usize ];
            if e_index == 13 {
                if is_god_available(EMBLEM_GIDS[x as usize], false) && m011_cleared {
                    if !GameVariableManager::get_bool(format!("G_Cleared_{}", para)) { available.push(para.to_string()); }
                }
            }
            else if ( ( x < 6 && m011_cleared ) || x >= 6 ) && is_god_available(EMBLEM_GIDS[x as usize], false) {
                if !GameVariableManager::get_bool(format!("G_Cleared_{}", para)) { available.push(para.to_string()); } 
            }
        }
    }
    let m022 = ["M011", "M013", "M014", "M015", "M016", "M017", "M019", "M020", "M021"].iter().filter(|&x|GameVariableManager::get_bool(format!("G_Cleared_{}", x))).count();
    if m022 == 9 { 
        if !GameVariableManager::get_bool("G_Cleared_M022") { available.push("M022".to_string()); }
        else if !GameVariableManager::get_bool("G_Cleared_S015") { available.push("S015".to_string()); }
    }
    if completed >= 20 && GameVariableManager::get_bool("G_Cleared_M022") { 
        ["M023", "M024", "M025"].iter().for_each(|key| if !GameVariableManager::get_bool(format!("G_Cleared_{}", key)) {available.push(key.to_string());} );
    }
    if dlc {
        for x in 1..7 {
            let god = format!("G00{}", x);
            if !GameVariableManager::get_bool(format!("G_Cleared_{}", god)) {available.push(god); }
        }
    }
    if let Some(pos) = available.iter().position(|key| *key == prefixless) { available.remove(pos); }
    if available.len() == 0 {
        current_chapter.set_next_chapter("CID_M026");
        return; 
    }
    let rng = Random::get_game();
    let mut key= available[ rng.get_value( available.len() as i32 ) as usize ].to_string();

    if dlc {
        let mut count = 0;
        while count < 3 {
            key = available[ rng.get_value( available.len() as i32 ) as usize ].to_string();
            if key.contains("G00") { count += 1; }
            else { break; }
        }
    }
    let cid = format!("CID_{}", key);

    if GameUserData::get_sequence() == 7 {
        let chapter = current_chapter.cid;
        if chapter.contains("M011") && !m011 || chapter.contains("M022") && m022 < 9 {
            GameUserData::set_chapter(ChapterData::get(cid).unwrap());
            return;
        }
    }
    println!("Current Chapter: {}", current_chapter.cid);
    println!("New Random Chapter: {} out of {} Possible", cid, available.len() );
    println!("Number of Map Completed: {}", completed);
    println!("Number of Story Maps Completed: {}", get_story_chapters_completed());
    unsafe { crate::randomizer::STATUS.continious_random_chapter = prefixless.to_string() };
    available.iter().for_each(|cid| println!("{}", cid));
    current_chapter.set_next_chapter(cid.as_str());
}

// Function to auto collect bond frags
fn bond_frags_from_achievement(this: &AchieveData) -> i32 {
    let status = unsafe { achieve_status(this, None) };
    if status == 1 || status == 2 {
        unsafe { achieve_set_status(this, 3, None) };
        return unsafe { achieve_reward(this, None) };
    }
    return 0;
}

fn collect_bond_frags_from_achievements() {
    AchieveData::get_list().unwrap().iter().for_each(|achieve| { GameUserData::add_bond( bond_frags_from_achievement( achieve ) ); });
}

pub fn update_bonds() {
    if GameVariableManager::get_number("G_Continuous") == 0 { return; }
    for x in 0..19 {
        let gid = format!("GID_{}", EMBLEM_ASSET[x]);
        let god = GodData::get(&gid).unwrap();
        let god_unit = unsafe { try_get_god(god, true, None) };

        if god_unit.is_none() { continue; }
        let g_unit = god_unit.unwrap();
        let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
        let mut max_level = 1;
        let mut bond_exp: u16 = 0;
    // Get highest bond level 
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            for unit in force_iter {
                let god_bond = unsafe { get_bond(g_unit, unit, None) };
                if god_bond.is_none() { continue; } 
                let g_bond = god_bond.unwrap();
                if g_bond.level == 4 { unsafe { level_up_bond(g_bond, None) }; }   // C Bond
                if g_bond.level == 19 { unsafe { level_up_bond(g_bond, None) }; }  // A Bond
                if max_level < g_bond.level { 
                    max_level = g_bond.level;
                    bond_exp = g_bond.exp;
                }
            }
        }
                // level up to highest bond level
        let force_type2: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
        for ff2 in force_type2 {
            let force_iter2 = Force::iter(Force::get(ff2).unwrap());
            for unit in force_iter2 {
                let god_bond = unsafe { get_bond(g_unit, unit, None) };
                if god_bond.is_none() { continue; } 
                let g_bond = god_bond.unwrap();
                if g_bond.level < max_level {
                    let n_levels = max_level - g_bond.level;
                    for _x in 0..n_levels { unsafe { level_up_bond(g_bond, None) }; }
                    g_bond.exp = bond_exp;
                    unsafe { inherit_apt_from_god(unit, g_unit, None) };   
                }   
            }  
        }
    }
}

fn create_bond_rings() {
    collect_bond_frags_from_achievements();
    if !GameVariableManager::get_bool("G_Cleared_M003") { return; }
    let ring_list = RingData::get_list().unwrap();
    let mut active_emblems = 0;
    for x in 0..12 {
        let god_unit = GodPool::try_get_gid(EMBLEM_GIDS[x], true);
        if god_unit.is_none() { continue; }
        let bond_variable_count = format!("G_{}_Ring_Count", EMBLEM_ASSET[x]);
        if !GameVariableManager::exist(&bond_variable_count) { GameVariableManager::make_entry(&bond_variable_count, 0); }
        if GameVariableManager::get_number(&bond_variable_count) == 0 {
            for _x in 0..10 {
                let rank = get_ring_rank() as usize; 
                let ring_index = Random::get_game().get_value(10) as usize;
                let ring_index = x * 40 + ring_index * 4 + rank;
                add_ring(ring_list[ring_index].rid);
            }
            GameVariableManager::set_number(&bond_variable_count, 10);
        }
        if !god_unit.unwrap().get_escape() { active_emblems += 1; }
    }
    // DLC
    for x in 12..19 {
        let god_unit = GodPool::try_get_gid(EMBLEM_GIDS[x], true);
        if god_unit.is_none() { continue; }
        if !god_unit.unwrap().get_escape() { active_emblems += 1; }
    }
    GameUserData::add_bond(500);
    let bond_frag = GameUserData::get_piece_bond();
    let mut ring_count = 0;
    //println!("Bond Frags: {}", bond_frag);
    //println!("Active Embelms: {}", active_emblems);
    let total_cost = 1100*active_emblems;
    if total_cost < bond_frag && ring_count < 700 {
        for x in 0..12 {
            let god_unit = GodPool::try_get_gid(EMBLEM_GIDS[x], true);
            if god_unit.is_none() { continue; }
            if god_unit.unwrap().get_escape() { continue; }
            for _x in 0..5 {
                let rank = get_ring_rank() as usize; 
                let ring_index = Random::get_game().get_value(10) as usize;
                let ring_index = x * 40 + ring_index * 4 + rank;
                add_ring(ring_list[ring_index].rid);
        }   }
        GameUserData::add_bond(-500*active_emblems);
    }
    // Auto merge rings
    let ring_cost = [game_parameter("指輪合成指輪コストB"), game_parameter("指輪合成指輪コストA"), game_parameter("指輪合成指輪コストS")];
    ring_count = 0;
    for x in 0..12 {    // Emblem Index
        for y in 0..10 {   // Ring Index
            for z in 0..3 {    //Rank Index
                let index = x*40 + y*4 + z;
                let ring = &ring_list[index];
                let count =  unsafe { unit_ring_pool_stock_count(ring, None) };
                if count == 0 { continue; }
                if count >= ring_cost[z] {
                    unsafe { sub_ring_to_pool(ring.rid, None, ring_cost[z], None) };
                    add_ring(ring_list[index + 1].rid); 
                }
            }
        }
    }
    for x in 0..ring_list.len()-3 {  unsafe { ring_count += unit_ring_pool_stock_count(ring_list[x], None); } } // getting new count of bond rings
}

fn get_ring_rank() -> i32 {
    let rng = Random::get_game();
    let value = rng.get_value(100);
    let s_rate = 2*game_parameter("指輪精製確率S");
    let a_rate = game_parameter("指輪精製確率A") + s_rate;
    let b_rate = game_parameter("指輪精製確率B") + a_rate;
    if value < s_rate { return 3; }
    else if value < a_rate { return 2; }
    else if value < b_rate { return 1; }
    else { return 0; }
}

fn game_parameter(value: &str) -> i32 { unsafe { get_game_parameter(value.into(), None) } }
fn add_ring(ring_id: &Il2CppString) { unsafe { add_ring_to_pool(ring_id, None, 1, None); } }

fn do_dlc() {
    if !continuous_mode_dlc_allowed() { return; }
    let current_cid = GameUserData::get_chapter().cid.to_string();
    let random = GameVariableManager::get_number("G_Continuous") == 3;
    let completed = get_story_chapters_completed();
    if (!random && current_cid == "CID_M006" ) || ( random && completed >= 4 ) {
        let god;
        if GameVariableManager::get_number("G_Emblem_Mode") != 1 { god = GodData::get("GID_エーデルガルト").unwrap(); }
        else {
            let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト").to_string(); 
            god = GodData::get(&gid).unwrap();
        }
        unsafe { godpool_create(god, None); }
    }
    if (!random && current_cid == "CID_M017" ) || ( random && completed == 16 ) {
        GameVariableManager::set_bool("G_CC_エンチャント", true);   // enable dlc seals
        GameVariableManager::set_bool("G_CC_マージカノン", true);
        GameVariableManager::set_number("G_所持_IID_マージカノン専用プルフ", GameVariableManager::get_number("G_所持_IID_マージカノン専用プルフ") + 1); // add dlc deals
        GameVariableManager::set_number("G_所持_IID_エンチャント専用プルフ", GameVariableManager::get_number("G_所持_IID_エンチャント専用プルフ") + 1);
        let persons = ["PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
        for x in persons {
            if GameVariableManager::get_number("G_Random_Recruitment") != 0 {
                let new_person = GameVariableManager::get_string(&format!("G_R_{}", x)).to_string();
                let person_data = PersonData::get(&new_person).unwrap();
                GameVariableManager::make_entry("MapRecruit", 1);
                GameVariableManager::set_bool("MapRecruit", true);
                unsafe { join_unit(person_data, None); }
                GameVariableManager::set_bool("MapRecruit", false);
            }
            else {
                let person_data = PersonData::get(x).unwrap();
                unsafe { join_unit(person_data, None); }
            }
        }
    }
}

pub fn update_ignots(){
    let add_iron = get_number_main_chapters_completed2()*5;
    let mut add_steel = 0;
    let mut add_silver = 0;
    GameUserData::add_iron( add_iron );
    if GameVariableManager::get_bool("G_Cleared_G001") { add_silver += 1; } 
    if GameVariableManager::get_bool("G_Cleared_M004") { add_steel += 1;} 
    if GameVariableManager::get_number("G_Continuous") < 3 {
        if GameVariableManager::get_bool("G_Cleared_M005") { add_steel += 5; } 
        if GameVariableManager::get_bool("G_Cleared_M006") { add_steel += 5; } 
        if GameVariableManager::get_bool("G_Cleared_M007") { add_silver += 1; } 
        if GameVariableManager::get_bool("G_Cleared_M009") { add_silver += 1; } 
        if GameVariableManager::get_bool("G_Cleared_M011") { add_silver += 1; } 
        if GameVariableManager::get_bool("G_Cleared_M016") { 
            add_silver += 1;
            add_steel += 5;
        }
    }
    else {
        let cleared = get_story_chapters_completed();
        if cleared >= 5 { add_steel += 1;} 
        if cleared >= 6 { add_steel += 1;} 
        if cleared >= 7 { add_silver += 1; } 
        if cleared >= 9 { add_silver += 1; } 
        if cleared >= 11 { add_silver += 1 }
        if cleared >= 16 { 
            add_silver += 1;
            add_steel += 5;
        }
    }
    GameUserData::add_steel( add_steel );
    GameUserData::add_silver( add_silver );
}
pub fn get_number_main_chapters_completed2() -> i32 {
    let mut number = 0;
    ChapterData::get_list().unwrap().iter()
        .for_each(|chapter| if GameUserData::is_chapter_completed(chapter) { number += 1; });
    number
}
pub fn get_story_chapters_completed() -> i32 {
    let mut number = 0;
    GameVariableManager::find_starts_with("G_Cleared_M0").iter()
        .for_each(|cleared| if GameVariableManager::get_bool(cleared.to_string()) { number += 1 });
    number
}

pub fn get_story_completed_prefix(next: bool) -> String {
    let value = if next { 1 } else { 0 } + get_story_chapters_completed();
    if value < 10 { format!("M00{}", value) }
    else { format!("M0{}", value) }
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

// Support Stuff 
pub fn add_support_points() {
    let unit_list = unsafe { my_room_reliance_select_get_unit_list(None) };
    for x in 0..unit_list.len() {
        let unit_a = &unit_list[x];
        let is_a_deployed = unit_a.force.unwrap().force_type == 0;
        let is_lueur = unit_a.person.pid.to_string() == "PID_リュール";
        for y in x+1..unit_list.len(){
            let unit_b = &unit_list[y];
            if let Some(reliance_data) = unsafe { unit_reliance_try_get(unit_a, unit_b, None) } {
                let is_b_deployed = unit_b.force.unwrap().force_type == 0;
                let exp_needed = unsafe { unit_get_exp_next_level(reliance_data, reliance_data.level, None) };
                if exp_needed != 100 && exp_needed != 0 { reliance_data.exp += 1 + (is_a_deployed || is_b_deployed) as i8 + is_lueur as i8; }
            }
        }
    }
}

pub struct ContiniousMode;
impl ConfigBasicMenuItemSwitchMethods for ContiniousMode {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result;
        if dlc_check() { result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().continuous, 0, 3, 1); }
        else { result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().continuous, 0, 2, 1); }
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
            else {"Game will progress with access to the Somniel and World Map" }
        }
        else {
            if mode == 1 { "Game will progress map to map." }
            else if mode == 2 { "Game will progress map to map in random order."}
            else { "Game will progress with access to the Somniel and World Map" }
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let mode = CONFIG.lock().unwrap().continuous;
        this.command_text =  if dlc_check() {
            if mode == 1 { "Enabled" }
            else if mode == 2 { "Enabled w/o DLC" }
            else if mode == 3 { "Random" }
            else { "Disabled" }
        }
        else {
            if mode == 1 { "Enabled" }
            else if mode == 2 { "Random" }
            else { "Disabled" }
        }.into();
    }
}

pub fn continous_rand_emblem_adjustment() {
    if GameVariableManager::get_number("G_Continuous") < 3 { return; }
    // if !in_map_chapter() { return; }
    unsafe { enable_map_rewind(None); }
    if GameVariableManager::get_bool("G_Cleared_M022") { for x in 0..12 { escape_god(EMBLEM_GIDS[x], false); }   return; }
    let current_chapter = GameUserData::get_chapter().cid.to_string();
    if current_chapter.contains("M011") && !GameVariableManager::get_bool("G_Cleared_M011") {
        for x in 0..6 { escape_god(EMBLEM_GIDS[x], true); }  
        return;
    }
    if current_chapter.contains("M022") {
        escape_god(EMBLEM_GIDS[0], false);
        for x in 1..12 { escape_god(EMBLEM_GIDS[x], true); } 
        return;
    }
    if GameVariableManager::get_bool("G_Cleared_M010") && !GameVariableManager::get_bool("G_Cleared_M011"){
        for x in 0..6 { escape_god(EMBLEM_GIDS[x], false); }  
    }
    if GameVariableManager::get_bool("G_Cleared_M021") && !GameVariableManager::get_bool("G_Cleared_M022"){
        for x in 0..12 { escape_god(EMBLEM_GIDS[x], false); }  
    }

}

pub fn escape_god(gid: &str , escape: bool) {
    if let Some(god_data) = if GameVariableManager::get_number("G_Emblem_Mode") == 0 { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = unsafe { try_get_god(god_data, true, None) } {
            unsafe { god_unit_set_escape(god_unit, escape, None) };
            if escape {
                if let Some(parent) = unsafe {god_unit_get_parent(god_unit, None) } {
                    unsafe { unit_clear_parent(parent, None);}
                }
            }
        }
    }
}
fn is_god_available(gid: &str, randomized: bool) -> bool {
    if let Some(god_data) = if GameVariableManager::get_number("G_Emblem_Mode") == 0 || !randomized { GodData::get(gid) }
        else { GodData::get( GameVariableManager::get_string(format!("G_R_{}", gid)))} {
        if let Some(god_unit) = unsafe { try_get_god(god_data, true, None) } {
            return !god_unit.get_escape();   
        }
        else { return false; }
    }
    false
}
#[skyline::from_offset(0x01ddfc50)]
fn enable_map_rewind(method_info: OptionalMethod);

#[skyline::from_offset(0x02280c20)]
fn get_game_parameter(value: &Il2CppString, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c5cf40)]
fn unit_ring_pool_stock_count(data: &RingData, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c5d420)]
fn add_ring_to_pool(rnid: &Il2CppString, owner: Option<&Unit>, count: i32, method_info: OptionalMethod) -> &'static UnitRing;

#[skyline::from_offset(0x01c5d5b0)]
fn sub_ring_to_pool(rnid: &Il2CppString, owner: Option<&Unit>, count: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x01c73960)]
fn join_unit(person: &PersonData, method_info: OptionalMethod) -> Option<&Unit>;

#[skyline::from_offset(0x020193e0)]
fn calc_rewards(name: &Il2CppString, method_info: OptionalMethod) -> &'static mut List<ItemData>;

#[skyline::from_offset(0x023405b0)]
fn get_bond(this: &GodUnit, unit: &Unit, method_info: OptionalMethod) -> Option<&'static mut GodBond>;

#[skyline::from_offset(0x02334600)]
pub fn try_get_god(gid: &GodData, included_reserved: bool,  method_info: OptionalMethod) -> Option<&GodUnit>;

#[skyline::from_offset(0x02b4dff0)]
fn level_up_bond(this: &GodBond, method_info: OptionalMethod);

#[skyline::from_offset(0x023349c0)]
fn godpool_create(this: &GodData, method_info: OptionalMethod) -> Option<&'static GodUnit>;

#[skyline::from_offset(0x0233eaf0)]
fn god_unit_set_escape(this: &GodUnit, escape: bool, method_info: OptionalMethod);

#[skyline::from_offset(0x01a4f4c0)]
pub fn unit_clear_parent(this: &Unit, method_info: OptionalMethod);

#[skyline::from_offset(0x0233ffb0)]
fn god_unit_get_parent(this: &GodUnit, method_info: OptionalMethod) -> Option<&'static Unit>;

#[skyline::from_offset(0x02939dc0)]
pub fn set_exchange_level(level: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x02532f40)]
fn create_common_reward_bind(proc: &ProcInst, exp: &Dictionary<&Unit, i32>, items: &List<ItemData>, money: i32, is_bg: bool, method_info: OptionalMethod);

#[skyline::from_offset(0x0293c890)]
fn calc_well_item(proc: &ProcInst, level: i32, random: &Random, method_info: OptionalMethod) -> &'static mut List<ItemData>;

#[skyline::from_offset(0x01a3dd90)]
fn inherit_apt_from_god(this: &Unit, god: &GodUnit, method_info: OptionalMethod);

#[skyline::from_offset(0x027c7380)]
fn achieve_status(this: &AchieveData, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x027c74b0)]
fn achieve_set_status(this: &AchieveData, status: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x027c6b30)]
fn achieve_reward(this: &AchieveData, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x01c5a930)]
pub fn reliance_can_level_up( unit_a: &Unit, unit_b: &Unit, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "MyRoomRelianceSelect","GetUnitList")]
pub fn my_room_reliance_select_get_unit_list(method_info: OptionalMethod) -> &'static List<Unit>;

#[skyline::from_offset(0x01c5a040)]
pub fn unit_reliance_try_get(unit_a: &Unit, unit_b: &Unit, method_info: OptionalMethod) -> Option<&'static mut UnitRelianceData>;

#[skyline::from_offset(0x01c5c450)]
pub fn unit_get_exp_next_level(this: & UnitRelianceData, current_level: i32, method_info: OptionalMethod ) -> i32;

#[skyline::from_offset(0x01c5ae10)]
fn can_be_a_plus_support(unit_a: &Unit, unit_b: &Unit, method_info: OptionalMethod ) -> bool;

#[skyline::from_offset(0x01c5abf0)]
fn level_up_support(unit_a: &Unit, unit_b: &Unit, method_info: OptionalMethod ) -> bool;

#[skyline::from_offset(0x01c5b020)]
fn set_level_a_plus(unit_a: &Unit, unit_b: &Unit, method_info: OptionalMethod );

#[skyline::from_offset(0x02939a80)]
pub fn set_well_use_flag(flag: i32, method_info: OptionalMethod);

#[skyline::from_offset(0x03cbca00)]
fn dictionary_ctor(this: &Dictionary<&Unit, i32>, method_info: OptionalMethod);
