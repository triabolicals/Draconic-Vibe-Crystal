use super::*;

pub fn install_tilebolical_effects(script: &EventScript) {
    register_action(script, "ShuffleEmblems", shuffle_emblems);
    register_action(script, "DanceTeam", dance_all);
    register_action(script, "EngageOn", all_engage);
    register_action(script, "EngageOff", all_disengage);
    register_action(script, "HP1", set_hp_to_1);

    register_action(script, "HP100", set_hp_to_max);
    register_action(script, "HP100Team", set_hp_to_max_team);
    register_action(script, "LevelUp", unit_level_up);
    register_action(script, "SP100", unit_get_sp);
    register_action(script, "SP500", unit_get_sp_500);
    
    register_action(script, "SP1000", unit_get_sp_1000);
    register_action(script, "RevivalStone", revival_stone);
    register_action(script, "EnemyLevelUp", enemy_level_up);
    register_action(script, "EnemyLevelDown", enemy_level_down);
    
    register_action(script, "Enemy1HP", enemy_1_hp);
    register_action(script, "EnemyMaxHP", enemy_max_hp);
    register_action(script, "InheritSkill", inherit_skill);
    register_action(script, "EnemyActive", enemy_all_active);
    register_action(script, "EnemyInactive", enemy_all_inactive);
    
    register_action(script, "EnemyGuard", enemies_guarding);
    register_action(script, "Enemy50Exp", enemy_exp_50);
    register_action(script, "VoidCurse", enemy_void_curse);
    register_action(script, "GetExpertise", unit_expertise);
    register_action(script, "DoubleExp", unit_double_exp);

    register_action(script, "CompleteMap", finish_map);
    register_action(script, "PlayerWait", player_all_inactive);
    register_action(script, "EnemyStones", revival_stone_enemy);
    register_action(script, "BondUp", bond_up);
    register_action(script, "Vision", enemy_vision);

    register_action(script, "AllInactive", all_inactive);
    register_action(script, "RemoveEmblems", remove_emblems);

}

fn nothing_message() { GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MTID_Nothing").to_string()); }

extern "C" fn finish_map(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    if GameUserData::is_encount_map() || !GameUserData::get_chapter().cid.contains("M0") { 
        GameVariableManager::set_bool("勝利", true); 
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MSG_STAGE_CLEAR").to_string() );
    }
    else { nothing_message(); }
}

extern "C" fn enemy_exp_50(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let message = format!("{}+50", Mess::get("MID_SYS_Exp"));
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.private_skill.add_sid("SID_撃破経験加算５０", 10, 0);
        unsafe { unit_map_effect("ステータス上昇".into(), unit, None); }
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(),  message);
}
extern "C" fn enemy_void_curse(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit| { unit.private_skill.add_sid("SID_虚無の呪い", 10, 0); } );
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MSID_CurseOfNihility").to_string());
}
extern "C" fn unit_expertise(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    unit.private_skill.add_sid("SID_努力の才", 10, 0);
    unsafe {  unit_map_effect("ステータス上昇".into(), unit, None); }
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MSID_H_PrivateSkill_Jean").to_string());
}
extern "C" fn unit_double_exp(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    unit.private_skill.add_sid("SID_経験値２倍", 10, 0);
    unsafe {  unit_map_effect("ステータス上昇".into(), unit, None); }
    let message = format!("2x {}", Mess::get("MID_SYS_Exp"));
    GameMessage::create_key_wait(ScriptUtil::get_sequence(),  message);
}
extern "C" fn enemy_all_active(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let count = Force::get(ForceType::Enemy).unwrap().iter().count();
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.ai.set_active(1);
        unit.clear_status(1);
    });
    let message = format!("{} {}", Mess::get("MID_MENU_REWIND_INFO_REST_UNIT"), count);
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}

extern "C" fn enemy_all_inactive(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.ai.set_active(0); 
        unit.set_status(1);
    });
    let message = format!("{} 0", Mess::get("MID_MENU_REWIND_INFO_REST_UNIT"));
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), message);
}

extern "C" fn dance_all(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.status.value & 70368744177857 != 0 { player.clear_status(7036874417785); }
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_HELP_DANCE").to_string());
    GameSound::post_event("SE_Effect_Dance_ActionAgain", None);
}

