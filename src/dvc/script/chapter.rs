use engage::{
    app::{
        mapimage::*, unit::*, persondata::*, goddata::*, eventscript::*, skilldata::*, random_2::*,
        GameMessage, AIValue_Order, GodData, UnitPool, Force_Type,
        ScriptMap, ScriptSystem, ScriptUnit, ScriptUtil, MapHistory,
        IStructData_1Methods, IMapImageUnitMethods, ISkillArrayMethods,
        IUnitAIMethods, ISingletonClass_1Methods,
    },
    EventScriptExt,
    moon_sharp::interpreter::dynvalue::*,
    DynValueArgs, Mess, GameVariableManager,
};
use outfit_core::Randomizer;
use unity::{Array, Cast, ClassIdentity, Il2CppString, OptionalMethod};
use crate::{DVCVariables, utils::il2str, DVCFlags, enums::{PIDS, MPIDS, RINGS}, utils::to_opt, gamedata::EmblemPool, DVCVariable};
pub fn emblem_emblem_engage_atk_variable(){
    let chapter_index = DVCVariables::get_chapter_index();
    match chapter_index {
        37 => { emblem_script_variable_change("竜脈_済", None); }
        53 => {
            for i in ["ヴェロニカ召喚1_済", "ヴェロニカ召喚2_済"] { emblem_script_variable_change(i, Some(emblem_script_change(None))); }
        }
        61 => {
            if let Some(atk) = il2str(GodData::get("GID_E001_敵チキ".into()).get_engage_attack()){
                let mut v = GameVariableManager::get_number("特殊召喚カウント");
                if !atk.contains("チキ") { v |= 16; } else { v &= 16; }
                GameVariableManager::set_number("特殊召喚カウント", v);
            }
        }
        63|65 => {
            emblem_script_variable_change("特殊召喚カウント", Some(emblem_script_change(None)));
        }
        66 => {
            let veronica = emblem_script_change(Some(15));
            emblem_script_variable_change("ヴェロニカ召喚カウント", Some(veronica));
        }
        _ => {}
    }
}
extern "C" fn m022_turn_1_boss_rewarp_event(_args: Array<DynValue>, _: OptionalMethod) {
    let unit = UnitPool::get_from_person_2("PID_M020_グリ", false);
    engage::app::MapCursor::set_2(unit, 0.0, true);
    engage::app::GameSound::field_bgm_play(Force_Type::player());
    engage::app::Talk::start_bind(ScriptUtil::get_sequence(), "MID_EV1", false);
}
extern "C" fn e001_engage_atk(_args: Array<DynValue>, _: OptionalMethod) {
    if let Some(boss) = to_opt(UnitPool::get_from_person_2("PID_E001_Boss", false)) {
        if crate::dvc::asset::engage_attack::EngageAttackType::from_unit(boss) == crate::dvc::asset::engage_attack::EngageAttackType::TikiDragon {
            if to_opt(UnitPool::get_from_person_2("PID_E001_超強_異形狼", false))
                .filter(|p| p.get_force_type().value == 1)
                .is_none_or(|u| u.get_hp_stock_count() > 0)
            {
                let ai = boss.get_ai();
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_Breath");
                ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy");
                boss.clear_status(Unit_Status::move_not_allow());
            }
        }
    }
}
pub extern "C" fn install_chapter_script_functions(script: EventScript) {
    let chapter = DVCVariables::get_chapter_index();
    script.register_function("EngageAttackCheck", emblem_unit_has_engage_atk);
    match chapter {
        4 => {
            if DVCFlags::Ironman.get_value() { script.register_action("青軍２ターン直前", m004_time_crystal); }
        }
        11 => {
            if DVCFlags::RandomDeploySpot.get_value() { script.register_function("condition_離脱地点が空いた",  condition_false); }
        }
        20 => { script.register_action("Turn1", m022_turn_1_boss_rewarp_event); }
        22 => {
            script.register_action("ユニット会話_ソロ時", m022_ring_pick_up_talk_1);
            script.register_action("ユニット会話_シンクロ中", m022_ring_pick_up_talk_2);
        }
        37|51|56 => {
            // 炎上配置
        }
        26 => {
            if DVCVariable::UnitDeployment.get_value() == 4 { script.register_action("味方キャラを再配置", m026_phase_2_positions); }
        }
        61 => { script.register_action("ボス移動開始", e001_engage_atk); }
        _ => {}
    }
    script.register_action("Dialog", dialog);
    if DVCFlags::Ironman.get_value() { script.register_action("MapHistoryRewindEnable", disable_rewind); }
}
fn emblem_script_variable_change(variable: &str, same: Option<bool>){
    let same = same.unwrap_or_else(|| emblem_script_change(None));
    let mut v = GameVariableManager::get_number(variable);
    if same { v |= 16; } else { v &= !16; }
    GameVariableManager::set_number(variable, v);
}
fn emblem_script_change(emblem_index: Option<i32>) -> bool {
    let chapter = DVCVariables::get_chapter_index();
    let mut active = true;
    match chapter {
        33..45 => {
            if let Some(emblem) = EmblemPool::get_emblem_by_paralogue_index(chapter - 30){
                let asset = EmblemPool::GIDS[emblem as usize];
                let pid = format!("PID_S{:03}_{}", chapter - 30,  asset);
                if let Some(emblem_person) = to_opt(PersonData::get(pid.into())) {
                    match chapter {
                        37 => { active = !emblem_person.get_common_skills().find("SID_竜脈").is_null() || !emblem_person.get_common_skills().find("SID_竜脈・異").is_null() }
                        53 => { active = to_opt(emblem_person.get_engage_skill()).is_some_and(|v| !v.get_sid().to_rust_string().contains(asset)); }
                        _ => {}
                    }
                }
            }
        }
        51..57 => {
            let asset = EmblemPool::GIDS[chapter as usize - 38];
            let pid = format!("PID_G00{}_{}", chapter - 50, asset);
            if let Some(emblem_person) = to_opt(PersonData::get(pid.into())) {
                match chapter {
                    53 => { active = to_opt(emblem_person.get_engage_skill()).is_some_and(|v| !v.get_sid().to_rust_string().contains(asset)); }
                    55 => { active = !emblem_person.get_common_skills().find("SID_竜脈").is_null() || !emblem_person.get_common_skills().find("SID_竜脈・異").is_null(); }
                    _ => {}
                }
            }
        }
        61 => { active = il2str(GodData::get("GID_E001_敵チキ".into()).get_engage_attack()).is_some_and(|v| v.contains("チキ")); }
        63|65 => {
            let c = chapter - 60;
            active = il2str(GodData::get(format!("GID_E00{}_敵ヴェロニカ", c).into()).get_engage_attack()).is_some_and(|v| v.contains("ヴェロニカ"));
        }
        66 => {
            if emblem_index == Some(15) { active = il2str(GodData::get("GID_E006_敵ヴェロニカ".into()).get_engage_attack()).is_some_and(|v| v.contains("ヴェロニカ")); }
        }
        _ => {}
    }
    active
}
extern "C" fn condition_false(_args: Array<DynValue>, _: OptionalMethod) -> DynValue { DynValue::new_boolean(false) }
extern "C" fn emblem_unit_has_engage_atk(_args: Array<DynValue>, _: OptionalMethod) -> DynValue {
    DynValue::new_boolean(emblem_script_change(None))
}
extern "C" fn disable_rewind(_arg: Array<DynValue>, _: OptionalMethod) { MapHistory::rewind_disable(); }
extern "C" fn dialog(arg: Array<DynValue>, _: OptionalMethod) {
    if DVCFlags::Ironman.get_value() && il2str(ScriptUtil::try_get_string(arg, 0, Il2CppString::null())).is_some_and(|c| c.contains("TIMECRYSTAL")) {
        return;
    }
    else if DVCVariables::get_chapter_index() == 22 && il2str(arg.try_get_string(0)).is_some_and(|v| v.contains("MID_TUT_NAVI_M022_GET")){
        if let Some(mid) = il2str(arg.try_get_string(0)).filter(|v| v.contains("MID_TUT_NAVI_M022_GET")){
            if let Some(pos) = RINGS.iter().position(|&r| mid.contains(r)){
                let original_ring_name = Mess::get(format!("MGID_Ring_{}", RINGS[pos])).to_rust_string();
                let original_message = Mess::get(mid).to_rust_string();
                if let Some(replacement) = EmblemPool::get_replacement_god_3(pos as i32, false) {
                    let new_ring_name = Mess::get(replacement.get_ringname()).to_rust_string();
                    let new_message = original_message.replace(original_ring_name.as_str(), new_ring_name.as_str());
                    GameMessage::create_key_wait(ScriptUtil::get_sequence(), new_message.as_str());
                    ScriptUtil::r#yield();
                    return;
                }
            }
        }
    }
    ScriptSystem::dialog(arg);
}
extern "C" fn m022_ring_pick_up_talk_1(arg: Array<DynValue>, _: OptionalMethod) {
    let pid = arg.try_get_string(0).to_rust_string();
    let pos = PIDS.iter().position(|p| *p == pid).filter(|&x| x > 0).unwrap_or_else(|| Random_2::get_system().get_value_2(35) as usize + 1);
    let mid = format!("{}1", MPIDS[pos].replace("MPID_", "MID_TK_"));
    let args: Array<DynValue> = Array::<DynValue>::new(DynValue::class().raw(), 1).unwrap();
    args.set(0, DynValue::new_string(mid.as_str()));
    ScriptSystem::talk(args);
}
extern "C" fn m022_ring_pick_up_talk_2(arg: Array<DynValue>, _: OptionalMethod) {
    let pid = arg.try_get_string(0).to_rust_string();
    let pos = PIDS.iter().position(|p| *p == pid).unwrap_or_else(|| Random_2::get_system().get_value_2(35) as usize + 1);
    let mid = format!("{}2", MPIDS[pos].replace("MPID_", "MID_TK_"));
    let args: Array<DynValue> = Array::<DynValue>::new(DynValue::class().raw(), 1).unwrap();
    args.set(0, DynValue::new_string(mid.as_str()));
    ScriptSystem::talk(args);
}
extern "C" fn m004_time_crystal(_args: Array<DynValue>, _: OptionalMethod) {
    let args: Array<DynValue> = Array::<DynValue>::new(DynValue::class().raw(), 2).unwrap();
    args.set(0, DynValue::new_string("Bandit"));
    args.set(1, DynValue::new_number(1.0));
    ScriptMap::dispos(args);
    let script = EventScript::get_instance();
    args.set(0, DynValue::new_number(0.5));
    args.get(0).assign_number(0.0);
    script.call("WaitTime", args);
    args.set(0, DynValue::new_string("MID_EV8"));
    ScriptSystem::talk(args);
}

