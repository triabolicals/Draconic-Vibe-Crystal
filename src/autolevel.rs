use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    force::*,
    mess::*,
    gamedata::{unit::*, dispos::ChapterData, *},
};
use engage::gamedata::dispos::DisposData;
use super::CONFIG;
use crate::utils::*;

pub const EMBLEMS: &[&str] = &[ "GID_M010_敵リン", "GID_M007_敵ルキナ", "GID_M014_敵ベレト", "GID_M024_敵マルス", "GID_M017_敵シグルド", "GID_M017_敵セリカ", "GID_M019_敵ミカヤ", "GID_M019_敵ロイ", "GID_M017_敵リーフ", "GID_E006_敵エーデルガルト", "GID_E006_敵クロム", "GID_E006_敵カミラ", "GID_E006_敵セネリオ", "GID_E006_敵ヴェロニカ", "GID_E006_敵ヘクトル", "GID_E006_敵チキ"];
pub const ENGAGE: &[&str] = &[ "AI_AT_EngageAttack", "AI_AT_EngageAttack", "AI_AT_EngageDance", "AI_AT_EngageAttack", "AI_AT_EngagePierce", "AI_AT_EngageAttack", "AI_AT_AttackToHeal", "AI_AT_EngageAttack", "AI_AT_EngageAttackNoGuard", "AI_AT_EngageClassPresident", "AI_AT_EngageAttack", "AI_AT_EngageCamilla", "AI_AT_EngageAttack", "AI_AT_EngageSummon", "AI_AT_EngageWait", "AI_AT_EngageBlessPerson"];
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

pub fn is_boss(this: &PersonData) -> bool { unsafe { !is_null_empty(this.get_combat_bgm(),None) }  } 
pub fn calculate_player_cap() -> i32 {
    let mut max_cap: [i32; 10] = [0; 10];
    let mut unit_name: [&Il2CppString; 10] = [" N/A".into(); 10];
    GameVariableManager::make_entry_norewind("G_Player_Rating_Average", 0);
    for force in 0..max_cap.len() {
        let force_type: [ForceType; 4] = [ForceType::Player, ForceType::Absent, ForceType::Dead, ForceType::Lost];
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            let i: usize = force.into();
            for unit in force_iter {
                if unit.person.get_asset_force() != 0 { continue; }
                let cur = unit_cap_total(unit, true);
                if force == 0 {
                    if max_cap[i] < cur {
                        max_cap[i] = cur;
                        unit_name[i] = unit.person.name; 
                    }
                }
                else {
                    if max_cap[i] < cur && cur < max_cap[i-1] {
                        max_cap[i] = cur;
                        unit_name[i] = unit.person.name; 
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
        println!("Rank {}: {}/{} with rating {}", i+1, Mess::get(unit_name[i]).get_string().unwrap(), unit_name[i].get_string().unwrap(), max_cap[i]);
    }
    println!("{} unit Average is {}", count_average, average);
    GameVariableManager::set_number("G_Player_Rating_Average", average);
    average
}

pub fn unit_cap_total(this: &Unit, with_hp: bool) -> i32 {
    let mut total = 0;
    if with_hp {
        if this.get_capability(CapabilityType::Str.value(), false) < this.get_capability(CapabilityType::Mag.value(), false) {
            total += 2*this.get_capability(CapabilityType::Mag.value(), false);
        }
        else { total += 2*this.get_capability(CapabilityType::Str.value(), false); }
        total += this.get_capability(CapabilityType::Hp.value(), false);
        total += this.get_capability(CapabilityType::Dex.value(), false);
        total += this.get_capability(CapabilityType::Luck.value(), false);
        total += this.get_capability(CapabilityType::Def.value(), false);
        total += this.get_capability(CapabilityType::Res.value(), false);
        total += this.get_capability(CapabilityType::Spd.value(), false);
        total += 2*this.get_capability(CapabilityType::Sight.value(), false);
        total += 2*this.get_capability(CapabilityType::Move.value(), false);
        total += 2*this.get_capability(CapabilityType::Build.value(), false);
    }
    else { for x in 1..8 { total = total + this.get_capability(x, false); } }
    total
}

pub fn promote_unit(this: &Unit, level: i32){
        // if class is already promoted or class is base but does not have promotions
    let job = this.get_job(); 
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
        return; 
    }
    let job_max_level = job.get_max_level() as i32;
    if job_max_level < level {
        if job.is_low() {
            let job_jid = job.jid.get_string().unwrap();
            let mut high_job_index: usize = 0;
            if job_jid == "JID_ソードペガサス" || job_jid == "JID_ランスペガサス" || job_jid == "JID_アクスペガサス" {
                high_job_index = 1;
            } 
            let new_job = &job.get_high_jobs()[high_job_index];
            this.class_change(new_job);
            this.set_level(level-job_max_level);
            this.set_weapon_mask_from_person();
            this.set_internal_level(job_max_level);
            if this.person.get_unit_icon_id().get_string().unwrap() == "702MorphLC" {
                this.person.set_unit_icon_id("702Morph".into())
            }
        }
    }
}

pub fn get_number_main_chapters_completed() -> i32 {
    let mut number = 0;
    let chapters = ChapterData::get_list_mut().expect(":D");
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
    unsafe{
        let diff = GameUserData::get_difficulty(false);
        let player = unit.person.get_asset_force() == 0;
        let mut level = calculate_average_level(get_sortie_unit_count(None)) + diff;
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
        let floor_cap;
        if player { floor_cap = player_cap;  }
        else { floor_cap = player_cap + diff*( get_number_main_chapters_completed() - 10 ); }
        while enemy_cap < floor_cap && count < 30 {
            unit.level_up(4);
            enemy_cap = unit_cap_total(unit, true);
            unit.level = unit_level;
            count += 1;
        }
        if starting_cap != enemy_cap { 
            println!("{} {} gain {} stat points to {} ( {} Ups )", Mess::get(unit.get_job().name).get_string().unwrap(), Mess::get(unit.person.get_name().unwrap()).get_string().unwrap(), enemy_cap-starting_cap, enemy_cap, count);
        }
        unit.set_hp(unit.get_capability(0, true));
        return;
    }
}

pub fn calculate_average_level(sortie_count: i32) -> i32 {
    let mut used: [bool; 64] = [false; 64];
    let mut index;
    let vander_replace = GameVariableManager::get_string("G_R_PID_ヴァンドレ").get_string().unwrap();
    let mut sum = 0;
    let count;
    if sortie_count == 0 { count = 10; }
    else { count = sortie_count; }
    for _x in 0..sortie_count {
        let force_type: [ForceType; 2] = [ForceType::Player, ForceType::Absent];
        index = 0;
        let mut level = 0;
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            for unit in force_iter {
                if unit.person.pid.get_string().unwrap() == vander_replace {
                    used[index as usize] = true;
                }
                let total_level = unit.level as i32 + unit.internal_level as i32; 
                if !used[index as usize] {
                    if level < total_level { level = total_level; }
                }
                index += 1;
            }
        }
        sum += level;
    }

    let average = sum / count;
    println!("Player Average Level: {}", average);
    average
}

pub struct AutolevelMod;
impl ConfigBasicMenuItemSwitchMethods for AutolevelMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().autolevel);
        if CONFIG.lock().unwrap().autolevel != result {
            CONFIG.lock().unwrap().autolevel = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().autolevel { this.help_text = "Units/enemies will be scaled to army's power. (Togglable)".into(); }
        else { this.help_text = "No changes to recruited/enemy unit's stats and levels. (Togglable)".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().autolevel { this.command_text = "On".into();  }
        else { this.command_text = "Off".into(); }
    }
}


