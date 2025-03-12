use super::*;
use super::{emblem_item::*, emblem_structs::*};
use super::super::skill::SkillIndex;
use std::sync::{OnceLock, Mutex};
use concat_string::concat_string;

use engage::menu::BasicMenuItemAttribute;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    gamedata::GodData,
};
use skyline::patching::Patch;
use crate::CONFIG;
const EMBLEM_WEAPON: [i32; 20] = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258];

pub static EMBLEM_PERSON: OnceLock<Vec<(i32, i32)>> = OnceLock::new();
pub static mut ENGAGE_SKILL_COUNT: i32 = 19;
pub static ENGAGE_SKILLS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static ADDED_ENGAGE:  Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static ADDED_SYNC: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static ENGAGE_SKILLS_CHAOS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static mut EIRIKA_INDEX: usize = 11;
static ENGAGE_ATTACKS: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());

pub static STAT_BONUS: OnceLock<[i32; 66]> = OnceLock::new();
pub static SYNCHO_RANDOM_LIST: Mutex<SynchoList> = Mutex::new(
    SynchoList { 
        sync_list: Vec::new(), inherit_list: Vec::new(), chaos_list: Vec::new(), randomized: false, sync_list_size: 0, non_random_skills: Vec::new(), inherit_cost: Vec::new(),
        chaos_inherit_list: Vec::new(), sync_rando: Vec::new()
    }
);


pub struct EmblemSkillChaos;
impl ConfigBasicMenuItemSwitchMethods for EmblemSkillChaos {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if GameUserData::get_sequence() != 0 { GameVariableManager::make_entry("ESkC", GameVariableManager::get_number("ESkC"));  }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_skill_chaos }
            else { GameVariableManager::get_number("ESkC")};
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_skill_chaos = result; }
            else { GameVariableManager::set_number("ESkC", result)};
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let main = DVCVariables::is_main_menu();
        let value = if main { CONFIG.lock().unwrap().emblem_skill_chaos } else { GameVariableManager::get_number("ESkC")};
        let changed = if main { "" }
            else if GameVariableManager::get_number("ESkC") != GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) { " (A to Confirm)"}
            else { "" };

        this.help_text = format!("{}{}", match value {
            1 => { "Expands pool for sync skills." },
            2 => { "Expands pool for engage skills." },
            3 => { "Expands pool for engage and sync skills." },
            _ => { "Default pool for sync and engage skills."},
        }, changed).into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().emblem_skill_chaos }
            else { GameVariableManager::get_number("ESkC")};
        let changed = DVCVariables::changed_setting_text("ESkC", DVCVariables::EMBLEM_SKILL_CHAOS_KEY);

        this.command_text = format!("{}{}",changed, match value {
            1 => { "Sync" },
            2 => { "Engage" },
            3 => { "Sync / Engage "},
            _ => { "Default"},
        }).into();
    }
}

pub struct EmblemSkillChaosConfirm;
impl TwoChoiceDialogMethods for EmblemSkillChaosConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY, GameVariableManager::get_number("ESkC"));
        unsafe { 
            let menu = std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap());
            let index = menu.select_index;
            EmblemSkillChaos::set_help_text(menu.menu_item_list[index as usize], None);
            EmblemSkillChaos::set_command_text(menu.menu_item_list[index as usize], None);
            menu.menu_item_list[index as usize].update_text();
        }
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn esc_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_number("ESkC") == GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) { return BasicMenuResult::new();}
    YesNoDialog::bind::<EmblemSkillChaosConfirm>(this.menu, "Change Randomization Setting?\nMust save and reload to take effect.", "Do it!", "Nah..");
    return BasicMenuResult::new();
}
pub fn esc_build_attr(_this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if can_rand() && GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) > 1 { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
}

pub extern "C" fn vibe_rand_esc() -> &'static mut ConfigBasicMenuItem {
    let switch = ConfigBasicMenuItem::new_switch::<EmblemSkillChaos>("Emblem Skill Chaos Mode");
    switch.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = esc_acall as _ );
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = esc_build_attr as _ );
    switch
}


