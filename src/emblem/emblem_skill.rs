use unity::prelude::*;
use engage::{
    gamevariable::*,
    random::*,
    mess::*,
    menu::{BasicMenuResult, config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods}},
    gamedata::{*, skill::*, GodData, god::*},
};
use std::sync::Mutex;
use super::emblem_item::ENGAGE_ITEMS;
use crate::{enums::*, skill::{SkillIndex}, utils::*, CONFIG};
use std::{fs::File, io::Write};
const EMBLEM_WEAPON: [i32; 20] = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66, 258];

pub static ENGAGE_SKILLS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static ENGAGE_SKILLS_CHAOS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
static ENGAGE_ATTACKS: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());
static ENGAGE_ATK_SWAP: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());

pub static STAT_BONUS:  Mutex<[i32; 66]> = Mutex::new([0; 66]);
pub static SYNCHO_RANDOM_LIST: Mutex<SynchoList> = Mutex::new(SynchoList { sync_list: Vec::new(), inherit_list: Vec::new(), chaos_list: Vec::new(), randomized: false,});
pub static mut EIRIKA_INDEX: usize = 11;

struct EngageAttackIndex {
    pub index_1: i32,
    pub index_2: i32,
    pub in_use: bool,
    pub linked_use: bool,
}
impl EngageAttackIndex {
    fn new(value_1: i32, value_2: i32) -> Self { Self { index_1: value_1, index_2: value_2, in_use:false, linked_use: false,}}
}

