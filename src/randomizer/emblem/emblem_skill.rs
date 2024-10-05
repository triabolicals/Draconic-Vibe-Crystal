use super::*;
use super::{emblem_item::*, emblem_structs::*};
use super::super::skill::SkillIndex;
use std::sync::Mutex;
use engage::{
    random::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    gamedata::{GodData, god::*},
};
use skyline::patching::Patch;
use crate::CONFIG;

const EMBLEM_WEAPON: [i32; 20] = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258];
pub static mut ENGAGE_SKILL_COUNT: i32 = 19;
pub static ENGAGE_SKILLS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static ADDED_ENGAGE:  Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static ADDED_SYNC: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static ENGAGE_SKILLS_CHAOS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());

static ENGAGE_ATTACKS: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());
static ENGAGE_ATK_SWAP: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());

pub static STAT_BONUS:  Mutex<[i32; 66]> = Mutex::new([0; 66]);
pub static SYNCHO_RANDOM_LIST: Mutex<SynchoList> = Mutex::new(
    SynchoList { sync_list: Vec::new(), inherit_list: Vec::new(), chaos_list: Vec::new(), randomized: false, sync_list_size: 0, non_random_skills: Vec::new()}
);
pub static mut EIRIKA_INDEX: usize = 11;

pub struct EmblemSkillChaos;
impl ConfigBasicMenuItemSwitchMethods for EmblemSkillChaos {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_skill_chaos, 0, 4, 1);
        if CONFIG.lock().unwrap().emblem_skill_chaos != result {
            CONFIG.lock().unwrap().emblem_skill_chaos = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = match CONFIG.lock().unwrap().emblem_skill_chaos {
            1 => { "Expands the randomization pool for sync skills." },
            2 => { "Expands the randomization pool for engage skills." },
            3 => { "Expands the randomization pool for engage and sync skills." },
            4 => { "Expands the randomization pool by DVC List." },
            _ => { "Default randomization pool for sync and engage skills."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_skill_chaos {
            1 => { "Sync Skills" },
            2 => { "Engage Skills" },
            3 => { "Sync / Engage "},
            4 => { "Use List"},
            _ => { "Default Skills"},
        }.into();
    }
}


pub fn create_emblem_skill_pool() {
    // get skill index of hidden stat boost for emblems stat sync bonuses.
    for x in 0..11 {
        if x == 9 { continue; } // No Sight
        for y in 1..7 { STAT_BONUS.lock().unwrap()[ 6*x + y-1 ] = find_emblem_stat_bonus_index(x as i32, y as i32); }
    }
    // Get all syncho skills to the random list  //Add Gambit    
    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let keys = GodGrowthData::get_level_data(&growth_id);
        if keys.is_some() {
            let level_data = keys.unwrap();
            let engage_skill = level_data[0].engage_skills[0].get_skill().unwrap();
            ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(engage_skill.parent.index));
            for y in 0..level_data.len() {
                for z in 0..level_data[y].synchro_skills.list.size {
                    let skill = level_data[y].synchro_skills[z as usize].get_skill().unwrap();
                    SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, true);
                }
            }
        }
    }
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 {
        let god_list = GodData::get_list().unwrap();
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1] as usize;
            if c_index >= god_list.len() { continue; }
            let growth_data = god_list[c_index].get_grow_table();
            if growth_data.is_none() { continue; }
            let lvl_data = GodGrowthData::get_level_data(&growth_data.unwrap().get_string().unwrap());
            if lvl_data.is_none() { continue; }
            let level_data = lvl_data.unwrap();
            let engage_skill = level_data[0].engage_skills[0].get_skill().unwrap();
            ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(engage_skill.parent.index));
            for y in 0..level_data.len() {
                for z in 0..level_data[y].synchro_skills.list.size {
                    let skill = level_data[y].synchro_skills[z as usize].get_skill().unwrap();
                    SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, true);
                }
            }
        }
    }


    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_勇空＋", true, false);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_太陽の腕輪＋", true, false);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_日月の腕輪＋", true, false);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_蒼穹＋", true, false);
    for x in crate::enums::EXTRA_SYNCS { SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade(x, true);  }
    SYNCHO_RANDOM_LIST.lock().unwrap().get_sync_list_size(); // Calc size
    let mut count = 0;
    for x in 0..20 {
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let engage_index = SkillData::get_index( god.get_engage_attack() );
        if engage_index != -1 {
            ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(engage_index as i32, count));
        }
        let link_engage = god.get_engage_attack_link();
        if link_engage.is_some() {
            let link_index = SkillData::get_index( link_engage.unwrap() );
            if link_index != -1 { ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(link_index as i32, count)); }
        }
        count += 1;
    }
    unsafe { ENGAGE_SKILL_COUNT = ENGAGE_ATTACKS.lock().unwrap().len() as i32; }
    println!("Number of Engage Attacks in pool: {}", ENGAGE_ATTACKS.lock().unwrap().len());
    println!("Number of Engage Skills in pool: {}", ENGAGE_SKILLS.lock().unwrap().len());
    for _x in 0..22 {  ENGAGE_ATK_SWAP.lock().unwrap().push(EngageAttackIndex::new(0, 0));  }
}

