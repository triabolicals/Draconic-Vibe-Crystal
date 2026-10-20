use engage::{
    app::{
        IFieldBgmManager, ISingletonMonoBehaviour_1Methods,
        unitmodel::*, gamesound::*,
        mapsequencebattleaction::*, mapsequencebattle::*, mapsituation::*,
        procinst::*, sortiesequence::*, battlecalculator::*, soundmanager::*,
        IBattleInfoSide, ICommonBattleSequence_1, IUnitItem,
        Proc, ProcVoidMethod,Force_Type,
        FieldBgmManager_ProcChangeBgm, MapHistory,
    },
    combat::{Character, combatsequence::*},
    GameVariableManager,
    ProcVoidMethodExt,
};
use super::*;
use outfit_core::anim::SpecialAttackType;
use crate::{utils::to_opt};
pub fn assign_random_bgm(){
    let rng = Random_2::get_game();
    let mut pool = GameData::get().bgm.clone();
    for i in 0..4 {
        if let Some(bgm) = pool.get_remove(rng) {
            DVCVariables::set_variable_key_string(format!("CBGM{}", i+1), bgm.as_str());
        }
    }
    if let Some(bgm) = pool.get_remove(rng) { DVCVariables::set_variable_key_string("SBGM", bgm.as_str()); }
}
pub fn update_bgm_for_force(proc: impl Into<ProcInst>, is_elapsed_turn: bool){
    let map_sit = MapSituation::get_instance();
    if map_sit.is_null() { return; }
    let force = map_sit.get_current_force_type();
    let manager = SoundManager::get_instance().get_field_bgm_manager();
    let mut return_to_normal_bgm = false;
    manager.set_m_prev_bgm_handle_name(manager.m_cur_bgm_handle_name());
    if force.value == 0 && is_elapsed_turn {
        let turn = manager.m_battle_bgm_continue_turn();
        if turn < 2 {
            MapHistory::field_bgm_special_turn(turn);
            manager.set_m_battle_bgm_continue_turn(turn + 1);
        }
        if manager.m_battle_bgm_continue_turn() == 2 { return_to_normal_bgm = true; }
    }
    let handle_array = manager.m_handle_name_array();
    if force.value < handle_array.len() as i32 {
        manager.set_m_cur_bgm_handle_name(handle_array.get(force.value as usize));
        FieldBgmManager_ProcChangeBgm::create(manager, force, return_to_normal_bgm, proc);
    }
}
pub fn set_random_bgm() {
    if DVCFlags::BGM.get_value() {
        let bgm1 = GameVariableManager::get_string("CBGM1");
        let bgm2 = GameVariableManager::get_string("CBGM2");
        let bgm3 = GameVariableManager::get_string("CBGM3");
        GameSound::field_bgm_set_phase_bgm_2(bgm1, bgm2, bgm3);
    }
}
fn get_random_special(set: bool) -> Il2CppString {
    if set {
        if let Some(bgm) = GameData::get().bgm.get_random_element(Random_2::get_game()) {
            DVCVariables::set_variable_key_string("SBGM", bgm.as_str());
        }
    }
    GameVariableManager::get_string("SBGM")
}
pub fn random_special_bgm(calculator: BattleCalculator) -> bool {
    if GameSound::field_bgm_is_special_battle_bgm(calculator) {
        GameSound::field_bgm_start_special_battle_bgm_continue_turn();
        let s = get_random_special(false);
        if !GameSound::is_event_playing(s) {
            if GameSound::field_bgm_set_phase_bgm_2(s, s, s) {
                GameSound::field_bgm_play(Force_Type::player());
                GameSound::set_state_special_battle("PreSpecialCombat");
            }
        }
        true
    }
    else { false }
}
pub fn proc_descs_bgm_edit(proc: ProcInst) {
    let descs = proc.m_descs();
    if let Some(proc) = proc.try_cast::<SortieSequence>() {
        let deployment = DVCVariable::UnitDeployment.get_value();
        if deployment != 0 && DVCVariables::is_main_chapter_complete(4){
            let mut pos_list = deployment::get_deployment_positions(None);
            let mut units = vec![];
            Force::get(Force_Type::player()).iter().for_each(|unit|{
                let x = unit.get_x();
                let z = unit.get_z();
                if let Some(i) = pos_list.iter().position(|&r| r.0 == x && r.1 == z) { pos_list.remove(i); } 
                else { units.push(unit); }
            });
            if !units.is_empty() && !pos_list.is_empty() {
                units.iter().zip(pos_list.into_iter()).for_each(|(unit, pos)|{
                    unit.set_position_2(pos.0, pos.1, true);
                    unit.set_rotation(pos.2);
                })
            }
        }
        descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, sortie_play_bgm).unwrap()));
        descs.set(39, Proc::call_2(ProcVoidMethod::from_fn(proc, sortie_stop_bgm).unwrap()));
    }
    else if let Some(proc) = proc.try_cast::<MapSequenceBattle>() {
        descs.set(60, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_battle_pre_bgm).unwrap()));
    }
    else if let Some(proc) = proc.try_cast::<MapSequenceBattleAction>() {
        descs.set(1, Proc::call_2(ProcVoidMethod::from_fn(proc, map_sequence_battle_action_pre_bgm).unwrap()));
    }
    else if let Some(proc) = proc.try_cast::<CombatSequence>() {
        descs.set(4, Proc::call_2(ProcVoidMethod::from_fn(proc, combat_sequence_pre_bgm).unwrap()));
    }
}

