use unity::prelude::*;
use engage::{
    force::*, 
    gamedata::{dispos::*, skill::*, unit::*, *}, 
    gameuserdata::*, 
    gamevariable::*, 
    menu::{config::*, BasicMenuItemAttribute, BasicMenuResult}, 
    mess::*,
};
use super::CONFIG;
use crate::utils::*;

pub const EMBLEMS: &[&str] = &[ "GID_M010_敵リン", "GID_M007_敵ルキナ", "GID_M014_敵ベレト", "GID_M024_敵マルス", "GID_M017_敵シグルド", "GID_M017_敵セリカ", "GID_M019_敵ミカヤ", "GID_M019_敵ロイ", "GID_M017_敵リーフ", "GID_E006_敵エーデルガルト", "GID_E006_敵クロム", "GID_E006_敵カミラ", "GID_E006_敵セネリオ", "GID_E006_敵ヴェロニカ", "GID_E006_敵ヘクトル", "GID_E006_敵チキ"];
pub const ENGAGE: &[&str] = &[ "AI_AT_EngageAttack", "AI_AT_EngageAttack", "AI_AT_EngageDance", "AI_AT_EngageAttack", "AI_AT_EngagePierce", "AI_AT_EngageAttack", "AI_AT_AttackToHeal", "AI_AT_EngageAttack", "AI_AT_EngageAttackNoGuard", "AI_AT_EngageAttack", "AI_AT_EngageAttack", "AI_AT_EngageCamilla", "AI_AT_EngageAttack", "AI_AT_EngageSummon", "AI_AT_EngageWait", "AI_AT_EngageBlessPerson"];
pub enum CapabilityType {
    Hp = 0,
    Str = 1,
    Dex = 2,
    Spd = 3,
    Luck = 4,
    Def = 5,
    Mag = 6,
    Res = 7,
    Build = 8,
    Sight = 9,
    Move = 10,
}
impl CapabilityType {
    pub fn value(&self) -> i32 {
        match *self {
            CapabilityType::Hp => { 0 },
            CapabilityType::Str => { 1 },
            CapabilityType::Dex => { 2 },
            CapabilityType::Spd => { 3 },
            CapabilityType::Luck => { 4 },
            CapabilityType::Def => { 5 },
            CapabilityType::Mag => { 6 },
            CapabilityType::Res => { 7 },
            CapabilityType::Build => { 8 },
            CapabilityType::Sight => { 9 },
            CapabilityType::Move => { 10 },
        }
    }
}

pub fn is_boss(this: &PersonData) -> bool { this.get_combat_bgm().is_some() }
pub fn calculate_player_cap() -> i32 {
    let mut max_cap: [i32; 10] = [0; 10];
    let mut unit_name: [&Il2CppString; 10] = [" N/A".into(); 10];
    GameVariableManager::make_entry_norewind("G_Player_Rating_Average", 0);
    for force in 0..max_cap.len() {
        let force_type: [ForceType; 4] = [ForceType::Player, ForceType::Absent, ForceType::Ally, ForceType::Temporary];
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            let i: usize = force.into();
            for unit in force_iter {
                if unit.person.get_asset_force() != 0 { continue; }
                let cur = unit_cap_total(unit, true);
                if force == 0 {
                    if max_cap[i] < cur {
                        max_cap[i] = cur;
                        unit_name[i] = unit.person.name.unwrap();
                    }
                }
                else {
                    if max_cap[i] < cur && cur < max_cap[i-1] {
                        max_cap[i] = cur;
                        unit_name[i] = unit.person.name.unwrap();
                    }
                }
            }
        }
    }   
    let mut average: i32 = 0;
    let diff = GameUserData::get_difficulty(false);
    let count_average: usize = max_cap.len() - (2*diff as usize);
    for i in 0..count_average {
        average += max_cap[i] / (count_average as i32 );
        println!("Rank {}: {}/{} with rating {}", i+1, Mess::get(unit_name[i]).to_string(), unit_name[i].to_string(), max_cap[i]);
    }
    println!("{} unit Average is {}", count_average, average);
    if average > GameVariableManager::get_number("G_Player_Rating_Average") {
        GameVariableManager::set_number("G_Player_Rating_Average", average);
    }

    average
}

