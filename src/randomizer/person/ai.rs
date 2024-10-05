use super::*;
use crate::randomizer::emblem;
use crate::utils;

static mut AISET: bool = false;

pub fn adjust_dispos_person_ai(data: &mut DisposData) {
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return; }

    if data.ai_action_value.is_some() {
        let string = data.ai_action_value.unwrap().get_string().unwrap();
        let found = PIDS.iter().position(|x| *x == string);
        if found.is_some() {
            let new_string = format!("G_R_{}", string);
            let new_pid = GameVariableManager::get_string(&new_string);
            data.ai_action_value = Some(new_pid);
        }
    }
    if data.ai_mind_value.is_some() {
        let string = data.ai_mind_value.unwrap().get_string().unwrap();
        let found = PIDS.iter().position(|x| *x == string);
        if found.is_some() {
            let new_string = format!("G_R_{}", string);
            let new_pid = GameVariableManager::get_string(&new_string);
            data.ai_mind_value = Some(new_pid);
        }
    }
    if data.ai_move_value.is_some() {
        let string = data.ai_move_value.unwrap().get_string().unwrap();
        let found = PIDS.iter().position(|x| *x == string);
        if found.is_some() {
            let new_string = format!("G_R_{}", string);
            let new_pid = GameVariableManager::get_string(&new_string);
            data.ai_move_value = Some(new_pid);
        }
    }
    if data.ai_attack_value.is_some() {
        let string = data.ai_attack_value.unwrap().get_string().unwrap();
        let found = PIDS.iter().position(|x| *x == string);
        if found.is_some() {
            let new_string = format!("G_R_{}", string);
            let new_pid = GameVariableManager::get_string(&new_string);
            data.ai_attack_value = Some(new_pid);
        }
    }
}


pub fn adjust_unit_ai(unit: &Unit, data: &mut DisposData) {
    let job = unit.get_job();
    let m022 = GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022";
    let jid = job.jid.get_string().unwrap();
    let old_ai_names: [&Il2CppString; 4] = [data.ai_action_name, data.ai_mind_name, data.ai_attack_name, data.ai_move_name];
    let old_ai_values: [Option<&Il2CppString>; 4] = [data.ai_action_value, data.ai_mind_value, data.ai_attack_value, data.ai_move_value];
    let not_ac_every_time = !utils::str_contains(data.ai_action_name, "AC_Everytime");

    // Allow Non-thieves to open doors if they have treasure AI
    if str_contains(old_ai_names[1], "Treasure") || str_contains(old_ai_names[3], "Treasure") {
        unit.private_skill.add_sid("SID_鍵開け", 10, 0); 
    }
    if jid == "JID_ダンサー" {
        data.ai_mind_name = "AI_MI_Irregular".into();
        if not_ac_every_time { data.ai_action_name = "AI_AC_AttackRange".into();  }
        // Special Dance for Dancer if Chapter 19 is completed
        if GameVariableManager::get_bool("G_Cleared_M019") { unit.private_skill.add_sid("SID_特別な踊り", 10, 0);   }
    }
    else if jid == "JID_エンチャント" {
        data.ai_attack_name = "AI_AT_Enchant".into();
        data.ai_attack_value = Some("".into());
    }
    // staff user, Chapter 22 needs to use Force due to Green Emblem Allies
    else if job.get_weapon_mask().value & ( 1 << 7 ) != 0 {
        if unit.item_list.has_item_iid("IID_ワープ") {
            data.ai_attack_name = "AI_AT_RodWarp".into();
            data.ai_attack_value = Some("1, 1".into());
            data.ai_move_name = "AI_MV_WeakEnemy".into();
        }
        else if unit.has_interfence_rod() {
            if m022 {  data.ai_attack_name = "AI_AT_InterferenceForceOnly".into();  }
            else {
                data.ai_attack_name = "AI_AT_Interference".into();
                data.ai_move_name =  "AI_MV_WeakEnemy".into();
            }
            if not_ac_every_time { data.ai_action_name = "AI_AC_AttackRange".into(); }
            data.ai_action_value =  Some("".into());
        }
        else if unit.has_heal_rod() {
            if m022 { data.ai_attack_name =  "AI_AT_AttackToHealForceOnly".into(); }
            else {
                data.ai_attack_name =  "AI_AT_HealToAttack".into();
                data.ai_move_name =  "AI_MV_WeakEnemy".into();
            }
        }
        else {
            data.ai_attack_name =  "AI_AT_Attack".into();
            data.ai_attack_value = None;
            data.ai_move_name =  "AI_MV_WeakEnemy".into();
        }
    }
    else {
        if str_contains(data.ai_action_name, "Guard") || str_contains(data.ai_mind_name, "Guard") { //Chain Guarder Unit
            unit.private_skill.add_sid("SID_チェインガード許可", 10, 0); 
        }
        // Healer turned non healer
        if str_contains(data.ai_action_name, "Heal") && not_ac_every_time { data.ai_action_name = "AI_AC_AttackRange".into(); }
        if str_contains(data.ai_attack_name, "Heal") {  
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
        }
        if str_contains(data.ai_move_name, "Heal") {  data.ai_move_name = "AI_MV_WeakEnemy".into(); }
        // No offensive staffs
        if str_contains(data.ai_action_name, "Interference") || str_contains(data.ai_attack_name, "Interference") {
            data.ai_action_name =  "AI_AC_AttackRange".into();
            data.ai_action_value = None;
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
            data.ai_move_name =  "AI_MV_WeakEnemy".into();
        }
        if str_contains(data.ai_attack_name, "RodWarp") { 
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
        }
    }
    if m022 {
        data.ai_move_name = "AI_MV_ForceOnly".into();
        data.ai_move_value = Some("FORCE_PLAYER".into());
        data.ai_attack_value = Some("FORCE_PLAYER".into());
    }
    if str_contains(old_ai_names[3], "Terrain") {  data.ai_move_name = old_ai_names[3]; }
    if data.get_flag().value & 16 != 0 ||  str_contains(old_ai_names[0], "Turn") { 
        data.ai_action_name = old_ai_names[0]; 
        data.ai_action_value = old_ai_values[0];
    }
    let engage_atk_ai = unsafe { emblem::get_engage_attack_type(unit_get_engage_atk(unit, None)) };
    if engage_atk_ai != -1 {
        data.ai_attack_name = ENGAGE_ATK_AI[engage_atk_ai as usize].into();
        if engage_atk_ai == 4 { data.ai_attack_value = Some("255, 255, 3, 3".into()); }
        else if engage_atk_ai == 8 { data.ai_attack_value = Some("2, 2, 255, 255".into()); }
        else { data.ai_attack_value = Some("2,2".into()); }
        if str_contains(data.ai_action_name, "AC_Null") {  data.ai_action_name = "AI_AC_AttackRange".into(); }
    }
    unsafe { unit_set_dispos_ai(unit, data, None) };
    data.ai_action_name = old_ai_names[0];
    data.ai_mind_name = old_ai_names[1];
    data.ai_attack_name = old_ai_names[2];
    data.ai_move_name = old_ai_names[3];
    data.ai_action_value = old_ai_values[0];
    data.ai_mind_value = old_ai_values[1];
    data.ai_attack_value = old_ai_values[2];
    data.ai_move_value = old_ai_values[3];
}

