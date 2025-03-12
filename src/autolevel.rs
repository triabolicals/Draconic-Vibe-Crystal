use unity::prelude::*;
use engage::{
    force::*, gamedata::{ai::UnitAI, dispos::*, unit::*, *}, 
    gameuserdata::*, 
    gamevariable::*, 
    map::situation::MapSituation, 
    menu::{config::*, BasicMenuItemAttribute, BasicMenuResult}, 
    mess::*, 
    util::get_instance
};
use super::CONFIG;
use crate::{randomizer::emblem::ENEMY_EMBLEM_LIST, utils::*, DVCVariables};

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
pub struct BenchAutoLevelOption;
impl ConfigBasicMenuItemSwitchMethods for BenchAutoLevelOption {
    fn init_content(_this: &mut ConfigBasicMenuItem)    {
        GameVariableManager::make_entry(DVCVariables::AUTOLEVEL_BENCH_KEY, 0); 
    } 
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle =  GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY);
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);
        if toggle != result {
            GameVariableManager::set_bool(DVCVariables::AUTOLEVEL_BENCH_KEY, result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else { return BasicMenuResult::new();  }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.help_text = 
            if !GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY) {"Undeployed will not be autoleveled at the end of the chapter." }
            else { "Undeployed units will autolevel to difficulty-adjusted average." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        this.command_text = 
            if !GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY) { "Disabled" }
            else { "Enabled" }.into();
    }
}
pub extern "C" fn autobench() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<BenchAutoLevelOption>("Post Chapter Autoleveling")   }

pub struct AutolevelMod;
impl ConfigBasicMenuItemSwitchMethods for AutolevelMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        let result = ConfigBasicMenuItem::change_key_value_b(value);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().autolevel = result; }
            else { GameVariableManager::set_bool(DVCVariables::DVC_AUTOLEVEL_KEY,result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        this.help_text = if value { "Units/enemies will be scaled to army's power." }
            else { "No changes to recruited/enemy unit's stats and levels." }.into();
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() {  CONFIG.lock().unwrap().autolevel }
            else { GameVariableManager::get_bool(DVCVariables::DVC_AUTOLEVEL_KEY) };
        this.command_text = if value { "Autoscale" } else { "No Scaling" }.into();
    }
}

pub fn auto_level_build(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if DVCVariables::is_random_map() { return BasicMenuItemAttribute::Hide }
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
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate as f32 / 100.0 }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) } / 10;
        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 10, 1);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate =result * 10; }
            else { GameVariableManager::set_number(DVCVariables::ENEMY_EMBLEM_KEY,  result * 10 ); }
            this.gauge_ratio = result as f32 / 10.0;
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        return BasicMenuResult::new(); 
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().enemy_emblem_rate }
            else { GameVariableManager::get_number(DVCVariables::ENEMY_EMBLEM_KEY) };

        this.help_text = format!("{}% chance of enemy units equipped with an emblem.", value).into();
    }
}

pub struct EnemyRevivalStones;
impl ConfigBasicMenuItemGaugeMethods  for EnemyRevivalStones {
    fn init_content(this: &mut ConfigBasicMenuItem){
        this.gauge_ratio = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate as f32 / 100.0 }
            else { GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY) as f32 / 100.0  };
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let value = if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate as f32 / 100.0  }
            else { GameVariableManager::get_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY) as f32 / 100.0 };
        let result = ConfigBasicMenuItem::change_key_value_f(value, 0.0, 1.0, 0.10);
        if value != result {
            if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().revival_stone_rate = ( result * 100.0 ) as i32; }
            else { GameVariableManager::set_number(DVCVariables::REVIVAL_STONE_GAUGE_KEY, ( result * 100.0 ) as i32); }
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