#[skyline::from_offset(0x01a3c3b0)]
fn unit_learn_job_skill(this: &Unit, job: &JobData, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[skyline::from_offset(0x01a3c290)]
fn unit_learn_job(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>; 

#[unity::from_offset("App", "Unit", "set_LearnedJobSkill")]
fn unit_set_learned_job_skill(this: &Unit, value: Option<&SkillData>, method_info: OptionalMethod);

pub fn update_learn_skills() {
    let force_type: [ForceType; 7] = [ForceType::Player, ForceType::Absent, ForceType::Dead, ForceType::Lost, ForceType::Enemy, ForceType::Ally, ForceType::Temporary];
    for ff in force_type {
        let force_iter = Force::iter(Force::get(ff).unwrap());
        for unit in force_iter {
            unit_update_learn_skill(unit);
        }
    }
}

pub fn unit_update_learn_skill(unit: &Unit) {
    if unit.learned_job_skill.is_some() {  unsafe { unit_set_learned_job_skill(unit, None, None) };
        let skill = unsafe { unit_learn_job_skill(unit, unit.job, None) }; 
        if skill.is_some() { unsafe { unit_set_learned_job_skill(unit, skill, None) }; }
    }
    else { unsafe { unit_learn_job(unit, None) }; }
}

pub fn unit_cap_total(this: &Unit, with_hp: bool) -> i32 {
    let mut total = 0;
    if with_hp {
        if this.get_capability(CapabilityType::Str.value(), false) < this.get_capability(CapabilityType::Mag.value(), false) {
            total += 2*this.get_capability(CapabilityType::Mag.value(), true) + this.get_capability(CapabilityType::Str.value(), true);
        }
        else { total += 2*this.get_capability(CapabilityType::Str.value(), true) + this.get_capability(CapabilityType::Mag.value(), true); }
        total += this.get_capability(CapabilityType::Hp.value(), false);
        total += this.get_capability(CapabilityType::Dex.value(), false);
        total += this.get_capability(CapabilityType::Luck.value(), false);
        total += this.get_capability(CapabilityType::Def.value(), false) * 5 / 4;
        total += this.get_capability(CapabilityType::Res.value(), false) * 5 / 4;
        total += this.get_capability(CapabilityType::Spd.value(), true) * 3 / 2;
        total += this.get_capability(CapabilityType::Sight.value(), false) * 3 / 2;
        total += this.get_capability(CapabilityType::Move.value(), true) * 4;
        total += this.get_capability(CapabilityType::Build.value(), false) * 2;
    }
    else { for x in 1..8 { total += this.get_capability(x, false); } }
    total
}

pub fn promote_unit(this: &Unit, level: i32){
    // if class is already promoted or class is base but does not have promotions
    let job = this.get_job();
    let job_max_level = job.get_max_level() as i32;
    if job.is_high() || (job.is_low() && !job.has_high()) {
        if job.is_high() && level > 40 {
            this.set_level(20);
            this.set_internal_level(level - 20);
        }
        else if job.is_high() {
            let total_level = this.level as i32 + this.internal_level as i32; 
            if total_level < level {
                let new_level = level - total_level + this.level as i32;
                if new_level > 20 {
                    this.set_level(20);
                    this.set_internal_level( new_level - 20 + this.internal_level as i32);
                }
                else { this.set_level(new_level); }
            }
         }
        if ( job.is_low() && !job.has_high() ) && level > 40 {
            this.set_level(40);
            this.set_internal_level(level - 40);
        }
    }
    else if job_max_level < level && job.is_low() {
        let job_jid = job.jid.to_string();
        let mut high_job_index: usize = 0;
        if job_jid == "JID_ソードペガサス" || job_jid == "JID_ランスペガサス" || job_jid == "JID_アクスペガサス" {
            high_job_index = 1; //Change Flier to Wyvern Instead
        } 
        let new_job = &job.get_high_jobs()[high_job_index];
        this.class_change(new_job);
        this.set_level(level-job_max_level);
        this.set_weapon_mask_from_person();
        this.set_internal_level(job_max_level);
        if this.person.get_unit_icon_id().to_string() == "702MorphLC" {
            this.person.set_unit_icon_id("702Morph".into());
        }
    }
}

pub fn demote_unit(this: &Unit, level: i32) {
    let job = this.get_job();
    let job_max_level = job.get_max_level() as i32;
    let current_level = this.level as i32 + this.internal_level as i32; 
  
    if level < current_level {
        if job_max_level == 40 { 
            this.set_level(level); 
            this.auto_grow_capability(level, level);
        }
        else if job.is_high(){
            if level < 20 {
                let low_jobs = job.get_low_jobs();
                if low_jobs.len() == 3 { 
                    let rnd = engage::random::Random::get_game();
                    this.class_change(low_jobs[rnd.get_value(3) as usize]);
                }
                else if low_jobs.len() == 1 { this.class_change(low_jobs[0]); }
                this.auto_grow_capability(level, level);
                this.set_level(level);
                this.set_internal_level(0);
                this.set_weapon_mask_from_person();
            }
            else {
                this.auto_grow_capability(level-20, level);
                this.set_level(level-20);
                this.set_internal_level(20);
            }
        }
        else {  // base class that stays a base class
            this.auto_grow_capability(level, level);
        }
    }
}

pub fn get_number_main_chapters_completed() -> i32 {
    let mut number = 0;
    let chapters = ChapterData::get_list_mut().unwrap();
    let length = chapters.len();
     for x in 0..length {
        if str_start_with(chapters[x].cid, "CID_M") || str_start_with(chapters[x].cid, "CID_S"){
            if GameUserData::is_chapter_completed(chapters[x]) { number += 1; }
        }
    }
    number
}

pub fn auto_level_unit(unit: &mut Unit){
    if !GameVariableManager::get_bool( "G_Cleared_M006") { return; } 
    if GameUserData::is_evil_map() { return; }
    let diff = GameUserData::get_difficulty(false);
    let player = unit.person.get_asset_force() == 0;
    let mut level =  unsafe{ calculate_average_level(get_sortie_unit_count(None)) } + diff;
    if is_boss(unit.person) { level += 3; }
    if player { level -= diff; }
    let current_level = unit.level as i32 + unit.internal_level as i32;
    if current_level < level {
        for _x in 0..(level - current_level) { unit.level_up(3); }
        if !player { promote_unit(unit, level); }
        else {
            let max_job_level = unit.job.get_max_level() as i32;
            if max_job_level < unit.level as i32 {
                let new_level = max_job_level;
                let new_internal_level = unit.internal_level as i32 + unit.level as i32 - max_job_level;
                unit.set_level(new_level.into());
                unit.set_internal_level(new_internal_level.into());
            } 
        }
    }
        //Adjust Stats 
    let player_cap = GameVariableManager::get_number("G_Player_Rating_Average");
    let starting_cap = unit_cap_total(unit, true);
    let mut enemy_cap = unit_cap_total(unit, true);
    let mut count = 0;
    let unit_level = unit.level;
    let floor_cap = if player {player_cap }
        else { player_cap + diff*( get_number_main_chapters_completed() - 10 ) };
    while enemy_cap < floor_cap && count < 30 {
        unit.level_up(4);
        enemy_cap = unit_cap_total(unit, true);
        unit.level = unit_level;
        count += 1;
    }
    if starting_cap != enemy_cap { println!("{} {} gain {} stat points to {} ( {} Ups )", Mess::get(unit.get_job().name).to_string(), Mess::get(unit.person.get_name().unwrap()).to_string(), enemy_cap-starting_cap, enemy_cap, count); }
    unit.set_hp(unit.get_capability(0, true));
    return;
}

pub fn auto_level_unit_for_random_map(unit: &mut Unit, leader: bool){
    if !GameVariableManager::get_bool( "G_Cleared_M004") { return; } 
    let diff = GameUserData::get_difficulty(false);
    let player = unit.person.get_asset_force() == 0;
    let mut level = crate::utils::max( (crate::continuous::get_story_chapters_completed()-6)*2, crate::continuous::get_story_chapters_completed() + 4);
    level = crate::utils::max( calculate_average_level(14 - diff*2) + diff - 1, level);
    if is_boss(unit.person) || leader { level += 3; }
    if player { level -= diff; }
    let current_level = unit.level as i32 + unit.internal_level as i32;
    for _x in 0..(level - current_level) { unit.level_up(3); }
    if current_level < level { promote_unit(unit, level); }
    else if level < current_level { demote_unit(unit, level); }

    let offset = unit.person.get_offset_by_difficulty();
    let maps = crate::continuous::get_number_main_chapters_completed2() + diff;

    unit.auto_grow_capability(unit.level as i32, level + maps / 4 );

    if unit.job.max_level == 20 {
        let div = level / 20;
        unit.set_internal_level(div*20);
        if div > 0 { unit.set_level( crate::utils::max(1, level - div*20));  }
        else { unit.set_level(level); }
    }
    else {
        if level > 40 {
            unit.set_level(level-20);
            unit.set_internal_level(20);
        }
        else { unit.set_level(level); }
    }
    for x in 0..10 { unit.base_capability.capability[x] -= offset[x]; }

    if GameVariableManager::get_number("G_Player_Rating_Average") == 0 { calculate_player_cap(); }
    let player_cap = GameVariableManager::get_number("G_Player_Rating_Average");

    let unit_level = unit.level;

    let floor_cap = if player { player_cap }
        else { player_cap + diff * ( maps - 10 )  } - 5;

    let ceil = floor_cap + maps;
    while ceil <  unit_cap_total(unit, true) {
        unit.level = unit_level + 1;
        unit.level_down();
    } 
    unit.set_hp(unit.get_capability(0, true));
    return;
}

pub fn calculate_average_level(sortie_count: i32) -> i32 {
    let vander_replace = if GameVariableManager::exist("G_R_PID_ヴァンドレ") {
        GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string()
    } 
    else { "PID_ヴァンドレ".to_string() };
    let mut sum = 0;
    let count = if sortie_count == 0 { 10 } else { sortie_count };
    let mut used: Vec<i32> = Vec::new();
    for _x in 0..count {
        let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
        let mut level = 0;
        let mut last_index = -1;
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            for unit in force_iter {
                if unit.person.pid.to_string() == vander_replace { continue; }
                let total_level = unit.level as i32 + unit.internal_level as i32; 
                let index = unit.person.parent.index;
                if level < total_level && !used.iter().any(|&i| i == index) {
                    level = total_level; 
                    last_index = index;
                }
            }
        }
        if last_index > 0 {
            used.push(last_index);
            sum += level;
        }
    }
    let average = sum / count;
    println!("Player Average Level: {}", average);
    average
}


//Get Average Level of Party
#[skyline::from_offset(0x02b4afa0)]
pub fn get_average_level(difficulty: i32, sortie_count: i32, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x024f2c10)]
pub fn get_sortie_unit_count(method_info: OptionalMethod) -> i32;