extern "C" fn player_all_inactive(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player| player.set_status(1) );
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_WAIT").to_string());
}

extern "C" fn all_inactive(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player| player.set_status(1) );
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player| player.set_status(1) );
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_WAIT").to_string());
}

extern "C" fn enemies_guarding(_args: &Array<DynValue>, _method_info: OptionalMethod){
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        if unit.person.get_asset_force() != 0 {
            unit.set_status(128|64);
            unit.private_skill.add_sid("SID_チェインガード許可", 10, 0);
            unsafe { unit_map_effect("チェインガードコマンド".into(), unit, None); }
        }
    });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_GUARD").to_string());
}

extern "C" fn shuffle_emblems(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let mut emblem_list = crate::deployment::get_emblem_list();
    if emblem_list.len() < 2 { 
        nothing_message();
        return;
    }
    utils::remove_equip_emblems();
    let rng = Random::get_game();
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit| unit.clear_parent() );
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit|{
        if emblem_list.len() > 0 { 
            let value = rng.get_value(emblem_list.len() as i32) as usize;
            let god_unit = GodPool::try_get_gid(emblem_list[value].as_str(), false).unwrap();
            if unit.try_connect_god(god_unit).is_some() { emblem_list.remove(value); }
        }
     });
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_KEYHELP_MENU_GOD_CHANGE").to_string());
    GameSound::post_event("Play_God_Appear", None);
}

extern "C" fn remove_emblems(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit| unit.clear_parent() );
    GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_REFINE_SHOP_ENGRAVE_EMPTY").to_string());
}

extern "C" fn all_engage(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let mut has_engage = false;
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        if player.god_unit.is_some() {
            player.set_engage(true, None);
            player.reload_actor();
            has_engage = true;
        }
    });
    if has_engage {
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MENU_ENGAGE_COMMAND").to_string());
        GameSound::post_event("SE_Effect_Engage_On", None);
    }
    else { nothing_message(); }
}

extern "C" fn all_disengage(_args: &Array<DynValue>, _method_info: OptionalMethod) {
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
    else { nothing_message(); }
}
extern "C" fn set_hp_to_1 (_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    unit.set_hp(1);
    unit.play_set_damage(999, false, true);
}

extern "C" fn set_hp_to_max (_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    unit.set_hp(unit.get_capability(0, true));
    unit.play_set_damage(-999, false, true);
}

extern "C" fn set_hp_to_max_team (_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Player).unwrap().iter().for_each(|player|{
        player.set_hp(player.get_capability(0, true));
        player.play_set_damage(-999, false, true);
    });
}

extern "C" fn enemy_1_hp(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player|{
        player.set_hp(1);
        player.play_set_damage(999, false, true);
    });
}
extern "C" fn enemy_max_hp(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|player|{
        player.set_hp(player.get_capability(0, true));
        player.play_set_damage(-999, false, true);
    });
}

extern "C" fn revival_stone(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    unit.extra_hp_stock_count += 1;
    unit.extra_hp_stock_count_max += 1;
    unsafe { unit_map_effect("エンゲージ技_発射_G1".into(), unit, None); }
}

extern "C" fn revival_stone_enemy(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut count = 0;
    UnitPool::class().get_static_fields_mut::<crate::randomizer::job::UnitPoolStaticFieldsMut>().s_unit
        .iter_mut().filter(|unit| unit.force.is_some_and(|f| f.force_type == 1 )).for_each(|unit|{
            if rng.get_value(5) < 1 {
                count += 1;
                unit.extra_hp_stock_count += 1;
                unit.extra_hp_stock_count_max += 1;
                //unsafe { unit_map_effect("エンゲージ技_発射_G1".into(), unit, None); }
            }
        }
    );
    if count == 0 { nothing_message(); }
    else { 
        GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_TUT_BMAP_HPSTOCK_0").to_string()); 
        GameSound::post_event("Status_Up", None);
    }
}