pub fn create_emblem_skill_pool() {
    // get skill index of hidden stat boost for emblems stat sync bonuses.
    STAT_BONUS.get_or_init(||{
        let mut values: [i32; 66] = [0; 66];
        for x in 0..11 {
            if x == 9 { continue; } // No Sight
            for y in 1..7 { values[ 6*x + y-1 ] = find_emblem_stat_bonus_index(x as i32, y as i32); }
        }
        values
    });
    // Get all syncho skills to the random list  //Add Gambit    
    for x in EMBLEM_ASSET {
        if x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        if let Some(level_data) = GodGrowthData::get_level_data(&growth_id) {
            let engage_skill = level_data[0].engage_skills[0].get_skill().unwrap();
            ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(engage_skill.parent.index));
            level_data.iter().flat_map(|l| l.synchro_skills.iter()).for_each(|k|{
                if let Some(skill )= k.get_skill(){
                    SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, true);
                }
            });
        }
    }
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 {
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(level_data) = get_god_level_data_by_index(c_index) {
                let engage_skill = level_data[0].engage_skills[0].get_skill().unwrap();
                ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(engage_skill.parent.index));
                level_data.iter().flat_map(|l| l.synchro_skills.iter()).for_each(|k|{
                    if let Some(skill )= k.get_skill(){
                        if skill.help.is_some_and(|h| Mess::get(h).to_string().len() > 5 ){
                            SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill, true);
                        }
                    }
                });
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
        let god = GodData::get(EMBLEM_GIDS[x]).unwrap();
        let engage_index = SkillData::get_index( god.get_engage_attack() );
        if engage_index != -1 {
            ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(engage_index as i32, count));
        }
        if let Some(atk) =  god.get_engage_attack_link() {
            let link_index = SkillData::get_index( atk );
            if link_index != -1 { ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(link_index as i32, count)); }
        }
        count += 1;
    }
    ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(SkillData::get("SID_ベレトエンゲージ技_闇").unwrap().parent.index, count));
    count +=1;
    ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(SkillData::get("SID_セリカエンゲージ技_闇_M020").unwrap().parent.index, count));
    unsafe { ENGAGE_SKILL_COUNT = ENGAGE_ATTACKS.lock().unwrap().len() as i32; }
    println!("Number of Engage Attacks in pool: {}", ENGAGE_ATTACKS.lock().unwrap().len());
    println!("Number of Engage Skills in pool: {}", ENGAGE_SKILLS.lock().unwrap().len());
}

pub fn reset_emblem_skills() {
    println!("Resetting skills to normal");
    let engage_skill_count = ENGAGE_SKILLS.lock().unwrap().len();
    let chaos_skill_count = ENGAGE_SKILLS_CHAOS.lock().unwrap().len();
    let engage_attack_count = ENGAGE_ATTACKS.lock().unwrap().len();
    for x in 0..engage_skill_count {  ENGAGE_SKILLS.lock().unwrap()[x as usize].in_use = false;   }
    let engage_skill_count_2 = unsafe { ENGAGE_SKILL_COUNT } as usize;
    if ENGAGE_SKILLS.lock().unwrap().len() > engage_skill_count_2 { ENGAGE_SKILLS.lock().unwrap().drain(engage_skill_count_2..); }

    for x in 0..chaos_skill_count {  ENGAGE_SKILLS_CHAOS.lock().unwrap()[x as usize].in_use = false;   }
    for x in 0..engage_attack_count { 
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].in_use = false;  
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].linked_use = false;
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().reset();
    unsafe { EIRIKA_INDEX = 11; }
}

pub fn randomized_inherit(grow_list: &mut Vec<&mut List<GodGrowthData>>) {
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_KEY) & 1 == 0 { return; }
    grow_list.iter_mut().flat_map(|ggid| ggid.iter_mut())
        .for_each(|level|{
            level.get_inheritance_skills()
                .iter_mut()
                .flat_map(|x|x.iter_mut())
                .for_each(|inherit|{
                    let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid(inherit, true);
                    if replacement_skill.parent.index > 0 { *inherit = replacement_skill.sid; }  
                }
            );
        }
    );
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(level_data) = get_god_grow_data_by_index(c_index) {
                level_data.iter_mut().for_each(|level|{
                    level.get_inheritance_skills()
                        .iter_mut()
                        .flat_map(|x|x.iter_mut())
                        .for_each(|inherit|{
                            let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid(inherit, true);
                            if replacement_skill.parent.index > 0 { *inherit = replacement_skill.sid; }  
                        }
                    );
                });
            }
        }
    }
}

