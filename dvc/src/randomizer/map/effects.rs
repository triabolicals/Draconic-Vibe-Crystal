use std::collections::HashMap;
use engage::{unit::UnitUtil, map::{effect::MapEffect, history::MapHistory}};
use super::*;
use EffectType::*;
use crate::script::chapter::*;
use crate::utils::min;


pub static SCRIPT_ACTIONS: OnceLock<HashMap<&'static str, MethodInfo>> = OnceLock::new();
pub const SCRIPT_FUNCTION_NAME: [(&str, u8); 13] = [
    ("ShuffleEmblems", 6), ("EngagedAll", 6), ("HP100", 2), ("GainSP", 4), ("EnemyLevel", 3), ("EnemyActive", 7),
    ("BondUp", 6), ("Vision", 7), ("ReviveDead", 7), ("StatUp!", 5), ("Gold", 10), ("SpawnAbsent", 7), ("RemoveStones", 7)
];
pub(crate) fn init_script_functions() -> HashMap<&'static str, MethodInfo> {
    let mut map = HashMap::new();
    let donor_method = Il2CppClass::from_name("App", "ScriptSystem").unwrap().get_method_from_name("Log", 1).unwrap();
    add_method_info_to_map(&mut map, "ShuffleEmblems",  shuffle_emblems as _, donor_method);
    add_method_info_to_map(&mut map,"EngagedAll", all_engage as _, donor_method);

    add_method_info_to_map(&mut map,"HP100", set_hp_to_max as _, donor_method);
    add_method_info_to_map(&mut map,"GainSP", unit_get_sp as _, donor_method);

    add_method_info_to_map(&mut map,"RevivalStone", revival_stone as _, donor_method);
    add_method_info_to_map(&mut map,"EnemyLevel", enemy_level_up as _, donor_method);
    add_method_info_to_map(&mut map,"EnemyActive", enemy_all_active as _, donor_method);

    add_method_info_to_map(&mut map,"BondUp", bond_up as _, donor_method);
    add_method_info_to_map(&mut map,"Vision", vision as _, donor_method);
    add_method_info_to_map(&mut map,"ReviveDead", revive_units as _, donor_method);

    add_method_info_to_map(&mut map,"RandomSkill", skill_gain as _, donor_method);
    add_method_info_to_map(&mut map,"StatUp!", stat_up_change as _, donor_method);
    add_method_info_to_map(&mut map,"Gold", gold_gain as _, donor_method);

    add_method_info_to_map(&mut map,"SpawnAbsent", spawn_absent_unit as _, donor_method);
    add_method_info_to_map(&mut map,"RemoveStones", remove_stones as _, donor_method);
    add_method_info_to_map(&mut map,"味方キャラを再配置", m026_phase_2_positions as _, donor_method);
    add_method_info_to_map(&mut map,"ユニット会話_ソロ時", ring_talk_1 as _, donor_method);
    add_method_info_to_map(&mut map,"ユニット会話_シンクロ中", ring_talk_2 as _, donor_method);
    add_method_info_to_map(&mut map, "Dialog", ring_dialog_up_dialog as _, donor_method);


    let function = Il2CppClass::from_name("App", "ScriptSystem").unwrap().get_method_from_name("MessIsExist", 1).unwrap();
    add_method_info_to_map(&mut map,"PlayerGender", crate::script::dvc_alear_is_female as _, function);
    add_method_info_to_map(&mut map,"IsAlearFemale", crate::script::is_alear_female as _, function);
    map
}
fn add_method_info_to_map(map: &mut HashMap<&'static str, MethodInfo>, name: &'static str, method: *mut u8, donor_method: &MethodInfo) {
    let mut copy = donor_method.clone();
    copy.method_ptr = method;
    map.insert(name, copy);
}