pub fn reset_emblem_skills() {
    println!("Resetting skills to normal");
    let engage_skill_count = ENGAGE_SKILLS.lock().unwrap().len();
    let chaos_skill_count = ENGAGE_SKILLS_CHAOS.lock().unwrap().len();
    let engage_attack_count = ENGAGE_ATTACKS.lock().unwrap().len();
    let swap_count =  ENGAGE_ATK_SWAP.lock().unwrap().len();
    for x in 0..engage_skill_count {  ENGAGE_SKILLS.lock().unwrap()[x as usize].in_use = false;   }
    let engage_skill_count_2 = unsafe { ENGAGE_SKILL_COUNT } as usize;
    if ENGAGE_SKILLS.lock().unwrap().len() > engage_skill_count_2 { ENGAGE_SKILLS.lock().unwrap().drain(engage_skill_count_2..); }

    for x in 0..chaos_skill_count {  ENGAGE_SKILLS_CHAOS.lock().unwrap()[x as usize].in_use = false;   }
    for x in 0..engage_attack_count { 
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].in_use = false;  
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].linked_use = false;
    }
    for x in 0..swap_count {
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].index_1 = 0;
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].index_2 = 0;
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].in_use = false;  
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].linked_use = false;  
    }
    // Reset Emblem Bond Data for Skills (Engaged/Engage/Sync)
    SYNCHO_RANDOM_LIST.lock().unwrap().reset();
    // Engage Attack Reset Weapon Restrictions
    change_weapon_restrict("SID_マルスエンゲージ技", 2);    //Marth
    change_weapon_restrict("SID_シグルドエンゲージ技", 6);  //Sigurd
    change_weapon_restrict("SID_ロイエンゲージ技", 2);  // Roy
    change_weapon_restrict("SID_ルキナエンゲージ技", 2);    //Lucina
    change_weapon_restrict("SID_リンエンゲージ技", 16);     //Lyn
    change_weapon_restrict("SID_アイクエンゲージ技", 10);   //Ike
    change_weapon_restrict("SID_エイリークエンゲージ技", 2);    //Eirika
    change_weapon_restrict("SID_ヘクトルエンゲージ技", 10);     //Hector
    change_weapon_restrict("SID_カミラエンゲージ技", 8);    //Camilla   
    change_weapon_restrict("SID_クロムエンゲージ技", 2);    //Chrom
    change_weapon_restrict("SID_リュールエンゲージ技", 2); //Alear Dragon Blast
    change_weapon_restrict("SID_リュールエンゲージ技共同",2); //Alear Bond Blast
    change_weapon_restrict("SID_重唱", 64); // Echo
    unsafe { EIRIKA_INDEX = 11; }
}

