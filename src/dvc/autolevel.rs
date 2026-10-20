use super::*;
use engage::{
    app::{
        CommonRewardSequence, ICapabilityBase_1Methods, IProcInstMethods, 
        Proc, ProcBoolMethod, ProcInst, ProcVoidMethod, UnitRecord_Kinds
    },
    ForceExt,
    prelude::{Dictionary_2, List_1},
    system::collections::generic::{IDictionary_2, IDictionary_2Methods, InsertionBehavior}
};
use crate::{
    utils::clamp_value,
    dvc::{
        unit::{
            *,
            grow::{adjust_enemy_level, get_original_total_level, get_scaling_level},
            item::adjust_unit_items
        }
    },
    utils::to_opt,
    DVCVariable::UnitDeployment
};
use unity::{Array, IlNull};

pub fn adjust_player_unit_level_scale(unit: Unit, target_level: i32) {
    if unit.get_job().is_high() {
        if target_level <= 20 {
            unit.set_level(1);
            unit.set_internal_level(target_level - 1);
        } else {
            unit.set_level(target_level - 20);
            unit.set_internal_level(20);
        }
    }
    else {
        let max_lvl = unit.get_job().get_max_level() as i32;
        if target_level > max_lvl {
            unit.set_level(max_lvl);
            unit.set_internal_level(max_lvl - target_level);
        }
        else {
            unit.set_level(target_level);
            unit.set_m_internal_level(0);
        }
    }
}
pub fn adjust_player_level_recruitment(unit: Unit, lvl: i32, il: i32) {
    if unit.get_job().is_high() {
        unit.set_level(lvl);
        unit.set_internal_level(il);
    }
    else {
        let max_lvl = unit.get_job().get_max_level() as i32;
        let total = lvl + il;
        if total <= max_lvl {
            unit.set_level(total);
            unit.set_internal_level(0);
        }
        else {
            unit.set_level(max_lvl);
            unit.set_internal_level(max_lvl - total);
        }
    }
}
pub fn try_change_unit_tier_for_scale(unit: Unit) {
    if let Some(level) = get_scaling_level() {
        let job = unit.get_person().get_job();
        let current_job = unit.get_job();
        if job.is_high() && level <= 20 {
            if !unit.has_private_skill("SID_リーダー") && current_job.is_high(){
                if let Some(base) = JobDataBase::get().get_reclass_job(unit, current_job, ClassTier::Base) {
                    unit.set_job(base);
                    unit.update_weapon_mask();
                    unit.set_weapon_mask_from_parson();
                    assign_selected_weapon_mask_by_apt(unit, None);
                    adjust_unit_items(unit);
                    unit::item::adjust_missing_weapons(unit);
                }
            }
        }
        else if current_job.is_low() && level >= 20 && current_job.has_high_jobs() {
            if let Some(promoted) = JobDataBase::get().get_reclass_job(unit, current_job, ClassTier::Promoted) {
                unit.set_job(promoted);
                unit.update_weapon_mask();
                unit.set_weapon_mask_from_parson();
                assign_selected_weapon_mask_by_apt(unit, None);
                adjust_unit_items(unit);
                unit::item::adjust_missing_weapons(unit);
            }
        }
    }
}

