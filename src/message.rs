use unity::prelude::*;
use engage::{
    gamevariable::*, gameuserdata::*, mess::*,
    gamedata::{cook::CookData, *, skill::*},
    hub::*,
};
use skyline::patching::Patch;
use crate::{
    enums::*,
    random::LINKED,
    emblem::{emblem_item::ENGAGE_ITEMS, emblem_skill::EIRIKA_INDEX}, 
    utils::*,

};

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(label, is_replaced, method_info);
    if !unsafe { LUEUR_CHANGE } { return result; }
    if !can_rand() { return result; }
    unsafe {
        if label.is_some() {
            let mess_label = label.unwrap().get_string().unwrap();
            match mess_label.as_str() {
                "MSID_H_EirikEngage" => {
                    let gid = format!("GID_{}", EMBLEM_ASSET[ EIRIKA_INDEX]);
                    let eirika_replacement = GodData::get( &gid ).unwrap().mid;
                    return replace_str(result, Mess::get("MGID_Eirik"), Mess::get(eirika_replacement), None);
                },
                "MID_RULE_M006_LOSE" => {
                    let yunakers_replacement = GameVariableManager::get_string("G_R_PID_ユナカ");
                    return replace_str(result, Mess::get("MPID_Yunaka"), Mess::get(PersonData::get(&yunakers_replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M015_LOSE"|"MID_RULE_M015_WIN"|"MID_TUT_NAVI_M015_ESCAPE" => {
                    let seadall_replacement = GameVariableManager::get_string("G_R_PID_セアダス");
                    return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get(&seadall_replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                },
                "MGID_Lueur"|"MPID_Lueur" => { return GameVariableManager::get_string("G_Lueur_Name");  },  
                "MID_RULE_M007_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_オルテンシア");
                    return replace_str(result, Mess::get("MPID_Hortensia"), Mess::get(PersonData::get(&replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M008_WIN"|"MID_RULE_M009_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_アイビー");
                    return replace_str(result, Mess::get("MPID_Ivy"), Mess::get(PersonData::get(&replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                }
                "MID_RULE_M014_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_オルテンシア");
                    let replacement2 = GameVariableManager::get_string("G_R_PID_モーヴ");
                    let str1 = replace_str(result, Mess::get("MPID_Hortensia"), Mess::get(PersonData::get(&replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                    return replace_str(str1, Mess::get("MPID_Mauve"), Mess::get(PersonData::get(&replacement2.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M016_WIN"|"MID_RULE_M019_WIN" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_モーヴ");
                    return replace_str(result, Mess::get("MPID_Mauve"), Mess::get(PersonData::get(&replacement.get_string().unwrap()).unwrap().get_name().unwrap()), None);
                },
                "MID_RULE_M024_WIN"|"MPID_H_Veyre"|"MID_SELECTRING_LUEUR_NOTES" => {
                    let replacement = GameVariableManager::get_string("G_R_PID_リュール");
                    return replace_str(result, Mess::get(PersonData::get(&replacement.get_string().unwrap()).unwrap().get_name().unwrap()), GameVariableManager::get_string("G_Lueur_Name") , None);
                },
                _ => {},
            }
            if string_start_with(label.unwrap(), "MTID_Ring_".into(), None) {
                for x in 1..12 {
                    let tid_label = format!("MTID_Ring_{}",  RINGS[x as usize ]);
                    if mess_label == tid_label { 
                        let index = crate::person::pid_to_index(&EMBLEM_GIDS[x].to_string(), false);
                        return Mess::get(format!("MGID_Ring_{}", RINGS[ index as usize ]));
                     }
                }
            }
            if string_start_with(label.unwrap(), "MIID_H_".into(), None) && GameVariableManager::get_bool("G_Random_Engage_Weps") {
                let found = ENGAGE_ITEMS.lock().unwrap().item_list.iter().position(|x| x.miid == mess_label);
                if found.is_some() {
                    let new_emblem = ENGAGE_ITEMS.lock().unwrap().item_list[found.unwrap()].new_emblem;
                    let old_emblem = ENGAGE_ITEMS.lock().unwrap().item_list[found.unwrap()].original_emblem;
                    if new_emblem == -1 { return result; }
                    let emblem_name = Mess::get( GodData::get(&format!("GID_{}", EMBLEM_ASSET[old_emblem as usize])).unwrap().mid);
                    let new_emblem_name = Mess::get( GodData::get(&format!("GID_{}", EMBLEM_ASSET[new_emblem as usize])).unwrap().mid);
                    return replace_str(result, emblem_name, new_emblem_name, None);
                }
                return result;
            }
            if string_start_with(label.unwrap(), "MID_TUT_NAVI_M022_GET_".into(), None){
                if GameVariableManager::get_number("G_Emblem_Mode") != 0 {
                    let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                    for x in RINGS {
                        if str_contains(label.unwrap(), x) {
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
/*
#[skyline::hook(offset=0x02d52340)]
pub fn get_face_path_god(god: &GodData, method_info: OptionalMethod) -> &Il2CppString {
    if god.gid.get_string().unwrap() == "GID_リュール" && GameVariableManager::exist("G_Lueur_Gender2") {
        if GameVariableManager::get_number("G_Lueur_Gender2") == 2 { return "LueurW_God".into(); }
        else { return "Lueur_God".into(); }
    }
    call_original!(god, method_info)
}
*/

pub fn god_engage_random_str(gid: &str) -> String {
    let god = GodData::get(gid).unwrap();
    let emblem_name = Mess::get( god.mid).get_string().unwrap();
    let engage_attack = Mess::get( SkillData::get( &god.get_engage_attack().get_string().unwrap() ).unwrap().name.unwrap() ).get_string().unwrap();
    let mut string = " ------  ".into();
    let mut string2 = "  ------ ".into();
    let mut string3 = " ------ ".into();
    if god.get_engage_attack_link().is_some() {
        let sid =  god.get_engage_attack_link().unwrap();
        string2 = Mess::get( SkillData::get(&sid.get_string().unwrap()).unwrap().name.unwrap()).get_string().unwrap();
    }
    if god.get_link_gid().is_some() {
        let gid = god.get_link_gid().unwrap();
        string = Mess::get( GodData::get(&gid.get_string().unwrap()).unwrap().mid).get_string().unwrap(); 
    }
    if god.get_link().is_some(){
        let pid = god.get_link().unwrap();
        string3 = Mess::get( PersonData::get(&pid.get_string().unwrap()).unwrap().get_name().unwrap()).get_string().unwrap(); 
    }
    else {
        let found = EMBLEM_GIDS.iter().position(|&r| r == gid); 
        if found.is_some() {
            unsafe {
                if LINKED[ found.unwrap() ] != -1 {
                    let pid = PIDS[ LINKED[ found.unwrap() ] as usize ];
                    string3 = Mess::get( PersonData::get(&pid).unwrap().get_name().unwrap()).get_string().unwrap(); 
                }
            }
        }
   }
    return format!("* {}: {} / {} ( {} | {} )", emblem_name, engage_attack, string2, string, string3);
}
#[skyline::hook(offset=0x021a3310)]
pub fn script_get_string(dyn_value: u64,  method_info: OptionalMethod) -> Option<&'static Il2CppString> {
   if GameUserData::get_sequence() == 6 { crate::emblem::emblem_gmap_spot_adjust(); }
   let result = call_original!(dyn_value, method_info);
   if result.is_none() { return result; }
   if !crate::utils::can_rand() { return result; }
   let result_string = result.unwrap();
   unsafe {
       if string_start_with(result_string, "GID_".into(), None) {
           if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
           if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M026" { return result; } //Do not shuffle emblems in endgame
           if crate::utils::str_contains(GameUserData::get_chapter().cid, "CID_S0") { return result; }
           let gid = result_string.get_string().unwrap();
           let string = format!("G_R_{}", gid);
           let new_gid = GameVariableManager::get_string(&string);
           if !is_null_empty(new_gid, None) { return Some(new_gid); }
       }
       else if string_start_with(result_string, "PID_".into(), None) {
           if !GameVariableManager::get_bool("G_Random_Recruitment") { return result; }
           if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022" && result_string.get_string().unwrap() != "PID_ヴェイル" { return result; }
           let pid = result_string.get_string().unwrap();
           let string = format!("G_R_{}", pid);
           let new_pid = GameVariableManager::get_string(&string);
           if !is_null_empty(new_pid, None) { return Some(new_pid);  }
       }
       else if string_start_with(result_string, "IID_".into(), None){
           if GameVariableManager::get_number("G_Random_Item") == 0 || GameVariableManager::get_number("G_Random_Item") == 2  { return result; }
           else { return Some(crate::item::get_random_item(result.unwrap(), false)); }
       }
       else if string_start_with(result_string, "TUTID_紋章士".into(), None){
           if GameVariableManager::get_number("G_Emblem_Mode") == 0 { return result; }
           let key = replace_str(result_string, "TUTID_紋章士".into(), "G_R_GID_".into(), None);
           let new_gid = GameVariableManager::get_string(&key.get_string().unwrap());
           let new_tut = replace_str(new_gid, "GID_".into(), "TUTID_紋章士".into(), None);
           return Some(new_tut);
       }
   }
   return result;
}
// Prevents Alear or anyone with no cooking data from cooking 
pub fn cooking_menu_build_attribute(_this: u64, _method_info: OptionalMethod) -> i32 {
    let chef = HubUtil::get_current_cooking_pid();
    if chef.is_some() { 
        if CookData::get(&chef.unwrap().get_string().unwrap()).is_none() {  return 4; }
    }
    if GameVariableManager::get_bool("G_拠点_料理済み") { 2 } else { 1 }
}

pub fn replace_hub_fxn() {
    let cooking_menu = Il2CppClass::from_name("App", "HubPlayTalkAfter").unwrap().get_nested_types().iter().find(|x| x.get_name() == "CookingMenu").unwrap();
    let cooking_menu_mut = Il2CppClass::from_il2cpptype(cooking_menu.get_type()).unwrap();
    cooking_menu_mut.get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = cooking_menu_build_attribute as _);
    println!("Replaced Virtual Method of CookingMenu");
}

#[unity::class("App", "HubPlayTalkAfter")]
pub struct HubPlayTalkAfter {}