pub fn get_difficulty_adjusted_average_level() -> i32 {
    unsafe {
        if get_sortie_unit_count(None) == 0 { return get_average_level(0, 10, None); }
        else { return get_average_level(0, get_sortie_unit_count(None), None); }
    }
}

pub fn str_start_with(this: &Il2CppString, value: &str) -> bool { unsafe { string_start_with(this, value.into(), None) } }

#[skyline::from_offset(0x023349c0)]
pub fn god_pool_create(data: &GodData, method_info: OptionalMethod) -> &'static GodUnit;

#[unity::from_offset("App", "Unit", "TryConnectGodUnit")]
pub fn unit_connect_god_unit(this: &Unit, god_unit: &GodUnit, method_info: OptionalMethod) -> &'static GodUnit;

pub fn try_equip_emblem(unit: &Unit, emblem: usize) -> bool {
    // triabolical config check
    println!("Attempting to equip emblems for enemies");

    let jobname = unit.person.get_job().unwrap().name.to_string();
    if emblem >= EMBLEMS.len() { return false; }
    if unit.person.get_engage_sid().is_some() { return false; }
    if GodData::get(EMBLEMS[emblem]).is_none() { return false; }
    let job = unit.get_job();
    if job.name.to_string() == "MJID_Emblem" || jobname == "MJID_Emblem" { return false; }
    if job.get_sort() == 9999 { return false;}
    //Prevents Wyrms/Wolves from getting emblems
    
    if jobname == "JID_異形飛竜" || jobname  == "JID_幻影飛竜" {  return false; } //Wyverns
    if jobname  == "JID_異形竜" || jobname == "JID_幻影竜" {  return false; } //Wyrms
    if job.parent.index < 10 { return false; }

    if ( job.get_flag().value == 0 && job.jid.to_string() != "JID_蛮族" ) || job.get_flag().value == 8 { return false; }
    let style_name = job.get_job_style();
    if style_name.is_some() {
                // Not Flying or Armored or wolf knight for Bow/Magic Emblems
        let god_data = GodData::get(EMBLEMS[emblem]).unwrap();
        if style_name.unwrap().to_string() == "飛行スタイ ル" || style_name.unwrap().to_string() == "重装スタイル" || job.jid.to_string() == "JID_ウルフナイト" {
            match emblem {
                0 | 1 | 5 | 6 | 11 | 12 | 13 => { false }
                _ => { 
                    unsafe {
                        let god_unit = god_pool_create(god_data, None);
                        unit_connect_god_unit(unit, god_unit, None);
                    }
                    true
                },
            }
        }
        else {
            unsafe {
                let god_unit = god_pool_create(god_data, None);
                unit_connect_god_unit(unit, god_unit, None);
            }
            true
        }
    }
    else { false }
}
#[unity::class("App", "UnitAI")]
pub struct UnitAI {}