pub fn is_boss(this: &PersonData) -> bool { this.get_combat_bgm().is_some() }
pub fn calculate_player_cap() -> i32 {
    let mut max_cap: [i32; 10] = [0; 10];
    for force in 0..max_cap.len() {
        let force_type: [ForceType; 3] = [ForceType::Player, ForceType::Absent, ForceType::Ally];
        for ff in force_type {
            let force_iter = Force::iter(Force::get(ff).unwrap());
            let i: usize = force.into();
            for unit in force_iter {
                if unit.person.get_asset_force() != 0 { continue; }
                let cur = unit_cap_total(unit, true);
                if force == 0 {
                    if max_cap[i] < cur {
                        max_cap[i] = cur;
                    }
                }
                else {
                    if max_cap[i] < cur && cur < max_cap[i-1] {
                        max_cap[i] = cur;
                    }
                }
            }
        }
    }   
    let mut average: i32 = 0;
    let diff = GameUserData::get_difficulty(false);
    let count_average: usize = max_cap.len() - (2*diff as usize);
    for i in 0..count_average { average += max_cap[i];  }
    average = average / ( count_average as i32 );
    println!("{} unit Average is {}", count_average, average);
    if average > GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP) {
        GameVariableManager::set_number(DVCVariables::PLAYER_AVERAGE_CAP, average);
    }
    average
}



pub fn update_learn_skills(forced: bool) {
    println!("LearnSkillKey: {}", GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY));
    if forced || !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().learn_skill {
        let force_type = [ForceType::Player, ForceType::Absent, ForceType::Dead, ForceType::Lost, ForceType::Enemy, ForceType::Ally];
        for ff in force_type { Force::get(ff).unwrap().iter().for_each(|unit| unit_update_learn_skill(unit) ); }
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.learn_skill = true);
        return;
    }
}

pub fn unit_update_learn_skill(unit: &Unit) { 
    if unit.learned_job_skill.is_some() && unit.job.learn_skill.is_some() {
        unit.set_learn_skill(None);
        crate::randomizer::skill::learn::unit_learn_job_skill_hook(unit, unit.job, None);
        //if let Some(skill) = crate::randomizer::skill::learn::unit_learn_job_skill_hook(unit, unit.job, None) {
        //    println!("{} Learned {}", Mess::get_name(unit.person.pid), Mess::get(skill.name.unwrap()));
       // }
    }
    else { unit.try_learn_job_skill(); }
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
        if let Some(icon) = this.person.get_unit_icon_id() {
            if icon.to_string() == "702MorphLC" {
                this.person.set_unit_icon_id("702Morph".into());
            }
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
    if !DVCVariables::is_main_chapter_complete(6) { return; } 
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
    if GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP) == 0 { calculate_player_cap(); }
    let player_cap = GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP);
    
    let mut enemy_cap = unit_cap_total(unit, true);
    let mut count = 0;
    let unit_level = unit.level;
    let floor_cap = if player {player_cap }
        else { player_cap + diff*( get_number_main_chapters_completed() - 10 ) };
    while enemy_cap < floor_cap && count < 30 {
        unit.level_up(2);
        enemy_cap = unit_cap_total(unit, true);
        unit.level = unit_level;
        count += 1;
    }
    unit.set_hp(unit.get_capability(0, true));
    return;
}

