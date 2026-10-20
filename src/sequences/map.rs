use engage::{
    app::{
        godunit::*, mapsequence::*, singletonpool_2::*, mapsituation::*, 
        ScriptUnit, MapHistory, TelopManager
    },
    ForceExt,
    ProcVoidMethodExt,
    moon_sharp::interpreter::dynvalue::*,
};
use outfit_core::Randomizer;
use crate::{
    gamedata::*,
    dvc::{
        self,
        mess, unit::for_each_unit, autolevel::get_difficulty_adjusted_average_level, DVC, unit::grow::enemy_unit_stat_scaling
    }
};
use super::*;

pub fn map_sequence_desc_edit(proc: ProcInst) {
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast() {
        if DVCFlags::Ironman.get_value() { MapHistory::rewind_disable(); } else { MapHistory::rewind_enable(); }
        descs.set(4, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_setup_chapter).unwrap()));
        descs.set(15, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_setup_field_a).unwrap()));
        descs.set(19, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_dispos_event).unwrap()));
        descs.set(21, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_dispos_unit).unwrap()));
        descs.set(45, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_map_opening).unwrap()));
        descs.set(81, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_map_begins).unwrap()));
        descs.set(98, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_turn_effect).unwrap()));
        // 204 TryEnding
        descs.set(207, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_unload_script).unwrap()));
        descs.set(209, Proc::call_2(ProcVoidMethod::from_fn(proc, try_restart_map_result).unwrap()));
    }
}
extern "C" fn map_sequence_unload_script(map_sequence: MapSequence, _: OptionalMethod) {
    DVC::get().reset_set_jobs();
    dvc::continuous::post_map_emblem_adjustment();
    map_sequence.unload_script();
}
extern "C" fn map_sequence_setup_chapter(map_sequence: MapSequence, _: OptionalMethod) {
    let user_data = DVCVariables::user_data();
    let chapter = user_data.get_chapter();
    let status = user_data.m_status().m_value();
    if !DVCConfig::new().debug || DVCVariable::AverageLevel.get_value() == 0 {
        DVCVariable::AverageLevel.init_var(get_difficulty_adjusted_average_level(), true);
    }
    if chapter.get_hold_level() > 0 {
        let resume = map_sequence.get_is_resume();
        user_data.setup_for_chapter(resume);
        if GameVariableManager::get_bool(chapter.get_cleared_flag_name()) { user_data.m_status().set_m_value(status|0x3000); }
        else { user_data.m_status().set_m_value(status|0x1000); }
        if !resume {
            dvc::randomizer::map::setup_for_fx();
            let map_grow_mode = MapSequence::get_grow_mode(user_data.get_grow_mode());
            if map_grow_mode != user_data.get_grow_mode() { user_data.set_grow_mode(map_grow_mode); }
        }
    }
    else { map_sequence.setup_chapter(); }
    GameVariableManager::set_bool("禁止_経験値", false);
    EmblemPool::setup_enemy_dummies();
    dvc::randomizer::bgm::assign_random_bgm();
    if !chapter.is_null() {
        let mess = chapter.get_mess();
        if !mess.is_null() { mess::edit_msgfile_for_chapter(mess); }
        let cid = chapter.get_cid().to_rust_string();
        if cid.contains("G00") {
            GameVariableManager::set_bool("G_拠点_神竜導入イベント再生済み", true);
            if let Some(edelgard) = EmblemPool::get_replacement_god_3(12, false){
                GodPool::create(edelgard);
            }
        }
        if cid.contains("M003") { dvc::unit::item::change_liberation_type(); }
        dvc::continuous::pre_map_emblem_adjustment();
    }

}
extern "C" fn map_sequence_setup_field_a(map_sequence: MapSequence, _optional_method: OptionalMethod) {
    map_sequence.setup_field_a();
    dvc::script::inspectors::adjust_person_map_inspectors();
}
pub extern "C" fn map_sequence_dispos_event(this: MapSequence, _method_info: OptionalMethod) {
    this.dispos_event();
    if DVCVariable::UnitDeployment.get_value() == 4 && !DVCFlags::RandomDeploySpot.get_value() { dvc::deployment::load_extra_deployment_slots(); }
    if DVCFlags::RandomDeploySpot.get_value() { dvc::deployment::shuffle_deployment(); }
    dvc::randomizer::map::fow::init_fog();
    dvc::randomizer::map::dispos::change_map_dispos();
    let emblem = DVCVariable::EmblemDeployment.get_value();
    let resume = this.get_is_resume();
    if !resume {
        if emblem == 1 || emblem == 3{ for_each_unit(25u32, |unit| unit.clear_god_unit()); }
    }
    else {
        if emblem == 3 { for_each_unit(25u32, |unit| unit.clear_god_unit()); }
    }
    dvc::randomizer::map::terrain::adjust_miasma_tiles();
}

