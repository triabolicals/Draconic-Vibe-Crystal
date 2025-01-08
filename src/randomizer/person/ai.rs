use super::*;
use crate::randomizer::emblem;
static mut AISET: bool = false;

pub fn adjust_dispos_person_ai(data: &mut DisposData) {
    if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return; }
    if let Some(value) = data.ai_action_value {
        let string = value.to_string();
        if PIDS.iter().any(|x| *x == string) {
            data.ai_action_value = Some(GameVariableManager::get_string(format!("G_R_{}", string)));
        }
    }
    if let Some(value) = data.ai_mind_value {
        let string = value.to_string();
        if PIDS.iter().any(|x| *x == string) {
            data.ai_mind_value = Some(GameVariableManager::get_string(format!("G_R_{}", string)));
        }
    }
    if let Some(value) = data.ai_move_value {
        let string = value.to_string();
        if PIDS.iter().any(|x| *x == string) {
            data.ai_move_value = Some(GameVariableManager::get_string(format!("G_R_{}", string)));
        }
    }
    if let Some(value) = data.ai_attack_value {
        let string = value.to_string();
        if PIDS.iter().any(|x| *x == string) {
            data.ai_attack_value = Some(GameVariableManager::get_string(format!("G_R_{}", string)));
        }
    }
}

pub fn engage_attack_ai(unit: &Unit, data:& mut DisposData) {
    let engage_atk_ai = unsafe { emblem::get_engage_attack_type(unit_get_engage_atk(unit, None)) };
    if engage_atk_ai != -1 {
        data.ai_attack_name = ENGAGE_ATK_AI[engage_atk_ai as usize].into();
        if engage_atk_ai == 4 { data.ai_attack_value = Some("255, 255, 3, 3".into()); }
        else if engage_atk_ai == 8 { data.ai_attack_value = Some("2, 2, 255, 255".into()); }
        else { data.ai_attack_value = Some("2,2".into()); }
        if data.ai_action_name.to_string().contains("AC_Null") {  data.ai_action_name = "AI_AC_AttackRange".into(); }
    }
    unsafe { unit_set_dispos_ai(unit, data, None) };
}