pub fn try_autoscale_unit(unit: Unit) {
    let player = unit.get_person().get_asset_force().value == 0 || PersonPool::is_playable(unit.get_person());
    let chapter = DVCVariables::get_chapter_index();
    let leader = unit.has_private_skill("SID_リーダー") && !player;
    let emblem_paralogue = chapter >= 33 && chapter < 45 && !player;
    if let Some(scaling_level) = get_scaling_level() {
        if player {
            grow::adjust_recruitment_stats(unit, scaling_level - 1).into_iter().enumerate().for_each(|i|{ unit.set_base_capability(i.0 as i32, i.1); });
            adjust_player_unit_level_scale(unit, scaling_level - 1);
        }
        else {
            let target_level =
                if leader { 3 }
                else if !emblem_paralogue && chapter < 16 && to_opt(unit.get_person().get_job()).is_some_and(|v| v.is_high()) && scaling_level < 21 { 2 }
                else { 0 } + scaling_level;

            let adjustment = get_level_adjustment(unit);
            set_enemy_adjustment(unit, adjustment);
            grow::enemy_unit_stat_scaling(unit, target_level, Some(adjustment));
            adjust_enemy_level(unit, target_level);
        }
    }
    else if player && unit.check_status(Unit_Status::join_here()) {
        let person = PersonPool::get_replacement_2(unit.get_person(), true).unwrap_or(unit.get_person());
        let mut il = person.get_internal_level() as i32;
        if il == 0 { il = person.get_job().get_internal_level() as i32; }
        let level = grow::get_playable_target_level(unit);
        grow::adjust_recruitment_stats(unit, level).into_iter().enumerate().for_each(|i|{ unit.set_base_capability(i.0 as i32, i.1); });
        adjust_player_level_recruitment(unit, person.get_level() as i32, il);
    }
    else if emblem_paralogue && DVCVariable::EmblemRecruitment.get_value() != 0 {
        if let Some(replaced) = EmblemPool::get_emblem_by_paralogue_index(chapter).and_then(|v| EmblemPool::get_replacement_index_2(v, true)){
            grow::emblem_paralogue_stat_adjustment(unit, replaced, DVCVariables::user_data().get_difficulty(false).value);
        }
        else { grow::enemy_unit_stat_scaling(unit, get_original_total_level(unit), None); }
        adjust_enemy_level(unit, get_original_total_level(unit));
    }
    else {
        if !player {
            grow::enemy_unit_stat_scaling(unit, get_original_total_level(unit), None);
            adjust_enemy_level(unit, get_original_total_level(unit));
        }
        else {
            let level = grow::get_playable_target_level(unit);
            unit.auto_grow_capability(unit.get_level(), level);
        }
    }
    unit.set_hp(unit.get_capability(0, true));
    unit.update_weapon_mask();
    learn_skill::update_learn_skill(unit);
}
fn get_level_adjustment(unit: Unit) -> i32 {
    let user_data = DVCVariables::user_data();
    let random_map = DVCVariable::Continuous.get_value() == 4 || DVCVariable::Continuous.get_value() == 2;
    let chapter = DVCVariables::get_chapter_index();
    let diff = user_data.get_difficulty(false).value;
    let main_completed = DVCVariables::chapter_number_complete(false);
    let m022 = DVCVariables::is_main_chapter_complete(22);
    if DVCFlags::Autolevel.get_value() {
        if diff == 0 { 0 }
        else if m022 && diff > 0 { 2 * diff - 1 }
        else if chapter < 17 && chapter > 11 { diff - 1 }
        else { unit.get_person().get_auto_grow_offset() }
    }
    else if random_map {
        let m011 = DVCVariables::is_main_chapter_complete(11);
        if diff == 0 { -4 }
        else if diff == 1 { if m022 { 1 } else { 0 } }
        else { 2 * (m022 as i32) + m011 as i32 + (main_completed > 17) as i32 }
    }
    else { unit.get_person().get_auto_grow_offset() }
}
pub fn get_difficulty_adjusted_average_level() -> i32 {
    let diff = DVCVariables::user_data().get_difficulty(false).value;
    let mut il_count = 0;
    let mut levels: Vec<(i32, i32)> = vec![];
    Force::get(Force_Type::absent()).iter().chain(Force::get(Force_Type::player()).iter())
        .for_each(|unit|{
            let l = unit.get_level();
            let il = unit.get_internal_level();
            if il > 10 { il_count += 1; }
            levels.push((l, il));
        });
    let mut levels =
        if il_count < 3 { levels.iter().filter(|v| v.1 < 10).map(|v| v.0).collect::<Vec<i32>>() }
        else { levels.iter().map(|v| v.0 + v.1).collect::<Vec<i32>>() };
    levels.sort_by(|a, b| b.cmp(a));
    let count = 12 - 2*diff;
    let mut i = 0;
    let mut total = 0;
    let mut iter = levels.iter();
    while let Some(&level) = iter.next() {
        i += 1;
        total += level;
        if i >= count { break;}
    }
    total / count
}
pub fn auto_level_units_bind(proc: impl Into<ProcInst>, background: bool){
    let exp_list = get_exp_list(false);
    if exp_list.count() == 0 { return; }
    let reward = CommonRewardSequence::new(exp_list, List_1::<ItemData>::null(), 0 , background, false);
    let descs = [
        Proc::call_2(ProcVoidMethod::new(reward.into(), CommonRewardSequence::load_res_method_info().into())),
        Proc::wait_while_true_2(ProcBoolMethod::new(reward.into(), CommonRewardSequence::is_loading_res_method_info().into())),
        Proc::label(0), // Exp
        Proc::call_2(ProcVoidMethod::new(reward.into(), CommonRewardSequence::open_reward_exp_method_info().into())),
        Proc::label(1),
        Proc::call_2(ProcVoidMethod::new(reward.into(), CommonRewardSequence::check_level_up_method_info().into())),
        Proc::label(2),
        Proc::call_2(ProcVoidMethod::new(reward.into(), CommonRewardSequence::close_method_info().into())),
        Proc::end(),
    ];
    let descs = Array::from_slice(&descs).unwrap();
    reward.create_bind(proc.into(), descs, Il2CppString::null());
}