pub fn randomized_god_data(){
    let mode = GameVariableManager::get_number("G_Random_God_Mode");
    if mode == 0 { return; }
    println!("Randomizing God Data...");
    let rng = Random::instantiate().unwrap();
    let seed = 3*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_list = SkillData::get_list().unwrap();
    let seed2 = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
    let rng2 = Random::instantiate().unwrap();
    rng2.ctor(seed2);
    if GameVariableManager::get_number("G_ChaosMode") == 4 {
        let list = GameVariableManager::find_starts_with("G_AddS");
        if list.len() == 0 {
            println!("Adding Sync Skills from List");
            crate::enums::get_added_skills();
            let mut count = 0;
            for x in ADDED_SYNC.lock().unwrap().iter() {
                let old_count_sync = SYNCHO_RANDOM_LIST.lock().unwrap().sync_list.len();
                let skill = &skill_list[*x as usize];
                SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, true);
                let new_count_sync = SYNCHO_RANDOM_LIST.lock().unwrap().sync_list.len();
                if new_count_sync > old_count_sync { 
                    GameVariableManager::make_entry(&format!("G_AddS{}", count), *x);
                    count += 1;
                }
            }
        }
        else {
            println!("Adding Sync Skills from GameVariables");
            for x in 0..list.len() {
                let skill_index = GameVariableManager::get_number(&list[x].get_string().unwrap());
                if !crate::utils::is_valid_skill_index( skill_index ) { continue; }
                SYNCHO_RANDOM_LIST.lock().unwrap().add_by_index(skill_index, true, true);
            }
        }
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng2);
    if mode & 1 != 0 {
        for x in 0..20 { 
            let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
            let ggid = GodGrowthData::try_get_from_god_data(god);
            if ggid.is_none() { continue; }
            let god_grow = ggid.unwrap(); 
            for y in 0..god_grow.len() {
                let level = god_grow[y].get_inheritance_skills();
                if level.is_none() {continue; }
                let inherit_skills = level.unwrap();
                for z in 0..inherit_skills.len() {
                    let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid(inherit_skills[z], true);
                    if replacement_skill.parent.index > 0 {
                        inherit_skills[z] = replacement_skill.sid;
                    }                   
                }
                god_grow[y].on_complete(); 
            }
        }
        if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
            let god_list = GodData::get_list().unwrap();
            let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
            for x in 0..n_customs {
                let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1] as usize;
                if c_index >= god_list.len() { continue; }
                let ggid = GodGrowthData::try_get_from_god_data(&god_list[c_index]);
                if ggid.is_none() { continue; }
                let god_grow = ggid.unwrap(); 
                for y in 0..god_grow.len() {
                    let level = god_grow[y].get_inheritance_skills();
                    if level.is_none() {continue; }
                    let inherit_skills = level.unwrap();
                    for z in 0..inherit_skills.len() {
                        let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid(inherit_skills[z], true);
                        if replacement_skill.parent.index > 0 { inherit_skills[z] = replacement_skill.sid; }                   
                    }
                    god_grow[y].on_complete(); 
                }
            }
        }
    }
    if mode & 2 != 0 {
        rng.ctor(seed);
        println!("Randomizing Engage Attacks");
        let engage_atk_size = ENGAGE_ATTACKS.lock().unwrap().len();
        let mut linked_gid: [bool; 19] = [false; 19];
        let mut count = 0;
        Patch::in_text(0x01c77620).bytes(&[0xc0,0x03, 0x5f, 0xd6]).unwrap();
        for x in 0..20 { 
            let god = GodData::get_mut(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
            // Engage Attack
            let mut value;
            let mut rng_counter = 0;
            loop {
                value = rng.get_value(engage_atk_size as i32) as usize;
                rng_counter += 1;
                if ( x == 9 || x == 13 ) && value == 7 {   continue; }  // No Astra Storm for Byleth/Tiki
                if !ENGAGE_ATTACKS.lock().unwrap()[value].in_use { break; }
                if rng_counter >= 100 { break; }
            }
            let engage_sid = skill_list[ ENGAGE_ATTACKS.lock().unwrap()[value].index_1 as usize ].sid;
            god.set_engage_attack( engage_sid );
            ENGAGE_ATTACKS.lock().unwrap()[value].in_use = true;
            // Linked Engage Attack
            let mut linked_index; // = rng.get_value(engage_atk_size as i32) as usize;
            rng_counter = 0;
            loop {
                linked_index = rng.get_value(engage_atk_size as i32) as usize;
                rng_counter += 1;
                if ( x == 9 || x == 13 ) && value == 7 {   continue; }  // No Astra Storm for Byleth/Tiki
                if !ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use && linked_index != value { 
                    break;
                }
                if rng_counter >= 100 && linked_index != value { break; }
            }
            let linked_engage_sid = skill_list[ ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_1 as usize ].sid;
            god.set_engage_attack_link(linked_engage_sid);
            ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use = true;

            ENGAGE_ATK_SWAP.lock().unwrap()[count as usize].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
            ENGAGE_ATK_SWAP.lock().unwrap()[count as usize].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
            
            // Linked Emblem
            if x != 19 {    // Not Emblem Alear
                let mut linked_god_index = rng.get_value(19) as usize;
                if count == 18 { 
                    for zz in 0..19 { if !linked_gid[zz as usize] { linked_god_index = zz; }  }
                }
                else {
                    rng_counter = 0; 
                    while linked_gid[linked_god_index] || x == linked_god_index { 
                        linked_god_index = rng.get_value(19) as usize; 
                        rng_counter += 1;
                        if rng_counter >= 100 && x != linked_god_index { break; }
                    }
                }
                linked_gid[linked_god_index] = true;
                let gid_linked = EMBLEM_GIDS[linked_god_index];
                god.set_link_gid(gid_linked.into());
                if x == 12 { // If Edelgard then change it for Dimitri and Claude
                    let war_criminals = ["GID_ディミトリ", "GID_クロード"];
                    for bg in war_criminals {
                        let war_crimes = GodData::get_mut(bg).unwrap();
                        war_crimes.set_engage_attack( engage_sid );
                        war_crimes.set_link_gid( gid_linked.into() );
                        war_crimes.set_engage_attack_link( linked_engage_sid);
                        war_crimes.on_complete();
                    }
                    ENGAGE_ATK_SWAP.lock().unwrap()[20].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
                    ENGAGE_ATK_SWAP.lock().unwrap()[20].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
                    ENGAGE_ATK_SWAP.lock().unwrap()[21].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
                    ENGAGE_ATK_SWAP.lock().unwrap()[21].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
                }
                else if x == 11 {
                    let ephirm = GodData::get_mut("GID_エフラム").unwrap();
                    ephirm.set_engage_attack( engage_sid );
                    ephirm.set_link_gid( gid_linked.into() );
                    ephirm.set_engage_attack_link( linked_engage_sid);
                    ephirm.on_complete();
                }
            }
            count += 1;
            god.on_complete();
        }
        adjust_engage_weapon_type();
    }
    if GameVariableManager::get_bool("G_Random_Engage_Weps") {
        println!("Randomizing Engage Weapons");
        let seed2 = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
        let rng2 = Random::instantiate().unwrap();
        rng2.ctor(seed2);
        ENGAGE_ITEMS.lock().unwrap().randomize_list(rng2);
        ENGAGE_ITEMS.lock().unwrap().commit();
        adjust_growth_data_weapons();
        adjust_engage_weapon_type();
    }
    randomize_engage_skills(rng);
    randomize_emblem_stat_bonuses(rng);
    randomized_emblem_syncho_skills(rng);
}