#[skyline::from_offset(0x01a522a0)]
pub fn get_unit_ai(this: &Unit, method_info: OptionalMethod) -> &'static UnitAI;

#[unity::from_offset("App", "Unit", "SetDisposAi")]
pub fn set_unit_ai_dispos(this: &Unit, data: &DisposData, method_info: OptionalMethod);

pub fn adjust_emblem_unit_ai(unit: &Unit, data: &DisposData, emblem_index: usize){
    //
    println!("Attempting to change AI");
    unsafe {
        let diff = GameUserData::get_difficulty(false);
        data.set_ai_attack_name(ENGAGE[emblem_index].into());
        if diff == 2 { data.set_ai_attack_value("2,2".into()); }
        else { data.set_ai_attack_value("3,3".into()); }
        if EMBLEMS[emblem_index] == "GID_M017_敵カムイ" { data.set_ai_attack_value("255, 255, 3, 3".into());  }
        set_unit_ai_dispos(unit, data, None);
    }
}

pub fn autolevel_party(){
    if GameVariableManager::get_bool("G_AutoBench"){
        let bench = Force::get(ForceType::Absent).unwrap();
        let avg_number = 8 + 2*GameUserData::get_difficulty(false);
        let player_average = calculate_average_level(avg_number) - GameUserData::get_difficulty(false);
        println!("Autoleveling Bench to average of {}", player_average);
        let mut force_iter = Force::iter(bench);
        while let Some(unit) = force_iter.next() {
            let total_level: i32 = (unit.level as i8 + unit.internal_level) as i32;
            let number_of_levelups = player_average - total_level;
            if number_of_levelups < 1 { continue; }
            multiple_level_ups(unit, number_of_levelups);
            println!("{}: gained {} levels", Mess::get_name(unit.person.pid), number_of_levelups);
        }
    }
}