pub fn randomized_god_data(){
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_data_randomized { return; }
    let mode = GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_KEY);
    println!("Randomizing God Data...");
    let rng = Random::instantiate().unwrap();
    let seed = DVCVariables::get_seed();
    rng.ctor(3*seed as u32 );
    let skill_list = SkillData::get_list().unwrap();
    let rng2 = Random::instantiate().unwrap();
    rng2.ctor( 2*seed as u32);
    let god_list: Vec<_> = EMBLEM_GIDS.iter().flat_map(|&gid| GodData::get_mut(gid)).collect();
    let mut level_data: Vec<_> = GGIDS.iter().flat_map(|&ggid| GodGrowthData::get_level_data(ggid)).collect();
    let mut grow_data: Vec<_> = EMBLEM_GIDS.iter().flat_map(|&gid| GodGrowthData::try_get_from_god_data(GodData::get(gid).unwrap()) ).collect();
    if mode != 0 {
        SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng2);
        if mode & 1 != 0 { randomized_inherit(&mut grow_data); }
        if mode & 2 != 0  {
            rng.initialize(3*seed as u32);
            println!("Randomizing Engage Attacks");
            let engage_atk_size = ENGAGE_ATTACKS.lock().unwrap().len();
            let mut linked_gid: [bool; 19] = [false; 19];
            let mut count = 0;
            Patch::in_text(0x01c77620).bytes(&[0xc0,0x03, 0x5f, 0xd6]).unwrap();
            for x in 0..20 { 
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
                god_list[x].set_engage_attack( engage_sid );
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
                god_list[x].set_engage_attack_link(linked_engage_sid);

                ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use = true;
            // Linked Emblem
                if x != 19 {    // Not Emblem Alear
                    let mut linked_god_index = rng.get_value(19) as usize;
                    if count == 18 {  for zz in 0..19 { if !linked_gid[zz as usize] { linked_god_index = zz; }  } }
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
                    god_list[x].set_link_gid(gid_linked.into());
                    GodData::try_index_get_mut( super::ENEMY_EMBLEM_LIST.get().unwrap()[x]).map(|god|{
                        god.set_engage_attack_link(linked_engage_sid);
                        god.set_engage_attack(engage_sid);
                        god.set_link_gid(format!("GID_相手{}", EMBLEM_ASSET[linked_god_index]).into());
                    });
                    if x == 12 { // If Edelgard then change it for Dimitri and Claude
                       ["ディミトリ", "クロード"].iter().for_each(|war|{
                            let god = GodData::get_mut(&format!("GID_{}", war)).unwrap();
                            god.set_engage_attack( engage_sid );
                            god.set_link_gid( gid_linked.into() );
                            god.set_engage_attack_link( linked_engage_sid);
                            GodData::get(format!("GID_相手{}", war)).map(|god|{
                                god.set_engage_attack_link(linked_engage_sid);
                                god.set_engage_attack(engage_sid);
                                god.set_link_gid(format!("GID_相手{}", EMBLEM_ASSET[linked_god_index]).into());
                            });
                        });
                    }
                    else if x == 11 {
                        let ephirm = GodData::get_mut("GID_エフラム").unwrap();
                        ephirm.set_engage_attack( engage_sid );
                        ephirm.set_link_gid( gid_linked.into() );
                        ephirm.set_engage_attack_link( linked_engage_sid);
                        GodData::get("GID_相手エフラム").map(|god|{
                            god.set_engage_attack_link(linked_engage_sid);
                            god.set_engage_attack(engage_sid);
                            god.set_link_gid(format!("GID_相手{}", EMBLEM_ASSET[linked_god_index]).into());
                        });
                    }
                } 
                count += 1;
            }
            adjust_engage_weapon_type();
        }
    }
    if GameVariableManager::get_bool(DVCVariables::EMBLEM_ITEM_KEY) {
        println!("Randomizing Engage Weapons");
        rng2.initialize(2*DVCVariables::get_seed() as u32);
        ENGAGE_ITEMS.lock().unwrap().randomize_list(rng2);
        ENGAGE_ITEMS.lock().unwrap().commit();
        adjust_growth_data_weapons(&mut grow_data);
        adjust_engage_weapon_type();
    }
    randomize_engage_skills(rng, &mut grow_data, &mut level_data);
    randomize_emblem_stat_bonuses(rng, &mut level_data);
    randomized_emblem_syncho_skills(rng, &mut grow_data, &mut level_data);
    if GameVariableManager::get_number(DVCVariables::SP_KEY) > 0 {
        let rng = get_rng();
        SYNCHO_RANDOM_LIST.lock().unwrap().randomized_skill_cost(rng);
    }
    update_god_grow(&mut grow_data);
    let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.emblem_data_randomized = true);
}