pub extern "C" fn m026_phase_2_positions(_args: Array<DynValue>, _method_info: OptionalMethod) {
    let map_image = MapImage::get_instance();
    let args: Array<DynValue> = Array::<DynValue>::new(DynValue::class().raw(), 3).unwrap();
    for x in 0..3 { args.set(x, DynValue::new_number(0.0)); }
    if DVCFlags::RandomDeploySpot.get_value() {
        let mut possible_pos = vec![];
        for z in 0..31 {
            for x in 0..31 {
                if z >= 13 && z < 18 && x >= 13 && x < 18 { continue; } // Big Dragon Position
                if map_image.m_unit().get_unit(x, z).is_null() {
                    args.get(0).assign_number(x as f64);
                    args.get(1).assign_number(z as f64);
                    if il2str(ScriptMap::terrain_get_move_cost(args).get_string()).is_some_and(|c| c == "COST_平地") {
                        possible_pos.push((x, z));
                    }
                }
            }
        }
        let rng = Random_2::get_game();
        for z in 17..23 {
            for x in 1..29 {
                if let Some(unit) = to_opt(map_image.m_unit().get_unit(x, z)) {
                    args.set(0, DynValue::new_string(unit.get_pid()));
                    if let Some(pos) = possible_pos.get_random_element(rng) {
                        args.get(1).assign_number(pos.0 as f64);
                        args.get(2).assign_number(pos.1 as f64);
                        ScriptUnit::unit_set_pos(args);
                    }
                }
            }
        }
    }
    else {
        let mut pos_list = [
            ( 11, 12 ), ( 19, 12 ), ( 12, 11 ), ( 18, 11 ), ( 11, 10 ), ( 12, 10 ), ( 14, 10 ), ( 16, 10 ), ( 18, 10 ), ( 19, 10 ),
            ( 13, 9  ), ( 15, 9  ), ( 17, 9  ), ( 14, 8 ), ( 16, 8 ), ( 13, 7), ( 17, 7 ), ( 14, 6 ), ( 16, 6 ), ( 15, 5 ),
            ( 10, 11 ), ( 10, 10 ), ( 9, 11 ), ( 20, 11 ), ( 20, 10 ), ( 21, 11 ), ( 9, 12 ), ( 21, 12 ), ( 11, 9 ),
            ( 19, 9 ), ( 10, 12 ), ( 20, 12 ), ( 13, 10 ), ( 17, 10 ), ( 14, 9 ), ( 16, 9 ), ( 15, 8 ), ( 14, 7 ),
            ( 16, 7 ), ( 15, 6 ),
        ].iter();
        for z in 17..23 {
            for x in 1..29 {
                if let Some((unit, pos)) = to_opt(map_image.m_unit().get_unit(x, z)).zip(pos_list.next()) {
                    args.set(0, DynValue::new_string(unit.get_pid()));
                    args.get(1).assign_number(pos.0 as f64);
                    args.get(2).assign_number(pos.1 as f64);
                    ScriptUnit::unit_set_pos(args);
                }
            }
        }
    }
}