pub fn multiple_level_ups(unit: &Unit, number_of_levels: i32){
    // Levels up unit and fixes their HP and internal Level
    if number_of_levels < 0 { return; }
    let job_max_level = unit.get_job().get_max_level();
    for _x in 0..number_of_levels { 
        if job_max_level <= unit.level {
            if unit.get_job().is_low() && unit.get_job().has_high() {
                unit.class_change(unit.get_job().get_high_jobs()[0]);
                let weapon_mask: &mut WeaponMask = unit.get_aptitude();
                let value = weapon_mask.value;
                unit.set_weapon_mask_from_person();
                weapon_mask.value = value;
            }
        }
        unit.level_up(3);
        unit.add_sp(100);
    }
    unit.set_hp(unit.get_capability(0, true));
    unit.get_learn_job_skill();
}

pub struct BenchAutoLevelOption;
impl ConfigBasicMenuItemSwitchMethods for BenchAutoLevelOption {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry("G_AutoBench", 0); } 
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle =  GameVariableManager::get_bool("G_AutoBench");
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);
        if toggle != result {
            GameVariableManager::set_bool("G_AutoBench", result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new();  }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = if !GameVariableManager::get_bool("G_AutoBench") {"Undeployed will not be autoleveled at the end of the chapter." }
            else { "Undeployed units will autolevel to difficulty-adjusted average."   }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = if !GameVariableManager::get_bool("G_AutoBench") { "Disabled" }
            else { "Enabled" }.into();
    }
}
pub extern "C" fn autobench() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<BenchAutoLevelOption>("Post Chapter Autoleveling")   }