pub fn install_tilebolical_effects(script: &EventScript) {
    GameVariableManager::make_entry("TileSkills", 0);
    let script_functions = SCRIPT_ACTIONS.get_or_init(||init_script_functions());
    for i in ["IsAlearFemale", "PlayerGender"] {
        if let Some(method_info) = script_functions.get(i).map(|m| EventScriptFunctionArgs::new_from_method_info(m)) {
            script.register_function2(i, method_info);
        }
    }
    let chapter = DVCVariables::get_chapter_index();
    if chapter == 26 {
        let action_name = "味方キャラを再配置";
        if let Some(method_info) = script_functions.get(action_name).map(|m| EventScriptActionArgs::new_from_method_info(m)) {
            script.register_action2(action_name, method_info);
        }
    }
    if chapter == 22 {
        for action_name in ["ユニット会話_ソロ時", "ユニット会話_シンクロ中", "Dialog"]{
            if let Some(method_info) = script_functions.get(action_name).map(|m| EventScriptActionArgs::new_from_method_info(m)) {
                script.register_action2(action_name, method_info);
            }
        }
    }
    SCRIPT_FUNCTION_NAME.iter().for_each(|(name, _)|{ register_action(script, name); });
    
    if let Some(cc) = Il2CppClass::from_name("App", "MapUnitCommandMenu")
        .unwrap().get_nested_types().iter().find(|cc| cc.get_name() == "VisitMenuItem")
    {
        if let Ok(class) = Il2CppClass::from_il2cpptype(cc.get_type()) {
            class.get_virtual_method_mut("GetName").map(|method| method.method_ptr = visit_command_name as _);
            class.get_virtual_method_mut("GetCommandHelp").map(|method| method.method_ptr = visit_command_help as _);
        }
    }
}
fn nothing_message() { GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MTID_Nothing").to_string()); }
fn nothing_message_with_name(mid: &str) {
    let message = format!("{}: {}", Mess::get(mid), Mess::get("MTID_Nothing"));
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}
fn added_skill_message(mut affected_units: i32, index: i32) {
    match index {
        0 => { affected_units = 2; }
        2..12 => { if affected_units % 2 == 0 { affected_units = 1; } else { affected_units = 0;} }
        _ => {}
    }
    let sid = SKILL_SIDS[index as usize];
    if let Some(skill) = SkillData::get(sid).and_then(|skill| skill.help)
        .or_else(||SkillData::get(sid).and_then(|skill| skill.name))
    {
        let mut name = String::new();
        match affected_units {
            1 => {
                Force::get(ForceType::Player).unwrap().iter()
                    .for_each(|unit| {
                        unit.private_skill.add_sid(sid, SkillDataCategorys::Private, 0);
                        if index == 1 { unit.private_skill.add_sid("SID_チェインアタック許可", SkillDataCategorys::Private, 0); }
                    });
                name = "Player Units".to_string();
            }
            2 => {
                Force::get(ForceType::Enemy).unwrap().iter()
                    .for_each(|unit| {
                        unit.private_skill.add_sid(sid, SkillDataCategorys::Private, 0);
                        if index == 1 { unit.private_skill.add_sid("SID_チェインアタック許可", SkillDataCategorys::Private, 0); }
                    });
                name = "Enemy Units".to_string();
            }
            3 => {
                Force::get(ForceType::Player).unwrap().iter().chain(Force::get(ForceType::Enemy).unwrap().iter())
                    .for_each(|unit| {
                        unit.private_skill.add_sid(sid, SkillDataCategorys::Private, 0);
                        if index == 1 { unit.private_skill.add_sid("SID_チェインアタック許可", SkillDataCategorys::Private, 0); }
                    });
                name = "All Units".to_string();
            }
            _ => {
                if let Some(unit) = MapMind::get_unit(){
                    unsafe { MapHistory::private_skill(unit); }
                    unit.private_skill.add_sid(sid, SkillDataCategorys::Private, 0);
                    if index == 2 { unit.private_skill.add_sid("SID_チェインアタック許可", SkillDataCategorys::Private, 0); }
                    name = Mess::get_name(unit.person.pid).to_string();
                }
            }
        }
        let message = format!("{}: {}",name, Mess::get(skill));
        let s = GameVariableManager::get_number("TileSkills") | (1 << index);
        GameVariableManager::set_number("TileSkills", s);
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
    }
    else { nothing_message(); }
}

extern "C" fn enemy_all_active(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let active = Random::get_system().get_value(2) == 0;
    for_each_unit(2, |unit|{
        if active {
            unit.ai.set_active(1);
            unit.clear_status(1);
        }
        else {
            unit.ai.set_active(0);
            unit.set_status(1);
        }
    });
    let message = if active { "Enemies are all active." } else { "Enemies are inactive." };
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}
extern "C" fn skill_gain(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let s = GameVariableManager::get_number("TileSkills");
    let mut indexes = vec![];
    for x in 0..SKILL_SIDS.len() {
        if s & (1 << x) == 0 { indexes.push(x as i32); }
    }
    if indexes.len() > 1 {
        let rng = Random::get_system();
        let affected = rng.get_value(4);
        if let Some(skill_index) = indexes.get(rng.get_value(indexes.len() as i32) as usize) {
            added_skill_message(affected, *skill_index);
            return;
        }
    }
    nothing_message();
}
extern "C" fn inactive(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system().get_value(4);
    match rng {
        0 => {
            Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
                if player.status.value & 70368744177857 != 0 { player.clear_status(7036874417785); }
            });
            GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_HELP_DANCE").to_string());
            GameSound::post_event("SE_Effect_Dance_ActionAgain", None);
        }
        1 => {
            for_each_unit(3, |u| u.set_status(1));
            let message = format!("All Units: {}", Mess::get("MID_MENU_WAIT"));
            GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
        }
        _ => {
            let force = if rng & 1 != 0 { 1 } else { 2};
            for_each_unit(force, |u| u.set_status(1));
            if rng & 1 != 0 {
                GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_TUT_BMAP_CHANGE_TITLE").to_string());
            }
        }
    }
}