//Get Average Level of Party
#[skyline::from_offset(0x02b4afa0)]
pub fn get_average_level(difficulty: i32, sortie_count: i32, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x024f2c10)]
pub fn get_sortie_unit_count(method_info: OptionalMethod) -> i32;

pub fn str_start_with(this: &Il2CppString, value: &str) -> bool { unsafe { string_start_with(this, value.into(), None) } }

#[skyline::from_offset(0x023349c0)]
pub fn god_pool_create(data: &GodData, method_info: OptionalMethod) -> &'static GodUnit;

#[unity::from_offset("App", "Unit", "TryConnectGodUnit")]
pub fn unit_connect_god_unit(this: &Unit, god_unit: &GodUnit, method_info: OptionalMethod) -> &'static GodUnit;

pub fn try_equip_emblem(unit: &Unit, emblem: usize) -> bool {
    // triabolical config check
    println!("Attempting to equip emblems for enemies");

    unsafe {
        let jobname = unit.person.get_job().unwrap().name.get_string().unwrap();

        if emblem >= EMBLEMS.len() { return false; }
        if GodData::get(EMBLEMS[emblem].into()).is_none() { return false; }
        let job = unit.get_job();
        if job.name.get_string().unwrap() == "MJID_Emblem" || jobname == "MJID_Emblem" { return false; }
        if job.get_sort() == 9999 { return false;}
        //Prevents Wyrms/Wolves from getting emblems
        if jobname == "JID_異形飛竜" ||jobname  == "JID_幻影飛竜" {  return false; } //Wyverns
        if jobname  == "JID_異形竜" || jobname == "JID_幻影竜" {  return false; } //Wyrms
        if job.parent.index < 10 { return false; }
        if ( job.get_flag().value == 0 && job.jid.get_string().unwrap() != "JID_蛮族" ) || job.get_flag().value == 8 { return false; }
        let style_name = job.get_job_style();

        if style_name.is_some() {
                // Not Flying or Armored or wolf knight for Bow/Magic Emblems
            let god_data = GodData::get(EMBLEMS[emblem].into()).unwrap();

            if style_name.unwrap().get_string().unwrap() == "飛行スタイ ル" || style_name.unwrap().get_string().unwrap() == "重装スタイル" || job.jid.get_string().unwrap() == "JID_ウルフナイト" {
                match emblem {
                    0 | 1 | 5 | 6 | 11 | 12 | 13 => { return false; }
                    _ => { 
                        let god_unit = god_pool_create(god_data, None);
                        unit_connect_god_unit(unit, god_unit, None);
                        return true;
                    }
                }
            }
            else {
                let god_unit = god_pool_create(god_data, None);
                unit_connect_god_unit(unit, god_unit, None);
                return true;
            }
        }
    }
    return false; 
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