pub struct AutolevelMod;
impl ConfigBasicMenuItemSwitchMethods for AutolevelMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool("G_DVC_Autolevel") };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().autolevel = result; }
            else { GameVariableManager::set_bool("G_DVC_Autolevel",result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool("G_DVC_Autolevel") };
        this.help_text = if value { "Units/enemies will be scaled to army's power." }
            else { "No changes to recruited/enemy unit's stats and levels." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if GameUserData::get_sequence() == 0 {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool("G_DVC_Autolevel") };
        this.command_text = if value { "Autoscale" } else { "No Scaling" }.into();
    }
}

pub fn auto_level_build(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if GameVariableManager::get_number("G_Continuous") == 3 { return BasicMenuItemAttribute::Hide }
    else { crate::menus::build_attribute_not_in_map(this, None) }
}
pub extern "C" fn vibe_autolevel() -> &'static mut ConfigBasicMenuItem { 
    let switch = ConfigBasicMenuItem::new_switch::<AutolevelMod>("Level Scale Units");
    switch.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = auto_level_build as _);
    switch
} 

pub struct EnemyEmblemGauge;
impl ConfigBasicMenuItemGaugeMethods  for EnemyEmblemGauge {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_emblem_rate as f32 / 100.0 }
            else { GameVariableManager::get_number("G_EnemyEmblemGauge") as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_emblem_rate  as f32 / 100.0  }
            else { GameVariableManager::get_number("G_EnemyEmblemGauge") as f32 / 100.0 };
        let result = ConfigBasicMenuItem::change_key_value_f(value, 0.0, 1.0, 0.25);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().enemy_emblem_rate = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_EnemyEmblemGauge", ( result * 100.0 ) as i32); }
            this.gauge_ratio = result;
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Chance of enemy units equipped with a dark emblem.".into();
    }
}

pub struct EnemyRevivalStones;
impl ConfigBasicMenuItemGaugeMethods  for EnemyRevivalStones {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().revival_stone_rate as f32 / 100.0 }
            else { GameVariableManager::get_number("G_EnemyRevivalStone") as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().revival_stone_rate as f32 / 100.0  }
            else { GameVariableManager::get_number("G_EnemyRevivalStone") as f32 / 100.0 };
        let result = ConfigBasicMenuItem::change_key_value_f(value, 0.0, 1.0, 0.25);
        if value != result {
            if GameUserData::get_sequence() == 0 { CONFIG.lock().unwrap().revival_stone_rate = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number("G_EnemyRevivalStone", ( result * 100.0 ) as i32); }
            this.gauge_ratio = result;
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = "Chance of enemy units gaining a revival stone.".into();
    }
}

pub extern "C" fn vibe_enemy_emblem() -> &'static mut ConfigBasicMenuItem { 
    let enemy_emblem = ConfigBasicMenuItem::new_gauge::<EnemyEmblemGauge>("Enemy Emblem Rate"); 
    enemy_emblem.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map as _);
    enemy_emblem
}
pub extern "C" fn vibe_enemy_stones() -> &'static mut ConfigBasicMenuItem { 
    let enemy_stones = ConfigBasicMenuItem::new_gauge::<EnemyRevivalStones>("Enemy Revival Stone Rate"); 
    enemy_stones.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::build_attribute_not_in_map as _);
    enemy_stones
}