pub fn auto_level_unit_for_random_map(unit: &mut Unit, leader: bool){
    if !GameVariableManager::get_bool( "G_Cleared_M004") || GameUserData::is_evil_map() { return; } 
    let diff = GameUserData::get_difficulty(false);
    let player = unit.person.get_asset_force() == 0;
    if player { 
        crate::randomizer::person::unit::random_map_unit_level(unit);
        return;
    }
    let mut level =  crate::continuous::random::random_map_mode_level();
    let map_sit = get_instance::<MapSituation>().average_level;
    if map_sit != 0 { level = ( level + map_sit ) / 2; }

    if leader || is_boss(unit.person) { level += 3; }
    if player { level -= diff; }
    let current_level = unit.level as i32 + unit.internal_level as i32;
    for _x in 0..(level - current_level) { unit.level_up(3); }
    if current_level < level { promote_unit(unit, level); }
    else if level < current_level { demote_unit(unit, level); }

    let offset = unit.person.get_offset_by_difficulty();
    let maps = crate::continuous::get_continious_total_map_complete_count();

    unit.auto_grow_capability(unit.level as i32, level + diff + maps / 4 );

    if unit.job.max_level == 20 {
        let div = level / 20;
        unit.set_internal_level(div*20);
        if div > 0 { 
            unit.set_level( crate::utils::max(1, level - div*20));  
        }
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

    if GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP) == 0 { calculate_player_cap(); }
    let player_cap = GameVariableManager::get_number(DVCVariables::PLAYER_AVERAGE_CAP);

    let unit_level = unit.level;

    let floor_cap = if player { player_cap }
        else { player_cap + diff * ( maps - 10 )  } - 5;

    let ceil = floor_cap + maps;
    while ceil <  unit_cap_total(unit, true) {
        unit.level = unit_level + 1;
        unit.level_down();
    } 
    unit.set_hp(unit.get_capability(0, true));
    if unit.internal_level < 0 { unit.set_internal_level(level - unit.level as i32); }
    return;
}

pub fn calculate_average_level(sortie_count: i32) -> i32 {
    let vander_replace = if GameVariableManager::exist("G_R_PID_ヴァンドレ") {
        GameVariableManager::get_string("G_R_PID_ヴァンドレ").to_string()
    } 
    else { "PID_ヴァンドレ".to_string() };
    let count = if sortie_count == 0 { 10 } else { sortie_count };
    let mut collection: Vec<i32> = Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Absent).unwrap().iter())
        .filter(|unit| unit.person.pid.to_string() != vander_replace)
        .map(|unit| unit.level as i32 + unit.internal_level as i32).collect();
    collection.sort_by(|a, b| b.cmp(a));

    let length = collection.len() as i32;
    let sum_count = if length < count { length } else { count };
    let mut sum = 0;
    for x in 0..sum_count { sum += collection[x as usize]; }
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

pub fn try_equip_emblem(unit: &Unit, emblem: usize) -> bool {
    if !GameVariableManager::exist("EnemyEmblemSet") { GameVariableManager::make_entry_norewind("EnemyEmblemSet", 0); }
    let mut emblem_set_flag = GameVariableManager::get_number("EnemyEmblemSet");
    if unit.person.gender == 0 { return false; }
    if emblem < 31 { if emblem_set_flag & (1 << emblem) != 0 { return false; }  }

    if let Some(god) = GodData::try_index_get(ENEMY_EMBLEM_LIST.get().unwrap()[emblem]) {
        if let Some(god_unit) = engage::godpool::GodPool::create(god) {
            let valid = unit.try_connect_god(god_unit).is_some();
            god_unit.set_escape(true);
            if valid && emblem < 31 {
                emblem_set_flag |= 1 << emblem;
                GameVariableManager::set_number("EnemyEmblemSet", emblem_set_flag);
            }
            return valid;
        }
    }
    return false;
}

#[skyline::from_offset(0x01a522a0)]
pub fn get_unit_ai(this: &Unit, method_info: OptionalMethod) -> &'static UnitAI;

pub fn adjust_emblem_unit_ai(unit: &Unit){
    let ai = &unit.ai;
    ai.set_sequence(2, "AI_AT_Versus");
    ai.set_flag(31);
    ai.set_flag(0x400);
    ai.set_flag(0x800);
}

pub fn autolevel_party(){
    if GameVariableManager::get_bool(DVCVariables::AUTOLEVEL_BENCH_KEY){
        let bench = Force::get(ForceType::Absent).unwrap();
        let avg_number = 8 + 2*GameUserData::get_difficulty(false);
        let player_average = calculate_average_level(avg_number) - GameUserData::get_difficulty(false);
        println!("Autoleveling Bench to average of {}", player_average);
        let mut force_iter = Force::iter(bench);
        while let Some(unit) = force_iter.next() {
            let total_level: i32 = unit.level as i32 + unit.internal_level as i32;
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
    if let Some(skill) = unit.try_learn_job_skill() { unit.set_learn_skill(Some(skill)); }
}