extern "C" fn map_sequence_battle_pre_bgm(this: MapSequenceBattle, _method_info: OptionalMethod) {
    if DVCFlags::BGM.get_value() { if random_special_bgm(this.m_calculator()) { return; } }
    this.to_pre_bgm();
}
fn is_human_body(unit: Unit) -> bool {
    to_opt(unit.get_unit_model()).and_then(|model| to_opt(model.m_body())).is_some_and(|o| {
        let body = o.get_name().to_rust_string().to_lowercase();
        body.contains("f_c") || body.contains("m_c")
    })
}
extern "C" fn map_sequence_battle_action_pre_bgm(this: MapSequenceBattleAction, _method_info: OptionalMethod) {
    if !this.m_calculator().is_null() {
        for value in 0..2 {
            let side = this.m_calculator().get_side(Force_Type { value });
            if !side.is_null() && !side.m_unit_item().is_null() && !side.m_unit().is_null() {
                let item = side.m_unit_item().m_item();
                let unit = side.m_unit();
                if is_human_body(unit) && !unit.check_status(Unit_Status::engage_attack()) && SpecialAttackType::determine_from_unit(unit, item) == SpecialAttackType::Transform {
                    side.m_status().set_or_clear(true, -2147483648);
                }
            }
        }
        if DVCFlags::BGM.get_value() { if random_special_bgm(this.m_calculator()) {return; } }
    }
    this.to_pre_bgm();
}

extern "C" fn sortie_play_bgm(this: SortieSequence, _: OptionalMethod) {
    if DVCFlags::BGM.get_value() { GameSound::post_event(GameVariableManager::get_string("CBGM4"), Character::null()); }
    else { this.play_bgm(); }
}

extern "C" fn sortie_stop_bgm(this: SortieSequence, _: OptionalMethod) {
    if DVCFlags::BGM.get_value() { GameSound::stop_sound_on_event(GameVariableManager::get_string("CBGM4"), GameSound_FadeSpeedType::very_slow()); }
    else { this.stop_bgm(); }
}

extern "C" fn combat_sequence_pre_bgm(this: CombatSequence, _: OptionalMethod) {
    if DVCFlags::BGM.get_value() && this.get_calculator().m_mode().value == 0 {
        if random_special_bgm(this.get_calculator()) { return; }
    }
    this.to_pre_bgm();
}