pub struct SynchoSkill {
    pub index: i32,
    pub max_priority: i32,
    pub min_priority: i32,
    pub randomized_index: i32,
    pub in_use : bool,
    pub eirika_twin_skill: bool,
}
impl SynchoSkill {
    fn new(skill_index: i32, priority: i32, eirika: bool) -> Self {
        Self { index: skill_index, max_priority: priority, min_priority: priority, in_use: false, randomized_index: 0, eirika_twin_skill: eirika} 
    }
}
pub struct SynchoList {
    pub sync_list: Vec<SynchoSkill>,
    pub inherit_list: Vec<SynchoSkill>,
    pub chaos_list: Vec<SynchoSkill>,
    pub randomized: bool, 
}
impl SynchoList {
    // For the three houses gambits to force them to be 4 separate skills instead of one 4-level skill
    pub fn add_to_non_upgrade(&mut self, sid: &str){
        let skill = SkillData::get(sid);
        if skill.is_none() { return; } 
        let skill_index = skill.unwrap().parent.index;
        let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, 0, false)); }
    }
    pub fn add_by_sid(&mut self, sid: &str, is_syncho: bool){
        let skill = SkillData::get(sid);
        if skill.is_some() { self.add_list(skill.unwrap(), is_syncho); }
    }
    pub fn add_inherit(&mut self, skill_index: i32, priority: i32, _in_sync: bool, in_inherit: bool, chaos_only: bool, is_eirika: bool) {
        let found = self.chaos_list.iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  self.chaos_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
        else {
            let skill = found.unwrap();
            if skill.max_priority < priority { skill.max_priority = priority; }
            if priority < skill.min_priority { skill.min_priority = priority; }
        }
        if chaos_only { return; }
        //if in_sync {
            let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
            else {
                let skill = found.unwrap();
                if skill.max_priority < priority { skill.max_priority = priority; }
                if priority < skill.min_priority { skill.min_priority = priority; }
            }
        //}
        if in_inherit {
            let found = self.inherit_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {  self.inherit_list.push(SynchoSkill::new(skill_index, priority, is_eirika));  }
            else {
                let skill = found.unwrap();
                if skill.max_priority < priority { skill.max_priority = priority; }
                if priority < skill.min_priority { skill.min_priority = priority; }
            }
        }
    }
    pub fn add_list(&mut self, skill: &SkillData, is_syncho: bool) {
        if skill.get_flag() & 1 != 0 {  return; }   // cannot be hidden
        // ignore "None" "Night and Day", "Friendly Riviary"
        let can_inherit = skill.get_inheritance_cost() != 0;
        let only_chaos = !can_inherit && !is_syncho;
        //if !can_inherit && !is_syncho { return; }   
        let sid = skill.sid.get_string().unwrap();
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" || sid == "SID_無し"  { return; }
        if sid == "SID_異界の力" { // if book of worlds
            let skill_index = skill.parent.index;
            self.add_inherit(skill_index, 0, is_syncho, can_inherit, only_chaos, false);
            return;
        }
        for x in 0..EIRIKA_TWIN_SKILLS.len() {  //Eirika Skills
            if EIRIKA_TWIN_SKILLS[x] == sid {
                let skill_index = if x < 6 { skill.parent.index } else { skill.parent.index - 3 };
                let priority = if x < 6 { 1 } else { 2 };
                self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, true);
                return;
            }
        }
        let priority = skill.get_priority();
        let skill_index = if priority >= 1 { skill.parent.index - (priority - 1) } else { skill.parent.index };
        self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, false);
    }
    pub fn reset(&mut self) {
        for x in self.sync_list.iter_mut() {
            x.in_use = false;
            x.randomized_index = 0;
        }
        for x in self.inherit_list.iter_mut() {
            x.in_use = false;
            x.randomized_index = 0;
        }
        for x in self.chaos_list.iter_mut() {
            x.in_use = false;
            x.randomized_index = 0;
        }
        self.sync_list[0].in_use = true;
        self.randomized = false;
    }
    pub fn randomized(&mut self, rng: &Random) {
        if self.randomized { return; }
        let i_size = self.inherit_list.len() as i32;
        let i_list = &mut self.inherit_list;
        let mut value;
        for x in 0..i_size {     //inherit skill -> inherit skill
            value = rng.get_value( i_size );
            let dp = i_list[x as usize].max_priority - i_list[x as usize].min_priority;
            let mut dp2 = i_list[value as usize].max_priority - i_list[value as usize].min_priority;
            let mut count = 0;
            while i_list[ value as usize ].in_use || ( dp != dp2 ) {
                if count == 200 { break; }
                value = rng.get_value( i_size );
                dp2 = i_list[value as usize].max_priority - i_list[value as usize].min_priority;
                count += 1;
            }
            if count == 200 { value = x as i32; }
            i_list[value as usize].in_use = true;
            i_list[x as usize ].randomized_index = value;
        }
        let i_list = &mut self.inherit_list;
        let s_list = &mut self.sync_list;
        let size = s_list.len() as i32;
        for x in 1..size-4 {
            s_list[x as usize].randomized_index = -1;
            let skill_index = s_list[x as usize].index;
            let found = i_list.iter().find(|z| z.index == skill_index);     // is inherit skill in sync list
            if found.is_some() {
                let random_index = i_list[ found.unwrap().randomized_index as usize ].index;
                let found2 = s_list.iter().position(|z| z.index == random_index);    // is the randomized skill in sync list in inherit list
                if found2.is_some() {
                    s_list[found2.unwrap()].in_use = true;
                    s_list[x as usize ].randomized_index = found2.unwrap() as i32;
                }
            }
        }
        value =  rng.get_value( size - 5) + 1;
        while s_list[ value as usize].max_priority != 0 || s_list[ value as usize].in_use { value =  rng.get_value( size - 5) + 1; }
        s_list[0].randomized_index = value; s_list[ value as usize].in_use = true;
        for x in 1..size-4 {
            if s_list[x as usize].randomized_index != -1 { continue; }
            let dp = s_list[x as usize].max_priority;
            let mut count = 0;
            value = rng.get_value( size - 1 ) + 1;
            let mut dp2 = s_list[value as usize].max_priority;
            while s_list[ value as usize ].in_use || ( dp != dp2 ) {
                if count >= 100 && !s_list[ value as usize ].in_use { break; }
                if count == 200 { break; }
                value = rng.get_value( size - 1 ) + 1;
                dp2 = s_list[value as usize].max_priority ;
                count += 1;
            }
            if count == 200 { value = x as i32; }
            s_list[value as usize].in_use = true;
            s_list[x as usize ].randomized_index = value;
        }
        let c_size = self.chaos_list.len() as i32;
        let c_list = &mut self.chaos_list;
        // Chaos
        for x in 0..c_size {
            let dp = c_list[x as usize].max_priority;
            let mut count = 0;
            value = rng.get_value( c_size );
            let mut dp2 = c_list[value as usize].max_priority;
            while c_list[ value as usize ].in_use || ( dp != dp2 ) {
                if count >= 100 && !c_list[ value as usize ].in_use { break; }
                if count == 200 { break; }
                value = rng.get_value( c_size );
                dp2 = c_list[value as usize].max_priority ;
                count += 1;
            }
            if count == 200 { value = x as i32; }
            c_list[value as usize].in_use = true;
            c_list[x as usize ].randomized_index = value;
        }
        self.randomized = true;
        self.print_inherit_list();
    }
    pub fn get_replacement_sid(&self, sid: &Il2CppString, is_inherit: bool) -> &'static SkillData {
        let skill = SkillData::get(&sid.get_string().unwrap());
        if skill.is_some() {
            let sk = skill.unwrap();
            let replacement = self.get_replacement(sk, is_inherit);
            return replacement;
        }
        return SkillData::get("SID_無し").unwrap();
    }
    pub fn get_replacement(&self, original_skill: &SkillData, is_inherit: bool) -> &'static SkillData {
        let skill_list = SkillData::get_list().unwrap();
        let o_skill = &skill_list[ original_skill.parent.index as usize];
        if original_skill.get_flag() & 1 != 0 {  return o_skill; }
        let sid = original_skill.sid.get_string().unwrap();         // ignore "None" "Night and Day", "Friendly Riviary"
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { return o_skill; }
        if sid == "SID_無し" { return o_skill; } 
        let mut priority = original_skill.get_priority();
        let skill_index;
        if sid == "SID_異界の力" { priority = 0; }
        let is_eirika = EIRIKA_TWIN_SKILLS.iter().position(|x| *x == sid);
        if is_eirika.is_some() {
            let x = is_eirika.unwrap();
            skill_index = if x < 6 { original_skill.parent.index } else { original_skill.parent.index - 3 };
            priority = if x < 6 { 1 } else { 2 };
        }
        else {
            skill_index = if priority == 0 { original_skill.parent.index } else { original_skill.parent.index - (priority - 1)};
        }
        let chaos_mode = GameVariableManager::get_number("G_ChaosMode");
        let mode = if chaos_mode == 3 || ( !is_inherit && chaos_mode == 1 ) { 2 }
                    else if is_inherit { 0 } else { 1 };
        let list = [&self.inherit_list, &self.sync_list, &self.chaos_list];
        let found = list[mode as usize].iter().find(|x| x.index == skill_index);
        if found.is_none() { return o_skill; }
        let new_skill_index = list[mode as usize][ found.unwrap().randomized_index as usize ].index;
        let new_max_priority = list[mode as usize][ found.unwrap().randomized_index as usize ].max_priority;
        let is_eirika_twin = list[mode as usize][ found.unwrap().randomized_index as usize ].eirika_twin_skill;
        let new_priority = priority - found.unwrap().min_priority + list[mode as usize][ found.unwrap().randomized_index as usize ].min_priority;

        if is_eirika_twin { // Replacement skill is an Eirika Twin Skill (Lunar Brace/Solar Brace/Eclipse Brace etc...)
            if new_max_priority <= priority { // new_max_priority is 2 for Lunar/Solar/Eclipse Brace +
                return &skill_list[ ( new_skill_index + 3 ) as usize]; 
            }
            return &skill_list[ new_skill_index as usize];
        }
        if new_max_priority == 0 || priority == 0 {  return &skill_list[new_skill_index as usize]; }
        if new_max_priority <= new_priority { return &skill_list[ (new_skill_index + new_max_priority - 1) as usize ]; }
        return &skill_list[ (new_skill_index + new_priority - 1  ) as usize];
    }
    pub fn print_inherit_list(&self) {
        let skill_list = SkillData::get_list().unwrap();
        let filename = format!("sd:/Draconic Vibe Crystal/Emblem Skills List.txt");
        let file = File::options().create(true).write(true).truncate(true).open(filename);
        if file.is_err() { println!("Cannot create output file"); return; }
        let mut f = file.unwrap();
        let i_list = &self.inherit_list;
        writeln!(&mut f, "Inherit Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..i_list.len() {
            let sid = skill_list[ i_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ i_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[x].index, sid).as_str())).get_string().unwrap();

            let random_index =  i_list[x].randomized_index as usize;
            let sid2 = skill_list[ i_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ i_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, i_list[x].index,  random_index, i_list[x].max_priority, i_list[random_index].max_priority).unwrap();
        }
        let s_list = &self.sync_list;
        writeln!(&mut f, "\nSync Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..s_list.len() {
            let sid = skill_list[ s_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ s_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[x].index, sid).as_str())).get_string().unwrap();
            if s_list[x].randomized_index == -1 { continue; }
            let random_index =  s_list[x].randomized_index as usize;
            let sid2 = skill_list[ s_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ s_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, s_list[x].index, random_index, s_list[x].max_priority, s_list[random_index].max_priority).unwrap();
        }
        let c_list = &self.chaos_list;
        writeln!(&mut f, "\nChaos Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..c_list.len() {
            let sid = skill_list[ c_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ c_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[x].index, sid).as_str())).get_string().unwrap();
            if c_list[x].randomized_index == -1 { continue; }
            let random_index =  c_list[x].randomized_index as usize;
            let sid2 = skill_list[ c_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ c_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, c_list[x].index, random_index, c_list[x].max_priority, c_list[random_index].max_priority).unwrap();
        }
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
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_勇空＋", true);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_太陽の腕輪＋", true);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_日月の腕輪＋", true);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_蒼穹＋", true);
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_引込の計");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_猛火計");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_聖盾の備え");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_毒矢");
    
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
            if link_index != -1 {
                ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(link_index as i32, count));
            }
        }
        count += 1;
    }
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
    change_weapon_restrict("SID_重唱", 64);
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
    SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng2);
    if mode == 1 || mode == 3 {
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
    }
    if mode >= 2 {
        rng.ctor(seed);
        println!("Randomizing Engage Attacks");
        let engage_atk_size = ENGAGE_ATTACKS.lock().unwrap().len();
        let mut linked_gid: [bool; 19] = [false; 19];
        let mut count = 0;
        for x in 0..20 { 
            let god = GodData::get_mut(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
            // Engage Attack
            let mut value = rng.get_value(engage_atk_size as i32) as usize;
            loop {
                if ( x == 9 || x == 13 ) && value == 7 {    // Prevent Byleth/Tiki from getting Astra Storm 
                    value = rng.get_value(engage_atk_size as i32) as usize; 
                    continue;
                }
                if ENGAGE_ATTACKS.lock().unwrap()[value].in_use { 
                    value = rng.get_value(engage_atk_size as i32) as usize; 
                    continue;
                }
                break;
            }
            let engage_sid = skill_list[ ENGAGE_ATTACKS.lock().unwrap()[value].index_1 as usize ].sid;
            god.set_engage_attack( engage_sid );
            ENGAGE_ATTACKS.lock().unwrap()[value].in_use = true;
            // Linked Engage Attack
            let mut linked_index = rng.get_value(engage_atk_size as i32) as usize;
            loop {
                if ( x == 9 || x == 13 ) && linked_index == 7 {  // Prevent Byleth/Tiki from getting Astra Storm 
                    linked_index = rng.get_value(engage_atk_size as i32) as usize;
                    continue;
                }
                if ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use || linked_index == value {
                    linked_index = rng.get_value(engage_atk_size as i32) as usize; 
                    continue;
                }
                break;
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
                    while linked_gid[linked_god_index] || x == linked_god_index { linked_god_index = rng.get_value(19) as usize; }
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
    let mut count: usize = 0;
    let mut engage_sid: [i32; 20] = [-1; 20];
    let skill_pool = if GameVariableManager::get_number("G_ChaosMode") >= 2 { &ENGAGE_SKILLS_CHAOS } else { &ENGAGE_SKILLS };
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
}
fn randomized_common_sids(name: String) {
    let person_list = PersonData::get_list_mut().unwrap();
    for x in 300..1250 {
        let person_x = &person_list[x as usize];
        if person_x.get_name().is_none() { continue; }
        let name2 = Mess::get( person_x.get_name().unwrap() ).get_string().unwrap();
        if name != name2 { continue; }
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
    }
    // Change for ring reference
    for x in 0..20 {
        let god = GodData::get(&format!("GID_{}", EMBLEM_ASSET[x as usize])).unwrap();
        let name = Mess::get(god.mid).get_string().unwrap();
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
                syncho_skills[z] = replacement_skill.sid;
            }
            god_grow[y].on_complete(); 
        }
        randomized_common_sids(name);
    }
    randomized_common_sids(Mess::get("MPID_Reflet").get_string().unwrap());
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
            let name = Mess::get(SkillData::get_mut(&style_sid).unwrap().name.unwrap()).get_string().unwrap();
            let w2 = SkillData::get_mut(&style_sid).unwrap().get_weapon_prohibit();
            println!("Engage Attack {} #{} Weapon: {}", name, SkillData::get(&style_sid).unwrap().parent.index, crate::utils::get_weapon_mask_str(value));    
            w2.value = 1023 - value;
        }
    }
}

pub struct EmblemSkillChaos;
impl ConfigBasicMenuItemSwitchMethods for EmblemSkillChaos {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().emblem_skill_chaos, 0, 3, 1);
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
            _ => { "Default randomization pool for sync and engage skills."},
        }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = match CONFIG.lock().unwrap().emblem_skill_chaos {
            1 => { "Sync Skills" },
            2 => { "Engage Skills" },
            3 => { "Sync / Engage "},
            _ => { "Default Skills"},
        }.into();
    }
}