fn adjust_growth_data_weapons() {
    for x in 0..20 {
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let ggid = GodGrowthData::try_get_from_god_data(god);
        if ggid.is_none() { continue; }
        let god_grow = ggid.unwrap(); 
        for y in 0..god_grow.len() {
            if god_grow[y].engage_items.is_some() {
                let item = god_grow[y].engage_items.as_mut().unwrap();
                for z in 0..item.len() { 
                    item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);
                }
            }
            if god_grow[y].engage_cooperations.is_some() {
                let item = god_grow[y].engage_cooperations.as_mut().unwrap();
                for z in 0..item.len() {  item[z] = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]); }
            }
            if god_grow[y].engage_horses.is_some() {
                let item = god_grow[y].engage_horses.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
            if god_grow[y].engage_coverts.is_some() {
                let item = god_grow[y].engage_coverts.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
            if god_grow[y].engage_heavys.is_some() {
                let item = god_grow[y].engage_heavys.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
            if god_grow[y].engage_flys.is_some() {
                let item = god_grow[y].engage_flys.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]); }
            }
            if god_grow[y].engage_magics.is_some() {
                let item = god_grow[y].engage_magics.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
            if god_grow[y].engage_pranas.is_some() {
                let item = god_grow[y].engage_pranas.as_mut().unwrap();
                for z in 0..item.len() { item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
            if god_grow[y].engage_dragons.is_some() {
                let item = god_grow[y].engage_dragons.as_mut().unwrap();
                for z in 0..item.len() {  item[z] =  ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item[z]);}
            }
        }
    }
}