pub fn get_exp_list(continuous_mode: bool) -> Dictionary_2<Unit, i32> {
    let user_data = DVCVariables::user_data();
    let diff = user_data.get_difficulty(false).value;
    let base_exp_gain = 30 + 10*(2 - diff);
    let mut total_level = 0;
    let count = Force::get(Force_Type::player()).iter().filter(|x|!x.is_vision() && !x.is_summon()).count();
    let dictionary = Dictionary_2::<Unit, i32>::new();
    let average;
    if continuous_mode {
        Force::get(Force_Type::player()).iter().filter(|x|!x.is_vision() && !x.is_summon())
            .for_each(|unit|{
                let exp = if unit.get_level() == unit.get_job().get_max_level() as i32 { 0 }
                else { base_exp_gain + unit.get_record(UnitRecord_Kinds::map_kill_count()) };
                let exp = clamp_value(exp, 0, 100);
                total_level += unit.get_level() + unit.get_internal_level();
                unit.add_skill_point(100);
                dictionary.try_insert(unit, exp, InsertionBehavior::overwrite_existing());
            });
        average = total_level / (count as i32 + 1);
    }
    else {
        average = get_difficulty_adjusted_average_level() - 2 * DVCVariables::user_data().get_difficulty(false).value;
    }
    let auto_level = DVCFlags::PostChapterAutolevel.get_value();
    Force::get(Force_Type::absent()).iter()
        .for_each(|unit|{
            let job_max = unit.get_job().get_max_level() as i32;
            let level = unit.get_level();
            let exp =
                if level == job_max { 0 }
                else {
                    let total_level = unit.get_level() + unit.get_internal_level();
                    if auto_level {
                        let mut n_levels = average - total_level;
                        if n_levels < 1 { 0 }
                        else {
                            if (level + n_levels) > job_max { n_levels = job_max - level; }
                            100 - (unit.m_exp() as i32) + (n_levels - 1) * 50
                        }
                    }
                    else {
                        if total_level >= average { base_exp_gain / 3 }
                        else { base_exp_gain + (average - total_level) * 5 }
                    }
                };
            if exp > 0 {
                unit.add_skill_point(exp / 2);
                dictionary.try_insert(unit, exp, InsertionBehavior::overwrite_existing());
            }
        });
    dictionary
}