fn adjust_growth_data_weapons(level_data: &mut Vec<&mut List<GodGrowthData>>) {
    level_data.iter_mut().flat_map(|ggld| ggld.iter_mut())
        .for_each(|level|{
            level.engage_items.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_cooperations.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_coverts.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_horses.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_magics.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_flys.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_pranas.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_dragons.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
            level.engage_heavys.iter_mut().flat_map(|x| x.iter_mut()).for_each(|item| *item = ENGAGE_ITEMS.lock().unwrap().get_replacement_iid(item) );
        }
    );
}

fn randomize_engage_skills(rng: &Random, grow_data: &mut Vec<&mut List<GodGrowthData>>, level_data: &mut Vec<&mut List<GodGrowthDataLevelData>> ){
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) & 2 == 0 { return; }
    println!("Random Engage Skills");
    let skill_list = SkillData::get_list().unwrap();
    let mut count: usize = 0;
    let mut engage_sid: [i32; 40] = [-1; 40];
    let skill_pool = if GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY) & 2 != 0 { &ENGAGE_SKILLS_CHAOS } else { &ENGAGE_SKILLS };
    let  engage_skill_size = skill_pool.lock().unwrap().len() as i32;
    level_data.iter_mut().for_each(|ggld|{
        let mut index = rng.get_value( engage_skill_size ) as usize;
        while skill_pool.lock().unwrap()[index].in_use   { index = rng.get_value(engage_skill_size) as usize; }
        let skill_index = skill_pool.lock().unwrap()[index].index;
        if let Some(engage_skill) = SkillData::try_index_get(skill_index)  {
            if engage_skill.sid.to_string() == "SID_双聖" { unsafe { EIRIKA_INDEX = count; }  }
            engage_sid[count] = engage_skill.parent.index;
            ggld.iter().for_each(|level| level.engage_skills.replace(0, engage_skill, 5));
            skill_pool.lock().unwrap()[index].in_use = true;
        }
        count += 1;
    });

    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(level_data) = get_god_level_data_by_index(c_index){
                let mut index = rng.get_value( engage_skill_size ) as usize;
                while skill_pool.lock().unwrap()[index].in_use { index = rng.get_value(engage_skill_size) as usize; }
                let engage_skill =  SkillData::try_index_get( skill_pool.lock().unwrap()[index].index ).unwrap();
                level_data.iter().for_each(|level| level.engage_skills.replace(0, engage_skill, 5));
                skill_pool.lock().unwrap()[index].in_use = true;
                if let Some(god_grow) = get_god_grow_data_by_index(c_index) {
                    god_grow.iter_mut().for_each(|level|{
                        if let Some(engage_skills) = level.engage_skills.as_mut() {
                            engage_skills[0] = skill_list[ engage_skill.parent.index as  usize].sid;
                        }
                    });
                }
            }
        }
    }
    // For Ring Reference 
    count = 0;
    grow_data.iter_mut()
        .for_each(|ggd| ggd.iter_mut()
            .for_each(|level|{
                if let Some(engage) = level.engage_skills.as_mut() {
                    engage[0] = skill_list[ engage_sid[count] as usize].sid;
                }
            })
        );

    GodData::get_list().unwrap().iter()
        .filter(|god| god.gid.to_string().contains("GID_M0") || god.gid.contains("GID_E0"))
        .for_each(|god|{
            if let Some(emblem_position) = EMBLEM_ASSET.iter().position(|&a| god.gid.to_string().contains(a)) {
                if emblem_position < 20 {
                    if let Some(engage_skill) = SkillData::try_index_get(engage_sid[emblem_position]){
                        if let Some(ggd) = god.get_level_data() {
                            ggd.iter().for_each(|level| level.engage_skills.replace(0, engage_skill, 5));
                        }
                    }
                }
            }
        }
    );
}
fn update_god_grow(god_data: &mut Vec<&mut List<GodGrowthData>>) {
    god_data.iter_mut().flat_map(|ggd| ggd.iter_mut()).for_each(|level| level.on_completed());

    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(grow_data) = get_god_grow_data_by_index(c_index) {
                grow_data.iter_mut().for_each(|level| level.on_completed() );
            }
        }
    }
}
fn randomize_emblem_stat_bonuses(rng: &Random, level_data: &mut Vec< &mut List<GodGrowthDataLevelData>>){
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) & 1 == 0 { return; }
    // Skill Range of Invisible Stat+ Skills
    println!("Random Stat Bonuses");
    let stats_skill = STAT_BONUS.get().unwrap();
    let min_index = stats_skill[0]; //Lowest HP Index
    let max_index = stats_skill[65]; //Highest Move Index
    level_data.iter_mut().for_each(|ggld|{
        let stats = get_stats_for_emblem(rng);
        ggld.iter_mut().for_each(|level|{
            let mut stat_index: usize = 0;
            level.synchro_skills.iter().for_each(|skill|{
                let stat_skill = skill.get_skill().unwrap();
                let index = stat_skill.parent.index;
                let priority = stat_skill.get_priority();
                if index <= max_index && min_index <= index && stat_index < 4 {
                    let sb_index = 
                        if priority == 0 {  stats[ stat_index ] * 6 } //Replace Move+1 stat boost
                        else { stats[ stat_index ] * 6  + ( priority  - 1 ) };
                    skill.set_index(stats_skill[ sb_index as usize]);
                    stat_index += 1;
                }
            });
            stat_index = 0;
            level.engaged_skills.iter().for_each(|skill|{
                let stat_skill = skill.get_skill().unwrap();
                let index = stat_skill.parent.index;
                let priority = stat_skill.get_priority();
                if index <= max_index && min_index <= index && stat_index < 4 {
                    let sb_index = 
                        if priority == 0 {  stats[ stat_index ] * 6 } //Replace Move+1 stat boost
                        else { stats[ stat_index ] * 6  + ( priority  - 1 ) };
                    skill.set_index(stats_skill[ sb_index as usize]);
                    stat_index += 1;
                }
            });
        });

    });
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 {
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(level_data) = get_god_level_data_by_index(c_index) {
                let stats = get_stats_for_emblem(rng);
                level_data.iter_mut().for_each(|level|{
                    let mut stat_index: usize = 0;
                    level.synchro_skills.iter().for_each(|skill|{
                        let stat_skill = skill.get_skill().unwrap();
                        let index = stat_skill.parent.index;
                        let priority = stat_skill.get_priority();
                        if index <= max_index && min_index <= index && stat_index < 4 {
                            let sb_index = 
                                if priority == 0 {  stats[ stat_index ] * 6 } //Replace Move+1 stat boost
                                else { stats[ stat_index ] * 6  + ( priority  - 1 ) };
                            skill.set_index(stats_skill[ sb_index as usize]);
                            stat_index += 1;
                        }
                    });
                    stat_index = 0;
                    level.engaged_skills.iter().for_each(|skill|{
                        let stat_skill = skill.get_skill().unwrap();
                        let index = stat_skill.parent.index;
                        let priority = stat_skill.get_priority();
                        if index <= max_index && min_index <= index && stat_index < 4 {
                            let sb_index = 
                                if priority == 0 {  stats[ stat_index ] * 6 } //Replace Move+1 stat boost
                                else { stats[ stat_index ] * 6  + ( priority  - 1 ) };
                            skill.set_index(stats_skill[ sb_index as usize]);
                            stat_index += 1;
                        }
                    });
                });
            }
        }
    }
}
fn randomized_common_sids(emblem_index: i32) {
    let index = emblem_index as usize;
    let emblem_pids = EMBLEM_PERSON.get().unwrap();
    let source_god = GodData::get(format!("GID_{}", EMBLEM_ASSET[index])).unwrap();
    let engage_attack = if emblem_index < 0 || emblem_index >= 19 { "none".to_string() }
    else {  source_god.engage_attack.unwrap().to_string() };
    let ggd = source_god.get_grow_table().unwrap();
    let level_data = GodGrowthData::get_level_data(&ggd.to_string()).unwrap();
    let engaged_skills = level_data[20].engaged_skills;
    emblem_pids.iter()
        .filter(|x| x.1 == emblem_index)
        .for_each(|pid|{
            if let Some(person) = PersonData::try_index_get_mut(pid.0) {
                let mut counter = 0;
                let common = person.get_common_skills();
                let normal = person.get_normal_skills();
                let hard = person.get_hard_skills();
                let lunatic = person.get_lunatic_skills();
                common.clear();
                hard.clear();
                lunatic.clear();
                normal.clear();
                engaged_skills.iter()
                    .for_each(|engaged|{
                        if let Some(skill) = engaged.get_skill() {
                            if skill.get_flag() & 1 == 0 && !skill.sid.to_string().contains("アイクエンゲージスキル") && counter < 4 {  // no laguz friend
                                common.add_skill(skill, 1, 0);
                                hard.add_skill(skill, 1, 0);
                                normal.add_skill(skill, 1, 0);
                                lunatic.add_skill(skill, 1, 0);
                                counter += 1;
                            }
                        }
                    }
                );
                if person.get_asset_force() != 0 { normal.add_sid("SID_命中回避－２０",1, 0); }
                person.set_sp(emblem_index+1);
                if person.get_engage_sid().is_some() && engage_attack != "none" {  person.set_engage_skill(SkillData::get(engage_attack.as_str()).unwrap()); }
            } 
        }
    );
}

