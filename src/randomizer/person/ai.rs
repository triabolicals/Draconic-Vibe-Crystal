use super::*;
use crate::randomizer::emblem;
use engage::gamedata::ai::*;
use unit::has_sid;
static mut AISET: bool = false;

pub fn adjust_person_unit_ai(unit: &mut Unit) {
    if GameVariableManager::get_number(DVCVariables::RECRUITMENT_KEY) == 0 { return; }
    for order in 0..4 {
        for index in 0..4 {
            let ai_value = unit.ai.get_value(order, index);
            if let Some(person) = ai_value.get_person() {
                let new_person = switch_person(person);
                ai_value.set_str_value(new_person.pid);
            }
        }
    }
}

pub fn adjust_enemy_emblem_unit_ai_flags(unit: &Unit){
    let ai = &unit.ai;
    ai.set_sequence(2, "AI_AT_Versus");
    ai.set_flag(31);
    ai.set_flag(0x400);
    ai.set_flag(0x800);
    ai.set_flag( 0x20000000 );
}

pub fn adjust_ai_by_skill(unit: &mut Unit) {
    if has_sid(unit, "SID_特別な踊り") && unit.person.gender != 0 {
        unit.private_skill.add_sid("SID_踊り", 10, 0);
        unit.ai.set_sequence(1, "AI_MI_Irregular");
    }
}

pub fn create_custom_ai() {
    if unsafe { AISET} { return; }
    AIData::get_list_mut().unwrap().iter_mut()
        .for_each(|ai|{
            let name = ai.array_name.to_string();
            if name.contains("AI_AT") && !name.contains("Engage") && !name.contains("Versus") {                 // Engage Attack
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
            
                add_to_ai_data(ai, 0, 3, -3, -128, "V_Default", "V_Default", true);  
                add_to_ai_data(ai, 0, 3, -5, -128, "0", "V_Default", false);    // Command
                add_to_ai_data(ai, 0, 3, -5, -128, "1", "V_Default", false);    // Command
                add_to_ai_data(ai, 0, 3, -3, -128, "V_Default", "V_Default", false); 
                add_to_ai_data(ai, 0, 0, 0, -128, "V_Default", "V_Default", false);  

            }
            else if name.contains("MV") {
                add_to_ai_data(ai, 0, 3, 108, -128, "V_Default", "V_Default", true); 
                add_to_ai_data(ai, 0, 3, 108, -128, "1", "1", true);    // Rewrap
            }
        }
    );
    unsafe {  AISET = true; }

}

