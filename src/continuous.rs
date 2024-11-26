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
        let current_chapter = GameUserData::get_chapter();
        let flag = current_chapter.get_flag();
        let current_chapter = GameUserData::get_chapter();
        let current_cid = current_chapter.cid.to_string();
        if current_cid == "CID_G006" { current_chapter.set_next_chapter("CID_M010"); }
        let mut new_flag = flag;
        if flag & 16 != 0 { new_flag -= 16; }   // hub  disable
        if flag & 32 != 0 { new_flag -= 32; }   // gmap  disable
        if flag & 2 != 0 { new_flag -= 2; } // can back disable
        current_chapter.set_flag(new_flag);
        set_next_chapter();
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
    }
}

pub fn continous_mode_post_battle_stuff(proc: &ProcInst){
    if GameVariableManager::get_number("G_Continuous") == 0 { return; }
    if GameUserData::get_chapter().cid.to_string() == "CID_M026" { return; }
    if GameVariableManager::get_bool("G_Cleared_M026") { return; }
    let chapter = GameUserData::get_chapter();
    let flag = chapter.get_flag();
    let mut new_flag = flag;
    if flag & 16 != 0 { new_flag = new_flag - 16; } // Hub Removal
    if flag & 32 != 0 { new_flag = new_flag - 32; } // Gmap Removal
    if flag & 2 != 0 { new_flag -= 2; } // Back Removal
    chapter.set_flag(new_flag);
    add_support_points();
    set_next_chapter();
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
            for ff in force_type {
                let force_iter = Force::iter(Force::get(ff).unwrap());
                for unit in force_iter {
                    if unit.person.pid.to_string() == "PID_残像" { continue; }    // Lyn doubles are a no-no
                    if unit.level == unit.job.max_level { 
                        unit.add_sp(base_exp_gain);
                        continue; 
                    }
                    // level scaling    
                    let total_level = unit.level as i32 + unit.internal_level as i32;
                    if total_level < level_cap { 
                        e_list.add(unit, base_exp_gain); 
                        unit.add_sp(base_exp_gain);
                    }
                    else {
                        let diff = total_level - level_cap;
                        let exp_gain = base_exp_gain / (  2 + diff );
                        e_list.add(unit, exp_gain);
                        unit.add_sp(exp_gain);
                    }
                }
                base_exp_gain = clamp_value(base_exp_gain * 5 / 3 , base_exp_gain, 100);
                level_cap -= 5; 
                if base_exp_gain == 0 { break; }
            }
            // Heroes DLC
            if !GameVariableManager::get_bool("G_拠点_コンテンツ報酬受け取り済") && has_content(1, None) {
                item_list.add(ItemData::get_mut( "IID_フェンサリル" ).unwrap());
                item_list.add(ItemData::get_mut( "IID_ノーアトゥーン" ).unwrap());
                item_list.add(ItemData::get_mut( "IID_フォルクヴァング" ).unwrap());
                add_ring_to_pool( "RNID_DLC1コモン_1_S".into(), None, 1, None);
                add_ring_to_pool( "RNID_DLC1コモン_2_S".into(), None, 1, None);
                add_ring_to_pool( "RNID_DLC1コモン_3_S".into(), None, 1, None);
                GameVariableManager::make_entry("G_拠点_コンテンツ報酬受け取り済", 1);
                GameVariableManager::set_bool("G_拠点_コンテンツ報酬受け取り済", true);
            }
            if GameUserData::get_chapter().cid.to_string() == "CID_S015" {
                GameVariableManager::make_entry("G_所持_IID_約束の指輪", 40);
                GameVariableManager::set_number("G_所持_IID_約束の指輪", 40);
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
        if !GameVariableManager::get_bool("G_Cleared_M006") {
            if current_cid == "CID_M004" {
                let patch_items = calc_rewards("Patch0特典".into(), None);
                if continuous_mode_dlc_allowed() {
                    let dlc_items = calc_rewards("DLC購入特典0".into(), None);
                    let n_items = dlc_items.len();
                    for x in 0..n_items {
                        let item = dlc_items[x].iid;
                        patch_items.add(ItemData::get_mut(&item.to_string()).unwrap());
                    } 
                }
                return patch_items;
            }
            else if current_cid == "CID_M005" {
                let patch_items = calc_rewards("Patch3特典".into(), None);
                if continuous_mode_dlc_allowed() {
                    let dlc_items = calc_rewards("DLC購入特典1".into(), None);
                    for x in 0..dlc_items.len() {
                        let item = dlc_items[x].iid;
                        patch_items.add(ItemData::get_mut(&item.to_string()).unwrap());
                    } 
                }
                return patch_items;
            }
            else {
                let well_items = calc_well_item(proc, 1, random, None);
                if current_cid == "CID_M006" {
                    well_items.add(ItemData::get_mut("IID_トライゾン").unwrap());
                    well_items.add(ItemData::get_mut("IID_ルヴァンシュ").unwrap());
                    if continuous_mode_dlc_allowed() { well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap()); }
                }
                return well_items;
            }
        }
        if !GameVariableManager::get_bool("G_Cleared_M010") {
            let well_items = calc_well_item(proc, 2, random, None);
            if current_cid == "CID_M008" || ( current_cid == "CID_G002" || current_cid == "CID_G005" ) {
                well_items.add(ItemData::get_mut("IID_マスタープルフ").unwrap());
                well_items.add(ItemData::get_mut("IID_チェンジプルフ").unwrap());
            }
            return well_items;
        }
        else if !GameVariableManager::get_bool("G_Cleared_M017") { return calc_well_item(proc, 3, random, None); }
        else if !GameVariableManager::get_bool("G_Cleared_M022") { return calc_well_item(proc, 4, random, None); }
        else { return calc_well_item(proc, 5, random, None);  }
    }
}