fn randomized_emblem_syncho_skills(rng: &Random, grow_data: &mut Vec<&mut List<GodGrowthData>>, level_data: &mut Vec<&mut List<GodGrowthDataLevelData>>) {
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) & 2 == 0 { return; }
    println!("Randomizing Syncho Skills");
    SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng);
    // For the SkillArray
    level_data.iter_mut()
        .flat_map(|ggld| ggld.iter_mut() )
        .for_each(|level|{
            level.synchro_skills.iter().for_each(|sync_skill|{
                if let Some(skill) = sync_skill.get_skill() {
                    if skill.get_flag() & 1 == 0 {
                        let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                        sync_skill.set_index(replacement_skill.parent.index);
                    }

                }
            });
            level.engaged_skills.iter().for_each(|sync_skill|{
                if let Some(skill) = sync_skill.get_skill() {
                    if skill.get_flag() & 1 == 0 {
                        let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                        sync_skill.set_index(replacement_skill.parent.index);
                    }
                }
            });
        }
    );
    eirika_change_skill_adjustment();
    if CUSTOM_EMBLEMS.lock().unwrap()[0] > 0 { 
        let n_customs = CUSTOM_EMBLEMS.lock().unwrap()[0] as usize;
        for x in 0..n_customs {
            let c_index = CUSTOM_EMBLEMS.lock().unwrap()[x+1];
            if let Some(god_grow) = get_god_grow_data_by_index(c_index) {
                god_grow.iter_mut().for_each(|level|{
                    level.synchro_skills.iter_mut()
                        .flat_map(|x|x.iter_mut())
                        .for_each(|sid|{
                            if let Some(skill) = SkillData::get(*sid) {
                                let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                                if replacement_skill.sid.to_string().contains("SID_計略") { *sid = "".into(); }
                                else { *sid = replacement_skill.sid; }
                            }
                        });
                    }
                );
            }
            if let Some(level_data) = get_god_level_data_by_index(c_index) {
                level_data.iter_mut().for_each(|level|{
                    level.synchro_skills.iter().for_each(|sync_skill|{
                        if let Some(skill) = sync_skill.get_skill() {
                            let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                            sync_skill.set_index(replacement_skill.parent.index);
                        }
                    });
                    level.engaged_skills.iter().for_each(|sync_skill|{
                        if let Some(skill) = sync_skill.get_skill() {
                            let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill, false);
                            sync_skill.set_index(replacement_skill.parent.index);
                        }
                    });
                });
            }
        }
    }
    let mut list = SYNCHO_RANDOM_LIST.lock().unwrap();
    // Adding unused/added skills
    let extra_skills = list.get_non_randomized_skill();
    let mut used_skill: [bool; 500] = [false; 500];
    let pool_size = extra_skills.len() as i32;
    let rng = crate::utils::get_rng();
    let mut count = 0;
    if pool_size > 10 {
        GGIDS.iter().for_each(|ggid|{
            let level_data = GodGrowthData::get_level_data(ggid).unwrap();
            let non_hidden_skills = level_data[0].synchro_skills.iter().filter(|s| s.get_skill().unwrap().get_flag() & 1 == 0 ).count();
            let mut non_engage_skill = level_data[0].engaged_skills.iter().filter(|s| s.get_skill().unwrap().get_flag() & 1 == 0 ).count();
            if non_hidden_skills < 4 {
                for _y in 0..4-non_hidden_skills {
                    let mut index;
                    let mut break_counter = 0;
                    loop {
                        index = rng.get_value( pool_size );
                        if index >= 400 { continue; }
                        if !used_skill[index as usize] { break; }
                        break_counter += 1;
                        if break_counter >= 150 { break; }
                    }
                    if break_counter >= 150 { break; }

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
            count += 1;
        });
    }
    grow_data.iter_mut()
        .flat_map(|ggd| ggd.iter_mut())
        .flat_map(|level|level.synchro_skills.iter_mut() )
        .flat_map(|x| x.iter_mut())
        .for_each(|sid|{
            if let Some(skill) = SkillData::get(*sid) {
                let replacement_skill = list.get_replacement(skill, false);
                if replacement_skill.sid.to_string().contains("SID_計略") { *sid = "".into(); }
                else { *sid = replacement_skill.sid; }
            }
        }
    );
    println!("Sync Skills Complete");
}

pub fn adjust_emblem_common_skills() {
    if GameVariableManager::get_number(DVCVariables::EMBLEM_SYNC_KEY ) & 2 == 0 || 
    !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_data_randomized { return; }
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_unit_skill_randomized  {
        for x in 0..23 { randomized_common_sids(x );  }
        println!("Enemy Emblem Unit Skills Changed");
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.emblem_unit_skill_randomized = true );
    }
}