extern "C" fn map_sequence_dispos_unit(proc: MapSequence, _method_info: OptionalMethod) {
    proc.dispos_unit();
    if !proc.get_is_resume() {
        let user_data = DVCVariables::user_data();
        if user_data.get_chapter().is_dlc_god() || user_data.is_encount_map() {
            for_each_unit(2, |u|{
                let level = u.get_internal_level() + u.get_level();
                enemy_unit_stat_scaling(u, level, None);
                u.set_hp(u.get_capability(0, true));
            });
        }
        dvc::deployment::deployment_modes();
    }
}
fn move_units_to_dispos_location() {
    let args = Array::<DynValue>::new(DynValue::class().raw(), 3).unwrap();
    UnitPool::s_units().iter().enumerate().filter(|(_i, u)| !u.is_null() && !u.get_person().is_null() && !u.get_force().is_null())
        .filter(|(_i, u)| u.get_force_type().value == 1)
        .for_each(|(i, unit)| {
            let current = (unit.m_x(), unit.m_z());
            let dispos = (unit.m_dispos_x(), unit.m_dispos_z());
            if current.0 != dispos.0 || current.1 != dispos.1 {
                args.set(0, DynValue::new_number(1.0 + i as f64));
                args.set(1, DynValue::new_number(dispos.0 as f64));
                args.set(2, DynValue::new_number(dispos.1 as f64));
                ScriptUnit::unit_move_pos(args);
            }
        });
}
extern "C" fn map_sequence_map_opening(proc: MapSequence, _optional_method: OptionalMethod) {
    dvc::unit::ai::update_engage_attack_ai();
    engage::app::EventSequence::map_opening(proc);
    dvc::script::chapter::emblem_emblem_engage_atk_variable();
    Force::get(Force_Type::enemy()).iter().for_each(|unit| { dvc::unit::enemy_check_soar(unit, true); });
    if !proc.get_is_resume() {
        if DVCFlags::RandomDeploySpot.get_value() { move_units_to_dispos_location(); }
        dvc::randomizer::map::terrain::randomized_emblem_power_spots();
    }
}
extern "C" fn map_sequence_turn_effect(this: MapSequence, _method_info: OptionalMethod) {
    if DVCFlags::BGM.get_value() {
        let map_sit = MapSituation::get_instance();
        let force = map_sit.get_current_force_type();
        TelopManager::create_phase_change(this, force, map_sit.m_turn(), map_sit.get_win_rule_limit_turn());
        dvc::randomizer::bgm::update_bgm_for_force(this, true);
    }
    else { this.turn_effect(); }
}
extern "C" fn map_sequence_map_begins(proc: MapSequence, _optional_method: OptionalMethod) {
    dvc::unit::ai::update_engage_attack_ai();
    if DVCVariables::is_main_chapter_complete(4) {
        let emblem_deployment = DVCVariable::EmblemDeployment.get_value();
        if emblem_deployment & 1 != 0 {
            Force::get(Force_Type::player()).iter().for_each(|unit| {
                let god_unit = unit.get_god_unit();
                if !god_unit.is_null() { unit.clear_god_unit(); }
            })
        }
        match emblem_deployment {
            1 => {
                let mut v: Vec<_> = GodPool::get_instance().m_sort().iter().filter(|g| g.get_force_type().value == 0 && !g.m_is_escaping()).collect();
                Force::get(Force_Type::player()).iter().for_each(|unit| {
                    if let Some(god_unit) = v.get_remove(Random_2::get_game()) { unit.set_god_unit(god_unit); }
                });
            }
            2 => {
                let mut god_units = vec![];
                Force::get(Force_Type::player()).iter().for_each(|unit| {
                    let god_unit = unit.get_god_unit();
                    if !god_unit.is_null() {
                        unit.clear_god_unit();
                        god_unit.clear_child();
                        god_unit.clear_parent();
                        god_units.push(god_unit);
                    }
                });
                if !god_units.is_empty() {
                    Force::get(Force_Type::player()).iter().for_each(|unit| {
                        if let Some(god_unit) = god_units.get_remove(Random_2::get_game()) { unit.set_god_unit(god_unit); }
                    });
                }
            }
            _ => {}
        }
    }
    if DVCVariables::get_chapter_index() == 22 {
        if let Some(unit) = PersonPool::get_replacement_unit(32, false) {
            if unit.get_force_type().value == 2 {
                unit.transfer(Force_Type::player(), true);
            }
        }
        dvc::script::change_g_pid_lueur();
    }
    Force::get(Force_Type::enemy()).iter().for_each(|unit| { dvc::unit::enemy_check_soar(unit, false); });
    dvc::randomizer::bgm::set_random_bgm();
    proc.map_begin();
}

extern "C" fn try_restart_map_result(proc: MapSequence, _: OptionalMethod) {
    let user_data = DVCVariables::user_data();
    if user_data.is_evil_map() && proc.m_is_completed() && !user_data.is_recollection_map(){
        let mut stats: Vec<(i32, [i8; 10])> = vec![];
        Force::get(Force_Type::player()).iter()
            .chain(Force::get(Force_Type::absent()).iter())
            .for_each(|unit| {
                let level = unit.get_level_capability();
                let mut total: i32 = 0;
                let mut stat: [i8; 10] = [0; 10];
                for i in 0..9 {
                    let gain = level.get(i as i32);
                    total += gain as i32;
                    stat[i] = gain;
                }
                let level_up_count = unit.get_record(engage::app::UnitRecord_Kinds::map_level_up_count()) as i8;
                total += level_up_count as i32;
                stat[9] = level_up_count;
                if total > 0 { stats.push((unit.get_person().hash(), stat)); }
            });
        proc.try_restart_map_result();
        stats.into_iter().for_each(|(hash, stat)| {
            let unit = UnitPool::get_from_person(PersonData::try_get_from_hash(hash), false);
            if !unit.is_null() {
                for i in 0..9 { unit.add_base_capability(i, stat[i as usize]); }
                unit.add_skill_point((stat[9] as i32) * 200);
            }
        });
    }
    else { proc.try_restart_map_result(); }
}