fn randomize_engage_skills(rng: &Random){
    if GameVariableManager::get_number("G_Random_God_Sync") <= 1 { return; }
    println!("Random Engage Skills");
    let skill_list = SkillData::get_list().unwrap();
    if GameVariableManager::get_number("G_ChaosMode") == 4 {
        let list = GameVariableManager::find_starts_with("G_AddE");
        if list.len() == 0 {
            crate::enums::get_added_skills();
            let mut count = 0;
            for x in ADDED_ENGAGE.lock().unwrap().iter() {
                if ENGAGE_SKILLS.lock().unwrap().iter().find(|y| y.index == *x).is_some() { continue; }  // must not be already contain the list
                if !crate::utils::is_valid_skill_index( *x ) { continue; }
                GameVariableManager::make_entry(&format!("G_AddE{}", count), *x);
                count += 1;
            }
        }
        else {
            for x in 0..list.len() {
                let skill_index = GameVariableManager::get_number(&list[x].get_string().unwrap());
                if ENGAGE_SKILLS.lock().unwrap().iter().find(|y| y.index == skill_index).is_some() { continue; }
                if !crate::utils::is_valid_skill_index( skill_index ) { continue; }
                ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(skill_index ));
            }
        }
    }
    let mut count: usize = 0;
    let mut engage_sid: [i32; 40] = [-1; 40];
    let skill_pool = if GameVariableManager::get_number("G_ChaosMode") == 2 || GameVariableManager::get_number("G_ChaosMode") == 3 { &ENGAGE_SKILLS_CHAOS } else { &ENGAGE_SKILLS };
    let  engage_skill_size = skill_pool.lock().unwrap().len() as i32;

    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let keys = GodGrowthData::get_level_data(&format!("GGID_{}", x));
        if keys.is_some() {
            let level_data = keys.unwrap();
            let mut index = rng.get_value( engage_skill_size ) as usize;
            while skill_pool.lock().unwrap()[index].in_use   { index = rng.get_value(engage_skill_size) as usize; }
            let engage_skill = &skill_list[ skill_pool.lock().unwrap()[index].index as usize ];
            if engage_skill.sid.get_string().unwrap() == "SID_双聖" { unsafe { EIRIKA_INDEX = count; }  }
            engage_sid[count] = engage_skill.parent.index;
            for y in 0..level_data.len() {
                level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
            }
            skill_pool.lock().unwrap()[index].in_use = true;
        }
        count += 1;
    }
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let god_list = GodData::get_list().unwrap();
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1] as usize;
            if c_index >= god_list.len() { continue; }
            let growth_data = god_list[c_index].get_grow_table();
            if growth_data.is_none() { continue; }
            let lvl_data = GodGrowthData::get_level_data(&growth_data.unwrap().get_string().unwrap());
            if lvl_data.is_none() { continue; }
            let level_data = lvl_data.unwrap();
            let mut index = rng.get_value( engage_skill_size ) as usize;
            while skill_pool.lock().unwrap()[index].in_use   { index = rng.get_value(engage_skill_size) as usize; }
            let engage_skill = &skill_list[ skill_pool.lock().unwrap()[index].index as usize ];
            if engage_skill.sid.get_string().unwrap() == "SID_双聖" { unsafe { EIRIKA_INDEX = count; }  }
            for y in 0..level_data.len() {
                level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
            }
            skill_pool.lock().unwrap()[index].in_use = true;
            let ggid = GodGrowthData::try_get_from_god_data(god_list[c_index]);
            if ggid.is_none() { continue; }
            let god_grow = ggid.unwrap(); 
            for y in 0..god_grow.len() {
                if god_grow[y].engage_skills.is_none() {continue; }
                let engage_skills = god_grow[y].engage_skills.as_mut().unwrap();
                engage_skills[0] = skill_list[ engage_skill.parent.index as  usize].sid;
                god_grow[y].on_complete(); 
            }
        }
    }

    // For Ring Reference 
    for x in 0..20 {
        let ggid = GodGrowthData::try_get_from_god_data(GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap());
        if ggid.is_none() { continue; }
        let god_grow = ggid.unwrap(); 
        for y in 0..god_grow.len() {
            if god_grow[y].engage_skills.is_none() {continue; }
            let engage_skills = god_grow[y].engage_skills.as_mut().unwrap();
            engage_skills[0] = skill_list[ engage_sid[x] as usize].sid;
            god_grow[y].on_complete(); 
        }
    }

    let god_list = GodData::get_list().unwrap();
    for x in 0..god_list.len() {
        if !str_contains(god_list[x].gid, "GID_M0") && !str_contains(god_list[x].gid, "GID_E00") { continue; }
        let ggd = god_list[x].get_grow_table();
        if ggd.is_none() { continue; }
        let ld = GodGrowthData::get_level_data(&ggd.unwrap().get_string().unwrap());
        if ld.is_none() { continue;}
        let emblem_position = EMBLEM_ASSET.iter().position(|&a| crate::utils::str_contains( god_list[x].gid, a));
        if emblem_position.is_none() { continue; }
        if emblem_position.unwrap() >= 20 { continue; }
        let engage_skill = &skill_list[ engage_sid[emblem_position.unwrap()] as usize] ;
        let level_data = ld.unwrap();
        for y in 0..level_data.len() {
            level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
        }
    }
}
fn randomize_emblem_stat_bonuses(rng: &Random){
    if GameVariableManager::get_number("G_Random_God_Sync") == 0 || GameVariableManager::get_number("G_Random_God_Sync") == 2 { return; }
    // Skill Range of Invisible Stat+ Skills
    println!("Random Stat Bonuses");
    let min_index = STAT_BONUS.lock().unwrap()[0]; //Lowest HP Index
    let max_index = STAT_BONUS.lock().unwrap()[65]; //Highest Move Index
    let skill_list = SkillData::get_list().unwrap();
    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        let stats = get_stats_for_emblem(rng);
        for y in 0..level_data.len() {
            let mut stat_index: usize = 0;
            for z in 0..level_data[y].synchro_skills.list.size {
                let skill_index = level_data[y].synchro_skills[z as usize].get_skill().unwrap().parent.index;
                if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                    let stat_skill = &skill_list[ skill_index as usize ];
                    let sb_index: usize = if stat_skill.get_priority() == 0 {  ( stats[ stat_index ] as usize ) * 6 } //Replace Move+1 stat boost
                                          else { ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize };
                    let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                    level_data[y as usize ].synchro_skills.replace(z as i32, new_skill, 5);
                    stat_index += 1;
                }
            }
            stat_index = 0;
            for z in 0..level_data[y].engaged_skills.list.size {
                let skill_index = level_data[y].engaged_skills[z as usize].get_skill().unwrap().parent.index;
                if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                    let stat_skill = &skill_list[ skill_index as usize ];
                    let sb_index = if stat_skill.get_priority() == 0 { ( stats[ stat_index ] as usize ) * 6  }
                                   else { ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize  };
                    let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                    level_data[y as usize ].engaged_skills.replace(z as i32, new_skill, 5);
                    stat_index += 1;
                }
            }
        }
    }
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 {
        let god_list = GodData::get_list().unwrap();
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1] as usize;
            if c_index >= god_list.len() { continue; }
            let growth_data = god_list[c_index].get_grow_table();
            if growth_data.is_none() { continue; }
            let lvl_data = GodGrowthData::get_level_data(&growth_data.unwrap().get_string().unwrap());
            if lvl_data.is_none() { continue; }
            let level_data = lvl_data.unwrap();
            let stats = get_stats_for_emblem(rng);
            for y in 0..level_data.len() {
                let mut stat_index: usize = 0;
                for z in 0..level_data[y].synchro_skills.list.size {
                    let skill_index = level_data[y].synchro_skills[z as usize].get_skill().unwrap().parent.index;
                    if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                        let stat_skill = &skill_list[ skill_index as usize ];
                        let sb_index: usize = if stat_skill.get_priority() == 0 {  ( stats[ stat_index ] as usize ) * 6 } //Replace Move+1 stat boost
                                              else { ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize };
                        let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                        level_data[y as usize ].synchro_skills.replace(z as i32, new_skill, 5);
                        stat_index += 1;
                    }
                }
                stat_index = 0;
                for z in 0..level_data[y].engaged_skills.list.size {
                    let skill_index = level_data[y].engaged_skills[z as usize].get_skill().unwrap().parent.index;
                    if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                        let stat_skill = &skill_list[ skill_index as usize ];
                        let sb_index = if stat_skill.get_priority() == 0 { ( stats[ stat_index ] as usize ) * 6  }
                                       else { ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize  };
                        let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                        level_data[y as usize ].engaged_skills.replace(z as i32, new_skill, 5);
                        stat_index += 1;
                    }
                }
            }
        }
    }
}
fn randomized_common_sids(mpid: String) {
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 300..1200 {
        let person_x = &person_list[x as usize];
        if person_x.get_name().is_none() { continue; }
        let name2 = person_x.get_name().unwrap();
        if !str_contains(name2, &mpid) { continue; }
        if person_x.get_common_sids().is_none() { continue; }
        let personal_sid = person_x.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid(personal_sid[y as usize], true);
            if replacement_skill.parent.index > 0 { personal_sid[y] = replacement_skill.sid; } 
        }
        person_x.on_complete();
    }
}
fn randomized_emblem_syncho_skills(rng: &Random) {
    if GameVariableManager::get_number("G_Random_God_Sync") <= 1 { return; }
    println!("Randomizing Syncho Skills");
    SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng);
    // For the SkillArray
    let mut count = 0;
    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        for y in 0..level_data.len() {
            for z in 0..level_data[y].synchro_skills.list.size {
                let skill = level_data[y].synchro_skills[z as usize].get_skill().unwrap();
                let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                level_data[y as usize ].synchro_skills.replace(z as i32, replacement_skill, 5);
            }
            for z in 0..level_data[y].engaged_skills.list.size {
                let skill = level_data[y].engaged_skills[z as usize].get_skill().unwrap();
                let replacement_skill =  SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                level_data[y as usize ].engaged_skills.replace(z as i32, replacement_skill, 5);
            }
        }
        println!("Emblem {} Sync is done", count);
        count +=1;
    }
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let god_list = GodData::get_list().unwrap();
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1] as usize;
            if c_index >= god_list.len() { continue; }
            let growth_data = god_list[c_index].get_grow_table();
            if growth_data.is_none() { continue; }
            let lvl_data = GodGrowthData::get_level_data(&growth_data.unwrap().get_string().unwrap());
            if lvl_data.is_none() { continue; }
            let level_data = lvl_data.unwrap();
            for y in 0..level_data.len() {
                for z in 0..level_data[y].synchro_skills.list.size {
                    let skill = level_data[y].synchro_skills[z as usize].get_skill().unwrap();
                    let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                    level_data[y as usize ].synchro_skills.replace(z as i32, replacement_skill, 5);
                }
                for z in 0..level_data[y].engaged_skills.list.size {
                    let skill = level_data[y].engaged_skills[z as usize].get_skill().unwrap();
                    let replacement_skill =  SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                    level_data[y as usize ].engaged_skills.replace(z as i32, replacement_skill, 5);
                }
            }
            let ggid = GodGrowthData::try_get_from_god_data(god_list[c_index]);
            if ggid.is_none() { continue; }
            let god_grow = ggid.unwrap();
            for y in 0..god_grow.len() {
                if god_grow[y].synchro_skills.is_none() {continue; }
                let syncho_skills = god_grow[y].synchro_skills.as_mut().unwrap();
                for z in 0..syncho_skills.len() {
                    let skill = SkillData::get(&syncho_skills[z].get_string().unwrap());
                    if skill.is_none() { continue; }
                    let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill.unwrap(), false);
                    if str_contains(replacement_skill.sid, "SID_計略") { syncho_skills[z] = "".into(); }
                    else { syncho_skills[z] = replacement_skill.sid; }
    
                }
                god_grow[y].on_complete(); 
            }
        }
    }
    // Adding unused/added skills
    let extra_skills = SYNCHO_RANDOM_LIST.lock().unwrap().get_non_randomized_skill();
    let mut used_skill: [bool; 500] = [false; 500];
    let pool_size = extra_skills.len() as i32;
    let rng = crate::utils::get_rng();
    println!("Extra Sync Pool Size: {}", pool_size);
    if pool_size > 10 {
        for x in EMBLEM_ASSET {
            if x == "ディミトリ" { break; }
            let growth_id = format!("GGID_{}", x);
            let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
            let mut non_hidden_skills = 0;
            for z in 0..level_data[0].synchro_skills.list.size {    // Get number of non-hidden skills
                if level_data[0].synchro_skills[z as usize].get_skill().unwrap().get_flag() & 1 == 0 { non_hidden_skills += 1;  }
            }
            let mut non_engage_skill = 0;
            for z in 0..level_data[0].engaged_skills.list.size {    // Get number of non-hidden skills
                if level_data[0].engaged_skills[z as usize].get_skill().unwrap().get_flag() & 1 == 0 { non_engage_skill += 1;  }
            }
            if non_hidden_skills < 4 {
                for _y in 0..4-non_hidden_skills {
                    let mut index;
                    loop {
                        index = rng.get_value( pool_size );
                        if index >= 400 { continue; }
                        if !used_skill[index as usize] { break; }
                    }
                    used_skill[index as usize] = true;
                    let skill_index = extra_skills[index as usize].index; 
                    let mut skill = SkillData::try_index_get(skill_index).unwrap();
                    let max_priority = extra_skills[index as usize].max_priority;
                    for y2 in 0..11 {  
                        if non_engage_skill < 4 { level_data[y2 as usize ].engaged_skills.add_skill(skill, 5, 0); }
                        level_data[y2 as usize ].synchro_skills.add_skill(skill, 5, 0); 
                    }
                    if max_priority >= 2 {
                        skill = if extra_skills[index as usize].eirika_twin_skill { SkillData::try_index_get(skill_index + 3 ) }
                        else { SkillData::try_index_get(skill_index + 1) }.unwrap();
                    }
                    for y2 in 11..17 { 
                        if non_engage_skill < 4 { level_data[y2 as usize ].engaged_skills.add_skill(skill, 5, 0); }
                        level_data[y2 as usize ].synchro_skills.add_skill(skill, 5, 0);  
                    }
                    if max_priority >= 3 { skill = SkillData::try_index_get(skill_index + 2).unwrap();  }
                    for y2 in 17..level_data.len() {  
                        if non_engage_skill < 4 { level_data[y2 as usize ].engaged_skills.add_skill(skill, 5, 0); }
                        level_data[y2 as usize ].synchro_skills.add_skill(skill, 5, 0);
                    }
                    non_engage_skill += 1;
                }
            }
        }
    }
    // Change for ring reference
    for x in 0..20 {
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let ggid = GodGrowthData::try_get_from_god_data(god);
        if ggid.is_none() { continue; }
        let god_grow = ggid.unwrap();
        for y in 0..god_grow.len() {
            if god_grow[y].synchro_skills.is_none() {continue; }
            let syncho_skills = god_grow[y].synchro_skills.as_mut().unwrap();
            for z in 0..syncho_skills.len() {
                let skill = SkillData::get(&syncho_skills[z].get_string().unwrap());
                if skill.is_none() { continue; }
                let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill.unwrap(), false);
                if str_contains(replacement_skill.sid, "SID_計略") { syncho_skills[z] = "".into(); }
                else { syncho_skills[z] = replacement_skill.sid; }

            }
            god_grow[y].on_complete(); 
        }
        if x < 19 { randomized_common_sids( RINGS[x as usize].to_string() );  }
    }
    randomized_common_sids("MPID_Reflet".to_string());
    // enemy and others
    let god_list = GodData::get_list().unwrap();
    for x in 0..god_list.len() {
        if !str_contains(god_list[x].gid, "GID_M0") && !str_contains(god_list[x].gid, "GID_E00") { continue; }
        let ggd = god_list[x].get_grow_table();
        if ggd.is_none() { continue; }
        let ld = GodGrowthData::get_level_data(&ggd.unwrap().get_string().unwrap());
        if ld.is_none() { continue;}
        let level_data = ld.unwrap();
        for y in 0..level_data.len() {
            for z in 0..level_data[y].synchro_skills.list.size {
                let skill = level_data[y].synchro_skills[z as usize].get_skill().unwrap();
                let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                level_data[y as usize ].synchro_skills.replace(z as i32, replacement_skill, 5);
            }
            for z in 0..level_data[y].engaged_skills.list.size {
                let skill = level_data[y].engaged_skills[z as usize].get_skill().unwrap();
                let replacement_skill =  SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                level_data[y as usize ].engaged_skills.replace(z as i32, replacement_skill, 5);
            }
        }
    }
}

