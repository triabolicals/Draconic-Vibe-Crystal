use unity::prelude::*;
use engage::{
    gamevariable::*, gameuserdata::*, mess::*,
    gamedata::{cook::CookData, *, skill::*},
    hub::*,
    script::*,
};
use crate::{
    enums::*,
    randomizer::{emblem::{emblem_item::ENGAGE_ITEMS, emblem_skill::EIRIKA_INDEX}, *},
    utils::*,
};
#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&'static Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(label, is_replaced, method_info);
    if !unsafe { LUEUR_CHANGE } { return result; }
    if !can_rand() { return result; }
    unsafe {
        if let Some(mess_il2cp) = label {
            let mess_label = mess_il2cp.to_string();
            match mess_label.as_str() {
                "MSID_H_EirikEngage" => {
                    let gid = format!("GID_{}", EMBLEM_ASSET[ EIRIKA_INDEX]);
                    let eirika_replacement = GodData::get( &gid ).unwrap().mid;
                    return replace_str(result, Mess::get("MGID_Eirik"), Mess::get(eirika_replacement), None);
                },
                "MID_RULE_M006_LOSE" => {
                    let yunakers_replacement = GameVariableManager::get_string("G_R_PID_ユナカ");
                    return replace_str(result, Mess::get("MPID_Yunaka"), Mess::get(PersonData::get(&yunakers_replacement.to_string()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M015_LOSE"|"MID_RULE_M015_WIN"|"MID_TUT_NAVI_M015_ESCAPE" => {
                    let seadall_replacement = GameVariableManager::get_string("G_R_PID_セアダス");
                    return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get(&seadall_replacement.to_string()).unwrap().get_name().unwrap()), None);
                },
                "MGID_Lueur"|"MPID_Lueur" => { return GameVariableManager::get_string("G_Lueur_Name");  },  
                "MID_RULE_M007_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_オルテンシア");
                    return replace_str(result, Mess::get("MPID_Hortensia"), Mess::get(PersonData::get(&replacement.to_string()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M008_WIN"|"MID_RULE_M009_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_アイビー");
                    return replace_str(result, Mess::get("MPID_Ivy"), Mess::get(PersonData::get(&replacement.to_string()).unwrap().get_name().unwrap()), None);
                }
                "MID_RULE_M014_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_オルテンシア");
                    let replacement2 = GameVariableManager::get_string("G_R_PID_モーヴ");
                    let str1 = replace_str(result, Mess::get("MPID_Hortensia"), Mess::get(PersonData::get(&replacement.to_string()).unwrap().get_name().unwrap()), None);
                    return replace_str(str1, Mess::get("MPID_Mauve"), Mess::get(PersonData::get(&replacement2.to_string()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M016_WIN"|"MID_RULE_M019_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_モーヴ");
                    return replace_str(result, Mess::get("MPID_Mauve"), Mess::get(PersonData::get(&replacement.to_string()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M024_WIN"|"MPID_H_Veyre"|"MID_SELECTRING_LUEUR_NOTES"|"MPID_PastLueur" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_リュール");
                    return replace_str(result, Mess::get(PersonData::get(&replacement.to_string()).unwrap().get_name().unwrap()), GameVariableManager::get_string("G_Lueur_Name") , None);
                },
                _ => {},
            }
            if mess_label.contains("MTID_Ring_") && !GameVariableManager::get_bool("G_CustomEmblem")  {
                for x in 1..12 {
                    if mess_label.contains(RINGS[x as usize ]) { 
                        let index = crate::randomizer::person::pid_to_index(&EMBLEM_GIDS[x].to_string(), false);
                        return Mess::get(format!("MGID_Ring_{}", RINGS[ index as usize ]));
                     }
                }
            }
            else if mess_label.contains("MIID_H_") && GameVariableManager::get_bool("G_Random_Engage_Weps") {
                let engage_weapons = ENGAGE_ITEMS.lock().unwrap();
                if let Some(found)= engage_weapons.item_list.iter().find(|x| x.miid.contains(mess_label.as_str())) {
                    let new_emblem = found.new_emblem;
                    let old_emblem = found.original_emblem;
                    let mess_emblem = found.mess_emblem;
                    if new_emblem == -1 || new_emblem > 20 || old_emblem > 19 { return result; }
                    if mess_emblem == -1 && old_emblem != 19 { return result; }
                    let emblem_name = 
                        if old_emblem == 19 {  Mess::get(PersonData::get(&GameVariableManager::get_string("G_R_PID_リュール").to_string()).unwrap().get_name().unwrap())  } // Alear
                        else {  Mess::get( format!("MGID_{}", RINGS[mess_emblem as usize])) };
                    let new_emblem_name =  Mess::get( format!("MGID_{}", RINGS[ new_emblem as usize]));
                    return replace_str(result, emblem_name, new_emblem_name, None);
                }
                return result;
            }
            else if mess_label.contains("MID_TUT_NAVI_M022_GET_") {
                if GameVariableManager::get_number("G_Emblem_Mode") != 0 {
                    let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                    for x in RINGS {
                        if  mess_label.contains(x) {
                            let new_ring = format!("MGID_Ring_{}", x);
                            return replace_str(mock_text, Mess::get("MGID_Ring_Siglud"),  Mess::get(new_ring), None);
                        }
                    }
                }
            }
        }
    }
    return result;
}

pub fn god_engage_random_str(gid: &str) -> String {
    let god = GodData::get(gid).unwrap();
    let emblem_name = Mess::get( god.mid).to_string();
    let engage_attack = Mess::get( SkillData::get( &god.get_engage_attack().to_string() ).unwrap().name.unwrap() ).to_string();
    let mut string = " ------  ".into();
    let mut string2 = "  ------ ".into();
    let mut string3 = " ------ ".into();
    if let Some(sid) = god.get_engage_attack_link() {
        string2 = Mess::get( SkillData::get(&sid.to_string()).unwrap().name.unwrap()).to_string();
    }
    if let Some(gid) = god.get_link_gid() {
        string = Mess::get( GodData::get(&gid.to_string()).unwrap().mid).to_string(); 
    }
    if let Some(pid) = god.get_link() {
        string3 = Mess::get_name(pid).to_string();
    }
    else {
        if let Some(found) = EMBLEM_GIDS.iter().position(|&r| r == gid){
            if unsafe { LINKED[ found ] } != -1 { string3 = Mess::get_name( PIDS[ unsafe { LINKED[ found ] as usize } ] ).to_string(); }
        }
    }
    return format!("* {}: {} / {} ( {} | {} )", emblem_name, engage_attack, string2, string, string3);
}

#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
    let result = call_original!(dyn_value, method_info);
    if result.is_none() || !crate::utils::can_rand() { return result; }
    let result_string = result.unwrap();
    if str_contains(result_string, "Kengen") && !GameVariableManager::get_bool("G_CustomEmblem") {
        if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
        let str1 = result_string.to_string();
        let emblem_index = KENGEN.iter().position(|x| *x == str1);
        if emblem_index.is_none() { return result;}
        let gid = EMBLEM_GIDS[emblem_index.unwrap()];
        let new_index = crate::randomizer::person::pid_to_index(&gid.to_string(), false);
        if new_index < 1 || new_index >= 20 { return result; }
        return Some(KENGEN[new_index as usize].into());
    }
    if unsafe { string_start_with(result_string, "GID_".into(), None) } {
        if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
        if GameUserData::get_chapter().cid.to_string() == "CID_M026" { return result; } //Do not shuffle emblems in endgame
        if crate::utils::str_contains(GameUserData::get_chapter().cid, "CID_S0") { return result; }
        let gid = result_string.to_string();
        let string = format!("G_R_{}", gid);
        let new_gid = GameVariableManager::get_string(&string);
        if unsafe { !is_null_empty(new_gid, None) } { return Some(new_gid); }
    }
    else if  unsafe { string_start_with(result_string, "PID_".into(), None) } {
        if GameVariableManager::get_number("G_Random_Recruitment") == 0 { return result; }
        let cid = GameUserData::get_chapter().cid.to_string();
        if cid == "CID_M022" && GameVariableManager::exist("VeyleRecruitment") {
            if result_string.to_string() == "PID_ヴェイル" {
                let new_gid = GameVariableManager::get_string("G_R_PID_ヴェイル");
                let veyle_replacement = unsafe { get_person_pid(new_gid, false, None) };
                if veyle_replacement.is_none() { return result; }
                let force = veyle_replacement.unwrap().force.unwrap().force_type;
                if force == 2 { return Some(new_gid);  }
                else if force == 0 { GameVariableManager::set_bool("VeyleRecruitment", true); }
            }
            return result; 
        }
        else if cid == "CID_M026" { return result; }
        let string = format!("G_R_{}",  result_string.to_string());
        let new_pid = GameVariableManager::get_string(&string);
        if unsafe { !is_null_empty(new_pid, None) } { return Some(new_pid);  }
    }
    else if  unsafe { string_start_with(result_string, "IID_".into(), None) } {
        if GameVariableManager::get_number("G_Random_Item") == 0 || GameVariableManager::get_number("G_Random_Item") == 2  { return result; }
        else { return Some( crate::randomizer::item::get_random_item(result.unwrap(), false)); }
    }
    else if  unsafe { string_start_with(result_string, "TUTID_紋章士".into(), None) }{
        if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
        let key =  unsafe { replace_str(result_string, "TUTID_紋章士".into(), "G_R_GID_".into(), None) };
        let new_gid = GameVariableManager::get_string(&key.to_string());
        let new_tut =  unsafe { replace_str(new_gid, "GID_".into(), "TUTID_紋章士".into(), None) };
        return Some(new_tut);
    }
    return result;
}

// Prevents Alear or anyone with no cooking data from cooking 
pub fn cooking_menu_build_attribute(_this: u64, _method_info: OptionalMethod) -> i32 {
    if let Some(chef) = HubUtil::get_current_cooking_pid() {
        if CookData::get(chef).is_none() {  return 4; }
    }
    if GameVariableManager::get_bool("G_拠点_料理済み") { 2 } else { 1 }
}
pub fn lol_map_attribute(_this: u64, _method_info: OptionalMethod) -> i32 { 
    return 1;
}

pub fn replace_hub_fxn() {
    let cooking_menu = Il2CppClass::from_name("App", "HubPlayTalkAfter").unwrap().get_nested_types().iter().find(|x| x.get_name() == "CookingMenu").unwrap();
    let cooking_menu_mut = Il2CppClass::from_il2cpptype(cooking_menu.get_type()).unwrap();
    cooking_menu_mut.get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = cooking_menu_build_attribute as _);
    println!("Replaced Virtual Method of CookingMenu");

    if let Some(cc) = Il2CppClass::from_name("App", "ClassChangeJobMenu").unwrap().get_nested_types().iter().find(|x| x.get_name() == "ClassChangeJobMenuItem"){
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut("ACall").map(|method| method.method_ptr = crate::randomizer::job::class_change_a_call_random_cc as _);
        println!("Replaced ACall of ClassChangeJobMenuItem");
    }
}

pub fn set_script_variable<'a>(key: impl Into<&'a Il2CppString>, value: &DynValue) {
    let script = EventScript::get_instance();
    unsafe { moonsharp_table_set(script.global_table, key.into(), value, None); }
}

#[skyline::from_offset(0x01c54fa0)]
fn get_person_pid(pid: &Il2CppString, relay: bool, method_info: OptionalMethod) -> Option<&'static  engage::gamedata::unit::Unit>;

#[skyline::from_offset(0x02e20010)]
pub fn dynvalue_new_string(string: &Il2CppString, method_info: OptionalMethod) -> &'static DynValue;

#[skyline::from_offset(0x02d24990)]
pub fn moonsharp_table_set(this: *const u8, key: &Il2CppString, value: &DynValue, method_info: OptionalMethod);