fn add_to_ai_data(ai: &mut StructDataArrayList<AIData>, active: i8, code: i8, mind: i8, trans: i8, str1: &str, str2: &str, is_top: bool) {
    let at_pos = if is_top || ai.len() < 2 { 0 }  else { ai.len() - 2 };
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

#[unity::from_offset("App", "Unit", "GetEngageAttack")]
fn unit_get_engage_atk(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[unity::from_offset("App", "AIData", ".ctor")]
fn ai_data_ctor(this: &AIData, method_info: OptionalMethod);

pub fn adjust_unitai(unit: &mut Unit) {
    let job = unit.get_job();
    let m022 = GameUserData::get_chapter().cid.to_string() == "CID_M022";
    let jid = job.jid.to_string();
    let not_ac_every_time = unit.ai.sequence[0].contains("AC_Everytime");
    let old_ai_names: [String; 4] = [unit.ai.sequence[0].to_string(), unit.ai.sequence[1].to_string(),  unit.ai.sequence[2].to_string(), unit.ai.sequence[3].to_string() ];
    let old_ac_values: [i16; 4] = [unit.ai.value[0].v16, unit.ai.value[1].v16, unit.ai.value[2].v16, unit.ai.value[3].v16];

    if unit.person.get_asset_force() == 2 {
        unit.ai.set_sequence(2, "AI_AT_Attack");
        unit.ai.set_sequence(3, "AI_MV_NearestEnemy");
    }
    let treasure = old_ai_names[1].contains("Treasure") || old_ai_names[3].contains("Treasure");
    // Allow Non-thieves to open doors if they have treasure AI
    if treasure { unit.private_skill.add_sid("SID_鍵開け", 10, 0);  }

    if old_ai_names[3].contains("Retreat") { unit.ai.set_sequence(3, "AI_MV_NearestEnemy"); }
    if jid == "JID_ダンサー" {
        unit.ai.set_sequence(1, "AI_MI_Irregular");
        if not_ac_every_time { unit.ai.set_sequence(0,  "AI_AC_AttackRange");  }
        // Special Dance for Dancer if Chapter 19 is completed
        if DVCVariables::is_main_chapter_complete(19) { unit.private_skill.add_sid("SID_特別な踊り", 10, 0);   }
    }
    else if jid == "JID_エンチャント" {
        unit.ai.set_sequence(2, "AI_AT_Enchant");
        set_ai_values_to_0(unit.ai, 2);
    }
    // staff user, Chapter 22 needs to use Force due to Green Emblem Allies
    else if job.get_weapon_mask().value & ( 1 << 7 ) != 0 {
        if unit.item_list.has_item_iid("IID_ワープ") {
            unit.ai.set_sequence(2, "AI_AT_RodWarp");
            unit.ai.set_value(2, 0, 1);
            unit.ai.set_value(2, 1, 1);
            unit.ai.set_sequence(3, "AI_MV_WeakEnemy");
        }
        else if unit.has_interfence_rod() {
            if m022 { unit.ai.set_sequence(2, "AI_AT_InterferenceForceOnly");  }
            else {
                unit.ai.set_sequence(2, "AI_AT_Interference");
                unit.ai.set_sequence(3, "AI_MV_WeakEnemy");
            }
            if not_ac_every_time { unit.ai.set_sequence(0,  "AI_AC_AttackRange"); }
            set_ai_values_to_0(unit.ai, 2);
        }
        else if unit.has_heal_rod() {
            if m022 { unit.ai.set_sequence(2,  "AI_AT_AttackToHealForceOnly"); }
            else {
                unit.ai.set_sequence(2,  "AI_AT_HealToAttack");
                unit.ai.set_sequence(3, "AI_MV_WeakEnemy");
            }
        }
        else {
            unit.ai.set_sequence(2,  "AI_AT_Attack");
            unit.ai.set_sequence(3, "AI_MV_WeakEnemy");
            set_ai_values_to_0(unit.ai, 2);
        }
    }
    else {
        if old_ai_names[0].contains("Guard") || old_ai_names[1].contains("Guard") { //Chain Guarder Unit
            unit.private_skill.add_sid("SID_チェインガード許可", 10, 0); 
        }
        // Healer turned non healer
        if old_ai_names[1].contains("Heal") && not_ac_every_time { unit.ai.set_sequence(0,  "AI_AC_AttackRange"); }
        if old_ai_names[2].contains("Heal") {  
            if m022 { unit.ai.set_sequence(2, "AI_AT_ForceOnly"); }
            else {  unit.ai.set_sequence(2, "AI_AT_Attack"); }
        }
        if  old_ai_names[3].contains( "Heal") {  unit.ai.set_sequence(3, "AI_MV_WeakEnemy"); }
        // No offensive staffs
        if  old_ai_names[1].contains("Interference") || old_ai_names[2].contains("Interference") {
            unit.ai.set_sequence(0,   "AI_AC_AttackRange");
            if m022 { unit.ai.set_sequence(2, "AI_AT_ForceOnly"); }
            else {  unit.ai.set_sequence(2, "AI_AT_Attack"); }
            set_ai_values_to_0(unit.ai, 2);
            set_ai_values_to_0(unit.ai, 0);
            unit.ai.set_sequence(3, "AI_MV_WeakEnemy");
        }
        if  old_ai_names[2].contains("RodWarp") { 
            if m022 { unit.ai.set_sequence(2, "AI_AT_ForceOnly"); }
            else {  unit.ai.set_sequence(2, "AI_AT_Attack"); }
            set_ai_values_to_0(unit.ai, 2);
        }
    }

    if old_ai_names[3].contains( "Terrain") {  unit.ai.set_sequence(3, old_ai_names[3].as_str()); }
    if unit.private_skill.find_sid("SID_リーダー".into()).is_some() ||  old_ai_names[0].contains("Turn") { 
        unit.ai.set_sequence(0,  old_ai_names[0].as_str());
        for x in 0..4 { unit.ai.set_value(0, x, old_ac_values[x as usize] as i32); }
    }
    let engage_atk_ai = unsafe { emblem::get_engage_attack_type(unit_get_engage_atk(unit, None)) };
    if engage_atk_ai != -1 {
        unit.ai.set_sequence(2, ENGAGE_ATK_AI[engage_atk_ai as usize]);
        if engage_atk_ai == 4 {
             unit.ai.set_value(2, 0, 255);
             unit.ai.set_value(2, 1, 255);
             unit.ai.set_value(2, 2, 3);
             unit.ai.set_value(2, 3, 3);
        }
        else if engage_atk_ai == 8 { 
            unit.ai.set_value(2, 0, 2);
            unit.ai.set_value(2, 1, 2);
            unit.ai.set_value(2, 2, 255);
            unit.ai.set_value(2, 3, 255);
        }
        else { 
            unit.ai.set_value(2, 0, 2);
            unit.ai.set_value(2, 1, 2);
        }
        if old_ai_names[0].contains("AC_Null") {  unit.ai.set_sequence(0, "AI_AC_AttackRange"); }
    }
    if m022 {
        if unit.ai.sequence[2].contains("Null") {
            unit.ai.set_sequence(2, "AI_AT_ForceOnly");
            unit.ai.set_value(2, 0, 0);
        }
        unit.ai.set_sequence(3, "AI_MV_ForceOnly");
        unit.ai.set_value(3, 0, 0);
        unit.ai.set_value(2, 0, 0);
    }
    else {
        if unit.ai.sequence[2].contains("Null") {
            unit.ai.set_sequence(2, "AI_AT_Attack");
            unit.ai.set_value(2, 0, 0);
        }
    }
}

fn set_ai_values_to_0(ai: &mut UnitAI, order: i32){
    ai.set_value(order, 0, 0);
    ai.set_value(order, 1, 0);
    ai.set_value(order, 2, 0);
    ai.set_value(order, 3, 0);
}