extern "C" fn shuffle_emblems(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let mut emblem_list = crate::deployment::get_emblem_list();
    if emblem_list.len() < 2 { 
        nothing_message_with_name("MID_KEYHELP_MENU_GOD_CHANGE");
        return;
    }
    utils::remove_equip_emblems();
    let rng = Random::get_game();
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit|{
        MapHistory::god_disconnect(unit);
        unit.clear_god_unit();
    });
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit|{
        if emblem_list.len() > 0 { 
            let value = rng.get_value(emblem_list.len() as i32) as usize;
            if let Some(god_unit) = GodPool::try_get_gid(emblem_list[value].as_str(), false){
                MapHistory::god_connect(unit);
                if unit.try_connect_god_unit(god_unit).is_some() { emblem_list.remove(value); }
            }
        }
     });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_KEYHELP_MENU_GOD_CHANGE").to_string());
    GameSound::post_event("Play_God_Appear", None);
}
extern "C" fn remove_stones(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    for_each_unit(15, |unit|{
        unit.extra_hp_stock_count = 0;
        unit.hp_stock_count = 0;
        unit.hp_stock_count_max = 0;
        unit.extra_hp_stock_count_max = 0;
    });
}
extern "C" fn all_engage(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let mut has_engage = false;
    let on = Random::get_system().get_value(2) == 0;
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.god_unit.is_some() {
            if on { player.clear_status(67108864); }
            player.set_engage(on, None);
            player.reload_actor();
            has_engage = true;
        }
    });
    let mid = if on { "MID_MENU_ENGAGE_COMMAND" } else { "MID_MENU_ENGAGE_COMMAND_RELEASE" };
    if has_engage {
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get(mid).to_string());
        GameSound::post_event("SE_Effect_Engage_On", None);
    }
    else { nothing_message_with_name(mid); }
}
extern "C" fn set_hp_to_max (_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_game().get_value(10);
    if rng < 4 {
        if let Some(unit) = MapMind::get_unit(){
            unit.set_hp(unit.get_capability(0, true));
            unit.play_set_damage(-999, false, true);
        }
    }
    else {
        let force = if rng & 1 != 0 { 1 } else { 2 };
        for_each_unit(force, |unit|{
            unit.set_hp(unit.get_capability(0, true));
            unit.play_set_damage(-999, false, true);
        });
    }
}
extern "C" fn revival_stone(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        MapHistory::plain_hp_stock(unit);
        MapEffect::play_on_unit("エンゲージ技_発射_G1".into(), unit);
        if unit.extra_hp_stock_count == 0 {
            unit.extra_hp_stock_count += 1;
            unit.extra_hp_stock_count_max += 1;
        }
        let rng = Random::get_system().get_value(5);
        if rng < 2 { return; }
        else {
            let ff = if rng == 2 { 1 } else { 2 };
            for_each_unit(ff, |unit|{
                if unit.extra_hp_stock_count == 0 {
                    unit.extra_hp_stock_count += 1;
                    unit.extra_hp_stock_count_max += 1;
                }
            });
        }
    }
}
extern "C" fn unit_get_sp(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        let rng = Random::get_system();
        let rng_v = rng.get_value(15);
        let sp =
            match rng_v {
                0..5 => 100,
                5|6 => 500,
                7|8 => 1000,
                _ => 2,
            };
        if sp == 2 { grow_sequence.set_unit_grow_data(unit, rng.get_value(10)*10, 0, false); }
        else { grow_sequence.set_unit_grow_data(unit, 0, sp, false); }
    }
}
extern "C" fn enemy_level_up(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let increase = Random::get_system().get_value(2) == 0;
    for_each_unit(2, |unit|{
        if increase { unit.level_up(2); }
        else {
            let hp = unit.get_display_hp();
            unit.level_down();
            let new_hp = min(unit.get_capability(0, true), hp);
            unit.set_hp(new_hp);
            MapEffect::play_on_unit("ステータス下降".into(), unit);
        }
    });
    let message =
    if increase {
        GameSound::post_event("FF_LevelUp_ST_Play", None);
        "Enemies: Level Up"
    }
    else { "Enemies: Level Down" };
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}