fn adjust_engage_weapon_type() {
    for x in 0..20 {
        let weapon_mask_1 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[x as usize];
        let engage_attack_sid = GodData::get(EMBLEM_GIDS[x]).unwrap().get_engage_attack();
        let mut weapon_mask_2 = EMBLEM_WEAPON[ x as usize];
        for y in 0..20 {
            if x == y { continue; }
            if let Some(linked_engage_attack_sid) =  GodData::get(EMBLEM_GIDS[y]).unwrap().get_engage_attack_link() {
                if engage_attack_sid == linked_engage_attack_sid {
                    weapon_mask_2 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[y as usize];
                    break;
                }
            }
        }
        let mut combine_weapon_mask = weapon_mask_1 | weapon_mask_2 ;
        let engage_atk = engage_attack_sid.to_string();
        if let Some(pos) = EMBLEM_ASSET.iter().position(|emblem| engage_atk.contains(emblem)) {
            match pos {
                0|1|4|6|8|11|14|17|18|19 => {
                    combine_weapon_mask |= EMBLEM_WEAPON[pos]; 
                    change_weapon_restrict(engage_atk.as_str(), combine_weapon_mask);
                }
                _ => {}
            }
        }
    }
    let emblem_list = EMBLEM_LIST.get().unwrap();
    let len = emblem_list.len();
    if len > 20 {
        for x in 20..len {
            if let Some(god) = GodData::try_get_hash(emblem_list[x]) {
                let engage_atk = god.get_engage_attack().to_string();
                change_weapon_restrict(engage_atk.as_str(), 1023);
            }
        }
    }
    change_weapon_restrict("SID_重唱", 1023);   //Echo
}
pub fn change_weapon_restrict(sid: &str, value: i32) {
    if let Some(engage_atk) = SkillData::get_mut(sid) {
        let w1 = engage_atk.get_weapon_prohibit();
        let weapon_mask_value = 1023 - value;
        if w1.value <= 2 { return; }
        w1.value = weapon_mask_value;
        (0..9).into_iter()
            .flat_map(|f| engage_atk.get_style_skill(f)).for_each(|s|{
                let w2 = s.get_weapon_prohibit(); 
                w2.value = weapon_mask_value;
            }
        );
    }
    if let Some(engage_atk) = SkillData::get_mut(concat_string!(sid, "＋")) {
        let w1 = engage_atk.get_weapon_prohibit();
        let weapon_mask_value = 1023 - value;
        if w1.value <= 2 { return; }
        w1.value = weapon_mask_value;
        (0..9).into_iter()
            .flat_map(|f| engage_atk.get_style_skill(f)).for_each(|s|{
                let w2 = s.get_weapon_prohibit(); 
                w2.value = weapon_mask_value;
            }
        );
    }
}