// When loading save at exploration
pub fn update_next_chapter() {
    if GameVariableManager::get_number("G_Continuous") != 0 && GameUserData::get_sequence() == 5 { set_next_chapter(); }
}
// DLC Check for continous mode
fn continuous_mode_dlc_allowed() -> bool {
    if dlc_check() {
        if GameVariableManager::get_number("G_Continuous") == 1 { return true; }
        else { return false; }
    }
    return false;
}

fn set_next_chapter() {
    if GameVariableManager::get_number("G_Continuous") == 0 { return; }
    if !GameVariableManager::get_bool("G_Cleared_M004") { return; }
    emblem::emblem_gmap_spot_adjust();
    let current_chapter = GameUserData::get_chapter();
    let current_cid = current_chapter.cid.to_string();
    if current_cid == "CID_M005" { current_chapter.set_next_chapter("CID_S001"); }
    if current_cid == "CID_S001" { current_chapter.set_next_chapter("CID_M006"); }
    let dlc = continuous_mode_dlc_allowed();
    if current_cid == "CID_M006" {
        if dlc { current_chapter.set_next_chapter("CID_G001"); }
        else { current_chapter.set_next_chapter("CID_S002"); }
        return;
    }
    if current_cid == "CID_S002" && !dlc { current_chapter.set_next_chapter("CID_M007"); }

    if !continuous_mode_dlc_allowed() &&  crate::autolevel::str_start_with(&current_chapter.cid, "CID_G") {   //switch or updated without DLC, moves back to main chapters from Divine Paralogue
        if !GameVariableManager::get_bool("G_Cleared_S002") { current_chapter.set_next_chapter("CID_S002");}
        else if !GameVariableManager::get_bool("G_Cleared_M007") { current_chapter.set_next_chapter("CID_M007");}
        else if !GameVariableManager::get_bool("G_Cleared_M008") { current_chapter.set_next_chapter("CID_M008");}
        else if !GameVariableManager::get_bool("G_Cleared_M009") { current_chapter.set_next_chapter("CID_M009");}
        else { current_chapter.set_next_chapter("CID_M010");} 
        return;
    }
    if dlc && !GameVariableManager::get_bool("G_Cleared_G006") {    // Divine Paralogue order with DLC
        if current_cid == "CID_G001" { current_chapter.set_next_chapter("CID_S002"); }
        if current_cid == "CID_S002" { current_chapter.set_next_chapter("CID_G002"); }
        if current_cid == "CID_G002" { current_chapter.set_next_chapter("CID_M007"); }
        if current_cid == "CID_M007" { current_chapter.set_next_chapter("CID_G003"); }
        if current_cid == "CID_G003" { current_chapter.set_next_chapter("CID_M008"); }
        if current_cid == "CID_M008" { current_chapter.set_next_chapter("CID_G004"); }
        if current_cid == "CID_G004" { current_chapter.set_next_chapter("CID_M009"); }
        if current_cid == "CID_M009" { current_chapter.set_next_chapter("CID_G005"); }
        if current_cid == "CID_G005" { current_chapter.set_next_chapter("CID_G006"); }
        if current_cid == "CID_G006" { current_chapter.set_next_chapter("CID_M010"); }
        return;
    }
    if current_cid == "CID_M015" || GameVariableManager::get_bool("G_Cleared_M015") {
        if current_cid == "CID_M021" { return; }
        let rec_level = get_recommended_level_main();
        let chapter_list = ChapterData::get_list().unwrap();
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
            if crate::utils::str_contains(paralogue.get_gmap_open_condition(), "G00") || 
               GameVariableManager::get_bool(&format!("G_Cleared_{}", paralogue.get_gmap_open_condition().to_string())) {
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
    let list = AchieveData::get_list().unwrap();
    for x in 0..list.len() {
        let bond = bond_frags_from_achievement(list[x]);
        GameUserData::add_bond(bond);
    }
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
    if current_cid == "CID_M006" {
        let god;
        if GameVariableManager::get_number("G_Emblem_Mode") != 1 { god = GodData::get("GID_エーデルガルト").unwrap(); }
        else {
            let gid = GameVariableManager::get_string("G_R_GID_エーデルガルト").to_string(); 
            god = GodData::get(&gid).unwrap();
        }
        unsafe { godpool_create(god, None); }
    }
    if current_cid == "CID_M017" {
        GameVariableManager::set_bool("G_CC_エンチャント", true);   // enable dlc seals
        GameVariableManager::set_bool("G_CC_マージカノン", true);
        GameVariableManager::set_number("G_所持_IID_マージカノン専用プルフ", GameVariableManager::get_number("G_所持_IID_マージカノン専用プルフ") + 1); // add dlc deals
        GameVariableManager::set_number("G_所持_IID_エンチャント専用プルフ", GameVariableManager::get_number("G_所持_IID_エンチャント専用プルフ") + 1);
        let persons = ["PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
        for x in persons {
            if GameVariableManager::get_bool("G_Random_Recruitment") {
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
    if GameVariableManager::get_bool("G_Cleared_M004") { add_steel += 1;} 
    if GameVariableManager::get_bool("G_Cleared_M005") { add_steel += 5; } 
    if GameVariableManager::get_bool("G_Cleared_M006") { add_steel += 5; } 
    if GameVariableManager::get_bool("G_Cleared_G001") { add_silver += 1; } 
    if GameVariableManager::get_bool("G_Cleared_M007") { add_silver += 1; } 
    if GameVariableManager::get_bool("G_Cleared_M009") { add_silver += 1; } 
    if GameVariableManager::get_bool("G_Cleared_M011") { add_silver += 1; } 
    if GameVariableManager::get_bool("G_Cleared_M016") { 
        add_silver += 1;
        add_steel += 5;
    }
    GameUserData::add_steel( add_steel );
    GameUserData::add_silver( add_silver );
}
pub fn get_number_main_chapters_completed2() -> i32 {
    let mut number = 0;
    let chapters = ChapterData::get_list_mut().expect(":D");
    for x in 0..chapters.len() { if GameUserData::is_chapter_completed(chapters[x]) { number += 1; } }
    number
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
fn try_get_god(gid: &GodData, included_reserved: bool,  method_info: OptionalMethod) -> Option<&GodUnit>;

#[skyline::from_offset(0x02b4dff0)]
fn level_up_bond(this: &GodBond, method_info: OptionalMethod);

#[skyline::from_offset(0x023349c0)]
fn godpool_create(this: &GodData, method_info: OptionalMethod) -> Option<&'static GodUnit>;

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