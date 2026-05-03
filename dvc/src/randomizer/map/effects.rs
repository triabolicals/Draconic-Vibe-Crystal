use engage::{unit::UnitUtil, map::{effect::MapEffect, history::MapHistory}};
use super::*;
use EffectType::*;

pub fn install_tilebolical_effects(script: &EventScript) {
    GameVariableManager::make_entry("DVC", 1);
    GameVariableManager::make_entry("TileSkills", 0);
    script.register_function("PlayerGender", crate::script::dvc_alear_is_female);
    script.register_function("IsAlearFemale", crate::script::is_alear_female);
    register_action(script, "ShuffleEmblems", shuffle_emblems, Emblem);
    register_action(script, "DanceTeam", dance_all, Other);
    register_action(script, "EngageOn", all_engage, Emblem);
    register_action(script, "EngageOff", all_disengage, Emblem);

    register_action(script, "HP100", set_hp_to_max, HP);
    register_action(script, "HP100Team", set_hp_to_max_team, HP);
    register_action(script, "LevelUp", unit_level_up, Level);
    register_action(script, "SP100", unit_get_sp, SP);
    register_action(script, "SP500", unit_get_sp_500, SP);

    register_action(script, "SP1000", unit_get_sp_1000, SP);
    register_action(script, "SPMax", unit_get_sp_max, SP);
    register_action(script, "RevivalStone", revival_stone, Other);
    register_action(script, "EnemyLevelUp", enemy_level_up, Level);
    register_action(script, "EnemyLevelDown", enemy_level_down, Level);
    
    register_action(script, "Enemy1HP", enemy_1_hp, HP);
    register_action(script, "EnemyMaxHP", enemy_max_hp, HP);
    register_action(script, "EnemyActive", enemy_all_active, Other);
    register_action(script, "EnemyInactive", enemy_all_inactive, Other);
    
    register_action(script, "EnemyGuard", enemies_guarding, Other);
    register_action(script, "Enemy50Exp", enemy_exp_50, Exp);
    register_action(script, "VoidCurse", enemy_void_curse, Exp);
    register_action(script, "GetExpertise", unit_expertise, Skill);
    register_action(script, "DoubleExp", unit_double_exp, Exp);

    register_action(script, "CompleteMap", finish_map, Other);
    register_action(script, "PlayerWait", player_all_inactive, Other);
    register_action(script, "EnemyStones", revival_stone_enemy, Other);
    register_action(script, "BondUp", bond_up, Emblem);
    register_action(script, "Vision", enemy_vision, Other);

    register_action(script, "AllInactive", all_inactive, Other);
    register_action(script, "ReviveDead", revive_units, Other);
    register_action(script, "PlayerVision", player_vision, Other);

    register_action(script, "RevivalAll", revival_all_stone, Other);
    register_action(script, "LevelReset", unit_level_reset, Level);

    register_action(script, "RandomSkill", skill_gain, Skill);
    register_action(script, "StatUp!", stat_up_change, Stat);
    register_action(script, "Gold", gold_gain, GoldItem);

    register_action(script, "SpawnAbsent", spawn_absent_unit, SpawnUnit);
    register_action(script, "RemoveStones", remove_stones, Other);

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
    let sid = super::SKILL_SIDS[index as usize];
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

extern "C" fn finish_map(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if GameUserData::is_encount_map() || !GameUserData::get_chapter().cid.contains("M0") { 
        GameVariableManager::set_bool("勝利", true); 
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MSG_STAGE_CLEAR").to_string() );
    }
    else { nothing_message_with_name("MID_MSG_STAGE_CLEAR"); }
}

extern "C" fn enemy_exp_50(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let message = format!("Enemies gives +50 {}", Mess::get("MID_SYS_Exp"));
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        MapHistory::private_skill(unit);
        unit.private_skill.add_sid("SID_撃破経験加算５０", SkillDataCategorys::Private, 0);
        MapEffect::play_on_unit("ステータス上昇".into(), unit);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(),  message);
}
extern "C" fn enemy_void_curse(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit| { 
        unsafe { MapHistory::private_skill(unit); }
        unit.private_skill.add_sid("SID_虚無の呪い", SkillDataCategorys::Private, 0);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MSID_CurseOfNihility").to_string());
}
extern "C" fn unit_expertise(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    added_skill_message(0, 3);
}

extern "C" fn unit_double_exp(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        MapHistory::private_skill(unit);
        MapEffect::play_on_unit("ステータス上昇".into(), unit);
        unit.private_skill.add_sid("SID_経験値２倍", SkillDataCategorys::Private, 0);
        let message = format!("{}: 2x {}", Mess::get_name(unit.person.pid), Mess::get("MID_SYS_Exp"));
        GameMessage::create_key_wait(ScriptUtil::get_sequence(),  message);
    }
}
extern "C" fn enemy_all_active(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.ai.set_active(1);
        unit.clear_status(1);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), "Enemies are all active.");
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

extern "C" fn enemy_all_inactive(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.ai.set_active(0); 
        unit.set_status(1);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), "Enemies are inactive.");
}

extern "C" fn dance_all(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.status.value & 70368744177857 != 0 { player.clear_status(7036874417785); }
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_HELP_DANCE").to_string());
    GameSound::post_event("SE_Effect_Dance_ActionAgain", None);
}

extern "C" fn player_all_inactive(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player| player.set_status(1) );
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_TUT_BMAP_CHANGE_TITLE").to_string());
}

extern "C" fn all_inactive(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player| player.set_status(1) );
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player| player.set_status(1) );
    let message = format!("All Units: {}", Mess::get("MID_MENU_WAIT"));
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}

