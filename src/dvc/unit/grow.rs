use super::*;
use engage::app::{CapabilityDefinition, Difficulty, ICapabilityBase_1Methods, IGameUserDataMethods, UnitUtil};
use crate::{
    gamedata::ClassTier,
    dvc::randomizer::map::dispos::PARALOGUE_LEVELS,
    DVCVariable::UnitRecruitment
};

const VANDER_MAX: [i8; 11] = [45, 12, 14, 11, 40, 12, 13, 12, 10, 5, 7];
const EMBLEM_AUTO: [i8; 36] = [
    -3, 0, 4,
    -3, -1, 1,
    -4, 0, 2,
    -3, 0, 2,
    -3, 2, 2,
    -3, 2, 2,
    -1, 0, 0,
    -2, 0, 0,
    -3, 0, 0,
    -6, -5, 2,
    -5, 0, 2,
    -5, 3, 2,
];
fn calc_max_recruit_stat(total_level: i32) -> [i8; 11] {
    let mut max: [i32; 11] = [0; 11];
    max[0] = 125 * total_level + 2600;
    max[8] = 20 * total_level + 1000;
    for x in [1, 2, 3, 6] { max[x] = 75*total_level + 1000; }
    for x in [5, 7] { max[x] = 60*total_level + 1100; }
    max[4] = 10000;
    max[9] = 1000;
    max[10] = 2000;
    let mut out = [0; 11];
    for x in 0..11 { out[x] = (max[x] / 100) as i8; }
    out
}
pub fn emblem_paralogue_stat_adjustment(unit: Unit, emblem_replace: i32, diff: i32){
    let person = unit.get_person();
    let hash = person.hash();
    let emblem = !unit.m_private_skill().find("SID_リーダー").is_null();
    let pid = person.get_pid().to_rust_string();
    let person_job = person.get_job().get_prefixless_jid().to_rust_string();
    if emblem_replace < 12 {
        let emblem_index = emblem_replace as usize;
        let diff = diff as usize;
        let (level, adjustment) =
            if emblem { (PARALOGUE_LEVELS[4*emblem_index + 1], unit.get_person().get_auto_grow_offset()) }
            else if unit.get_ai().get_sequence(AIValue_Order::mind()).to_rust_string().contains("Treasure") {
                (PARALOGUE_LEVELS[4*emblem_index + 2] + 15, EMBLEM_AUTO[3*emblem_index + diff] as i32)
            }
            else {
                let level = get_original_total_level(unit);
                let target_level = clamp_value(PARALOGUE_LEVELS[4*emblem_index + 2] as i32 + (level - DVCVariable::EmblemParalogueGenericLevel.get_value()), 15, 45) as u8;
                (target_level, EMBLEM_AUTO[3*emblem_index + diff] as i32)
            };
        set_enemy_adjustment(unit, adjustment);
        enemy_unit_stat_scaling(unit, level as i32, Some(adjustment));
        adjust_enemy_level(unit, level as i32);
    }
    else if let Some(i) = EmblemPool::get_emblem_by_paralogue_index(DVCVariables::get_chapter_index()) {
        let avg = DVCVariable::AverageLevel.get_value();
        if DVCVariable::AverageLevel.get_value() >= PARALOGUE_LEVELS[4 * i as usize] as i32 {
            let original_level = get_original_total_level(unit);
            enemy_unit_stat_scaling(unit, get_original_total_level(unit), None);
            adjust_enemy_level(unit, original_level);
        }
        else {
            let level =
                if emblem { avg + 6 }
                else if unit.get_ai().get_sequence(AIValue_Order::mind()).to_rust_string().contains("Treasure") { avg - 5 }
                else if pid.ends_with(&person_job) { avg + 1 }
                else if hash == 1093315877 { avg + 10 }
                else { avg + 3 };

            enemy_unit_stat_scaling(unit, if level < 1 { 1 } else { level }, None);
            let displayed = if level < 1 { 1 } else if level > 40 { 40 } else { level };
            adjust_enemy_level(unit, displayed);
        }
    }
}
pub fn adjust_enemy_level(unit: Unit, target_level: i32) {
    let job = unit.get_job();
    let max_level = job.get_max_level() as i32;
    let (lv, il) =
    if job.is_high() { if target_level > 20 { (target_level - 20, 20) } else { (target_level, 0) } }
    else if target_level > max_level { (max_level, target_level - max_level) }
    else { (target_level, 0) };
    unit.set_level(lv);
    unit.set_internal_level(il);
}
pub fn get_original_total_level(unit: Unit) -> i32 {
    let (lv, il) = get_unit_lil(unit);
    if (lv + il) > 0 { return lv + il; }
    let person = unit.get_person();
    let level = person.get_level() as i32;
    let il =
        if person.get_internal_level() == 0 { person.get_job().get_internal_level() as i32 }
        else { person.get_internal_level() as i32 };
    level + il
}
pub fn enemy_player_character_replacement(this: Unit) {
    let person = this.get_person();
    if UnitRecruitment.get_value() != 0 {
        if let Some(enemy) = PersonPool::get_enemy_player_replacement(person, false){
            let edit = this.m_edit();
            if enemy.hash() == PersonPool::LUEUR {
                edit.set_m_gender(Gender { value: DVCVariable::LueurGender.get_value() });
                edit.set_m_name(GameVariableManager::get_string(DVCVariables::LUEUR_NAME));
            } else {
                let gender = enemy.get_gender();
                edit.set_m_gender(gender);
                if PersonPool::is_nil(person) { edit.set_m_name(Mess::get(person.get_name())); }
                else { edit.set_m_name(Mess::get(enemy.get_name())); }
            }
            let old_job = enemy.get_job();
            let current_job = this.get_job();
            let current_max_level = current_job.get_max_level();
            let old_max_level = old_job.get_max_level();
            let il = this.get_internal_level();
            let lvl = this.get_level();
            let total = il + lvl;
            let apt = enemy.get_aptitude().m_value() | enemy.get_sub_aptitude().m_value();
            this.get_original_aptitude().set_m_value(apt);
            this.m_aptitude().set_m_value(apt);
            let target_class =
                if current_job.get_rank() == old_job.get_rank() {
                    if current_max_level > old_max_level && old_job.has_high_jobs() && total > 20 { old_job.get_high_jobs().get(0) }
                    else { old_job }
                }
                else if current_job.is_high() && old_job.is_low() {
                    if old_max_level > 20 || !old_job.has_high_jobs() { old_job }
                    else { old_job.get_high_jobs().get(0) }
                }
                else if current_max_level > 20 { old_job }
                else { GameData::get().job_db.get_reclass_job(this, old_job, ClassTier::Base).unwrap_or(old_job) };

            this.set_job(target_class);
            assign_selected_weapon_mask_by_apt(this, None);
            adjust_unit_items(this);
        }
    }
}
pub fn test_enemy_scaling(person_data: PersonData, diff: i32) {
    DVCVariables::user_data().set_difficulty(Difficulty{value: diff});
    let unit = Unit::new(true);
    unit.set_person(person_data);
    unit.set_m_job(person_data.get_job());
    let target = person_data.get_level() as i32 + if person_data.get_job().is_high() { 20 } else { 0 };
    enemy_unit_stat_scaling(unit, target, None);
    println!("Stats for Level: {} {} in {}", target, unit.get_name(), Mess::get(unit.get_job().get_name()));
    for i in 0..9 { println!("{}: {}", CapabilityDefinition::get_name(i), unit.get_capability(i, false)); }
}
pub fn enemy_unit_stat_scaling(unit: Unit, target_level: i32, level_offset: Option<i32>) {
    let mut grow: [i32; 10] = [0; 10];
    let person = unit.get_person();
    let job = unit.get_job();
    let diff = DVCVariables::user_data().get_difficulty(false).value;
    let mut person_grow = person.get_grow();
    if UnitRecruitment.get_value() != 0 && DVCFlags::RandomEnable.get_value() {
        if let Some(enemy) = PersonPool::get().enemy_character
            .iter()
            .find(|v|v.hash == person.hash())
            .and_then(|v| v.get_enemy_replacement())
        {
            person_grow = enemy.get_grow();
        }
    }
    let no_personal_growths = ICapabilityBase_1Methods::is_zero(person_grow);
    if no_personal_growths {
        let job_base_grow = job.get_base_grow();
        if ICapabilityBase_1Methods::is_zero(job_base_grow) {
            let limit = job.get_limit();
            let base = job.get_base();
            grow[0] = 50;
            grow[8] = 20;
            for i in 1..8 { grow[i] = limit.get(i as i32) as i32 - base.get(i as i32) as i32; }
        }
        else { for i in 0..10 { grow[i] = job_base_grow.get_item(i as i32) as i32; } }
    }
    else {
        for i in 0..10 { grow[i] = person_grow.get(i as i32) as i32; }
    }
    if unit.get_person().get_asset_force().value == 1{
        if JobDataBase::get().get_by_hash(job.hash()).is_none_or(|v| !v.zero_diff_grow) || (job.hash() != person.get_job().hash()) {
            let diff_grow =
                match diff {
                    1 => job.get_diff_grow_hard(),
                    2 => job.get_diff_grow_lunatic(),
                    _ => job.get_diff_grow_normal()
                };
            for i in 0..10 { grow[i] += diff_grow.get(i as i32) as i32; }
        }
    }
    let level_off = level_offset.unwrap_or_else(|| person.get_auto_grow_offset()) + DVCVariable::EnemyCapabilityOffset.get_value();
    let offset = person.get_capability_offset();
    let mut n_level = target_level + level_off - 1;
    if n_level < 1 { n_level = 0; }
    for i in 0..10 {
        let v = UnitUtil::get_round_grow(n_level * grow[i]);
        let stat = v + offset.get_item(i as i32) as i32;
        let stat = clamp_value(stat, (i == 0) as i32, 127);
        unit.set_base_capability(i as i32, stat);
    }
}
pub fn get_scaling_level() -> Option<i32> {
    if !DVCVariables::is_main_chapter_complete(4) { None }
    else if DVCFlags::Autolevel.get_value() { Some(DVCVariable::AverageLevel.get_value()) }
    else {
        let con_mode = DVCVariable::Continuous.get_value();
        if con_mode == 2 || con_mode == 4 {
            let total = DVCVariables::chapter_number_complete(true);
            Some(if total < 5 { total } else if total < 16 { total + 3 } else { 20 + (total - 16) * 2 })
        }
        else { None }
    }
}
pub fn get_playable_target_level(unit: Unit) -> i32 {
    let unit_person = unit.get_person();
    let person = PersonPool::get_replacement_2(unit_person, true).unwrap_or(unit_person);
    let level = person.get_level() as i32;
    if person.get_job().is_high() { level + 20 } else { level }
}
pub fn adjust_recruitment_stats(unit: Unit, target_level: i32) -> [i8; 11]{
    let person = unit.get_person();
    let mut out: [i8; 11] = [0; 11];
    if PersonPool::get_replacement_2(person, true).is_none_or(|v| v.hash() == person.hash()) {
        let offset = person.get_capability_offset();
        if target_level <= 1 {
            for i in 0..11 { out[i] = clamp_value(offset.get(i as i32), 0, 127); }
        }
        else {
            let n_level = target_level - 1;
            let mut grow = [50, 45, 45, 45, 45, 35, 35, 20, 5, 0, 0];
            let pgrow = person.get_grow();
            if !ICapabilityBase_1Methods::is_zero(pgrow){
                for i in 0..11 { grow[i] = pgrow.get(i as i32) as i32; }
            }
            else {
                let job = person.get_job().get_base_grow();
                if !ICapabilityBase_1Methods::is_zero(job) { 
                    for i in 0..11 { grow[i] = job.get(i as i32) as i32; } 
                }
            }
            for i in 0..11 {
                let s =  UnitUtil::get_round_grow(n_level * grow[i]) + offset.get(i as i32) as i32;
                out[i] = clamp_value(s, (i == 0) as i32, 127) as i8;
            }
        }
        out
    }
    else {
        let is_vander = PersonPool::get_replacement_index_3(person, true).is_some_and(|v| v == 1);
        let max = if is_vander { VANDER_MAX } else { calc_max_recruit_stat(target_level) };
        let old_adjusted_offset = PersonPool::get_grow_offset(person);
        let job = unit.get_job();
        let job_base = job.get_base();
        let base_grow = person.get_grow();
        let new_job_grow = job.get_diff_grow();
        let n_level = if target_level <= 1 { 0 } else { target_level - 1 };
        for x in 0..9 {
            let job_base = job_base.get(x as i32) as i8;
            let growth = base_grow.get(x as i32) as i32 + new_job_grow.get(x as i32) as i32;
            let grow = UnitUtil::get_round_grow(n_level * growth);
            let stat = (old_adjusted_offset[x] as i32 + grow) as i8;
            let total = stat + job_base;
            let b = if total >= max[x] { max[x] - job_base } else { stat };
            out[x] = clamp_value(b, (x == 0) as i8, 127);
        }
        out
    }
}