pub fn get_pid_emblems() {
    EMBLEM_PERSON.get_or_init(||{
        let mut list: Vec<(i32, i32)> = Vec::new();
        PersonData::get_list().unwrap().iter().filter(|&p|
            if let Some(jid) = p.get_jid() { jid.to_string().contains("JID_紋章士") }
            else { false }
        ).for_each(|emblem|{
            let jid = emblem.get_jid().unwrap().to_string();
            if let Some(position) = EMBLEM_ASSET.iter().position(|&asset| jid.contains(asset)) {
                list.push( (emblem.parent.index, position as i32 )); 
            }
        });
        list
    });
}
pub fn get_god_level_data_by_index(index: i32) -> Option<&'static mut List<GodGrowthDataLevelData>> {
    if index < GodData::get_count() {
        if let Some(custom_god) = GodData::try_index_get(index) { return custom_god.get_level_data();}
    }
    return None;
}

pub fn get_god_grow_data_by_index(index: i32) -> Option<&'static mut List<GodGrowthData>> {
    if index < GodData::get_count() {
        if let Some(custom_god) = GodData::try_index_get(index) { return GodGrowthData::try_get_from_god_data(custom_god);}
    }
    return None;
}

fn eirika_change_skill_adjustment() {
    println!("Edelgard Gambit / Eirika Twin Skill Adjustment");
    EIRIKA_TWIN_SKILLS.iter()
        .for_each(|&sid| {
            if let Some(eirika_skill) = SkillData::get_mut(sid) {
                let replacement = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(eirika_skill, false);
                let eirika_change_len = eirika_skill.change_skills.len();
                if eirika_change_len > 1 && replacement.parent.index > 1 && replacement.parent.index != eirika_skill.parent.index {
                    if let Some(replace_skill_mut) = SkillData::get_mut(replacement.sid) {
                        let change_array = Array::<&SkillData>::new_specific(eirika_skill.change_skills.get_class(), eirika_change_len).unwrap();
                        for x in 0..eirika_change_len {
                            change_array[x] = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(eirika_skill.change_skills[x], false);
                          // println!("Changed: {} with {}", Mess::get(eirika_skill.change_skills[x].name.unwrap()), Mess::get( change_array[x].name.unwrap()));
                        }
                        replace_skill_mut.change_skills = change_array;
                        replace_skill_mut.flag &= !140737488355329;
                    }   
                }
            }
        }
    );
    let gambit = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement_sid("SID_計略".into() , false);
    let mut list = SYNCHO_RANDOM_LIST.lock().unwrap();
    if let Some(replace_skill_mut) = SkillData::get_mut( gambit.sid) {
        let mut s_list = list.get_non_randomized_skill();
        let change_array = Array::<&SkillData>::new_specific(replace_skill_mut.change_skills.get_class(), 3).unwrap();
        let rng = get_rng();
        for x in 0..3 {
            let mut index = rng.get_value( s_list.len() as i32 ) as usize;
            while s_list[ index ].skill_used { index = rng.get_value( s_list.len() as i32 ) as usize;  }
            s_list[ index ].skill_used = true;
            change_array[x] = SkillData::try_index_get( s_list[ index ].index ).unwrap();
           //  println!("Edelgard Skill {}: #{} {}", x+1, change_array[x].parent.index, Mess::get(change_array[x].name.unwrap()));
        }
        replace_skill_mut.change_skills = change_array;
    }   
}