extern "C" fn unit_level_up(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    if unit.level >= unit.job.max_level {
        unit_get_sp(_args, None);
        return;
    }
    let exp_need = 100 - unit.exp as i32;
    let proc = ScriptUtil::get_sequence();
    let grow_sequence = UnitGrowSequence::create_bind(proc);
    grow_sequence.set_unit_grow_data(unit, exp_need, 0, false);
}

extern "C" fn unit_get_sp(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    let proc = ScriptUtil::get_sequence();
    let grow_sequence = UnitGrowSequence::create_bind(proc);
    grow_sequence.set_unit_grow_data(unit, 0, 100, false);
}

extern "C" fn unit_get_sp_500(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    let proc = ScriptUtil::get_sequence();
    let grow_sequence = UnitGrowSequence::create_bind(proc);
    grow_sequence.set_unit_grow_data(unit, 0, 500, false);
}

extern "C" fn unit_get_sp_1000(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    let proc = ScriptUtil::get_sequence();
    let grow_sequence = UnitGrowSequence::create_bind(proc);
    grow_sequence.set_unit_grow_data(unit, 0, 1000, false);
}

extern "C" fn enemy_level_up(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.level_up(2);
        unsafe { unit_map_effect("ステータス上昇".into(), unit, None); }
    });
    //GameMessage::create_key_wait(ScriptUtil::get_sequence(), Mess::get("MID_MSG_LEVEL_UP").to_string());
    GameSound::post_event("FF_LevelUp_ST_Play", None);
}

extern "C" fn enemy_level_down(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        unit.level_down();
        unsafe { unit_map_effect("ステータス下降".into(), unit, None); }
    });
}

extern "C" fn bond_up(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    let proc = ScriptUtil::get_sequence();
    if let Some(g_unit) = unit.god_unit {
        if let Some(bond) = g_unit.get_bond(unit) {
            bond.level_up();
            unit.inherit_apt(g_unit);
            GameMessage::create_key_wait(proc, Mess::get("MID_MSG_POPUP_BOND_EXP_UP").to_string());
            GameSound::post_event("Heart_Up", None);
            return;
        }
    }
    unit_get_sp(_args, None);
}
extern "C" fn enemy_vision(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let rng = Random::get_system();
    let mut created = false;
    Force::get(ForceType::Enemy).unwrap().iter().for_each(|unit|{
        if rng.get_value(10) < 2 && unit.person.gender != 0 && unit.person.get_bmap_size() == 1 {
            unit.private_skill.add_sid("SID_残像", 10, 0);
            unit.mask_skill.unwrap().add_sid("SID_残像", 10, 0);
            unsafe { vision_create(unit, None); }
            created = true;
        }
    });
    if !created { nothing_message();  }
}
extern "C" fn inherit_skill(_args: &Array<DynValue>, _method_info: OptionalMethod) {
    let unit = MapMind::get_unit();
    if let Some(god_unit) = unit.get_god_unit() {
        let mut hash = 0;
        if let Some(growth) = GodGrowthData::try_get_from_god_data(god_unit.data) {
            growth.iter()
                .flat_map(|ggd| ggd.inheritance_skills.iter() )
                .flat_map(|inherit| inherit.iter())
                .for_each(|sid|{
                    if hash == 0 {
                        if let Some(skill) = SkillData::get(*sid) {
                            if unit.equip_skill_pool.find_sid(sid).is_none() && !skill.can_override_skill() {
                                unit.add_to_equip_skill_pool(skill);
                                hash = skill.parent.hash;
                            }
                        }
                    }
                }
            );
        }
        if hash != 0 {
            if let Some(skill) = SkillData::try_get_hash(hash) {
                let proc = ScriptUtil::get_sequence();
                let sid_substring = &skill.sid.to_string()[4..];
                let tag = Mess::create_sprite_tag(1, sid_substring.into());
                Mess::set_argument(0, tag);
                let skill_name = Mess::get(skill.name.unwrap());
                Mess::set_argument(1, skill_name.to_string());
                let message = Mess::get("MID_Hub_Inheritance_Skill_Finish");
                GameMessage::create_key_wait(proc, message.to_string());
                GameSound::post_event("SkillInherit", None);
                return;
            }
        }
    }
    unit_get_sp(_args, None);
}