extern "C" fn bond_up(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit() {
        let proc = ScriptUtil::get_sequence();
        if let Some(g_unit) = unit.god_unit {
            if let Some(bond) = g_unit.get_bond(unit) {
                MapHistory::god_level_up(g_unit, unit);
                bond.level_up();
                unit.inherit_apt(g_unit);
                GameMessage::create_key_wait(proc, Mess::get("MID_MSG_POPUP_BOND_EXP_UP").to_string());
                GameSound::post_event("Heart_Up", None);
                return;
            }
        }
    }
}
extern "C" fn vision(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut created = false;
    let mut mpid_str = String::new();
    let force = if rng.get_value(2) == 0 { ForceType::Player } else { ForceType::Enemy };
    Force::get(force).unwrap().iter().for_each(|unit|{
        if rng.get_value(10) < 1 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 {
            unit.private_skill.add_sid("SID_残像", SkillDataCategorys::Private, 0);
            MapHistory::private_skill(unit);
            let name = Mess::get_name(unit.person.pid).to_string();
            if mpid_str.is_empty() {
                mpid_str = Mess::get_name(unit.person.pid).to_string();
            }
            else if !mpid_str.contains(name.as_str()) {
                mpid_str.push_str(", ");
                mpid_str.push_str(name.as_str());
            }
            UnitUtil::vision_create(unit);
            created = true;
        }
    });
    if !created { nothing_message_with_name("MSID_LinEngage");  }
    else {
        let message = format!("{}: {}", mpid_str, Mess::get("MSID_LinEngage"));
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
    }
}
extern "C" fn revive_units(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let count = Force::get(ForceType::Dead).unwrap().iter().count();
    if count > 0 {
        let proc = ScriptUtil::get_sequence();
        GameVariableManager::make_entry_norewind("G_Revive", 1);
        GameVariableManager::set_number("G_Revive", 1);
        GameMessage::create_key_wait(proc, Mess::get("MID_H_Mode_Casual").to_string());
    }
    else { nothing_message_with_name("MID_H_Mode_Casual"); }
}

extern "C" fn stat_up_change(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        let rng = Random::get_system();
        let stat = rng.get_value(10);
        let value = rng.get_value(4) + 1;
        MapHistory::base_capability(unit, stat);
        unit.add_base_capability(stat, value);
        if stat == 0 {
            let hp = unit.get_capability(0, true);
            unit.set_hp(hp);
            unit.play_set_damage(-hp, false, false);
        }
        Mess::set_argument(0, unsafe { capability_name(stat, None) });
        Mess::set_argument_number(1, value);
        GameMessage::create_key_wait_mess(ScriptUtil::get_sequence(), "MID_MENU_ITEM_USE_DOPING_FOREVER");
    }
}
extern "C" fn gold_gain(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let amount = Random::get_system().get_value(100) * 10;
    let s = GameUserData::get_gold();
    GameUserData::set_gold( s + amount);
    GameMessage::create_gold_gain(ScriptUtil::get_sequence(), amount, None);
}

extern "C" fn spawn_absent_unit(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        if let Some(s) = Force::get(ForceType::Absent).unwrap().iter().chain(Force::get(ForceType::Dead).unwrap().iter())
            .find(|x| x.person.get_asset_force() == 0)
        {
            s.transfer(ForceType::Player, true);
            if s.try_create_actor() {
                let mut new_x = 0;
                let mut new_z = 0;
                unsafe {
                    get_rescue_pos(&mut new_x, &mut new_z, s, unit.x as i32, unit.z as i32, false, None);
                    unit_set_position(s, new_x, new_z, true, None);
                }
                s.reload_actor();
                MapEffect::play_on_unit("スキルコマンド_G3".into(), s);
                MapHistory::dispos(s);
            }
        }
        else { nothing_message(); }
    }
}

#[skyline::from_offset(0x01dece70)]
fn get_rescue_pos(dst: &mut i32, dst_z: &mut i32, target: &Unit, src_x: i32, src_z: i32, is_here: bool, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "SetPosition")]
fn unit_set_position(this: &Unit, x: i32, z: i32, update: bool, method_info: OptionalMethod);

#[unity::from_offset("App", "CapabilityDefinition", "GetName")]
fn capability_name(index: i32, method_info: OptionalMethod) -> &'static Il2CppString;
