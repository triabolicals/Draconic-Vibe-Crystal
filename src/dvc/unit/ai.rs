use engage::app::IStructDataArray_1Methods;
use super::*;
use engage::{
    app::{
        aivalue::*, aithink::*, EventScript,
        IEventScriptMethods, IMapInspector, IStructBase, IStructData_1Methods, MapInspectors,
    },
    GameVariableManager,
    List_1Ext
};
use engage::app::AIData;
use unity::Cast;
use unity::il2cpp::Il2CppClass;
use crate::{gamedata::PersonPool, utils::to_opt, DVCFlags};
pub const ENGAGE_ATK_AI: [&str; 11] =
    ["AI_AT_EngageAttack","AI_AT_EngagePierce", "AI_AT_EngageVision", "AI_AT_EngageDance",
        "AI_AT_EngageOverlap", "AI_AT_EngageBless", "AI_AT_EngageWaitGaze", "AI_AT_EngageSummon",
        "AI_AT_EngageCamilla", "AI_AT_Versus", "AI_AT_EngageWait"
    ];
pub fn ai_think_method_edit() {
    if let Some(ai_think) = Class::try_lookup("App", "AIThink").ok().map(|c| c.raw_mut()){
        set_ai_think_method_ptr(ai_think, "ActionHealMiddleLow", ai_action_heal_middle_low as _);
        set_ai_think_method_ptr(ai_think, "ActionAttackDefault", ai_action_attack_default as _);
        set_ai_think_method_ptr(ai_think, "ActionEngageBless", engage_bless as _ );
        set_ai_think_method_ptr(ai_think, "ActionEngageSummon", engage_summon as _ );
        set_ai_think_method_ptr(ai_think, "ActionEngageAttack", enemy_action_engage_attack as _);
        set_ai_think_method_ptr(ai_think, "ActionMoveAttackRange", action_move_attack_range as _);
        set_ai_think_method_ptr(ai_think, "ActionCommandSkillOverlap", ai_command_overlap as _);
    }
}
fn set_ai_think_method_ptr(ai_think: &mut Il2CppClass, method_name: &str, method_fn: *mut u8) {
    if let Some(m) = ai_think.get_method_from_name(method_name, 3) { m.method_ptr =  method_fn; }
}
fn engage_bless(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    if DVCFlags::EngageAttacks.get_value() { engage_ai_by_type(this, command, v0, v1) }
    else { this.action_engage_bless(command, v0, v1) }
}
fn engage_summon(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    if DVCFlags::EngageAttacks.get_value() { engage_ai_by_type(this, command, v0, v1) }
    else { this.action_engage_summon(command, v0, v1) }
}
pub fn update_engage_attack_ai() {
    let sequence = DVCVariables::get_sequence();
    if sequence != 2 && sequence != 3 { return; }
    let chapter = DVCVariables::get_chapter_index();
    if chapter == 20 {
        if let Some(unit) = to_opt(UnitPool::get_from_person_2("PID_M020_グリ", false)){
            let ai = unit.m_ai();
            let engage_attack_hash = to_opt(unit.get_engage_attack()).map(|skill| skill.hash()).unwrap_or(0);
            let ran_warp = engage_attack_hash == -1279013693;
            if ran_warp {
                if GameVariableManager::get_bool("グリ_ライナワープ終了_済") { ai.set_sequence(AIValue_Order::attack(), "AI_AT_AttackCS"); }
                else {
                    ai.set_sequence(AIValue_Order::attack(), "AI_AT_EngageAttack");
                    ai.set_sequence(AIValue_Order::r#move(), "AI_MV_Null");
                }
            }
            else {
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_EngageAttack");
                ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy");
            }
            let person_index = unit.get_person().index();
            MapInspectors::get_inspectors().iter().filter(|i| i.m_kind().value >= 16 && i.m_kind().value < 19 && unity::field_get_value_at_offset::<i32>(*i, 0x30) == person_index)
                .for_each(|i| { unity::field_set_value_at_offset(i, 0x34, 2 - ran_warp as i32) });

            if let Some(turn) = MapInspectors::get_inspectors().iter().find(|i| i.m_kind().value == 1){
                let function = if ran_warp { "青軍１ターン直前" } else { "Turn1" };
                turn.set_m_function(EventScript::get_instance().get_func(function));
            }
        }
    }
}


fn enemy_action_engage_attack(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    let unit = this.m_unit();
    let god_unit = unit.get_god_unit();
    if !god_unit.is_null() {
        if god_unit.get_data().get_gid().to_rust_string().contains("GID_相手") {
            if !unit.is_engaging_2() { this.engage_to(false) }
            else if unit.can_engage_attack() && unit.is_engaging_2() && !unit.check_status(Unit_Status::engage_attacked()) {
                let r = engage_ai_by_type(this, -3, v0, v1);
                if r == AIThink_Result::none() { this.engage_attack_to(command, v0, v1) } else { r }
            }
            else { AIThink_Result::none() }
        }
        else { this.action_engage_attack(command, v0, v1) }
    }
    else {
        let r = engage_ai_by_type(this, command, v0, v1);
        if r == AIThink_Result::none() { this.action_engage_attack(command, v0, v1) }
        else { r }
    }
}

fn engage_ai_by_type(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    let unit = this.m_unit();
    let engage_attack_type = get_engage_attack_type(unit.get_engage_attack());
    match engage_attack_type {
        0|2 => this.action_engage_attack(command, v0, v1),
        1 => this.action_engage_pierce(command, v0, v1),
        3 => this.action_engage_dance(command, v0, v1),
        4 => this.action_engage_overlap(command, v0, v1),
        5 => this.action_engage_bless(command, v0, v1),
        6 => this.action_engage_wait(command, v0, v1),
        7 => this.action_engage_summon(command, v0, v1),
        8 => this.action_engage_wait(command, v0, v1),
        9 => {
            let engage_attack = unit.get_engage_attack();
            let count = this.engage_heal_to_count_target(unit, engage_attack);
            if count > 1 { this.engage_heal_to(command, v0, v1) } else { AIThink_Result::none() }
        }
        _ => { AIThink_Result::none() }
    }
}
fn ai_command_overlap(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    let unit = this.m_unit();
    if unit.get_person().hash() == -1132784745 { this.action_command_skill_overlap(command, v0, v1) }
    else { command_skill_ai_base(this, command, v0, v1) }
}
fn ai_action_heal_middle_low(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    if !DVCFlags::RandomEnable.get_value() || DVCVariable::EnemySkillGauge.get_value() == 0 { this.action_heal_middle_low(command, v0, v1) }
    else {
        let r = command_skill_ai_base(this, command, v0, v1);
        if r.value == 0 { this.action_heal_middle_low(command, v0, v1) } else { r }
    }
}
fn ai_action_attack_default(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    if !DVCFlags::RandomEnable.get_value() { this.action_attack_default(command, v0, v1) }
    else {
        let r = command_skill_ai_base(this, command, v0, v1);
        if r.value == 0 { this.action_attack_default(command, v0, v1) }
        else { r }
    }
}
fn command_skill_ai_base(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    let unit = this.m_unit();
    let mut r = AIThink_Result::none();
    if unit.has_skill(SkillData::s_dance_skill()) && r.value == 0 {
        r = this.dance_to(command, v0, v1);
        if r.value != 0 { return r; }
    }
    let vision = SkillData::get("SID_残像".into());
    if !vision.is_null() {
        let skill = vision.get_style_skill(unit.get_job().get_style());
        if unit.has_skill(skill) {
            r = this.engage_vision_to(false);
            if r.value != 0 { return r; }
        }
    }
    let mask_skills = unit.m_mask_skill().m_list();
    let count = mask_skills.count();
    if r.value == 0 {   // Overlap / Support Commands
        for i in 0..count {
            let skill = mask_skills.get(i).get_skill();
            let timing = skill.get_timing();
            if timing == SkillData_Timings::overlap_command() {
                r = this.command_skill_overlap_to(0x70, v0, v1, skill);  // command = 70 or GodSkill -5
                if r.value != 0 { break; }
            }
            if timing == SkillData_Timings::support_command() {
                r = this.engage_support_to(command, v0, v1, skill);
                if r.value != 0 { break; }
            }
        }
    }
    if r.value == 0 { r = this.action_vs_god_skill(command, v0, v1); }
    r
}
fn action_move_attack_range(this: AIThink, command: i32, v0: AIValue, v1: AIValue) -> AIThink_Result {
    let mut result = this.action_move_rewarp(command, v0, v1);
    if result.value == 0 { result = this.action_move_attack_range(command, v0, v1); }
    result
}

pub fn adjust_person_unit_ai(unit: Unit) {
    if DVCVariable::UnitRecruitment.get_value()  == 0 { return; }
    let ai = unit.get_ai();
    for order in 0..4 {
        for index in 0..4 {
            let ai_value = ai.get_value(AIValue_Order{value: order}, index);
            if let Some(new_person) = to_opt(ai_value.get_person()).and_then(|p| PersonPool::get_replacement_2(p, false)){
                ai_value.set_value_2(new_person.get_pid());
            }
        }
    }
}
fn ai_sequence_contains_mind(ai: UnitAI, order: AIValue_Order, mind: i8) -> bool {
    to_opt(AIData::get(ai.get_sequence(order)))
        .map(|ai|
            ai.base().base().iter().filter(|ai| unity::field_get_value_at_offset::<i8>(*ai, 0x28) == 3)
                .map(|ai| unity::field_get_value_at_offset::<i8>(ai, 0x29))
                .any(|m| m == mind)
        ).unwrap_or(false)
}

fn ai_sequence_contains_mind_2(ai: UnitAI, order: AIValue_Order, condition: impl Fn(i8) -> bool) -> bool {
    to_opt(AIData::get(ai.get_sequence(order)))
        .map(|ai|
            ai.base().base().iter().filter(|ai| unity::field_get_value_at_offset::<i8>(*ai, 0x28) == 3)
                .map(|ai| unity::field_get_value_at_offset::<i8>(ai, 0x29))
                .any(|mind| condition(mind))
        )
        .unwrap_or(false)
}
fn can_change_ai_mv(ai: UnitAI) -> bool {
    !ai_sequence_contains_mind_2(ai, AIValue_Order::r#move(), |mind| mind >= 89 && mind <= 108)
}
fn is_escape_breakdown_treasure_terrain(ai: UnitAI) -> bool {
    ai_sequence_contains_mind_2(ai, AIValue_Order::r#move(), |mind| mind >= 94 && mind <= 101)
}
/*
    Action Mind, Code == 3
    60 - ActionMindTalk (MI)
    61 - ActionMindTreasure (MI_Treasure)
    62 - ActionMindVillage (MI_Village)
    63, 64 - ActionMindEscape (MI_Escape / MI_EscapeSlow)
    65 - ActionMindBreakDown (MI_BreakDown)
    66 - ActionMindDance    (MI_Irregular)
    68 - ActionMindTerrainDestroy ("MV_TerrainDestroy")
    70 - ActionMindTorch (MI_Torch (Action=0 and MI_TorchAlways (Action=-1))
    71..74 - ActionMindGuard Various AI_AT / 73 used in MI_GuardPerson
    80 - ActionMoveReturn
    81 - ActionMoveIdle (Last in All AI_MV)
    82..87,109 - ActionMoveAttackRange (AI_MV ranges/force/attack..)
    88 - ActionMoveHealRange (AI_MV Heals)
    89 - ActionMoveHero AI_MV_Hero / AI_MV_HeroAlways
    90 - ActionMovePerson,
    91 - ActionMovePosition,
    92,92 - ActionMoveForce
    94 - ActionMoveTreasure
    95 - ActionMoveVillage
    96 - ActionMoveEscape
    97 - ActionMoveNearestFriend
    98 - ActionMoveTerrainDestroy
    100 - ActionMoveBreakDown
    101 - ActionMoveTerrain
    108 - ActionMoveRewarp
 */
pub fn adjust_unitai(unit: Unit) {
    let job = unit.get_job();
    let m022 = DVCVariables::get_chapter_index() == 22;
    let ai = unit.get_ai();
    let activation = ai.get_sequence(AIValue_Order::cause()).to_rust_string();
    let ac_every_time = activation.contains("AC_Everytime");
    let ac_turn =  activation.contains("Turn");
    let mut old_ai_names =  vec![];
    let mut old_ac_values: [i32; 4] = [0; 4];
    for i in 0..4 {
        old_ai_names.push(ai.get_sequence(AIValue_Order{value: i as i32}).to_rust_string());
        old_ac_values[i] = ai.m_a_value().get(i).get_value();
    }
    if unit.get_person().get_asset_force().value == 2 {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_Attack");
        ai.set_sequence(AIValue_Order::r#move(), "AI_AT_Turn");
    }
    let can_change_move = can_change_ai_mv(ai);
    let treasure = is_escape_breakdown_treasure_terrain(ai);
    // Allow Non-thieves to open doors if they have treasure AI
    if is_escape_breakdown_treasure_terrain(ai){
        if ai_sequence_contains_mind(ai, AIValue_Order::r#move(), 94) {
            unit.add_private_skill("SID_鍵開け");
        }
        return;
    }
    if old_ai_names[3].contains("Retreat") { ai.set_sequence(AIValue_Order::r#move(), "AI_MV_NearestEnemy"); }
    let j_mask = unit.get_job().get_mask_skill();
    let p_mask = unit.get_mask_skill();
    if !j_mask.find("SID_踊り").is_null() || !p_mask.find("SID_踊り").is_null() {
        ai.set_sequence(AIValue_Order::mind(), "AI_MI_Irregular");
        if ac_every_time { ai.set_sequence(AIValue_Order::cause(), "AI_AC_AttackRange"); }
        if DVCVariables::is_main_chapter_complete(19) { unit.add_private_skill("SID_特別な踊り"); }
    }
    if old_ai_names[2] != "AI_AT_Null" {
        if !j_mask.find("SID_エンチャント").is_null() || !p_mask.find("SID_エンチャント").is_null() {
            ai.set_sequence(AIValue_Order::attack(), "AI_AT_Enchant");
        }
        // staff user, Chapter 22 needs to use Force due to Green Emblem Allies
        else if job.get_weapon_mask_2().m_value() & ( 1 << 7 ) != 0 {
            if unit.has_item_2(ItemData::get("IID_ワープ".into())) {
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_RodWarp");
                ai.set_value(AIValue_Order::attack(), 0, 1);
                ai.set_value(AIValue_Order::attack(), 1, 1);
                if can_change_move { ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy"); }

            }
            else if unit.has_interference_rod() {
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_Interference");
                ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy");
                if ac_every_time { ai.set_sequence(AIValue_Order::cause(),  "AI_AC_AttackRange"); }
                set_ai_values_to_0(ai, AIValue_Order::attack());
            }
            else if unit.has_heal_rod() {
                ai.set_sequence(AIValue_Order::attack(),  "AI_AT_HealToAttack");
                if can_change_move { ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy"); }
            }
            else {
                ai.set_sequence(AIValue_Order::attack(),  "AI_AT_Attack");
                if can_change_move { ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy"); }
                set_ai_values_to_0(ai, AIValue_Order::attack());
            }
        }
        else {
            if old_ai_names[0].contains("Guard") || old_ai_names[1].contains("Guard") {
                unit.add_private_skill("SID_チェインガード許可");
            }
            // Healer turned non healer
            if old_ai_names[1].contains("Heal") && ac_every_time { ai.set_sequence(AIValue_Order::cause(),  "AI_AC_AttackRange"); }
            if old_ai_names[2].contains("Heal") { ai.set_sequence(AIValue_Order::attack(), "AI_AT_Attack"); }
            if old_ai_names[3].contains( "Heal") && can_change_move {  ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy"); }
            // No offensive staffs
            if old_ai_names[1].contains("Interference") || old_ai_names[2].contains("Interference") {
                ai.set_sequence(AIValue_Order::cause(),   "AI_AC_AttackRange");
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_Attack");
                set_ai_values_to_0(ai, AIValue_Order::attack());
                set_ai_values_to_0(ai, AIValue_Order::cause());
                if can_change_move { ai.set_sequence(AIValue_Order::r#move(), "AI_MV_WeakEnemy"); }
            }
            if old_ai_names[2].contains("RodWarp") {
                ai.set_sequence(AIValue_Order::attack(), "AI_AT_Attack");
                set_ai_values_to_0(ai, AIValue_Order::attack());
            }
        }
    }

    if m022 {
        set_ai_for_m022(unit);
        return;
    }
    if old_ai_names[3].contains( "Terrain") {  ai.set_sequence(AIValue_Order::r#move(), old_ai_names[3].as_str()); }
    if !unit.has_private_skill("SID_リーダー") ||  ac_turn {
        ai.set_sequence(AIValue_Order::cause(),  old_ai_names[0].as_str());
        for x in 0..4 { ai.set_value(AIValue_Order::cause(), x, old_ac_values[x as usize]); }
    }
    if to_opt(unit.get_god_unit()).is_some_and(|v| v.get_gid().to_rust_string().contains("GID_相手")) {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_EngageAttack");
    }
    else { adjust_ai_for_engage_attack(unit); }
    if ai.get_sequence(AIValue_Order::attack()).to_rust_string().contains("Null") && !treasure  {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_Attack");
        ai.set_value(AIValue_Order::attack(), 0, 0);
    }
    if unit.get_person().hash() == -1631515167 {    // Chapter 7 Hortensia
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_EngageAttack");
    }
}
pub fn set_ai_for_m022(unit: Unit){
    let ai = unit.get_ai();
    ai.set_sequence(AIValue_Order::r#move(), "AI_MV_Force");
    ai.set_value(AIValue_Order::r#move(), 0, 0);
    let attack = ai.get_sequence(AIValue_Order::attack()).to_rust_string();
    if attack.contains("Heal") {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_AttackToHealForceOnly");
        ai.set_value(AIValue_Order::attack(), 2, 0);
    }
    else if attack.contains("Interference") {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_InterferenceForceOnly");
        ai.set_value(AIValue_Order::attack(), 2, 0);
    }
    else {
        ai.set_sequence(AIValue_Order::attack(), "AI_AT_ForceOnly");
        ai.set_value(AIValue_Order::attack(), 0, 0);
    }
}

fn set_ai_values_to_0(ai: UnitAI, order: AIValue_Order){
    ai.set_value(order, 0, 0);
    ai.set_value(order, 1, 0);
    ai.set_value(order, 2, 0);
    ai.set_value(order, 3, 0);
}

pub fn adjust_ai_for_engage_attack(unit: Unit) {
    let ai = unit.get_ai();
    let engage_atk_ai = get_engage_attack_type(unit.get_engage_attack());
    if engage_atk_ai != -1 {
        ai.set_sequence(AIValue_Order::attack(), ENGAGE_ATK_AI[engage_atk_ai as usize]);
        if engage_atk_ai == 4 {
            ai.set_value(AIValue_Order::attack(), 0, 255);
            ai.set_value(AIValue_Order::attack(), 1, 255);
            ai.set_value(AIValue_Order::attack(), 2, 3);
            ai.set_value(AIValue_Order::attack(), 3, 3);
        }
        else if engage_atk_ai == 8 {
            ai.set_value(AIValue_Order::attack(), 0, 2);
            ai.set_value(AIValue_Order::attack(), 1, 2);
            ai.set_value(AIValue_Order::attack(), 2, 255);
            ai.set_value(AIValue_Order::attack(), 3, 255);
        }
        else {
            ai.set_value(AIValue_Order::attack(), 0, 2);
            ai.set_value(AIValue_Order::attack(), 1, 2);
        }
    }
}
pub fn get_engage_attack_type(skill: SkillData) -> i32 {
    if skill.is_null() { return -1; }
    let sid = skill.get_sid().to_rust_string();
    if let Some(engage_type) = EmblemPool::GIDS.iter().position(|s| sid.contains(s)){
        match engage_type {
            0|2|4|5|6|11|12|16|18|19|20|21 => { 0 }, //AI_AT_EngageAttack
            1 => { 1 }, //AI_AT_EngagePierce
            3 => { 9 }, //AI_AT_Versus
            7 => { 2 }, // AI_AT_EngageVision
            8 => { 10 }, // AI_AT_EngageWait
            9 => { 3 }, // AI_AT_EngageDance
            10 => { 4 }, // AI_AT_EngageOverlap
            13 => { 5 }, // AI_AT_EngageBless
            14 => { 6 }, // AI_AT_EngageWaitGaze
            15 => { 7 }, // AI_AT_EngageSummon
            17 => { 8 }, // AI_AT_EngageCamilla
            _ => { -1 },    // None
        }
    }
    else { -1 }
}