#[unity::from_offset("App", "Unit", "SetDisposAi")]
pub fn unit_set_dispos_ai(this: &Unit, data: &mut DisposData, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "GetEngageAttack")]
fn unit_get_engage_atk(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[unity::class("App", "AIData")]
pub struct AIData {
    pub parent: StructDataArrayFields,
    pub code: i8,
    pub mind: i8,
    pub active: i8,
    pub trans: i8,
    __: i32,
    pub str_value1: &'static Il2CppString,
    pub str_value2: &'static Il2CppString,
}
impl GamedataArray for AIData {}

#[unity::from_offset("App", "AIData", ".ctor")]
fn ai_data_ctor(this: &AIData, method_info: OptionalMethod);

pub fn create_custom_ai() {
    if unsafe { AISET} { return; }
    let ai_data_list = AIData::get_list_mut().unwrap();
    for x in 0..ai_data_list.len() {
        if str_contains(ai_data_list[x].array_name, "Engage") || !str_contains(ai_data_list[x].array_name, "AI_AT") {
            add_to_ai_data(ai_data_list[x], 0, 3, 21, -128, "V_0", "V_1"); 
            add_to_ai_data(ai_data_list[x], 0, 3, 23, -128, "1", "1"); 
            add_to_ai_data(ai_data_list[x], 0, 3, 110, -128, "V_Default", "V_Default"); //Command
            add_to_ai_data(ai_data_list[x], 0, 3, 119, -128, "V_Default", "V_Default"); // Contract
            add_to_ai_data(ai_data_list[x], 0, 3, 52, -128, "V_Default", "V_Default"); //Create Doubles
            add_to_ai_data(ai_data_list[x], 0, 3, -5, -128, "0", "V_Default"); // Command
            add_to_ai_data(ai_data_list[x], 0, 3, -5, -128, "1", "V_Default"); // Command
            continue;
        }
        add_to_ai_data(ai_data_list[x], 0, 3, 23, -128, "1", "1"); //Use Siege Weapons
        add_to_ai_data(ai_data_list[x], 0, 3, -8, -128, "V_Default", "V_Default"); //Use Siege Weapons
        add_to_ai_data(ai_data_list[x], 0, 3, 110, -128, "V_Default", "V_Default"); //Command
        add_to_ai_data(ai_data_list[x], 0, 3, 117, -128, "V_Default", "V_Default"); //Gambit
        add_to_ai_data(ai_data_list[x], 0, 3, 119, -128, "V_Default", "V_Default"); // Contract
        add_to_ai_data(ai_data_list[x], 0, 3, 120, -128, "V_Default", "V_Default"); // Rally
        add_to_ai_data(ai_data_list[x], 0, 3, 52, -128, "V_Default", "V_Default"); //Create Doubles
        add_to_ai_data(ai_data_list[x], 0, 3, -5, -128, "0", "V_Default"); // Command
        add_to_ai_data(ai_data_list[x], 0, 3, -5, -128, "1", "V_Default"); // Command
    }
    unsafe {  AISET = true; }

}

fn add_to_ai_data(ai: &mut StructDataArrayList<AIData>, active: i8, code: i8, mind: i8, trans: i8, str1: &str, str2: &str) {
    let at_pos = if ai.len() < 2 { 0 } 
        else { ai.len() - 2 };
    
    let new_ai_data = AIData::instantiate().unwrap();
    unsafe { ai_data_ctor(new_ai_data, None);}
    new_ai_data.code = code;
    new_ai_data.mind = mind;
    new_ai_data.active = active;
    new_ai_data.trans = trans;
    new_ai_data.str_value1 = str1.into();
    new_ai_data.str_value2 = str2.into();
    ai.insert(at_pos as i32, new_ai_data);
}