extern "C" fn enemies_guarding(_args: &Array<&DynValue>, _method_info: OptionalMethod){
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        if unit.person.get_asset_force() != 0 {
            unit.set_status(128|64);
            unit.private_skill.add_sid("SID_チェインガード許可", SkillDataCategorys::Private, 0);
            MapEffect::play_on_unit("チェインガードコマンド".into(), unit);
        }
    });
    let messages = format!("Enemies: {}", Mess::get("MID_MENU_GUARD"));
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), messages);
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
    UnitPool::class().get_static_fields_mut::<job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type < 3 )).for_each(|unit| {
        unit.extra_hp_stock_count = 0;
        unit.hp_stock_count = 0;
        unit.hp_stock_count_max = 0;
        unit.extra_hp_stock_count_max = 0;
    });
}
extern "C" fn all_engage(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let mut has_engage = false;
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.god_unit.is_some() {
            player.clear_status(67108864);
            player.set_engage(true, None);
            player.reload_actor();
            has_engage = true;
        }
    });
    if has_engage {
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_ENGAGE_COMMAND").to_string());
        GameSound::post_event("SE_Effect_Engage_On", None);
    }
    else { nothing_message_with_name("MID_MENU_ENGAGE_COMMAND"); }
}
extern "C" fn all_disengage(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let mut has_engage = false;
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.god_unit.is_some() {
            player.set_engage(false, None);
            player.reload_actor();
            has_engage = true;
        }
    });
    if has_engage {
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_ENGAGE_COMMAND_RELEASE").to_string());
        GameSound::post_event("SE_Effect_Engage_Off", None);
    }
    else { nothing_message_with_name("MID_MENU_ENGAGE_COMMAND_RELEASE"); }
}

extern "C" fn set_hp_to_max (_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        unit.set_hp(unit.get_capability(0, true));
        unit.play_set_damage(-999, false, true);
    }
}

extern "C" fn set_hp_to_max_team (_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        player.set_hp(player.get_capability(0, true));
        player.play_set_damage(-999, false, true);
    });
}

extern "C" fn enemy_1_hp(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player|{
        player.set_hp(1);
        player.play_set_damage(999, false, true);
    });
}
extern "C" fn enemy_max_hp(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player|{
        player.set_hp(player.get_capability(0, true));
        player.play_set_damage(-999, false, true);
    });
}

extern "C" fn revival_stone(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        MapHistory::plain_hp_stock(unit);
        MapEffect::play_on_unit("エンゲージ技_発射_G1".into(), unit);
        unit.extra_hp_stock_count += 1;
        unit.extra_hp_stock_count_max += 1;
    }
}
extern "C" fn revival_all_stone(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        MapHistory::plain_hp_stock(unit);
        MapEffect::play_on_unit("エンゲージ技_発射_G1".into(), unit);
        UnitPool::class().get_static_fields_mut::<job::UnitPoolStaticFieldsMut>().s_unit
            .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type < 3 )).for_each(|unit| {
            unit.extra_hp_stock_count += 1;
            unit.extra_hp_stock_count_max += 1;
        });
    }
}
extern "C" fn revival_stone_enemy(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut count = 0;
    UnitPool::class().get_static_fields_mut::<job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type == 1 )).for_each(|unit|{
            if rng.get_value(10) < 1 {
                MapHistory::plain_hp_stock(unit);
                count += 1;
                unit.extra_hp_stock_count += 1;
                unit.extra_hp_stock_count_max += 1;
            }
        }
    );
    if count == 0 { nothing_message(); }
    else { 
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_TUT_BMAP_HPSTOCK_0").to_string());
        GameSound::post_event("Status_Up", None);
    }
}
extern "C" fn unit_level_up(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        if unit.level >= unit.job.max_level {
            unit_get_sp(_args, None);
            return;
        }
        let exp_need = 100 - unit.exp as i32;
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, exp_need, 0, false);
    }
}
extern "C" fn unit_level_reset(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        unit.set_level(0);
        unit.set_internal_level(0);
        let exp_need = 100 - unit.exp as i32;
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, exp_need, 0, false);
    }
}

extern "C" fn unit_get_sp(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, 0, 100, false);
    }
}

extern "C" fn unit_get_sp_500(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit(){
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, 0, 500, false);
    }
}

extern "C" fn unit_get_sp_1000(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit() {
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, 0, 1000, false);
    }
}
extern "C" fn unit_get_sp_max(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    if let Some(unit) = MapMind::get_unit() {
        let proc = ScriptUtil::get_sequence();
        let grow_sequence = UnitGrowSequence::create_bind(proc);
        grow_sequence.set_unit_grow_data(unit, 0, 9999, false);
    }
}
extern "C" fn enemy_level_up(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.level_up(2);
        MapEffect::play_on_unit("ステータス下降".into(), unit);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), "Enemies: Level Up");
    GameSound::post_event("FF_LevelUp_ST_Play", None);
}

extern "C" fn enemy_level_down(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.level_down();
        MapEffect::play_on_unit("ステータス下降".into(), unit);
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), "Enemies: Level Down");
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
        unit_get_sp(_args, None);
    }
}
extern "C" fn enemy_vision(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut created = false;
    let mut mpid_str = String::new();
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
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
extern "C" fn player_vision(_args: &Array<&DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut created = false;
    let mut mpid_str = String::new();
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit|{
        if rng.get_value(10) < 2 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 {
            MapHistory::private_skill(unit);
            unit.private_skill.add_sid("SID_残像", SkillDataCategorys::Private, 0);
            if mpid_str.is_empty() { mpid_str = Mess::get_name(unit.person.pid).to_string(); }
            else {
                mpid_str.push_str(", ");
                mpid_str.push_str(Mess::get_name(unit.person.pid).to_string().as_str());
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