fn adjust_engage_weapon_type() {
    for x in 0..20 {
        let weapon_mask_1 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[x as usize];
        let gid1 = format!("GID_{}", EMBLEM_ASSET[ x as usize]);
        let engage_attack_sid = GodData::get(&gid1).unwrap().get_engage_attack().get_string().unwrap();
        let mut weapon_mask_2 = EMBLEM_WEAPON[ x as usize];
        for y in 0..20 {
            let gid2 = format!("GID_{}", EMBLEM_ASSET[ y as usize]);
            if GodData::get(&gid2).unwrap().get_engage_attack_link().is_none() { continue; }
            let linked_engage_attack_sid = GodData::get(&gid2).unwrap().get_engage_attack_link().unwrap().get_string().unwrap();
            if engage_attack_sid == linked_engage_attack_sid {
                weapon_mask_2 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[y as usize];
                break;
            }
        }
        let mut combine_weapon_mask = weapon_mask_1 | weapon_mask_2 ;
        if engage_attack_sid == "SID_リンエンゲージ技" { combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 16; }
        if engage_attack_sid == "SID_マルスエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 2; } 
        if engage_attack_sid == "SID_シグルドエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 6; }
        if engage_attack_sid == "SID_ロイエンゲージ技" { combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 2; }
        if engage_attack_sid == "SID_ルキナエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 2; }
        if engage_attack_sid == "SID_アイクエンゲージ技" { combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 10; }
        if engage_attack_sid == "SID_エイリークエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 2; } 
        if engage_attack_sid == "SID_ヘクトルエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 10; }
        if engage_attack_sid == "SID_カミラエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 8; } 
        if engage_attack_sid == "SID_クロムエンゲージ技"{ combine_weapon_mask = (weapon_mask_1 | weapon_mask_2 ) | 2; }
        change_weapon_restrict(&engage_attack_sid, combine_weapon_mask);
    }
    change_weapon_restrict("SID_重唱", 1023);
}
fn change_weapon_restrict(sid :&str, value: i32) {
    let engage_skill = SkillData::get_mut(sid).unwrap();
    let w1 = engage_skill.get_weapon_prohibit();
    if w1.value <= 2 { return; }
    w1.value = 1023 - value;
    let style = ["_気功", "_隠密", "_連携", "_通常", "_通常", "_重装", "_飛行", "_魔法", "_通常", "_竜族", "＋", "＋_連携", "＋_通常", "＋_通常", "＋_重装", "＋_飛行", "＋_魔法", "＋_通常", "＋_竜族", "＋_気功", "＋_隠密"];
    for x in style {
        let style_sid = format!("{}{}", sid, x);
        if SkillData::get(&style_sid).is_some() {
            let w2 = SkillData::get_mut(&style_sid).unwrap().get_weapon_prohibit(); 
            w2.value = 1023 - value;
        }
    }
}