pub fn adjust_unit_ai(unit: &Unit, data: &mut DisposData) {
    println!("Adjusting AI");
    let job = unit.get_job();
    let m022 = GameUserData::get_chapter().cid.to_string() == "CID_M022";
    let jid = job.jid.to_string();
    let old_ai_names: [String; 4] = [data.ai_action_name.to_string(), data.ai_mind_name.to_string(), data.ai_attack_name.to_string(), data.ai_move_name.to_string()];
    let old_ai_values: [Option<&Il2CppString>; 4] = [data.ai_action_value, data.ai_mind_value, data.ai_attack_value, data.ai_move_value];
    let not_ac_every_time = data.ai_action_name.to_string().contains("AC_Everytime");

    if unit.person.get_asset_force() == 2 {
        data.ai_attack_name = "AI_AT_Attack".into();
        data.ai_move_name = "AI_MV_NearestEnemy".into();
    }
    // Allow Non-thieves to open doors if they have treasure AI
    if old_ai_names[1].contains("Treasure") || old_ai_names[3].contains("Treasure") { unit.private_skill.add_sid("SID_鍵開け", 10, 0);  }

    if old_ai_names[3].contains("Retreat") { data.ai_move_name = "AI_MV_NearestEnemy".into(); }
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
        if old_ai_names[0].contains("Guard") || old_ai_names[1].contains("Guard") { //Chain Guarder Unit
            unit.private_skill.add_sid("SID_チェインガード許可", 10, 0); 
        }
        // Healer turned non healer
        if  old_ai_names[1].contains("Heal") && not_ac_every_time { data.ai_action_name = "AI_AC_AttackRange".into(); }
        if  old_ai_names[2].contains("Heal") {  
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
        }
        if  old_ai_names[3].contains( "Heal") {  data.ai_move_name = "AI_MV_WeakEnemy".into(); }
        // No offensive staffs
        if  old_ai_names[1].contains("Interference") || old_ai_names[2].contains("Interference") {
            data.ai_action_name =  "AI_AC_AttackRange".into();
            data.ai_action_value = None;
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
            data.ai_move_name =  "AI_MV_WeakEnemy".into();
        }
        if  old_ai_names[2].contains("RodWarp") { 
            if m022 { data.ai_attack_name = "AI_AT_ForceOnly".into(); }
            else {  data.ai_attack_name = "AI_AT_Attack".into(); }
            data.ai_attack_value = None;
        }
    }

    if  old_ai_names[3].contains( "Terrain") {  data.ai_move_name = old_ai_names[3].as_str().into(); }
    if data.get_flag().value & 16 != 0 || old_ai_names[0].contains("Turn") { 
        data.ai_action_name = old_ai_names[0].as_str().into();
        data.ai_action_value = old_ai_values[0];
    }
    let engage_atk_ai = unsafe { emblem::get_engage_attack_type(unit_get_engage_atk(unit, None)) };
    if engage_atk_ai != -1 {
        data.ai_attack_name = ENGAGE_ATK_AI[engage_atk_ai as usize].into();
        if engage_atk_ai == 4 { data.ai_attack_value = Some("255, 255, 3, 3".into()); }
        else if engage_atk_ai == 8 { data.ai_attack_value = Some("2, 2, 255, 255".into()); }
        else { data.ai_attack_value = Some("2,2".into()); }
        if old_ai_names[1].contains("AC_Null") {  data.ai_action_name = "AI_AC_AttackRange".into(); }
    }
    if m022 {
        data.ai_move_name = "AI_MV_ForceOnly".into();
        data.ai_move_value = Some("FORCE_PLAYER".into());
        data.ai_attack_value = Some("FORCE_PLAYER".into());
    }
    unsafe { unit_set_dispos_ai(unit, data, None) };
    data.ai_action_name = old_ai_names[0].as_str().into();
    data.ai_mind_name = old_ai_names[1].as_str().into();
    data.ai_attack_name = old_ai_names[2].as_str().into();
    data.ai_move_name = old_ai_names[3].as_str().into();
    data.ai_action_value = old_ai_values[0];
    data.ai_mind_value = old_ai_values[1];
    data.ai_attack_value = old_ai_values[2];
    data.ai_move_value = old_ai_values[3];
    println!("Finshed adjusting AI");
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
    AIData::get_list_mut().unwrap().iter_mut()
        .for_each(|ai|{
            let name = ai.array_name.to_string();
            if name.contains("AI_AT") && !name.contains("Engage") {                 // Engage Attack
                add_to_ai_data(ai, 0, 3, -3, -128, "V_Default", "V_Default", true);  
                if name.contains("Interference") || name.contains("Rod") || name.contains("Heal") {
                    add_to_ai_data(ai, 0, 3, 21, -128, "1", "1", true);    // Warp
                    add_to_ai_data(ai, 0, 3, 23, -128, "1", "1", true);    // Rescue
                }
                add_to_ai_data(ai, 0, 3, 117, -128, "V_Default", "V_Default", false);   // Gambit
                add_to_ai_data(ai, 0, 3, 120, -128, "V_Default", "V_Default", false);   // Rally
                add_to_ai_data(ai, 0, 3, 119, -128, "V_Default", "V_Default", false); // Contract
                add_to_ai_data(ai, 0, 3, -6, -128, "V_Default", "V_Default", false);    // Dance
                add_to_ai_data(ai, 0, 3, 52, -128, "V_Default", "V_Default", false); // Call Doubles
                add_to_ai_data(ai, 0, 3, 110, -128, "V_Default", "V_Default", false); //Battle Commands
                add_to_ai_data(ai, 0, 3, -3, -128, "V_Default", "V_Default", true);  
                add_to_ai_data(ai, 0, 3, -5, -128, "0", "V_Default", false);    // Command
                add_to_ai_data(ai, 0, 3, -5, -128, "1", "V_Default", false);    // Command
                add_to_ai_data(ai, 0, 3, -3, -128, "V_Default", "V_Default", false); 
                add_to_ai_data(ai, 0, 0, 0, -128, "V_Default", "V_Default", false);  
            }
            else if ai.array_name.contains("MV") {
                add_to_ai_data(ai, 0, 3, 108, -128, "V_Default", "V_Default", true); 
                add_to_ai_data(ai, 0, 3, 108, -128, "1", "1", true);    // Rewrap
            }
        }
    );
    unsafe {  AISET = true; }

}

fn add_to_ai_data(ai: &mut StructDataArrayList<AIData>, active: i8, code: i8, mind: i8, trans: i8, str1: &str, str2: &str, is_top: bool) {
    let at_pos = if is_top || ai.len() < 2 { 0 } 
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

